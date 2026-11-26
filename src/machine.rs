//! The VTL-2 machine: variables, memory with the program in it, and the
//! loop that runs statements.
//!
//! The machine never blocks. A host feeds it lines with [`Machine::enter`],
//! lets it work with [`Machine::run`] and collects what it printed with
//! [`Machine::take_output`]. When a statement needs input (`?` or `$`), `run`
//! returns [`Status::NeedLine`] or [`Status::NeedChar`]; the statement is
//! tried again from its start once the host has entered the input, with the
//! input it already consumed played back, so a statement either happens
//! completely or not at all.

use crate::syntax::{Expr, ParseError, Stmt, Syntax, Target, parse_expression, parse_statement};
use std::collections::VecDeque;

/// The address where the program begins. Arrays begin where it ends (`&`).
pub const PROGRAM_START: u16 = 264;
/// The memory a machine gets unless told otherwise: the value of `*`.
pub const DEFAULT_MEMORY: u16 = 32768;
/// Room for a statement: a stored line keeps its length in one byte, next to
/// its two bytes of line number and a closing zero.
pub const MAX_LINE: usize = 255 - 4;

/// How a machine is set up.
#[derive(Clone, Debug)]
pub struct Config {
    /// The end of memory, the initial value of `*`. The program and arrays
    /// live between [`PROGRAM_START`] and here.
    pub memory: u16,
    /// The dialect of the language.
    pub syntax: Syntax,
    /// The seed of the random numbers in `'`.
    pub seed: u64,
    /// Whether to print `OK` when the machine is ready for the next line.
    pub prompt: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            memory: DEFAULT_MEMORY,
            syntax: Syntax::default(),
            seed: 0x5eed_1977,
            prompt: true,
        }
    }
}

/// What the machine waits for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Waiting for a line at the `OK` prompt.
    Ready,
    /// Busy; call [`Machine::run`] again.
    Running,
    /// A `?` waits for a line of input.
    NeedLine,
    /// A `$` waits for a character.
    NeedChar,
}

/// The input a statement could not do without.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Need {
    Line,
    Char,
}

struct Line {
    number: u16,
    /// Where the line is stored: two bytes number, one byte length, the text
    /// and a zero.
    addr: u16,
    stmt: Result<Stmt, ParseError>,
}

enum Exec {
    Idle,
    Direct(Stmt),
    /// The line at `index` runs next; `number` finds it again if the program
    /// was read anew from memory.
    Program {
        index: usize,
        number: u16,
    },
}

/// Everything a statement reads and writes, apart from the parsed program.
struct Core {
    syntax: Syntax,
    vars: [u16; 128],
    mem: Vec<u8>,
    /// The program in memory was written to: read it anew before going on.
    dirty: bool,
    rng: u64,
    out: Vec<u8>,
    column: usize,
    after_cr: bool,
    lines_in: VecDeque<Vec<u8>>,
    chars_in: VecDeque<u8>,
    lines_used: usize,
    chars_used: usize,
    last_value: u16,
}

/// A VTL-2 interpreter.
pub struct Machine {
    config: Config,
    core: Core,
    lines: Vec<Line>,
    exec: Exec,
    waiting: Option<Status>,
    errors: usize,
    statements: u64,
}

const AMP: usize = b'&' as usize;
const STAR: usize = b'*' as usize;
const POUND: usize = b'#' as usize;
const BANG: usize = b'!' as usize;
const PERCENT: usize = b'%' as usize;
const LT: usize = b'<' as usize;

