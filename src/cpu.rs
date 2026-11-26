use crate::helpers::Value;
use crate::memory::Memory;
use std::collections::HashMap;
use std::io::{BufWriter, Write};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct CPU {
    // Storage
    pub program_counter: usize,
    pub accumulator: i64, // To store results (TODO switchable between 8, 16, 64-bit)
    pub variables: HashMap<char, i64>, // User and system variables

    // Math
    pub division_error: bool, // Tracks if a division by zero occurred
    pub remainder: i64,       // Stores the remainder of the last division
    pub pemdas: bool,         // Use PEMDAS operator order instead of left-to-right

    // I/O
    pub stdout: BufWriter<Vec<u8>>, // Output for e.g. ?="HELLO" statements
    pub rng: StdRng,                // PRNG instance
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            program_counter: 0, // Start at the first line
            accumulator: 0,
            variables: HashMap::new(),
            division_error: false,
            remainder: 0,
            pemdas: false,
            stdout: BufWriter::new(Vec::new()),
            rng: StdRng::from_entropy(), // Random seed for production
        }
    }

    // Add a constructor for testing, allowing a deterministic seed
    pub fn new_for_tests(seed: Option<u64>, left_to_right: Option<bool>) -> Self {
        CPU {
            program_counter: 0,
            accumulator: 0,
            variables: HashMap::new(),
            division_error: false,
            remainder: 0,
            pemdas: !left_to_right.unwrap_or(false),
            stdout: BufWriter::new(Vec::new()),
            rng: StdRng::seed_from_u64(seed.unwrap_or(0)), // Known seed for testing
        }
    }

    // Method to advance to the next line
    pub fn advance_program_counter(&mut self) {
        self.program_counter += 1;
    }

    // Method to set the program counter directly via the system variable `#`
    pub fn set_program_counter(&mut self, line_number: usize) {
        self.program_counter = line_number;
    }

    // Get the current value of the program counter (the system variable `#`)
    pub fn get_program_counter(&self) -> usize {
        self.program_counter
    }

    // Method to jump to a specific line number
    pub fn goto_line(&mut self, line_number: usize, memory: &Memory) -> Result<(), String> {
        if let Some(_line_code) = memory.read_program_line(line_number) {
            self.program_counter = line_number; // Set the program counter to the new line number
            Ok(())
        } else {
            Err(format!("No such line: {}", line_number))
        }
    }

    pub fn set_accumulator(&mut self, value: i64) {
        self.accumulator = value;
    }

    pub fn get_accumulator(&self) -> i64 {
        self.accumulator
    }

    // Set a variable value
    pub fn set_variable(&mut self, var: char, value: i64) {
        self.variables.insert(var, value);
    }

    // Get a variable value
    pub fn get_variable(&self, var: char) -> Option<i64> {
        self.variables.get(&var).cloned()
    }

    // Method to write output to the buffered `stdout`
    pub fn write_to_stdout(&mut self, output: &str) {
        self.stdout.write_all(output.as_bytes()).unwrap();
        self.stdout.write_all(b"\n").unwrap(); // Add newline
        self.stdout.flush().unwrap(); // Flush after each write (optional)
    }

    // Method to retrieve the buffered content of `stdout` as a String
    pub fn get_stdout(&mut self) -> String {
        self.stdout.flush().unwrap(); // Ensure all data is written to the underlying buffer
        String::from_utf8(self.stdout.get_ref().clone()).unwrap()
    }

    // Method to clear `stdout` buffer
    pub fn clear_stdout(&mut self) {
        self.stdout.get_mut().clear();
        self.stdout.flush().unwrap(); // Ensure that BufWriter's internal buffer is flushed
    }

    // Convert numeric to ASCII or ASCII to numeric when necessary
    pub fn convert_value(&self, value: &Value) -> i64 {
        match value {
            Value::Number(num) => *num,
            Value::Char(ch) => *ch as i64,
        }
    }

    // Convert numeric result back to ASCII character if it's within the valid range
    pub fn convert_to_char_if_possible(&self, result: i64) -> Value {
        if (0..=255).contains(&result) {
            Value::Char(result as u8 as char)
        } else {
            Value::Number(result)
        }
    }

    // Method to generate a random number
    pub fn generate_random_number(&mut self) -> i64 {
        self.rng.gen_range(0..=65535) as i64
    }
}
