//! The behaviour of the machine, statement by statement.

use vtl_rs::machine::{PROGRAM_START, Status};
use vtl_rs::syntax::Syntax;
use vtl_rs::{Config, Machine};

fn machine() -> Machine {
    Machine::new(Config {
        prompt: false,
        ..Config::default()
    })
}

/// Enters the lines and runs until the machine waits; returns the output.
fn session(vtl: &mut Machine, lines: &[&str]) -> String {
    for line in lines {
        let mut status = vtl.enter(line);
        while status == Status::Running {
            status = vtl.run(1000);
        }
    }
    String::from_utf8(vtl.take_output()).unwrap()
}

fn run(lines: &[&str]) -> String {
    session(&mut machine(), lines)
}

#[test]
fn printing() {
    assert_eq!(
        run(&["?=1", "?=\"A\"", "?=\"B\";", "?=\"\"", "?=2"]),
        "1A\nB\n2"
    );
    assert_eq!(run(&["$=72", "$=73", "$=13", "$=10", "$=33"]), "HI\n!");
    assert_eq!(run(&["$=10", "$=10"]), "\n\n");
    assert_eq!(run(&["?=\"OPEN STRING"]), "OPEN STRING\n");
    // Any variable on the left prints a string, as in VTL02.
    assert_eq!(run(&["A=\"TEXT\";"]), "TEXT");
}

#[test]
fn arithmetic_is_16_bit_and_left_to_right() {
    assert_eq!(run(&["?=2+5*2"]), "14");
    assert_eq!(run(&["?=0-1"]), "65535");
    assert_eq!(run(&["?=300*300"]), "24464");
    assert_eq!(run(&["?=17/5", "?=\" \";", "?=%"]), "3 2");
    assert_eq!(run(&["?=7/0", "?=\" \";", "?=%"]), "65535 7");
    assert_eq!(run(&["?=3>3", "?=3<3", "?=3=3", "?=2>3"]), "1010");
    assert_eq!(run(&["?=(2+5)*(1+1)"]), "14");
    assert_eq!(run(&["?=2*(3+(4*5))"]), "46");
    assert_eq!(
        run(&["?=12&10", "?=\" \";", "?=12|3", "?=\" \";", "?=12^10"]),
        "8 15 6"
    );
    assert_eq!(run(&["?=0[5", "?=1[5", "?=0]5", "?=1]5"]), "0550");
    assert_eq!(run(&["?=1{4", "?=\" \";", "?=256}8"]), "16 1");
}

#[test]
fn precedence_on_request() {
    let mut vtl = Machine::new(Config {
        prompt: false,
        syntax: Syntax {
            extended: true,
            precedence: true,
        },
        ..Config::default()
    });
    assert_eq!(session(&mut vtl, &["?=2+5*2", "?=1+1=2"]), "121");
}

#[test]
fn goto_and_if() {
    // #=0 does nothing, which makes #=condition*line an IF.
    let program = ["10 A=1", "20 #=A=2*50", "30 ?=\"NOT TWO\"", "50 ?=\"END\""];
    let mut vtl = machine();
    assert_eq!(session(&mut vtl, &program), "");
    assert_eq!(session(&mut vtl, &["#=1"]), "NOT TWO\nEND\n");
    assert_eq!(session(&mut vtl, &["10 A=2", "#=1"]), "END\n");
    // A jump to a missing line goes on at the next one after it.
    assert_eq!(session(&mut vtl, &["#=40"]), "END\n");
    assert_eq!(session(&mut vtl, &["#=51"]), "");
}

#[test]
fn jumping_to_the_same_line_goes_on() {
    assert_eq!(run(&["10 #=10", "20 ?=\"ON\"", "#=1"]), "ON\n");
}

#[test]
fn return_with_bang() {
    let program = [
        "10 #=100",
        "20 ?=\"BACK\"",
        "30 #=999",
        "100 ?=!",
        "110 ?=\" \";",
        "120 #=!",
        "#=1",
    ];
    assert_eq!(run(&program), "11 BACK\n");
}

