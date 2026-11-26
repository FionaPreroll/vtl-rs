//! Parsing VTL-2 statements and expressions into a small tree.
//!
//! A statement is `target=expression`, `target="text"` (with an optional `;`
//! after the closing quote) or a comment that starts with `)`. A target is a
//! variable (any printable character that is not a digit) or an array element
//! `:index)`. An expression is a chain of terms joined by operators, evaluated
//! strictly from left to right unless [`Syntax::precedence`] says otherwise.
//! It ends at the end of the line or at an unmatched `)`; whatever follows
//! that `)` is a comment.

use std::fmt;

/// A binary operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`, which also leaves the remainder in `%`.
    Div,
    /// `=`, 1 if both sides are equal, else 0.
    Eq,
    /// `<`, 1 if the left side is less, else 0.
    Lt,
    /// `>`, 1 if the left side is greater **or equal**, else 0 (as in VTL-2).
    Ge,
    /// `&`, bit-wise and (VTL02 extension).
    And,
    /// `|`, bit-wise or (VTL02 extension).
    Or,
    /// `^`, bit-wise exclusive or (VTL02 extension).
    Xor,
    /// `[`, "then": 0 if the left side is 0, else the right side (VTL02).
    Then,
    /// `]`, "else": the right side if the left side is 0, else 0 (VTL02).
    Else,
    /// `{`, shift left (VTL02sg extension).
    Shl,
    /// `}`, shift right (VTL02sg extension).
    Shr,
}

impl Op {
    fn from_byte(byte: u8, extended: bool) -> Option<Op> {
        let op = match byte {
            b'+' => Op::Add,
            b'-' => Op::Sub,
            b'*' => Op::Mul,
            b'/' => Op::Div,
            b'=' => Op::Eq,
            b'<' => Op::Lt,
            b'>' => Op::Ge,
            _ if !extended => return None,
            b'&' => Op::And,
            b'|' => Op::Or,
            b'^' => Op::Xor,
            b'[' => Op::Then,
            b']' => Op::Else,
            b'{' => Op::Shl,
            b'}' => Op::Shr,
            _ => return None,
        };
        Some(op)
    }

    /// The character that stands for the operator.
    pub fn symbol(self) -> char {
        match self {
            Op::Add => '+',
            Op::Sub => '-',
            Op::Mul => '*',
            Op::Div => '/',
            Op::Eq => '=',
            Op::Lt => '<',
            Op::Ge => '>',
            Op::And => '&',
            Op::Or => '|',
            Op::Xor => '^',
            Op::Then => '[',
            Op::Else => ']',
            Op::Shl => '{',
            Op::Shr => '}',
        }
    }

    /// Binding strength when [`Syntax::precedence`] is on: higher binds tighter.
    fn precedence(self) -> u8 {
        match self {
            Op::Mul | Op::Div => 6,
            Op::Add | Op::Sub => 5,
            Op::Shl | Op::Shr => 4,
            Op::Eq | Op::Lt | Op::Ge => 3,
            Op::And | Op::Xor | Op::Or => 2,
            Op::Then | Op::Else => 1,
        }
    }

    /// Applies the operator to two 16-bit unsigned values. Division stores
    /// its remainder in `remainder`; division by zero gives 65535 and leaves
    /// the dividend as the remainder, like VTL02 (the Altair 680 froze).
    #[inline]
    pub fn apply(self, a: u16, b: u16, remainder: &mut u16) -> u16 {
        match self {
            Op::Add => a.wrapping_add(b),
            Op::Sub => a.wrapping_sub(b),
            Op::Mul => a.wrapping_mul(b),
            Op::Div => {
                if b == 0 {
                    *remainder = a;
                    u16::MAX
                } else {
                    *remainder = a % b;
                    a / b
                }
            }
            Op::Eq => u16::from(a == b),
            Op::Lt => u16::from(a < b),
            Op::Ge => u16::from(a >= b),
            Op::And => a & b,
            Op::Or => a | b,
            Op::Xor => a ^ b,
            Op::Then => {
                if a == 0 {
                    0
                } else {
                    b
                }
            }
            Op::Else => {
                if a == 0 {
                    b
                } else {
                    0
                }
            }
            Op::Shl => a.checked_shl(u32::from(b)).unwrap_or(0),
            Op::Shr => a.checked_shr(u32::from(b)).unwrap_or(0),
        }
    }
}

