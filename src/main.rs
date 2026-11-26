//! `vtl-rs`, the command line program: an interactive VTL-2 machine with
//! line editing and history, or a runner for program files.

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use vtl_rs::machine::{DEFAULT_MEMORY, PROGRAM_START};
use vtl_rs::syntax::Syntax;
use vtl_rs::{Config, Machine, Status};

/// Set by Ctrl-C; the machine stops at the next slice.
static BREAK: AtomicBool = AtomicBool::new(false);

/// Statements between two looks at Ctrl-C and the output.
const SLICE: u32 = 50_000;

const USAGE: &str = "\
Usage: vtl-rs [OPTIONS] [FILE]

Runs the VTL-2 program in FILE, or starts the interactive machine.
Lines of FILE that start with a number are stored as the program; other
lines run as if typed. If FILE runs nothing itself, the program runs from
its first line. Lines that start with # (and are no #= statement) are
comments.

Options:
  -i, --interactive     stay in the interactive machine after FILE
  -e, --eval LINE       enter LINE before FILE (may be given more than once)
  -m, --memory BYTES    end of memory, the value of * (default 32768)
      --strict          only the VTL-2 of the Altair 680, without the VTL02
                        extensions (& | ^ [ ] { } and @)
      --precedence      * and / before + and -, instead of left to right
      --seed N          seed of the random numbers in '
      --baud RATE       print no faster than a serial line at RATE baud
                        (110 is a Teletype ASR-33)
  -q, --quiet           no greeting
  -h, --help            this help
  -V, --version         the version
";

const HELP: &str = "\
VTL-2 IN SHORT (SEE docs/language.md)
  10 A=1          STORE LINE 10; 10 ALONE DELETES IT; 0 LISTS ALL
  ?=A  ?=\"HI\"     PRINT A NUMBER OR TEXT (TEXT;  KEEPS THE LINE OPEN)
  A=?  A=$  $=65  INPUT A NUMBER OR A CHARACTER, PRINT A CHARACTER
  #=100  #=X>5*100  GOTO 100 (#=0 DOES NOTHING: IF), #=! RETURNS
  :I)=5           ARRAY ELEMENT I, FROM & UP TO *
  + - * / = < >   LEFT TO RIGHT; > IS >=; % IS THE LAST REMAINDER
  '  RANDOM NUMBER   &  END OF PROGRAM   *  END OF MEMORY
  ) COMMENT
COMMANDS: RUN  LIST [FROM[-TO]]  NEW  LOAD FILE  SAVE FILE  HELP  BYE
CTRL-C STOPS A PROGRAM, CTRL-D LEAVES.
";

struct Options {
    file: Option<PathBuf>,
    interactive: bool,
    eval: Vec<String>,
    config: Config,
    baud: u32,
    quiet: bool,
}

enum Parsed {
    Run(Box<Options>),
    Exit(ExitCode),
}

fn parse_args() -> Result<Parsed, String> {
    let mut options = Options {
        file: None,
        interactive: false,
        eval: Vec::new(),
        // Another game every time, unless --seed says otherwise.
        config: Config {
            seed: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos() as u64),
            ..Config::default()
        },
        baud: 0,
        quiet: false,
    };
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        let Some(text) = arg.to_str() else {
            options.file = Some(arg.into());
            continue;
        };
        let (name, inline) = match text.split_once('=') {
            Some((name, value)) if name.starts_with("--") => (name, Some(value.to_string())),
            _ => (text, None),
        };
        let mut value = |what: &str| -> Result<String, String> {
            inline
                .clone()
                .or_else(|| args.next().and_then(|v| v.into_string().ok()))
                .ok_or_else(|| format!("{name} needs {what}"))
        };
        let number = |text: String, what: &str| -> Result<u64, String> {
            text.parse()
                .map_err(|_| format!("{name}: {what} expected, got {text:?}"))
        };
        match name {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(Parsed::Exit(ExitCode::SUCCESS));
            }
            "-V" | "--version" => {
                println!(
                    "vtl-rs {} (git {}, {})",
                    env!("CARGO_PKG_VERSION"),
                    env!("GIT_HASH"),
                    env!("RUSTC_VERSION")
                );
                return Ok(Parsed::Exit(ExitCode::SUCCESS));
            }
            "-i" | "--interactive" => options.interactive = true,
            "-q" | "--quiet" => options.quiet = true,
            "-e" | "--eval" => options.eval.push(value("a line")?),
            "-m" | "--memory" => {
                let bytes = number(value("a number of bytes")?, "bytes")?;
                if !(u64::from(PROGRAM_START) + 64..=65535).contains(&bytes) {
                    return Err(format!(
                        "{name}: between {} and 65535 bytes, please",
                        PROGRAM_START + 64
                    ));
                }
                options.config.memory = bytes as u16;
            }
            "--seed" => options.config.seed = number(value("a number")?, "a number")?,
            "--baud" => {
                options.baud = number(value("a rate")?, "a rate")?.min(1_000_000) as u32;
            }
            "--strict" => {
                options.config.syntax = Syntax {
                    extended: false,
                    ..options.config.syntax
                }
            }
            "--precedence" => {
                options.config.syntax = Syntax {
                    precedence: true,
                    ..options.config.syntax
                }
            }
            "-" => options.file = Some(arg.into()),
            _ if name.starts_with('-') => return Err(format!("unknown option {name}")),
            _ if options.file.is_some() => return Err("only one FILE, please".into()),
            _ => options.file = Some(arg.into()),
        }
    }
    Ok(Parsed::Run(Box::new(options)))
}