#[test]
fn pound_reads_the_current_line() {
    assert_eq!(run(&["10 ?=#", "20 ?=#", "#=1", "?=#"]), "10200");
}

#[test]
fn line_editing() {
    let mut vtl = machine();
    session(
        &mut vtl,
        &["30 C=3", "10 A=1", "20 B=2", "20 B=22", "  40   D=4  "],
    );
    assert_eq!(
        vtl.listing(0, u16::MAX),
        "10 A=1\n20 B=22\n30 C=3\n40 D=4\n"
    );
    session(&mut vtl, &["30"]);
    assert_eq!(vtl.listing(0, u16::MAX), "10 A=1\n20 B=22\n40 D=4\n");
    assert_eq!(session(&mut vtl, &["0"]), "10 A=1\n20 B=22\n40 D=4\n");
    assert_eq!(session(&mut vtl, &["LIST 20"]), "20 B=22\n");
    assert_eq!(session(&mut vtl, &["list 15-"]), "20 B=22\n40 D=4\n");
    assert_eq!(session(&mut vtl, &["LIST -10"]), "10 A=1\n");
    // Each line takes its text and four bytes.
    assert_eq!(vtl.var(b'&'), PROGRAM_START + 3 * 4 + 3 + 4 + 3);
    session(&mut vtl, &["NEW"]);
    assert_eq!(vtl.listing(0, u16::MAX), "");
    assert_eq!(vtl.var(b'&'), PROGRAM_START);
}

#[test]
fn ampersand_zero_clears_the_program() {
    let mut vtl = machine();
    session(&mut vtl, &["10 ?=1", "&=0"]);
    assert!(!vtl.has_program());
    assert_eq!(session(&mut vtl, &["#=1", "0"]), "");
}

#[test]
fn memory_runs_out() {
    let mut vtl = Machine::new(Config {
        prompt: false,
        memory: PROGRAM_START + 20,
        ..Config::default()
    });
    assert_eq!(session(&mut vtl, &["10 A=12345678"]), "");
    assert_eq!(vtl.free(), 6);
    assert_eq!(session(&mut vtl, &["20 B=1"]), "ERROR: OUT OF MEMORY\n");
    // Replacing a line counts the room of the old one.
    assert_eq!(session(&mut vtl, &["10 A=1234567890"]), "");
    assert_eq!(session(&mut vtl, &["?=*-&"]), "4");
    let long = format!("10 ?=\"{}\"", "X".repeat(300));
    assert_eq!(session(&mut vtl, &[&long]), "\nERROR: LINE TOO LONG\n");
}

#[test]
fn arrays_live_after_the_program() {
    let mut vtl = machine();
    let out = session(&mut vtl, &[":0)=513", ":3)=7", "?=:0)+:3)", "?=@"]);
    assert_eq!(out, "5200");
    let amp = vtl.var(b'&');
    assert_eq!((vtl.peek(amp), vtl.peek(amp + 1)), (1, 2));
    assert_eq!(vtl.peek(amp + 6), 7);
    // Storing a line moves & and with it where the arrays begin.
    session(&mut vtl, &["10 )"]);
    assert_eq!(vtl.var(b'&'), amp + 5);
}

#[test]
fn a_program_can_change_itself() {
    // Line 10 is stored at PROGRAM_START: number, length, then "A=1".
    let mut vtl = machine();
    session(&mut vtl, &["10 A=1", "20 ?=A"]);
    let digit = PROGRAM_START + 5;
    vtl.set_var(b'<', digit);
    assert_eq!(session(&mut vtl, &["@=57", "#=1"]), "9");
    assert_eq!(vtl.listing(0, u16::MAX), "10 A=9\n20 ?=A\n");
    // An array index below & wraps around into the program too: the word
    // at digit - 1 holds "=" and the digit.
    let index = ((digit - 1).wrapping_sub(vtl.var(b'&')) / 2) | 0x8000;
    let word = u16::from(b'=') | u16::from(b'7') << 8;
    let poke = format!(":{index})={word}");
    assert_eq!(session(&mut vtl, &[&poke, "#=1"]), "7");
}

