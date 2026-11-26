//! The VTL-2 machine for the browser, as a WebAssembly module without any
//! glue generator: a handful of functions with numbers in and out, and two
//! buffers in the module's memory for text. `web/www/vtl.mjs` wraps them.
//!
//! The page writes a line into the buffer that [`vtl_buffer`] returns and
//! hands its length to [`vtl_enter`]; it reads what the machine printed at
//! [`vtl_output`] ([`vtl_output_len`] bytes) and clears it with
//! [`vtl_output_clear`]. Statuses are 0 ready, 1 running, 2 waiting for a
//! line, 3 waiting for a key.

use std::cell::RefCell;
use vtl_rs::syntax::Syntax;
use vtl_rs::{Config, Machine, Status};

struct State {
    machine: Machine,
    input: Vec<u8>,
    text: Vec<u8>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        machine: Machine::default(),
        input: Vec::new(),
        text: Vec::new(),
    });
}

fn with<T>(f: impl FnOnce(&mut State) -> T) -> T {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

fn code(status: Status) -> u32 {
    match status {
        Status::Ready => 0,
        Status::Running => 1,
        Status::NeedLine => 2,
        Status::NeedChar => 3,
    }
}

/// A new machine: `memory` is the value of `*`, bit 0 of `flags` the strict
/// VTL-2 of the Altair 680, bit 1 operator precedence.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_init(memory: u32, flags: u32, seed: u32) {
    let config = Config {
        memory: memory.clamp(1024, 65535) as u16,
        syntax: Syntax {
            extended: flags & 1 == 0,
            precedence: flags & 2 != 0,
        },
        seed: u64::from(seed),
        prompt: true,
    };
    with(|s| s.machine = Machine::new(config));
}

/// A buffer of `len` bytes for the next [`vtl_enter`].
#[unsafe(no_mangle)]
pub extern "C" fn vtl_buffer(len: u32) -> *mut u8 {
    with(|s| {
        s.input.clear();
        s.input.resize(len as usize, 0);
        s.input.as_mut_ptr()
    })
}

/// Enters the first `len` bytes of the buffer as a line.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_enter(len: u32) -> u32 {
    with(|s| {
        let len = (len as usize).min(s.input.len());
        let line = String::from_utf8_lossy(&s.input[..len]).into_owned();
        code(s.machine.enter(&line))
    })
}

/// A key press for `$`.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_key(key: u32) -> u32 {
    with(|s| code(s.machine.input_char(key as u8)))
}

/// Runs up to `budget` statements.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_run(budget: u32) -> u32 {
    with(|s| code(s.machine.run(budget)))
}

/// What the machine waits for.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_status() -> u32 {
    with(|s| code(s.machine.status()))
}

/// Stops the program, as Ctrl-C.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_stop() {
    with(|s| s.machine.stop());
}

/// Where the output begins.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_output() -> *const u8 {
    with(|s| s.machine.output().as_ptr())
}

/// How many bytes of output there are.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_output_len() -> u32 {
    with(|s| s.machine.output().len() as u32)
}

/// Forgets the output, once read.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_output_clear() {
    with(|s| s.machine.clear_output());
}

/// Writes the program's listing into the text buffer; returns its length.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_listing() -> u32 {
    with(|s| {
        s.text = s.machine.listing(0, u16::MAX).into_bytes();
        s.text.len() as u32
    })
}

/// Where the text buffer begins.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_text() -> *const u8 {
    with(|s| s.text.as_ptr())
}

/// Clears the program.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_new() {
    with(|s| s.machine.new_program());
}

/// The line that runs, or ran last (`#`): the front panel's address lights.
/// 0 after a statement typed at the prompt.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_line() -> u32 {
    with(|s| u32::from(s.machine.var(b'#')))
}

/// The value last stored: the front panel's data lights.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_value() -> u32 {
    with(|s| u32::from(s.machine.last_value()))
}

/// Free bytes between the program and `*`.
#[unsafe(no_mangle)]
pub extern "C" fn vtl_free() -> u32 {
    with(|s| u32::from(s.machine.free()))
}

/// Statements run so far (as a float, for JavaScript's numbers).
#[unsafe(no_mangle)]
pub extern "C" fn vtl_statements() -> f64 {
    with(|s| s.machine.statements() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enter(text: &str) -> u32 {
        let ptr = vtl_buffer(text.len() as u32);
        with(|s| assert_eq!(s.input.as_ptr(), ptr as *const u8));
        with(|s| s.input.copy_from_slice(text.as_bytes()));
        vtl_enter(text.len() as u32)
    }

    fn output() -> String {
        let text = with(|s| String::from_utf8_lossy(s.machine.output()).into_owned());
        assert_eq!(vtl_output_len() as usize, text.len());
        vtl_output_clear();
        text
    }

    #[test]
    fn a_session() {
        vtl_init(4096, 0, 1);
        assert_eq!(vtl_free(), 4096 - 264);
        assert_eq!(enter("10 C=$"), 0);
        enter("20 ?=C");
        assert_eq!(enter("#=1"), 1);
        assert_eq!(vtl_run(10), 3);
        assert_eq!(vtl_line(), 10);
        assert_eq!(vtl_key(65), 1);
        assert_eq!(vtl_run(10), 0);
        assert_eq!(vtl_status(), 0);
        assert_eq!(vtl_value(), 65);
        assert_eq!(output(), "65\nOK\n");
        assert_eq!(vtl_listing(), 14);
        assert!(!vtl_text().is_null());
        vtl_new();
        assert_eq!(vtl_listing(), 0);
        assert!(vtl_statements() >= 2.0);
    }

    #[test]
    fn stopping_and_dialects() {
        vtl_init(65535, 3, 1);
        enter("?=2+5*2");
        vtl_run(10);
        assert_eq!(output(), "12\nOK\n");
        enter("?=6&3");
        assert!(output().starts_with("ERROR: BAD OPERATOR"));
        enter("10 #=11");
        enter("11 #=10");
        enter("#=1");
        assert_eq!(vtl_run(100), 1);
        vtl_stop();
        assert!(output().starts_with("STOP AT 1"));
        assert!(!vtl_output().is_null());
    }
}