/// Standard output, optionally as slow as a serial line.
struct Output {
    stdout: io::Stdout,
    /// Characters per second; 0 for as fast as possible.
    cps: u32,
    since: Instant,
    sent: u64,
}

impl Output {
    fn new(baud: u32) -> Self {
        Output {
            stdout: io::stdout(),
            // A start bit, eight data bits and a stop bit (the ASR-33 had
            // two, which makes 110 baud the famous 10 characters a second).
            cps: if baud == 0 { 0 } else { (baud / 10).max(1) },
            since: Instant::now(),
            sent: 0,
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut out = self.stdout.lock();
        if self.cps == 0 {
            // A closed pipe is no reason to stop the machine.
            let _ = out.write_all(bytes).and_then(|()| out.flush());
            return;
        }
        for &byte in bytes {
            if BREAK.load(Ordering::Relaxed) {
                return;
            }
            self.sent += 1;
            let due = self.since + Duration::from_secs_f64(self.sent as f64 / f64::from(self.cps));
            if let Some(wait) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(wait);
            }
            let _ = out.write_all(&[byte]).and_then(|()| out.flush());
        }
    }

    /// The line was idle (waiting for input): no catching up afterwards.
    fn idle(&mut self) {
        self.since = Instant::now();
        self.sent = 0;
    }
}

enum Input {
    Line(String),
    Interrupted,
    End,
}

/// Standard input: with line editing and history at the prompt when it is a
/// terminal, plain lines for a program's input.
struct Terminal {
    editor: Option<DefaultEditor>,
    history: PathBuf,
}

impl Terminal {
    fn new() -> Self {
        let editor = if io::stdin().is_terminal() && io::stdout().is_terminal() {
            DefaultEditor::new().ok()
        } else {
            None
        };
        // Windows: %APPDATA%\vtl-rs_history, elsewhere ~/.vtl-rs_history.
        let (var, name) = if cfg!(windows) {
            ("APPDATA", "vtl-rs_history")
        } else {
            ("HOME", ".vtl-rs_history")
        };
        let history = std::env::var_os(var)
            .map(PathBuf::from)
            .unwrap_or_default()
            .join(name);
        let mut terminal = Terminal { editor, history };
        if let Some(editor) = &mut terminal.editor {
            let _ = editor.load_history(&terminal.history);
        }
        terminal
    }

    fn prompt_line(&mut self) -> Input {
        let Some(editor) = &mut self.editor else {
            return self.program_line();
        };
        match editor.readline("") {
            Ok(line) => {
                if !line.trim().is_empty() {
                    let _ = editor.add_history_entry(line.as_str());
                }
                Input::Line(line)
            }
            Err(ReadlineError::Interrupted) => Input::Interrupted,
            Err(_) => Input::End,
        }
    }

    fn program_line(&mut self) -> Input {
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => Input::End,
            Ok(_) if BREAK.load(Ordering::Relaxed) => Input::Interrupted,
            Ok(_) => Input::Line(line),
        }
    }

    fn save_history(&mut self) {
        if let Some(editor) = &mut self.editor {
            let _ = editor.save_history(&self.history);
        }
    }
}

/// Lets the machine work until it is ready for the next line. Returns false
/// if the input ended while a program waited for it.
fn drive(vtl: &mut Machine, terminal: &mut Terminal, out: &mut Output) -> bool {
    let mut status = vtl.status();
    loop {
        if status == Status::Running {
            status = vtl.run(SLICE);
        }
        out.write(&vtl.take_output());
        if BREAK.swap(false, Ordering::Relaxed) {
            vtl.stop();
            out.write(&vtl.take_output());
            return true;
        }
        match status {
            Status::Running => {}
            Status::Ready => return true,
            Status::NeedLine | Status::NeedChar => {
                out.idle();
                match terminal.program_line() {
                    Input::Line(line) => status = vtl.enter(&line),
                    Input::Interrupted => status = Status::Running,
                    Input::End => {
                        vtl.stop();
                        out.write(&vtl.take_output());
                        return false;
                    }
                }
            }
        }
    }
}

/// Enters a line and lets the machine work.
fn enter(vtl: &mut Machine, line: &str, terminal: &mut Terminal, out: &mut Output) -> bool {
    BREAK.store(false, Ordering::Relaxed);
    vtl.enter(line);
    drive(vtl, terminal, out)
}

