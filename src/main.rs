use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use std::path::PathBuf;
use vtl_rs::cpu::CPU;
use vtl_rs::helpers::get_build_info;
use vtl_rs::interpreter::evaluate_expression;
use vtl_rs::memory::Memory;

fn main() {
    println!("VTL-RS Interpreter");
    println!("{}", get_build_info());
    println!("Type 'help' to do something, I forgot what it was.");

    // Initialize CPU and memory
    let mut cpu = CPU::new();
    let mut memory = Memory::new(1024);

    // Initialize Rustyline editor with default helper and history
    let history_path = get_history_file();
    let mut rl = Editor::<(), DefaultHistory>::new().unwrap();
    if rl.load_history(&history_path).is_err() {
        println!("No previous history.");
    }

    // Enhanced REPL loop
    loop {
        // Read user input with history support
        let readline = rl.readline("VTL> ");
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input == "exit" {
                    println!("Exiting REPL.");
                    break;
                }

                // Add input to history
                rl.add_history_entry(input).unwrap();

                // clear output buffer
                cpu.clear_stdout();

                if let Err(_err) = handle_input(&line, &mut cpu, &mut memory) {
                    // Command failed, this is already handled by handle_input
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C, Exiting REPL.");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D, Exiting REPL.");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save command history
    rl.save_history(&history_path).unwrap();
}

fn handle_input(input: &str, cpu: &mut CPU, memory: &mut Memory) -> Result<(), String> {
    // Call evaluate_expression to handle parsing and evaluation
    match evaluate_expression(input, cpu, memory) {
        Ok(result) => {
            // Print the current stdout buffer
            let output = cpu.get_stdout();
            if !output.is_empty() {
                println!("{}", output);
            }
            println!("OK ({})", result);
            Ok(())
        }
        Err(err) => {
            // Print error message
            println!("ERROR: {}", err);
            Err(err)
        }
    }
}

fn get_history_file() -> PathBuf {
    // Windows: %APPDATA%\vtl-rs_history, elsewhere ~/.vtl-rs_history.
    let (var, name) = if cfg!(target_os = "windows") {
        ("APPDATA", "vtl-rs_history")
    } else {
        ("HOME", ".vtl-rs_history")
    };
    std::env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(name)
}
