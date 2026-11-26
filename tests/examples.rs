//! The programs in examples/ through the command line program, against
//! their output in tests/expected/ (with the input in <name>.in, if any).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn run(name: &str, args: &[&str]) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected = std::fs::read_to_string(root.join(format!("tests/expected/{name}.out")))
        .expect("expected output");
    let input = std::fs::read(root.join(format!("tests/expected/{name}.in"))).unwrap_or_default();
    let mut child = Command::new(env!("CARGO_BIN_EXE_vtl-rs"))
        .args(args)
        .arg(root.join(format!("examples/{name}.vtl")))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("vtl-rs runs");
    child.stdin.take().unwrap().write_all(&input).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{name}: {:?}", output.status);
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected, "{name}");
}

#[test]
fn hello() {
    run("hello", &[]);
}

#[test]
fn primes() {
    run("primes", &[]);
}

#[test]
fn fibonacci() {
    run("fibonacci", &[]);
}

#[test]
fn alphabet() {
    run("alphabet", &[]);
}

#[test]
fn subroutine() {
    run("subroutine", &[]);
}

#[test]
fn sieve() {
    run("sieve", &[]);
}

#[test]
fn echo() {
    run("echo", &[]);
}

#[test]
fn guess() {
    run("guess", &["--seed", "1"]);
}

/// The factorials of the VTL-2 manual, "about 208!" with 1K of memory: all
/// 208 of them, checked digit for digit when the expected output was made.
#[test]
fn factorials() {
    run("factorials", &["--memory", "1024"]);
}
