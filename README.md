# vtl-rs

*vtl-rs* is a Rust implementation of the *Very Tiny Language (VTL-2)*, a simple, high-level language originally designed for the Altair microcomputers in the 1970s. This project aims to be a spiritual successor with simplicity and hackability in mind, but it can of course not rival the compactness of the original.

## What is VTL-2?

![Introduction excerpt from the VTL-2 manual. Source: [1]](./doc/vtl2intro.png)

VTL-2, or **Very Tiny Language**, was a minimalist programming language developed to run on microcomputers like the Altair 680b, often with as little as 1-2 KB of memory. Despite its size, VTL-2 provided essential programming constructs, including:
- Basic arithmetic operations (`+`, `-`, `*`, `/`, `%`)
- Variables and simple assignments
- Simple control structures (IF statements and GOTO, but originally in a weird way)
- Support for integer division and remainder operations

## What vtl-rs Currently Does

The current implementation of **vtl-rs** in Rust is an early-stage interpreter with several core features already working:

### 1. **CPU and Memory Structures:**
   - **CPU**: Manages internal registers, handles variables.
   - **Memory**: Allows for reading and writing at specific memory addresses (Work In Progress).

### 2. **Parser and Lexer:**
   - The **Lexer** converts user input (expressions) into tokens (e.g., numbers, variables, and operators).
   - The **Parser** converts tokens into an **Abstract Syntax Tree (AST)**, which can then be evaluated by the CPU.

### 3. **Evaluator:**
   - Supports evaluation of arithmetic expressions, including addition (`+`), subtraction (`-`), multiplication (`*`), division (`/`), and modulo (`%`).
   - **Variable handling**: You can assign values to variables like `?`, and they will store intermediate results for use in further calculations.
   - **String Handling**: The evaluator supports string output using the `?=` syntax. For example, `?="HELLO"` will print *HELLO* to the output.

### 4. **REPL (Read-Eval-Print Loop):**
   - The interpreter runs in a REPL loop, allowing you to enter expressions interactively. The REPL supports expression evaluation and variable assignment.
   - It also has a shell history file, which is `~/.vtl-rs_history` on Linux.

## Example Usage

Here's a simple example of using **vtl-rs** in its current state ("OK" lines skipped for clarity):

```console
VTL-RS Interpreter
VTL> ?=2+2
4
VTL> ?="HELLO WORLD!"
HELLO WORLD!
VTL> A=123
OK (123)
VTL> B=456
OK (456)
VTL> ?=A+B
579
VTL> ?=7/3
2
VTL> ?=%
1
VTL> ?=2+5*2
14
```

### Key Points from the Output:

1. **Basic Math Operations**:
   - `?=2+2` gives `4`, which is expected in both left-to-right and operator precedence scenarios.

2. **String Output**:
   - `?="HELLO WORLD!"` correctly outputs the string as is.

3. **Variables**:
   - `A=123` and `B=456` store values in variables `A` and `B`, respectively.
   - `?=A+B` evaluates `123 + 456` to give `579`.

4. **Division and Remainder**:
   - `?=7/3` gives `2` because it performs integer division.
   - `?=%` gives `1`, which is the remainder of the previous division (`7 % 3`).

5. **Strict Left-to-Right Evaluation**:
   - The expression `?=2+5*2` produces `14`, following strict left-to-right evaluation, as `(2 + 5) * 2` rather than `2 + (5 * 2)`.

## Build/Run

You can easily build the REPL (a library build is possible, too!) on Linux/macOS/Windows or wherever Rust is running.
You will need current Rust and Cargo installed, then:

```
# This will give you the REPL prompt shown in the example:
cargo run

# <or>

# to build vtl-rs:
cargo build
# the executable will be somewhere inside the target/ directory after that
```

## References

For those interested in the history of VTL-2, or the original manuals and documents, here are some useful references:
- [1] [Altair 680 VTL-2 Programming Manual](https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/40-Software/VTL-2/VTL%20archive/Altair_680_VTL-2_Programming_Language_Manual.pdf)
- [2] [VTL-2 Magazine Article (Gary Shannon)](https://deramp.com/downloads/altair/software/altair_680/VTL-2%20(Very%20Tiny%20Language)/VTL-2%20Magazine%20Article.pdf)
- [3] [VTL-2 Source Code (Klaus2m5)](https://github.com/Klaus2m5/VTL02)
- [4] [Altair Information (Deramp)](https://deramp.com/altair.html)
