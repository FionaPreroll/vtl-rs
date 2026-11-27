//! The command line program: options, the interactive machine on a pipe,
//! files and exit codes.

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn vtl(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vtl-rs"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("vtl-rs runs");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn file(name: &str, text: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("vtl-rs-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn interactive_session() {
    let out = vtl(
        &["-q"],
        "?=2+5*2\n10 ?=\"HI\"\n20 ?=A\n0\nA=7\n#=1\nBYE\n?=99\n",
    );
    assert_eq!(
        stdout(&out),
        "14\nOK\n10 ?=\"HI\"\n20 ?=A\nOK\nOK\nHI\n7\nOK\n"
    );
}

#[test]
fn greeting() {
    let out = vtl(&[], "");
    let text = stdout(&out);
    assert!(text.starts_with("VTL-RS "), "{text}");
    assert!(text.contains("32504 BYTES FREE"), "{text}");
}

#[test]
fn eval_and_precedence() {
    assert_eq!(stdout(&vtl(&["-q", "-e", "?=2+5*2"], "")), "14\nOK\n");
    assert_eq!(
        stdout(&vtl(&["-q", "--precedence", "-e", "?=2+5*2"], "")),
        "12\nOK\n"
    );
}

#[test]
fn strict_refuses_extensions() {
    let out = vtl(&["-q", "--strict", "-e", "?=6&3"], "");
    assert!(stdout(&out).starts_with("ERROR: BAD OPERATOR"));
    assert_eq!(stdout(&vtl(&["-q", "-e", "?=6&3"], "")), "2\nOK\n");
}

#[test]
fn file_with_comments_and_direct_lines() {
    let path = file(
        "direct.vtl",
        "#!/usr/bin/env vtl-rs\n# a comment\n) another\n10 ?=A\nA=5\n#=1\n",
    );
    let out = vtl(&[path.to_str().unwrap()], "");
    assert_eq!(stdout(&out), "5");
    assert!(out.status.success());
}

#[test]
fn errors_set_the_exit_code() {
    let path = file("broken.vtl", "10 A=1+\n20 ?=1\n");
    let out = vtl(&[path.to_str().unwrap()], "");
    assert_eq!(
        stdout(&out),
        "ERROR IN LINE 10: MISSING VALUE AT COLUMN 5\nERROR IN LINE 10: MISSING VALUE AT COLUMN 5\n"
    );
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn input_ends_the_program() {
    let path = file("input.vtl", "10 A=?\n20 ?=A\n");
    let out = vtl(&[path.to_str().unwrap()], "");
    assert_eq!(stdout(&out), "STOP AT 10\n");
}

#[test]
fn save_and_load() {
    let path = file("saved.vtl", "");
    let path = path.to_str().unwrap();
    let out = vtl(
        &["-q"],
        &format!("10 ?=\"SAVED\"\nSAVE {path}\nNEW\n0\nLOAD \"{path}\"\nRUN\n"),
    );
    assert_eq!(stdout(&out), "OK\nOK\nOK\nOK\nSAVED\nOK\n");
    assert_eq!(std::fs::read_to_string(path).unwrap(), "10 ?=\"SAVED\"\n");
}

#[test]
fn bad_options() {
    let out = vtl(&["--memory", "12"], "");
    assert_eq!(out.status.code(), Some(2));
    let out = vtl(&["--nonsense"], "");
    assert_eq!(out.status.code(), Some(2));
    let out = vtl(&["/nonexistent/file.vtl"], "");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn help_and_version() {
    assert!(stdout(&vtl(&["--help"], "")).starts_with("Usage: vtl-rs"));
    assert!(stdout(&vtl(&["-V"], "")).starts_with("vtl-rs "));
    assert!(stdout(&vtl(&["-q"], "HELP\n")).contains("VTL-2 IN SHORT"));
}

#[test]
fn hostile_input_is_refused_not_a_crash() {
    // Deeper than the stack would take, typed and as the reply to `?`.
    let deep = "(".repeat(100_000);
    let out = vtl(&["-q", "-e", "A=?", "-e", "?=A"], &format!("{deep}\n"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "OK\n0\nOK\n");
    let out = vtl(&["-q"], &format!("?={deep}1\n?=42\n"));
    assert!(stdout(&out).contains("EXPRESSION TOO DEEP"));
    assert!(stdout(&out).ends_with("42\nOK\n"));
}

#[test]
fn large_files_are_refused() {
    let path = file("large.vtl", &"\n".repeat((1 << 20) + 1));
    let out = vtl(&[path.to_str().unwrap()], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("larger than 1024 KB"));
}

#[test]
fn output_to_a_pipe_stays_raw() {
    // Escape sequences are left out on a terminal only; --raw is accepted.
    let out = vtl(&["-q", "--raw", "-e", "$=27"], "");
    assert_eq!(out.stdout, b"\x1b\nOK\n");
    let out = vtl(&["-q", "-e", "$=27"], "");
    assert_eq!(out.stdout, b"\x1b\nOK\n");
}
