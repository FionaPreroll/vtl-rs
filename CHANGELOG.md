# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.0 - Unreleased

A rewrite: vtl-rs is now a complete VTL-2 with a command line program and a
web version.

### Added

- The complete VTL-2 of the Altair 680: programs of numbered lines stored in
  memory, `0` to list them, jumps and returns with `#` and `!`, input and
  output with `?` and `$`, arrays `:I)` after the program, and the system
  variables `%`, `&`, `*` and `'`.
- The VTL02 extensions `&`, `|`, `^`, `[`, `]`, `{`, `}` and `@`, on by
  default; `--strict` turns them off.
- Command line program: runs program files, has `RUN`, `LIST`, `NEW`, `LOAD`,
  `SAVE`, `HELP` and `BYE`, line editing with history, stops a program on
  Ctrl-C, and prints at the speed of a Teletype with `--baud`. Further
  options: `--interactive`, `--eval`, `--memory`, `--seed`, `--quiet`.
- Web version on GitHub Pages: the machine as WebAssembly behind a front
  panel in the manner of the Altair, whose lights follow the running
  program.
  - Starts as a terminal of 1977: amber phosphor at 300 baud.
  - CRT themes (green, amber, white), Altair blue, Teletype paper and modern
    dark and light; old and modern fonts; serial line speeds down to 110 baud.
  - Long bat-handle switches, full screen for the terminal or the whole
    machine, memory up to 64 KB, optional steady cursor.
  - Examples, programs saved in the browser, download and sharing as links.
  - Installs as an app and works offline; an About box shows version and
    build.
- Example programs with their expected output, and a language reference in
  `docs/language.md`.
- Release builds for Linux (x86_64, aarch64), macOS and Windows.

### Changed

- Numbers are 16-bit unsigned and expressions are evaluated strictly from
  left to right, as in VTL-2.
- `--precedence` replaces the former PEMDAS switch.
- The interpreter core has no dependencies and never blocks, so the same
  machine runs on the command line and in the browser.
- Rust edition 2024, minimum supported Rust version 1.85.

### Security

- An expression holds at most 512 terms and parentheses, and a reply to `?`
  holds a further `?` at most four levels deep. Deeper input, typed at the
  prompt or as a reply, ran out of stack: the command line program aborted
  and the web page stopped responding until it was reloaded.
- The web page starts the machine anew, with a message, should the module
  ever crash.
- A shared link stores only the numbered lines of its program and runs
  nothing. Before, its other lines ran as if typed, `SAVE` too, which could
  overwrite a program saved in the browser.
- The command line program leaves out escape sequences and other control
  characters that a program prints to a terminal, so that a program cannot
  set the window title, rewrite the screen or the clipboard; `--raw` passes
  them on. Into a file or a pipe the output stays as printed.
- Program files larger than 1 MB are refused.
- The web page cuts lines longer than 4096 characters, lets a program print
  at most 64 KB a frame, and has a Content Security Policy.
- `Machine::listing` stays within memory after `set_var` changed the program.
- CI installs the npm packages without their install scripts.

### Removed

- The dependencies `dirs`, `flexi_logger`, `log`, `rand` and `chrono`.

## 0.1.0 - 2024-10-06

### Added

- Interactive prompt (REPL) with a greeting and a history file
  (`~/.vtl-rs_history`, on Windows in `AppData/Roaming`).
- String output with `?="..."`.
- Variables and assignments.
- Random numbers with `'`.
- Remainder of the last division with `%`.
- Program counter register, bound to the variable `#`.
- Switchable evaluation order: strict left to right (as in VTL-2) or
  operator precedence (PEMDAS).

## 0.0.1 - 2024-10-04

### Added

- First version: lexer, parser and evaluator for arithmetic expressions
  with `+`, `-`, `*`, `/` and unary minus; `?=2+2` prints `4`.
