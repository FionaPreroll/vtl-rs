use std::fmt;

/// Lexer Token

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Variable(char),        // Variables like '?', 'A', etc.
    Number(i64),           // Numbers like 2, 3, etc.
    Plus,                  // '+'
    Minus,                 // '-'
    Multiply,              // '*'
    Divide,                // '/'
    Remainder,             // '%'
    Assign,                // '='
    GreaterThan,           // '>'
    LessThan,              // '<'
    Colon,                 // ':' for array subscripting
    RightParen,            // ')' for closing subscripting expression
    StringLiteral(String), // String literals like "HELLO"
    RandomNumber,          // Token for random number generation (`'`)
    EndOfInput,            // End of input indicator
    Invalid(char),         // Invalid characters
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Handle variables (e.g., '?', 'A', etc.)
            Token::Variable(var) => write!(f, "{}", var),

            // Handle numbers (e.g., '2', '3')
            Token::Number(value) => write!(f, "{}", value),

            // Handle operators
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Remainder => write!(f, "%"),

            // Handle assignment
            Token::Assign => write!(f, "="),

            // Handle operations
            Token::RandomNumber => write!(f, "'"),

            // Handle comparison operators
            Token::GreaterThan => write!(f, ">"),
            Token::LessThan => write!(f, "<"),

            // Handle array subscript tokens
            Token::Colon => write!(f, ":"),
            Token::RightParen => write!(f, ")"),

            // Handle string literals (quoted strings)
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),

            // Handle end of input indicator
            Token::EndOfInput => write!(f, "End of Input"),

            // Handle invalid characters
            Token::Invalid(ch) => write!(f, "Invalid: '{}'", ch),
        }
    }
}

/// Abstract Syntax Tree Node

#[derive(Clone, Debug)]
pub enum ASTNode {
    Number(i64),
    Variable(char),
    String(String),
    NullaryOp {
        operator: Token,
    },
    UnaryOp {
        operator: Token,
        expr: Box<ASTNode>,
    },
    BinaryOp {
        left: Box<ASTNode>,
        operator: Token,
        right: Box<ASTNode>,
    },
    Assignment {
        variable: char,
        expr: Box<ASTNode>,
    },
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Display a number
            ASTNode::Number(value) => write!(f, "{}", value),

            // Display a variable (like `A`, `B`, or `?`)
            ASTNode::Variable(var) => write!(f, "{}", var),

            // Display a string literal
            ASTNode::String(value) => write!(f, "\"{}\"", value),

            // Display a unary operation (e.g., `-X`)
            ASTNode::NullaryOp { operator } => {
                write!(f, "({})", operator)
            }

            // Display a unary operation (e.g., `-X`)
            ASTNode::UnaryOp { operator, expr } => {
                write!(f, "({}{})", operator, expr)
            }

            // Display a binary operation (e.g., `X + Y`)
            ASTNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", left, operator, right,)
            }

            // Display an assignment (e.g., `A = X`)
            ASTNode::Assignment { variable, expr } => {
                write!(f, "{} = {}", variable, expr)
            }
        }
    }
}
