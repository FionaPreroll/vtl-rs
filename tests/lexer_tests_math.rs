#[cfg(test)]
mod lexer_tests {
    use vtl_rs::lexer::lex;
    use vtl_rs::types::Token;

    #[test]
    fn test_lexer_large_number_overflow() {
        let result = lex("10101010100000000000000000000000000022929299292");
        assert!(result.is_err(), "Expected an overflow error");
        assert_eq!(result.unwrap_err(), "Overflow error");
    }

    #[test]
    fn test_lexer_remainder_unary_operator() {
        let tokens = lex("?=%").unwrap();
        assert_eq!(tokens.len(), 4); // ? = % EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('?'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Remainder);
        assert_eq!(tokens[3], Token::EndOfInput);
    }

    #[test]
    fn test_lexer_remainder_binary_operator() {
        // NOTE anachronistic
        let tokens = lex("?=7%3").unwrap();
        assert_eq!(tokens.len(), 6); // ? = 7 % 3 EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('?'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Number(7));
        assert_eq!(tokens[3], Token::Remainder);
        assert_eq!(tokens[4], Token::Number(3));
        assert_eq!(tokens[5], Token::EndOfInput);
    }

    #[test]
    fn test_lexer_division_operation() {
        let tokens = lex("?=20/5").unwrap();
        assert_eq!(tokens.len(), 6); // ? = 20 / 5 EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('?'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Number(20));
        assert_eq!(tokens[3], Token::Divide);
        assert_eq!(tokens[4], Token::Number(5));
        assert_eq!(tokens[5], Token::EndOfInput);
    }

    #[test]
    fn test_lexer_greater_than_operator() {
        let tokens = lex("?=5>3").unwrap();
        assert_eq!(tokens.len(), 6); // ? = 5 > 3 EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('?'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Number(5));
        assert_eq!(tokens[3], Token::GreaterThan);
        assert_eq!(tokens[4], Token::Number(3));
        assert_eq!(tokens[5], Token::EndOfInput);
    }

    #[test]
    fn test_lexer_less_than_operator() {
        let tokens = lex("?=3<5").unwrap();
        assert_eq!(tokens.len(), 6); // ? = 3 < 5 EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('?'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Number(3));
        assert_eq!(tokens[3], Token::LessThan);
        assert_eq!(tokens[4], Token::Number(5));
        assert_eq!(tokens[5], Token::EndOfInput);
    }
}