/// Whether a line of a file is a comment: it starts with `)`, or with `#`
/// and is no `#=` (such as `#!/usr/bin/env vtl-rs`).
fn is_comment(line: &str) -> bool {
    line.starts_with(')')
        || line
            .strip_prefix('#')
            .is_some_and(|rest| !rest.trim_start().starts_with('='))
}

/// Enters the lines of a file. Returns whether any of them ran something.
fn load(
    vtl: &mut Machine,
    path: &PathBuf,
    terminal: &mut Terminal,
    out: &mut Output,
) -> io::Result<bool> {
    let bytes = if path.as_os_str() == "-" {
        let mut bytes = Vec::new();
        io::Read::read_to_end(&mut io::stdin(), &mut bytes)?;
        bytes
    } else {
        std::fs::read(path)?
    };
    let text = String::from_utf8_lossy(&bytes);
    let mut ran = false;
    let prompt = vtl.config().prompt;
    vtl.set_prompt(false);
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || is_comment(line) {
            continue;
        }
        if line.starts_with(|c: char| c.is_ascii_digit()) {
            vtl.enter(line);
            out.write(&vtl.take_output());
        } else {
            ran = true;
            if !enter(vtl, line, terminal, out) {
                break;
            }
        }
    }
    vtl.set_prompt(prompt);
    Ok(ran)
}

/// The commands of this program rather than of the machine.
fn host_command(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let end = line
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(line.len());
    let word = line[..end].to_ascii_uppercase();
    let arg = line[end..].trim();
    let known = ["BYE", "EXIT", "QUIT", "HELP", "LOAD", "SAVE"];
    (known.contains(&word.as_str()) && !arg.starts_with('='))
        .then(|| (word, arg.trim_matches('"').to_string()))
}

fn repl(vtl: &mut Machine, terminal: &mut Terminal, out: &mut Output) {
    loop {
        let line = match terminal.prompt_line() {
            Input::Line(line) => line,
            Input::Interrupted => continue,
            Input::End => break,
        };
        let Some((command, arg)) = host_command(&line) else {
            if !enter(vtl, &line, terminal, out) {
                break;
            }
            continue;
        };
        let reply = match command.as_str() {
            "BYE" | "EXIT" | "QUIT" => break,
            "HELP" => HELP.to_string(),
            _ if arg.is_empty() => format!("{command} NEEDS A FILE NAME\n"),
            "LOAD" => {
                vtl.new_program();
                match load(vtl, &PathBuf::from(&arg), terminal, out) {
                    Ok(_) => String::new(),
                    Err(e) => format!("CANNOT LOAD {arg}: {e}\n"),
                }
            }
            _ => match std::fs::write(&arg, vtl.listing(0, u16::MAX)) {
                Ok(()) => String::new(),
                Err(e) => format!("CANNOT SAVE {arg}: {e}\n"),
            },
        };
        out.write(reply.as_bytes());
        out.write(b"OK\n");
    }
    terminal.save_history();
}

fn main() -> ExitCode {
    let options = match parse_args() {
        Ok(Parsed::Run(options)) => options,
        Ok(Parsed::Exit(code)) => return code,
        Err(message) => {
            eprintln!("vtl-rs: {message}\nTry 'vtl-rs --help'.");
            return ExitCode::from(2);
        }
    };
    // Without a handler (if one cannot be set), Ctrl-C ends the program.
    let _ = ctrlc::set_handler(|| BREAK.store(true, Ordering::Relaxed));

    let interactive = options.file.is_none() || options.interactive;
    let mut vtl = Machine::new(Config {
        prompt: interactive,
        ..options.config
    });
    let mut out = Output::new(options.baud);
    let mut terminal = Terminal::new();

    if interactive && !options.quiet {
        let memory = if vtl.config().memory == DEFAULT_MEMORY {
            String::new()
        } else {
            format!(", * = {}", vtl.config().memory)
        };
        let greeting = format!(
            "VTL-RS {} - VERY TINY LANGUAGE{memory}\n{} BYTES FREE. TYPE HELP FOR HELP.\nOK\n",
            env!("CARGO_PKG_VERSION"),
            vtl.free()
        );
        out.write(greeting.as_bytes());
    }
    let mut going = true;
    for line in &options.eval {
        going = going && enter(&mut vtl, line, &mut terminal, &mut out);
    }
    if let Some(path) = &options.file {
        match load(&mut vtl, path, &mut terminal, &mut out) {
            Ok(false) => {
                vtl.set_prompt(false);
                BREAK.store(false, Ordering::Relaxed);
                vtl.start(1);
                going = drive(&mut vtl, &mut terminal, &mut out);
                vtl.set_prompt(interactive);
                if interactive {
                    out.write(b"OK\n");
                }
            }
            Ok(true) => {}
            Err(e) => {
                eprintln!("vtl-rs: {}: {e}", path.display());
                return ExitCode::from(2);
            }
        }
    }
    if interactive && going {
        repl(&mut vtl, &mut terminal, &mut out);
        return ExitCode::SUCCESS;
    }
    if vtl.take_errors() > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
