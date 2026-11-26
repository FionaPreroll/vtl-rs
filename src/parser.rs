use crate::types::{ASTNode, Token};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    // Peek at the current token
    pub fn peek(&self) -> Result<&Token, String> {
        self.tokens
            .get(self.current)
            .ok_or_else(|| "Unexpected end of input (#1 peek)".to_string())
    }

    // Consume and return the current token
    pub fn consume(&mut self) -> Result<&Token, String> {
        if self.current < self.tokens.len() {
            self.current += 1;
            Ok(&self.tokens[self.current - 1])
        } else {
            Err("Unexpected end of input (#2 consume)".to_string())
        }
    }

    // Parse a number token
    pub fn parse_number(&mut self) -> Result<ASTNode, String> {
        if let Token::Number(value) = self.consume()? {
            Ok(ASTNode::Number(*value))
        } else {
            Err("Expected a number (#3 parse_number)".to_string())
        }
    }

    // Parse a variable token
    pub fn parse_variable(&mut self) -> Result<ASTNode, String> {
        if let Token::Variable(var) = self.consume()? {
            match var {
                // Handle special system variable cases
                '?' => {
                    // Currently we treat `?` as a regular variable, but eventually,
                    // it can also be used for user input (e.g., `A = ?`)
                    Ok(ASTNode::Variable('?'))
                }
                // TODO Handle other system variables like `#`, `!`, `$`, etc.
                '#' | '!' | '$' | '&' => Ok(ASTNode::Variable(*var)),
                // Default case for regular variables (A-Z)
                _ => Ok(ASTNode::Variable(*var)),
            }
        } else if let Ok(Token::RandomNumber) = self.peek().cloned() {
            self.consume()?; // Consume the random number token
            Ok(ASTNode::NullaryOp {
                operator: Token::RandomNumber,
            })
        } else {
            Err("Expected a variable (#4 parse_variable)".to_string())
        }
    }

    // Parse an expression (handles '+' and '-' operators)
    pub fn parse_expression(&mut self) -> Result<ASTNode, String> {
        let mut node = self.parse_term()?;

        while matches!(self.peek(), Ok(Token::Plus) | Ok(Token::Minus)) {
            let operator = self.consume()?.clone();
            let right = self.parse_term()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    // Parse a term (handles '*' and '/' operators)
    pub fn parse_term(&mut self) -> Result<ASTNode, String> {
        let mut node = self.parse_factor()?;

        while matches!(
            self.peek(),
            Ok(Token::Multiply) | Ok(Token::Divide) | Ok(Token::Remainder)
        ) {
            let operator = self.consume()?.clone();
            let right = self.parse_factor()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    pub fn parse_string_literal(&mut self) -> Result<ASTNode, String> {
        if let Ok(Token::StringLiteral(value)) = self.peek().cloned() {
            self.consume()?; // Consume the string literal token
            return Ok(ASTNode::String(value.clone()));
        }

        Err("Expected a string literal".to_string())
    }

    // Parse a factor (number, variable, or unary operator)
    pub fn parse_factor(&mut self) -> Result<ASTNode, String> {
        match self.peek()? {
            // Handle numbers
            Token::Number(_) => self.parse_number(),

            // Handle variables
            Token::Variable(_) => self.parse_variable(),

            // Handle string literals
            Token::StringLiteral(_) => self.parse_string_literal(),

            // Handle unary minus
            Token::Minus => {
                self.consume()?; // Consume the '-' token
                let expr = self.parse_factor()?; // Parse the expression after '-'
                Ok(ASTNode::UnaryOp {
                    operator: Token::Minus,
                    expr: Box::new(expr),
                })
            }

            // Handle remainder (unary operator in this context)
            Token::Remainder => {
                self.consume()?; // Consume the '%' token
                Ok(ASTNode::Variable('%'))
            }

            // Handle random number (nullary operator)
            Token::RandomNumber => {
                self.consume()?; // Consume the random number token
                Ok(ASTNode::NullaryOp {
                    operator: Token::RandomNumber,
                })
            }

            // Unexpected token handling
            _ => Err(format!(
                "Unexpected token (#5 parse_factor): {:?}",
                self.peek()?
            )),
        }
    }

    // Main entry point for parsing, including assignments
    pub fn parse(&mut self) -> Result<ASTNode, String> {
        // First, check if it's a variable assignment
        if let Ok(Token::Variable(var)) = self.peek() {
            let var = *var; // Copy the variable first to avoid holding the immutable borrow

            self.consume()?; // Consume the variable token

            // Check for the '=' sign
            if let Ok(Token::Assign) = self.peek() {
                self.consume()?; // Consume the '=' sign
                let expr = self.parse_expression()?; // Parse the expression on the right-hand side

                return Ok(ASTNode::Assignment {
                    variable: var, // Use the copied variable
                    expr: Box::new(expr),
                });
            }
        }

        // If not an assignment, just parse a normal expression
        self.parse_expression()
    }
}