impl Core {
    fn amp(&self) -> u16 {
        self.vars[AMP]
    }

    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.mem[usize::from(addr)];
        let hi = self.mem[usize::from(addr.wrapping_add(1))];
        u16::from_le_bytes([lo, hi])
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        self.mem[usize::from(addr)] = value;
        if (PROGRAM_START..self.amp()).contains(&addr) {
            self.dirty = true;
        }
    }

    fn random(&mut self) -> u16 {
        // xorshift64*, the top 16 bits.
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        (self.rng.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 48) as u16
    }

    fn seed(&mut self, seed: u64) {
        // splitmix64, so that small seeds give good states; never zero.
        let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        self.rng = (z ^ (z >> 31)) | 1;
    }

    fn put(&mut self, byte: u8) {
        match byte {
            b'\r' => {
                self.out.push(b'\n');
                self.column = 0;
                self.after_cr = true;
                return;
            }
            b'\n' if self.after_cr => {}
            b'\n' => {
                self.out.push(b'\n');
                self.column = 0;
            }
            _ => {
                self.out.push(byte);
                self.column += 1;
            }
        }
        self.after_cr = false;
    }

    fn put_str(&mut self, text: &[u8]) {
        for &b in text {
            self.put(b);
        }
    }

    fn put_number(&mut self, value: u16) {
        let digits = itoa(value);
        self.put_str(digits.as_bytes());
    }

    fn newline_if_needed(&mut self) {
        if self.column != 0 {
            self.put(b'\n');
        }
    }

    fn eval(&mut self, expr: &Expr) -> Result<u16, Need> {
        Ok(match expr {
            Expr::Num(n) => *n,
            Expr::Var(v) => self.vars[usize::from(*v & 127)],
            Expr::Binary(op, a, b) => {
                let a = self.eval(a)?;
                let b = self.eval(b)?;
                op.apply(a, b, &mut self.vars[PERCENT])
            }
            Expr::Array(index) => {
                let index = self.eval(index)?;
                self.read_word(self.amp().wrapping_add(index.wrapping_mul(2)))
            }
            Expr::Random => self.random(),
            Expr::Peek => u16::from(self.mem[usize::from(self.vars[LT])]),
            Expr::InputChar => {
                let Some(&c) = self.chars_in.get(self.chars_used) else {
                    return Err(Need::Char);
                };
                self.chars_used += 1;
                u16::from(c)
            }
            Expr::InputNumber => {
                let Some(line) = self.lines_in.get(self.lines_used) else {
                    return Err(Need::Line);
                };
                self.lines_used += 1;
                // A reply that is no expression counts as 0.
                match parse_expression(line, self.syntax) {
                    Ok(expr) => self.eval(&expr)?,
                    Err(_) => 0,
                }
            }
        })
    }

    /// Runs one statement. If it needs input that is not there yet, nothing
    /// of it happens.
    fn exec(&mut self, stmt: &Stmt) -> Result<(), Need> {
        match stmt {
            Stmt::Nothing => {}
            Stmt::Print { text, newline } => {
                self.put_str(text);
                if *newline {
                    self.put(b'\n');
                }
            }
            Stmt::Assign(target, expr) => {
                let (remainder, rng) = (self.vars[PERCENT], self.rng);
                let evaluated = (|| {
                    let index = match target {
                        Target::Array(index) => Some(self.eval(index)?),
                        Target::Var(_) => None,
                    };
                    Ok((index, self.eval(expr)?))
                })();
                let (index, value) = match evaluated {
                    Ok(v) => v,
                    Err(need) => {
                        self.vars[PERCENT] = remainder;
                        self.rng = rng;
                        self.lines_used = 0;
                        self.chars_used = 0;
                        return Err(need);
                    }
                };
                self.lines_in.drain(..self.lines_used);
                self.chars_in.drain(..self.chars_used);
                self.lines_used = 0;
                self.chars_used = 0;
                self.last_value = value;
                match (target, index) {
                    (_, Some(index)) => {
                        let addr = self.amp().wrapping_add(index.wrapping_mul(2));
                        let [lo, hi] = value.to_le_bytes();
                        self.write_byte(addr, lo);
                        self.write_byte(addr.wrapping_add(1), hi);
                    }
                    (Target::Var(var), None) => self.store(*var, value),
                    (Target::Array(_), None) => {}
                }
            }
        }
        Ok(())
    }

    fn store(&mut self, var: u8, value: u16) {
        match var {
            b'?' => self.put_number(value),
            b'$' => self.put(value as u8),
            b'\'' => self.seed(u64::from(value)),
            b'@' if self.syntax.extended => self.write_byte(self.vars[LT], value as u8),
            b'&' => {
                self.vars[AMP] = value.max(PROGRAM_START);
                self.dirty = true;
            }
            _ => self.vars[usize::from(var & 127)] = value,
        }
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Machine {
    /// A machine with an empty program.
    pub fn new(config: Config) -> Self {
        let mut core = Core {
            syntax: config.syntax,
            vars: [0; 128],
            mem: vec![0; 65536],
            dirty: false,
            rng: 0,
            out: Vec::new(),
            column: 0,
            after_cr: false,
            lines_in: VecDeque::new(),
            chars_in: VecDeque::new(),
            lines_used: 0,
            chars_used: 0,
            last_value: 0,
        };
        core.seed(config.seed);
        core.vars[AMP] = PROGRAM_START;
        core.vars[STAR] = config.memory.max(PROGRAM_START);
        Machine {
            config,
            core,
            lines: Vec::new(),
            exec: Exec::Idle,
            waiting: None,
            errors: 0,
            statements: 0,
        }
    }

    /// How the machine was set up.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Whether to print `OK` when ready for the next line.
    pub fn set_prompt(&mut self, prompt: bool) {
        self.config.prompt = prompt;
    }

    /// What the machine waits for now.
    pub fn status(&self) -> Status {
        match (&self.waiting, &self.exec) {
            (Some(status), _) => *status,
            (None, Exec::Idle) => Status::Ready,
            (None, _) => Status::Running,
        }
    }

    /// The value of a variable.
    pub fn var(&self, name: u8) -> u16 {
        self.core.vars[usize::from(name & 127)]
    }

    /// Sets a variable as `name=value` would, without its output.
    pub fn set_var(&mut self, name: u8, value: u16) {
        if !matches!(name, b'?' | b'$') {
            self.core.store(name, value);
        }
    }

    /// A byte of memory.
    pub fn peek(&self, addr: u16) -> u8 {
        self.core.mem[usize::from(addr)]
    }

    /// The line number that runs now, 0 when no program runs.
    pub fn current_line(&self) -> u16 {
        match self.exec {
            Exec::Program { .. } => self.core.vars[POUND],
            _ => 0,
        }
    }

    /// The value the last assignment stored.
    pub fn last_value(&self) -> u16 {
        self.core.last_value
    }

    /// How many statements ran since the machine was made.
    pub fn statements(&self) -> u64 {
        self.statements
    }

    /// How many errors were reported since the last call.
    pub fn take_errors(&mut self) -> usize {
        std::mem::take(&mut self.errors)
    }

    /// What was printed since the last call. Line ends are `\n`.
    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.core.out)
    }

    /// What was printed and not yet taken.
    pub fn output(&self) -> &[u8] {
        &self.core.out
    }

    /// Forgets what was printed and not yet taken.
    pub fn clear_output(&mut self) {
        self.core.out.clear();
    }

    /// The column the output stands at.
    pub fn column(&self) -> usize {
        self.core.column
    }

    /// Bytes between the end of the program and `*`.
    pub fn free(&self) -> u16 {
        self.core.vars[STAR].saturating_sub(self.core.amp())
    }

    /// Whether a program is stored.
    pub fn has_program(&self) -> bool {
        self.core.amp() > PROGRAM_START
    }

    /// Takes a line from the user: at the prompt a program line, a direct
    /// statement or one of the commands `LIST`, `RUN` and `NEW`; while a
    /// statement waits for input, that input.
    pub fn enter(&mut self, text: &str) -> Status {
        let text = text.trim_end_matches(['\r', '\n']).as_bytes();
        match self.waiting.take() {
            Some(Status::NeedChar) => {
                self.core.chars_in.extend(text);
                self.core.chars_in.push_back(b'\r');
                return Status::Running;
            }
            Some(_) => {
                self.core.lines_in.push_back(text.to_vec());
                return Status::Running;
            }
            None => {}
        }
        if !matches!(self.exec, Exec::Idle) {
            // Typed ahead while a program runs: kept for its next `?`.
            self.core.lines_in.push_back(text.to_vec());
            return Status::Running;
        }
        let start = text
            .iter()
            .take_while(|&&b| b == b' ' || b == b'\t')
            .count();
        let line = &text[start..];
        if line.first().is_some_and(u8::is_ascii_digit) {
            let digits = line.iter().take_while(|b| b.is_ascii_digit()).count();
            let number = line[..digits].iter().fold(0u16, |n, d| {
                n.wrapping_mul(10).wrapping_add(u16::from(d - b'0'))
            });
            if number == 0 {
                self.list(0, u16::MAX);
                self.finish();
            } else {
                self.edit_line(number, trim(&line[digits..]));
            }
            return self.status();
        }
        if let Some(word) = command_word(line) {
            let arg = trim(&line[word.len()..]);
            match word.to_ascii_uppercase().as_slice() {
                b"LIST" => {
                    let (from, to) = parse_range(arg);
                    self.list(from, to);
                    self.finish();
                }
                b"RUN" => self.start(1),
                b"NEW" => {
                    self.new_program();
                    self.finish();
                }
                _ => {
                    self.report(None, "UNKNOWN COMMAND");
                    self.finish();
                }
            }
            return self.status();
        }
        match parse_statement(line, self.config.syntax) {
            Ok(stmt) => {
                self.core.vars[POUND] = 0;
                self.exec = Exec::Direct(stmt);
            }
            Err(e) => {
                self.report(None, &e.to_string());
                self.finish();
            }
        }
        self.status()
    }

    /// Runs up to `budget` statements.
    pub fn run(&mut self, budget: u32) -> Status {
        if let Some(status) = self.waiting {
            return status;
        }
        for _ in 0..budget {
            match std::mem::replace(&mut self.exec, Exec::Idle) {
                Exec::Idle => return Status::Ready,
                Exec::Direct(stmt) => {
                    if let Err(need) = self.core.exec(&stmt) {
                        self.exec = Exec::Direct(stmt);
                        return self.wait(need);
                    }
                    self.statements += 1;
                    if self.core.dirty {
                        self.reload();
                    }
                    match self.core.vars[POUND] {
                        0 => self.finish(),
                        target => self.start(target),
                    }
                }
                Exec::Program { mut index, number } => {
                    if self.core.dirty {
                        self.reload();
                        index = self.first_from(number);
                    }
                    let Some(line) = self.lines.get(index) else {
                        self.finish();
                        continue;
                    };
                    let number = line.number;
                    self.core.vars[POUND] = number;
                    let result = match &line.stmt {
                        Ok(stmt) => self.core.exec(stmt),
                        Err(e) => {
                            let message = e.to_string();
                            self.report(Some(number), &message);
                            self.finish();
                            continue;
                        }
                    };
                    if let Err(need) = result {
                        self.exec = Exec::Program { index, number };
                        return self.wait(need);
                    }
                    self.statements += 1;
                    if self.core.dirty {
                        self.reload();
                        index = self.first_from(number);
                    }
                    let target = self.core.vars[POUND];
                    let index = if target == 0 || target == number {
                        // Fall through to the next line.
                        if self.lines.get(index).is_some_and(|l| l.number == number) {
                            index + 1
                        } else {
                            self.first_from(number.saturating_add(1))
                        }
                    } else {
                        // A jump: `!` remembers where to come back to, and a
                        // missing line means the next one after it.
                        self.core.vars[BANG] = number.wrapping_add(1);
                        self.first_from(target)
                    };
                    match self.lines.get(index) {
                        Some(line) => {
                            self.exec = Exec::Program {
                                index,
                                number: line.number,
                            }
                        }
                        None => self.finish(),
                    }
                }
            }
        }
        self.status()
    }

    /// Stops what runs, as Ctrl-C does, and goes back to the prompt.
    pub fn stop(&mut self) {
        if matches!(self.exec, Exec::Idle) && self.waiting.is_none() {
            return;
        }
        self.core.newline_if_needed();
        self.core.put_str(b"STOP");
        if let Exec::Program { .. } = self.exec {
            self.core.put_str(b" AT ");
            let line = self.core.vars[POUND];
            self.core.put_number(line);
        }
        self.core.put(b'\n');
        self.finish();
    }

    /// Runs the program from the first line at or after `line`, as `#=line`.
    pub fn start(&mut self, line: u16) {
        self.waiting = None;
        let index = self.first_from(line.max(1));
        if index < self.lines.len() {
            self.exec = Exec::Program {
                index,
                number: self.lines[index].number,
            };
        } else {
            self.finish();
        }
    }

    /// Clears the program, as `&=0` does.
    pub fn new_program(&mut self) {
        self.core.store(b'&', PROGRAM_START);
        self.reload();
    }

    /// Stores, replaces or (with an empty statement) deletes a program line.
    /// Returns false if the line was refused, which it reports.
    pub fn edit_line(&mut self, number: u16, text: &[u8]) -> bool {
        if self.core.dirty {
            self.reload();
        }
        let index = self.first_from(number);
        let old = self
            .lines
            .get(index)
            .filter(|l| l.number == number)
            .map(|l| usize::from(self.core.mem[usize::from(l.addr) + 2]));
        if text.len() > MAX_LINE {
            self.report(None, "LINE TOO LONG");
            return false;
        }
        let len = if text.is_empty() { 0 } else { text.len() + 4 };
        let end = usize::from(self.core.amp());
        let free = usize::from(self.core.vars[STAR]).saturating_sub(end) + old.unwrap_or(0);
        if len > 0 && len >= free {
            self.report(None, "OUT OF MEMORY");
            return false;
        }
        let addr = self.lines.get(index).map_or(end, |l| usize::from(l.addr));
        let mem = &mut self.core.mem;
        let old = old.unwrap_or(0);
        mem.copy_within(addr + old..end, addr + len);
        if len > 0 {
            mem[addr..addr + 2].copy_from_slice(&number.to_le_bytes());
            mem[addr + 2] = len as u8;
            mem[addr + 3..addr + 3 + text.len()].copy_from_slice(text);
            mem[addr + len - 1] = 0;
        }
        let new_end = end + len - old;
        self.core.vars[AMP] = new_end as u16;
        let shift = |a: u16| (usize::from(a) + len - old) as u16;
        for line in &mut self.lines[index..] {
            line.addr = shift(line.addr);
        }
        if old > 0 {
            self.lines.remove(index);
        }
        if len > 0 {
            let stmt = parse_statement(text, self.config.syntax);
            if let Err(e) = &stmt {
                self.report(Some(number), &e.to_string());
            }
            self.lines.insert(
                index,
                Line {
                    number,
                    addr: addr as u16,
                    stmt,
                },
            );
        }
        true
    }

    /// The program as text, one line each, as `0` lists it.
    pub fn listing(&self, from: u16, to: u16) -> String {
        let mut text = String::new();
        for line in &self.lines {
            if (from..=to).contains(&line.number) {
                text.push_str(&line.number.to_string());
                text.push(' ');
                text.push_str(&String::from_utf8_lossy(self.line_text(line)));
                text.push('\n');
            }
        }
        text
    }

    fn line_text(&self, line: &Line) -> &[u8] {
        let addr = usize::from(line.addr);
        let len = usize::from(self.core.mem[addr + 2]);
        let text = &self.core.mem[addr + 3..addr + len.max(4) - 1];
        let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
        &text[..end]
    }

    fn list(&mut self, from: u16, to: u16) {
        if self.core.dirty {
            self.reload();
        }
        let text = self.listing(from, to);
        self.core.newline_if_needed();
        self.core.put_str(text.as_bytes());
    }

    fn first_from(&self, number: u16) -> usize {
        self.lines.partition_point(|l| l.number < number)
    }

    fn wait(&mut self, need: Need) -> Status {
        let status = match need {
            Need::Line => Status::NeedLine,
            Need::Char => Status::NeedChar,
        };
        self.waiting = Some(status);
        status
    }

    fn finish(&mut self) {
        self.exec = Exec::Idle;
        self.waiting = None;
        self.core.lines_in.clear();
        self.core.chars_in.clear();
        if self.config.prompt {
            self.core.newline_if_needed();
            self.core.put_str(b"OK\n");
        }
    }

    fn report(&mut self, line: Option<u16>, message: &str) {
        self.errors += 1;
        self.core.newline_if_needed();
        self.core.put_str(b"ERROR");
        if let Some(line) = line {
            self.core.put_str(b" IN LINE ");
            self.core.put_number(line);
        }
        self.core.put_str(b": ");
        self.core.put_str(message.as_bytes());
        self.core.put(b'\n');
    }

    /// Reads the program anew from memory, after something wrote into it or
    /// moved its end `&`.
    fn reload(&mut self) {
        self.core.dirty = false;
        self.lines.clear();
        let end = usize::from(self.core.amp());
        let mut addr = usize::from(PROGRAM_START);
        while addr + 4 <= end {
            let mem = &self.core.mem;
            let number = u16::from_le_bytes([mem[addr], mem[addr + 1]]);
            let len = usize::from(mem[addr + 2]);
            if len < 4 || addr + len > end {
                break;
            }
            let text = &mem[addr + 3..addr + len - 1];
            let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
            self.lines.push(Line {
                number,
                addr: addr as u16,
                stmt: parse_statement(text, self.config.syntax),
            });
            addr += len;
        }
    }
}