/// An expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    /// A decimal constant (overflow wraps, as in VTL-2).
    Num(u16),
    /// A variable, read as it is stored.
    Var(u8),
    /// An array element `:index)`, two bytes at `& + 2 * index`.
    Array(Box<Expr>),
    /// `?`: a line typed by the user, evaluated as an expression.
    InputNumber,
    /// `$`: one character typed by the user.
    InputChar,
    /// `'`: a pseudo-random number.
    Random,
    /// `@`: the byte at the address in `<` (VTL02 extension).
    Peek,
    /// Two expressions joined by an operator.
    Binary(Op, Box<Expr>, Box<Expr>),
}

/// The left side of a statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// A simple or system variable.
    Var(u8),
    /// An array element `:index)`.
    Array(Expr),
}

/// One statement, the whole of a program line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stmt {
    /// An empty line or a comment.
    Nothing,
    /// `target=expression`.
    Assign(Target, Expr),
    /// `target="text"`: prints the text, then a newline unless `;` follows.
    Print {
        /// The text between the quotes.
        text: Vec<u8>,
        /// Whether a newline follows the text.
        newline: bool,
    },
}

/// The dialect a line is read in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Syntax {
    /// The VTL02 extensions: the operators `& | ^ [ ] { }` and `@` for the
    /// byte at the address in `<`. Without them only the language of the
    /// Altair 680 version is accepted.
    pub extended: bool,
    /// The usual operator precedence (`*` and `/` before `+` and `-`, then
    /// comparisons) instead of VTL-2's strict left to right.
    pub precedence: bool,
}

impl Default for Syntax {
    fn default() -> Self {
        Syntax {
            extended: true,
            precedence: false,
        }
    }
}

/// Why a line could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// The byte offset in the line where the problem is.
    pub column: usize,
    /// What is wrong, in the upper case of the era.
    pub message: &'static str,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} AT COLUMN {}", self.message, self.column + 1)
    }
}

impl std::error::Error for ParseError {}

/// Reads a statement.
pub fn parse_statement(src: &[u8], syntax: Syntax) -> Result<Stmt, ParseError> {
    Parser::new(src, syntax).statement()
}

/// Reads an expression that fills the whole input, such as a reply to `?`.
pub fn parse_expression(src: &[u8], syntax: Syntax) -> Result<Expr, ParseError> {
    let mut p = Parser::new(src, syntax);
    let expr = p.expression()?;
    p.skip_spaces();
    if p.pos < src.len() {
        return Err(p.error("UNEXPECTED CHARACTER"));
    }
    Ok(expr)
}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
    syntax: Syntax,
}

impl<'a> Parser<'a> {
    fn new(src: &'a [u8], syntax: Syntax) -> Self {
        Parser {
            src,
            pos: 0,
            syntax,
        }
    }

    fn error(&self, message: &'static str) -> ParseError {
        ParseError {
            column: self.pos,
            message,
        }
    }

