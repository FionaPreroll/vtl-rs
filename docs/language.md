# The VTL-2 language

VTL-2, the *Very Tiny Language*, was written in 1977 by Frank McCoy and Gary Shannon for the Altair 680, where the whole interpreter fit into 768 bytes. vtl-rs follows the behaviour documented for that version, as VTL02 for the 6502 duplicates it ([Klaus2m5/VTL02](https://github.com/Klaus2m5/VTL02)), and adds the extensions of VTL02 that do not change what a classic program does. `--strict` (or *Strict VTL-2* on the web page) switches the extensions off.

## Lines and the prompt

The machine prints `OK` when it is ready. A line that starts with a number is stored as program line with that number (1 to 65535), replacing a line with the same number. A number alone deletes its line, and `0` lists the program. Any other line runs at once.

```
OK
10 ?="HELLO"
20 #=10
0
10 ?="HELLO"
20 #=10
OK
#=1
```

`#=1` runs the program from its first line. It ends after its last line or when it jumps past the end. Ctrl-C (or the STOP switch on the web page) stops it.

## Statements

Every line holds one statement of the form *variable* `=` *expression*. What the statement does depends on the variable on the left:

| Statement | Effect |
| --- | --- |
| `A=expr` | stores the value in `A` |
| `?=expr` | prints the value as an unsigned decimal number, without a newline |
| `?="text"` | prints the text and a newline; `?="text";` keeps the line open, `?=""` prints only a newline |
| `$=expr` | prints the character with that code (13 is a newline) |
| `#=expr` | jumps to the line with that number, or the next one after it; `#=0` does nothing |
| `:expr)=expr` | stores the value in an array element |
| `&=expr` | moves the end of the program; `&=0` clears the program |
| `'=expr` | seeds the random numbers |
| `)` text | a comment; so is everything after an unmatched `)`, e.g. `A=1) start` |

Spaces are ignored outside of strings: `# = A < 10 * 100` is `#=A<10*100`.

### Conditions

There is no `IF`. Comparisons give 1 or 0, and `#=0` does nothing, so `#=A<10*100` jumps to line 100 if `A` is less than 10, and goes on with the next line otherwise.

### Subroutines

Every jump leaves the number of the line it was made from, plus one, in `!`. A jump to a missing line goes on at the next line after it, so `#=!` returns to the line after the jump:

```
10 #=1000
20 ?="BACK"
30 #=9999
1000 ?="IN THE SUBROUTINE"
1010 #=!
```

## Expressions

Numbers are 16 bits without a sign, 0 to 65535: `0-1` is 65535 and `300*300` is 24464. Expressions are evaluated strictly from left to right, so `2+5*2` is 14. Parentheses group: `2+(5*2)` is 12. `--precedence` (or *Operator precedence* on the web page) evaluates `*` and `/` before `+` and `-` instead.

| Operator | Meaning |
| --- | --- |
| `+ - * /` | arithmetic; `/` also leaves the remainder in `%`; a division by 0 gives 65535 |
| `=` | 1 if equal, else 0 |
| `<` | 1 if less, else 0 |
| `>` | 1 if greater **or equal**, else 0 |
| `& \| ^` | bit-wise and, or, exclusive or (VTL02) |
| `[` | "then": `A[B` is 0 if `A` is 0, else `B` (VTL02) |
| `]` | "else": `A]B` is `B` if `A` is 0, else 0 (VTL02) |
| `{ }` | shift left and right: `A{2` is `A*4` (VTL02sg) |

A term is a decimal number, a variable, a parenthesised expression or one of these:

| Term | Value |
| --- | --- |
| `?` | reads a line from the user and evaluates it as an expression |
| `$` | reads one character from the user (the end of a line is 13) |
| `'` | a random number from 0 to 65535 |
| `:expr)` | the array element with that index |
| `%` | the remainder of the last division |
| `#` | the number of the line that runs |
| `!` | the return line of the last jump |
| `&` | the end of the program, where the array begins |
| `*` | the end of memory |
| `@` | the byte at the address in `<` (VTL02) |

## Variables

Every printable character that has no other meaning is a variable: `A` to `Z`, `a` to `z` (distinct from the capitals) and most signs. Variables start at 0.

## Memory and arrays

The program is stored in memory from address 264 on, each line as its number (two bytes), its length (one byte), its text and a zero byte. `&` points behind the last line, `*` at the end of memory (32768 unless set with `--memory`), so `*-&` is the free memory. The array `:I)` holds two bytes per element from `&` on: element `I` is at `&+2*I`. A program that grows moves `&` and with it the array. An index can wrap around into the program, which then changes, as it did in the original.

With the VTL02 extensions, `<=address` points at a byte and `@` reads or writes it: `<=&` and `@=65` writes 65 into the first byte of the array.

## Commands

Besides `0`, a few words work at the prompt. No VTL-2 statement looks like them.

| Command | Effect |
| --- | --- |
| `RUN` | runs the program, like `#=1` |
| `LIST`, `LIST 100`, `LIST 100-200` | lists the program, or a part of it |
| `NEW` | clears the program |
| `LOAD name`, `SAVE name` | loads or saves the program (a file, or in the browser's storage on the web page) |
| `HELP` | a short summary |
| `BYE` | leaves the command line program |

## Differences from the original

- Errors are reported (`ERROR IN LINE 20: MISSING = AT COLUMN 2`) where the original went on and did something undefined. A line with an error is still stored, so that a program can be fixed line by line.
- A division by zero gives 65535, with the dividend as remainder, as in VTL02; the Altair 680 froze.
- Lower-case letters are variables of their own.
- An expression holds at most 512 terms and parentheses (`EXPRESSION TOO LONG`), far more than a stored line of 251 characters can. A reply to `?` that holds a `?` itself reads the next line, but only four levels deep; deeper, `?` is 0.
- On a terminal the command line program leaves out escape sequences and the control characters other than bell, backspace, tab, line feed, form feed and carriage return, so `$=27` prints nothing; `--raw` passes them on.
- The VTL02 extensions are on unless `--strict` is given.