fn trim(text: &[u8]) -> &[u8] {
    let start = text.iter().take_while(|b| b.is_ascii_whitespace()).count();
    let text = &text[start..];
    let end = text.len()
        - text
            .iter()
            .rev()
            .take_while(|b| b.is_ascii_whitespace())
            .count();
    &text[..end]
}

/// A word of at least two letters that starts the line and is not followed
/// by `=`: no VTL-2 statement looks like that.
fn command_word(line: &[u8]) -> Option<&[u8]> {
    let len = line.iter().take_while(|b| b.is_ascii_alphabetic()).count();
    let rest = trim(&line[len..]);
    (len >= 2 && rest.first() != Some(&b'=')).then(|| &line[..len])
}

/// `LIST`, `LIST 100`, `LIST 100-200`, `LIST -200` or `LIST 100-`.
fn parse_range(arg: &[u8]) -> (u16, u16) {
    let text = String::from_utf8_lossy(arg);
    let mut parts = text.splitn(2, '-');
    let from = parts.next().unwrap_or("").trim();
    let to = parts.next().map(str::trim);
    let from: u16 = from.parse().unwrap_or(0);
    let to = match to {
        None if from > 0 => from,
        None | Some("") => u16::MAX,
        Some(to) => to.parse().unwrap_or(u16::MAX),
    };
    (from, to)
}

/// A number in decimal without allocating.
struct Digits {
    buf: [u8; 5],
    start: usize,
}

impl Digits {
    fn as_bytes(&self) -> &[u8] {
        &self.buf[self.start..]
    }
}

fn itoa(mut value: u16) -> Digits {
    let mut buf = [0; 5];
    let mut start = 5;
    loop {
        start -= 1;
        buf[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    Digits { buf, start }
}