#[test]
fn peek_and_poke() {
    let mut vtl = machine();
    assert_eq!(session(&mut vtl, &["<=40000", "@=300", "?=@"]), "44");
    assert_eq!(vtl.peek(40000), 44);
}

#[test]
fn strict_dialect_keeps_at_as_a_variable() {
    let mut vtl = Machine::new(Config {
        prompt: false,
        syntax: Syntax {
            extended: false,
            precedence: false,
        },
        ..Config::default()
    });
    assert_eq!(session(&mut vtl, &["<=40000", "@=300", "?=@"]), "300");
    assert_eq!(vtl.peek(40000), 0);
}

#[test]
fn current_line_follows_the_program() {
    let mut vtl = machine();
    vtl.enter("10 A=A+1");
    vtl.enter("20 #=10");
    assert_eq!(vtl.current_line(), 0);
    vtl.enter("#=1");
    let seen: Vec<u16> = (0..4)
        .map(|_| {
            vtl.run(1);
            vtl.current_line()
        })
        .collect();
    assert_eq!(seen, [10, 20, 10, 20]);
    vtl.stop();
    assert_eq!(vtl.current_line(), 0);
}

#[test]
fn line_input() {
    let mut vtl = machine();
    vtl.enter("10 A=?");
    vtl.enter("20 ?=A*2");
    assert_eq!(vtl.enter("#=1"), Status::Running);
    assert_eq!(vtl.run(100), Status::NeedLine);
    assert_eq!(vtl.current_line(), 10);
    // The reply is an expression, evaluated with the program's variables.
    assert_eq!(vtl.enter("20+1"), Status::Running);
    assert_eq!(vtl.run(100), Status::Ready);
    assert_eq!(vtl.take_output(), b"42");
}

#[test]
fn a_statement_waiting_for_input_happens_once() {
    // Two replies, a division and random numbers in one statement: tried
    // three times, it must come out as if it had run once.
    let mut vtl = machine();
    let mut expected = machine();
    session(&mut expected, &["B=17/5", "X='"]);
    let random = session(&mut expected, &["?='"]);
    vtl.enter("B=17/5");
    vtl.run(10);
    assert_eq!(vtl.enter("A='*0+%+?+?"), Status::Running);
    assert_eq!(vtl.run(10), Status::NeedLine);
    vtl.enter("10");
    assert_eq!(vtl.run(10), Status::NeedLine);
    vtl.enter("%");
    assert_eq!(vtl.run(10), Status::Ready);
    assert_eq!(vtl.var(b'A'), 2 + 10 + 2);
    assert_eq!(session(&mut vtl, &["?='"]), random);
}

#[test]
fn char_input() {
    let mut vtl = machine();
    vtl.enter("A=$*0+$");
    assert_eq!(vtl.run(10), Status::NeedChar);
    vtl.enter("AB");
    assert_eq!(vtl.run(10), Status::Ready);
    assert_eq!(vtl.var(b'A'), u16::from(b'B'));
    // What is left of the line went with the statement's end.
    vtl.enter("C=$");
    assert_eq!(vtl.run(10), Status::NeedChar);
    vtl.enter("");
    assert_eq!(vtl.run(10), Status::Ready);
    assert_eq!(vtl.var(b'C'), 13);
}

#[test]
fn chars_stay_queued_within_a_program() {
    let program = ["10 C=$", "20 #=C=13*50", "30 $=C+1", "40 #=10", "#=1"];
    let mut vtl = machine();
    for line in program {
        vtl.enter(line);
    }
    assert_eq!(vtl.run(100), Status::NeedChar);
    vtl.enter("HAL");
    assert_eq!(vtl.run(100), Status::Ready);
    assert_eq!(vtl.take_output(), b"IBM");
}