    fn skip_spaces(&mut self) {
        while matches!(self.src.get(self.pos), Some(b' ' | b'\t')) {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_spaces();
        self.src.get(self.pos).copied()
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let target = match self.peek() {
            None | Some(b')') => return Ok(Stmt::Nothing),
            Some(b':') => {
                self.pos += 1;
                Target::Array(self.subexpression()?)
            }
            Some(byte) => {
                let var = self.variable(byte)?;
                self.pos += 1;
                Target::Var(var)
            }
        };
        if self.peek() != Some(b'=') {
            return Err(self.error("MISSING ="));
        }
        self.pos += 1;
        if self.peek() == Some(b'"') {
            self.pos += 1;
            let rest = &self.src[self.pos..];
            let end = rest.iter().position(|&b| b == b'"');
            let text = rest[..end.unwrap_or(rest.len())].to_vec();
            let newline = match end {
                Some(end) => {
                    self.pos += end + 1;
                    self.peek() != Some(b';')
                }
                None => true,
            };
            return Ok(Stmt::Print { text, newline });
        }
        let expr = self.expression()?;
        match self.peek() {
            // An unmatched `)` ends the statement: the rest is a comment.
            None | Some(b')') => Ok(Stmt::Assign(target, expr)),
            Some(_) => Err(self.error("BAD OPERATOR")),
        }
    }

    fn variable(&self, byte: u8) -> Result<u8, ParseError> {
        if byte.is_ascii_graphic() && !byte.is_ascii_digit() {
            Ok(byte)
        } else {
            Err(self.error("BAD VARIABLE"))
        }
    }

    /// An expression that ends at a `)`, which it consumes, or at the end of
    /// the line: an array index or a parenthesised sub-expression.
    fn subexpression(&mut self) -> Result<Expr, ParseError> {
        let expr = self.expression()?;
        match self.peek() {
            Some(b')') => {
                self.pos += 1;
                Ok(expr)
            }
            None => Ok(expr),
            Some(_) => Err(self.error("BAD OPERATOR")),
        }
    }

    /// Terms and operators up to the end of the line, an unmatched `)` or
    /// anything that is not an operator, which the caller judges.
    fn expression(&mut self) -> Result<Expr, ParseError> {
        let mut terms = vec![self.term()?];
        let mut ops = Vec::new();
        while let Some(byte) = self.peek() {
            let Some(op) = Op::from_byte(byte, self.syntax.extended) else {
                break;
            };
            self.pos += 1;
            ops.push(op);
            terms.push(self.term()?);
        }
        let mut terms = terms.into_iter();
        let first = terms.next().unwrap_or(Expr::Num(0));
        if self.syntax.precedence {
            let rest: Vec<(Op, Expr)> = ops.into_iter().zip(terms).collect();
            let mut rest = rest.into_iter().peekable();
            Ok(climb(first, &mut rest, 0))
        } else {
            Ok(ops.into_iter().zip(terms).fold(first, |left, (op, right)| {
                Expr::Binary(op, Box::new(left), Box::new(right))
            }))
        }
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let Some(byte) = self.peek() else {
            return Err(self.error("MISSING VALUE"));
        };
        if byte.is_ascii_digit() {
            let mut value: u16 = 0;
            while let Some(&d) = self.src.get(self.pos).filter(|b| b.is_ascii_digit()) {
                value = value.wrapping_mul(10).wrapping_add(u16::from(d - b'0'));
                self.pos += 1;
            }
            return Ok(Expr::Num(value));
        }
        if byte == b')' {
            return Err(self.error("MISSING VALUE"));
        }
        self.pos += 1;
        Ok(match byte {
            b'(' => self.subexpression()?,
            b':' => Expr::Array(Box::new(self.subexpression()?)),
            b'?' => Expr::InputNumber,
            b'$' => Expr::InputChar,
            b'\'' => Expr::Random,
            b'@' if self.syntax.extended => Expr::Peek,
            _ => {
                self.pos -= 1;
                let var = self.variable(byte)?;
                self.pos += 1;
                Expr::Var(var)
            }
        })
    }
}

/// Precedence climbing over the terms after `left`.
fn climb(
    mut left: Expr,
    rest: &mut std::iter::Peekable<std::vec::IntoIter<(Op, Expr)>>,
    min: u8,
) -> Expr {
    while let Some(&(op, _)) = rest.peek() {
        if op.precedence() < min {
            break;
        }
        let Some((op, mut right)) = rest.next() else {
            break;
        };
        while let Some(&(next, _)) = rest.peek() {
            if next.precedence() <= op.precedence() {
                break;
            }
            right = climb(right, rest, next.precedence());
        }
        left = Expr::Binary(op, Box::new(left), Box::new(right));
    }
    left
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stmt(src: &str) -> Result<Stmt, ParseError> {
        parse_statement(src.as_bytes(), Syntax::default())
    }

    fn bin(op: Op, a: Expr, b: Expr) -> Expr {
        Expr::Binary(op, Box::new(a), Box::new(b))
    }

    #[test]
    fn left_to_right() {
        let Stmt::Assign(Target::Var(b'A'), e) = stmt("A=2+5*2").unwrap() else {
            panic!()
        };
        assert_eq!(
            e,
            bin(
                Op::Mul,
                bin(Op::Add, Expr::Num(2), Expr::Num(5)),
                Expr::Num(2)
            )
        );
    }

    #[test]
    fn precedence() {
        let syntax = Syntax {
            extended: true,
            precedence: true,
        };
        let Stmt::Assign(_, e) = parse_statement(b"A=2+5*2-1", syntax).unwrap() else {
            panic!()
        };
        assert_eq!(
            e,
            bin(
                Op::Sub,
                bin(
                    Op::Add,
                    Expr::Num(2),
                    bin(Op::Mul, Expr::Num(5), Expr::Num(2))
                ),
                Expr::Num(1)
            )
        );
    }

    #[test]
    fn strings_and_comments() {
        assert_eq!(
            stmt("?=\"HELLO\";").unwrap(),
            Stmt::Print {
                text: b"HELLO".to_vec(),
                newline: false
            }
        );
        assert_eq!(
            stmt("?=\"OPEN").unwrap(),
            Stmt::Print {
                text: b"OPEN".to_vec(),
                newline: true
            }
        );
        assert_eq!(stmt(") A COMMENT").unwrap(), Stmt::Nothing);
        assert_eq!(stmt("   ").unwrap(), Stmt::Nothing);
        assert!(matches!(stmt("A=1) set A"), Ok(Stmt::Assign(..))));
    }

    #[test]
    fn spaces_are_ignored() {
        assert_eq!(stmt("# = A < 10 * 1010 ) loop"), stmt("#=A<10*1010"));
    }

    #[test]
    fn arrays_and_subexpressions() {
        let Stmt::Assign(Target::Array(i), e) = stmt(":I+1)=(1+2)*:I)").unwrap() else {
            panic!()
        };
        assert_eq!(i, bin(Op::Add, Expr::Var(b'I'), Expr::Num(1)));
        assert_eq!(
            e,
            bin(
                Op::Mul,
                bin(Op::Add, Expr::Num(1), Expr::Num(2)),
                Expr::Array(Box::new(Expr::Var(b'I')))
            )
        );
    }

    #[test]
    fn special_terms() {
        let Stmt::Assign(_, e) = stmt("A=?+$+'+@+%").unwrap() else {
            panic!()
        };
        let mut found = Vec::new();
        let mut e = &e;
        while let Expr::Binary(_, l, r) = e {
            found.push((**r).clone());
            e = l;
        }
        found.push(e.clone());
        found.reverse();
        assert_eq!(
            found,
            [
                Expr::InputNumber,
                Expr::InputChar,
                Expr::Random,
                Expr::Peek,
                Expr::Var(b'%')
            ]
        );
    }

    #[test]
    fn strict_dialect() {
        let strict = Syntax {
            extended: false,
            precedence: false,
        };
        assert!(parse_statement(b"A=B&1", strict).is_err());
        assert_eq!(
            parse_statement(b"A=@", strict).unwrap(),
            Stmt::Assign(Target::Var(b'A'), Expr::Var(b'@'))
        );
    }

    #[test]
    fn errors() {
        assert_eq!(stmt("A").unwrap_err().message, "MISSING =");
        assert_eq!(stmt("A=").unwrap_err().message, "MISSING VALUE");
        assert_eq!(stmt("A=1 2").unwrap_err().message, "BAD OPERATOR");
        assert_eq!(stmt("A=1+").unwrap_err().message, "MISSING VALUE");
        assert_eq!(stmt("A=1;").unwrap_err().column, 3);
    }

    #[test]
    fn numbers_wrap() {
        assert_eq!(
            stmt("A=65537").unwrap(),
            Stmt::Assign(Target::Var(b'A'), Expr::Num(1))
        );
    }

    #[test]
    fn operators() {
        let mut r = 0;
        assert_eq!(Op::Div.apply(7, 2, &mut r), 3);
        assert_eq!(r, 1);
        assert_eq!(Op::Div.apply(7, 0, &mut r), 65535);
        assert_eq!(r, 7);
        assert_eq!(Op::Sub.apply(0, 1, &mut r), 65535);
        assert_eq!(Op::Ge.apply(3, 3, &mut r), 1);
        assert_eq!(Op::Lt.apply(3, 3, &mut r), 0);
        assert_eq!(Op::Then.apply(0, 9, &mut r), 0);
        assert_eq!(Op::Then.apply(1, 9, &mut r), 9);
        assert_eq!(Op::Else.apply(0, 9, &mut r), 9);
        assert_eq!(Op::Else.apply(1, 9, &mut r), 0);
        assert_eq!(Op::Shl.apply(1, 16, &mut r), 0);
        assert_eq!(Op::Shr.apply(256, 4, &mut r), 16);
        assert_eq!(Op::Mul.apply(300, 300, &mut r), 24464);
    }
}
