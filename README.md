# vtl-rs

[![CI](https://github.com/FionaPreroll/vtl-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/FionaPreroll/vtl-rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/FionaPreroll/vtl-rs/graph/badge.svg)](https://codecov.io/gh/FionaPreroll/vtl-rs)

*vtl-rs* is a Rust implementation of the *Very Tiny Language (VTL-2)*, a minimalist language written in 1977 by Frank McCoy and Gary Shannon for the Altair 680, where the whole interpreter fit into 768 bytes. It runs classic VTL-2 programs as the original did, as a small and fast command line program for Linux, macOS and Windows, and in the browser.

**Try it: [fionapreroll.github.io/vtl-rs](https://fionapreroll.github.io/vtl-rs/)**

![The web version with a green phosphor screen and the front panel](https://fionapreroll.github.io/vtl-rs/screenshots/green.png)

![Introduction excerpt from the VTL-2 manual. Source: [1]](./doc/vtl2intro.png)

## What VTL-2 looks like

```
10 ?="PRIME NUMBERS BELOW 50"
20 N=2
30 D=2
40 M=N+1
50 #=D*D>M*90
60 Q=N/D
70 #=%=0*110
80 D=D+1
85 #=50
90 ?=N
100 ?=" ";
110 N=N+1
120 #=N<50*30
```

Every line is one assignment. `?=` prints, `#=` jumps (and `#=0` does not, which makes `#=condition*line` an IF), `%` is the remainder of the last division, and expressions run strictly from left to right on 16-bit numbers. [docs/language.md](docs/language.md) describes the whole language; [examples/](examples/) holds programs to try.

## What vtl-rs does

- The complete VTL-2 of the Altair 680: programs of numbered lines, `0` to list them, jumps and returns with `#` and `!`, input and output of numbers and characters with `?` and `$`, arrays `:I)` in the memory after the program, `%`, `&`, `*` and random numbers with `'`.
- The VTL02 extensions (`& | ^ [ ] { }` and `@`), which classic programs do not notice; `--strict` turns them off.
- Errors are reported with line and column instead of doing something undefined.
- About 30 million statements a second on a laptop, a 64 KB address space and no dependencies in the interpreter itself.
- An interactive machine with line editing and history, `RUN`, `LIST`, `NEW`, `LOAD`, `SAVE` and `HELP`, Ctrl-C to stop a program.
- `--baud 110` prints as slowly as a Teletype ASR-33.

## In the browser

[The web version](https://fionapreroll.github.io/vtl-rs/) runs the same interpreter as WebAssembly behind a front panel in the manner of the Altair, whose lights show the line that runs and the value stored last. Its terminal is a CRT with green, amber or white phosphor (with scan lines, glow and curved glass, each optional), the blue of the Altair, the paper of a Teletype ASR-33, or plainly modern in dark or light. The fonts are VT323, Courier Prime, Share Tech Mono, IBM Plex Mono or the system's own, and output can come at the speed of a serial line down to the ASR-33's 110 baud. It loads the examples and files, saves programs in the browser (`SAVE`, `LOAD`, `DIR`) or as files, and shares a program as a link. [web/README.md](web/README.md) says how it is built.

| Teletype ASR-33 | Amber phosphor | Altair blue |
| --- | --- | --- |
| ![Teletype theme](https://fionapreroll.github.io/vtl-rs/screenshots/teletype.png) | ![Amber theme](https://fionapreroll.github.io/vtl-rs/screenshots/amber.png) | ![Blue theme](https://fionapreroll.github.io/vtl-rs/screenshots/blue.png) |

## Install

Download the program for your system from the [releases](https://github.com/FionaPreroll/vtl-rs/releases), or build it with a current Rust:

```
cargo install --git https://github.com/FionaPreroll/vtl-rs
```

## Use

```console
$ vtl-rs
VTL-RS 0.2.0 - VERY TINY LANGUAGE
32504 BYTES FREE. TYPE HELP FOR HELP.
OK
?=2+5*2
14
OK
10 ?="HELLO WORLD!"
#=1
HELLO WORLD!
OK
BYE
```

`vtl-rs program.vtl` runs a program file: its numbered lines are stored, other lines run as if typed, and the program starts at its first line unless the file starts it itself. Lines that start with `#` (and are no `#=`) are comments, so a file can start with `#!/usr/bin/env vtl-rs`.

```
vtl-rs [OPTIONS] [FILE]
  -i, --interactive     stay in the interactive machine after FILE
  -e, --eval LINE       enter LINE before FILE (may be given more than once)
  -m, --memory BYTES    end of memory, the value of * (default 32768)
      --strict          only the VTL-2 of the Altair 680, without the VTL02 extensions
      --precedence      * and / before + and -, instead of left to right
      --seed N          seed of the random numbers in '
      --baud RATE       print no faster than a serial line at RATE baud
  -q, --quiet           no greeting
```

The interpreter is also a library without dependencies (`default-features = false`): `vtl_rs::Machine` never blocks, takes lines with `enter`, runs a given number of statements with `run` and says when it waits for input.

## Development

The checks CI runs, before every push:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For the web version, `web/build.sh` and then `node --test web/test/*.test.mjs`. After changing `Cargo.toml` or `Cargo.lock`, also `cargo audit` and `cargo deny check` (exceptions go into `deny.toml` with their reason). `./pre-commit.sh install` runs the checks before every commit. The expected output of the examples is in `tests/expected/`.

## References

- [1] [Altair 680 VTL-2 Programming Manual](https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/40-Software/VTL-2/VTL%20archive/Altair_680_VTL-2_Programming_Language_Manual.pdf)
- [2] [VTL-2 Magazine Article (Gary Shannon)](https://deramp.com/downloads/altair/software/altair_680/VTL-2%20(Very%20Tiny%20Language)/VTL-2%20Magazine%20Article.pdf)
- [3] [VTL02, VTL-2 for the 6502 (Michael T. Barry, Klaus2m5)](https://github.com/Klaus2m5/VTL02), whose source documents the behaviour of the original in detail

## License

GPL-3.0-only, see [LICENSE](LICENSE).