#[test]
fn random_numbers_follow_the_seed() {
    let a = run(&["'=7", "?='", "?=\" \";", "?='"]);
    let b = run(&["'=7", "?='", "?=\" \";", "?='"]);
    let c = run(&["'=8", "?='"]);
    assert_eq!(a, b);
    assert!(!a.starts_with(&c));
    let mut seen = std::collections::HashSet::new();
    let mut vtl = machine();
    for _ in 0..1000 {
        session(&mut vtl, &["A='/10"]);
        seen.insert(vtl.var(b'%'));
    }
    assert_eq!(seen.len(), 10);
}

#[test]
fn stopping() {
    let mut vtl = Machine::new(Config::default());
    vtl.enter("10 #=10+0*0+1");
    vtl.enter("11 #=10");
    vtl.take_output();
    vtl.enter("#=1");
    assert_eq!(vtl.run(1000), Status::Running);
    vtl.stop();
    let out = String::from_utf8(vtl.take_output()).unwrap();
    assert!(out.starts_with("STOP AT 1"), "{out}");
    assert!(out.ends_with("OK\n"));
    assert_eq!(vtl.status(), Status::Ready);
    assert_eq!(vtl.statements(), 1000);
}

#[test]
fn errors() {
    let mut vtl = Machine::new(Config::default());
    assert_eq!(
        session(&mut vtl, &["A=1+"]),
        "ERROR: MISSING VALUE AT COLUMN 5\nOK\n"
    );
    assert_eq!(
        session(&mut vtl, &["HELLO"]),
        "ERROR: UNKNOWN COMMAND\nOK\n"
    );
    assert_eq!(
        session(&mut vtl, &["10 ?=1", "20 X", "30 ?=3"]),
        "ERROR IN LINE 20: MISSING = AT COLUMN 2\n"
    );
    assert_eq!(
        session(&mut vtl, &["RUN"]),
        "1\nERROR IN LINE 20: MISSING = AT COLUMN 2\nOK\n"
    );
    assert_eq!(vtl.take_errors(), 4);
    assert_eq!(vtl.take_errors(), 0);
}

#[test]
fn ok_prompt() {
    let mut vtl = Machine::new(Config::default());
    assert_eq!(session(&mut vtl, &["?=5"]), "5\nOK\n");
    assert_eq!(session(&mut vtl, &["?=\"X\""]), "X\nOK\n");
    assert_eq!(session(&mut vtl, &["10 A=1"]), "");
    assert_eq!(session(&mut vtl, &["0"]), "10 A=1\nOK\n");
}

#[test]
fn typing_ahead() {
    let mut vtl = machine();
    vtl.enter("10 A=?");
    vtl.enter("20 ?=A");
    vtl.enter("#=1");
    // Entered while the program is still busy: kept for its `?`.
    assert_eq!(vtl.enter("5"), Status::Running);
    while vtl.run(100) == Status::Running {}
    assert_eq!(vtl.take_output(), b"5");
}

#[test]
fn direct_jump_runs_from_there() {
    assert_eq!(run(&["10 ?=1", "20 ?=2", "30 ?=3", "#=20"]), "23");
}

#[test]
fn star_and_ampersand() {
    let mut vtl = Machine::new(Config {
        prompt: false,
        memory: 4096,
        ..Config::default()
    });
    assert_eq!(session(&mut vtl, &["?=*", "?=\" \";", "?=&"]), "4096 264");
    assert_eq!(vtl.free(), 4096 - 264);
}

#[test]
fn key_presses_one_by_one() {
    let mut vtl = machine();
    vtl.enter("10 A=$");
    vtl.enter("20 B=$");
    vtl.enter("30 ?=A*0+B");
    assert_eq!(vtl.input_char(b'X'), Status::Ready, "nothing runs yet");
    vtl.enter("RUN");
    assert_eq!(vtl.run(100), Status::NeedChar);
    assert_eq!(vtl.input_char(b'X'), Status::Running);
    // Pressed before the program asks: kept for the next $.
    assert_eq!(vtl.input_char(b'Y'), Status::Running);
    assert_eq!(vtl.run(100), Status::Ready);
    assert_eq!(vtl.take_output(), b"89");
}
