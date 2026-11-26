use crate::types::*;

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                chars.next(); // skip whitespace
            }
            '?' => {
                tokens.push(Token::Variable('?'));
                chars.next();
            }
            '%' => {
                tokens.push(Token::Remainder); // '%' is used for remainder/modulus
                chars.next();
            }
            '$' | '#' | '!' | '&' => {
                tokens.push(Token::Variable(ch)); // system variables
                chars.next();
            }
            // Handle random number symbol
            '\'' => {
                tokens.push(Token::RandomNumber);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon); // for array subscripting
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen); // close subscript expression
                chars.next();
            }
            '>' => {
                tokens.push(Token::GreaterThan);
                chars.next();
            }
            '<' => {
                tokens.push(Token::LessThan);
                chars.next();
            }
            '=' => {
                tokens.push(Token::Assign);
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Multiply);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Divide);
                chars.next();
            }
            '0'..='9' => {
                // Handle numbers
                let mut num: i64 = 0;
                while let Some(&digit) = chars.peek() {
                    if digit.is_numeric() {
                        // Use checked_mul and checked_add to prevent overflow
                        num = num
                            .checked_mul(10)
                            .and_then(|n| n.checked_add(digit as i64 - '0' as i64))
                            .ok_or_else(|| "Overflow error".to_string())?;
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num));
            }
            'A'..='Z' => {
                // Handle variable names A-Z
                tokens.push(Token::Variable(ch));
                chars.next();
            }
            '"' => {
                // Handle string literals
                chars.next(); // Skip the opening quote
                let mut string_literal = String::new();
                let mut found_closing_quote = false;

                while let Some(&next_char) = chars.peek() {
                    if next_char == '"' {
                        chars.next(); // Skip the closing quote
                        tokens.push(Token::StringLiteral(string_literal));
                        found_closing_quote = true;
                        break;
                    }
                    string_literal.push(next_char);
                    chars.next();
                }

                // If we reached the end of input without finding a closing quote
                if !found_closing_quote {
                    return Err("Unclosed string literal".to_string());
                }
            }
            _ => {
                tokens.push(Token::Invalid(ch)); // Invalid tokens
                chars.next();
            }
        }
    }

    tokens.push(Token::EndOfInput); // Mark the end of input
    Ok(tokens)
}
