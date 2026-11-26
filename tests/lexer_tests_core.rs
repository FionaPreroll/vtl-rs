#[cfg(test)]
mod lexer_tests {
    use vtl_rs::lexer::lex;
    use vtl_rs::types::Token;

    #[test]
    fn test_lexer_basic_arithmetic() {
        let tokens = lex("?=2+2").unwrap();
        assert_eq!(tokens.len(), 6); // ? = 2 + 2 EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('?'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Number(2));
        assert_eq!(tokens[3], Token::Plus);
        assert_eq!(tokens[4], Token::Number(2));
        assert_eq!(tokens[5], Token::EndOfInput);
    }

    #[test]
    fn test_lexer_empty_input() {
        let tokens = lex("").unwrap();
        assert_eq!(tokens.len(), 1); // Should return only EndOfInput
        assert_eq!(tokens[0], Token::EndOfInput);
    }

    #[test]
    fn test_lexer_garbage_input() {
        let tokens = lex("abcd").unwrap();
        assert_eq!(tokens.len(), 5); // Invalid token for each character and EndOfInput

        // Check for invalid tokens followed by EndOfInput
        assert!(matches!(tokens[0], Token::Invalid('a')));
        assert!(matches!(tokens[1], Token::Invalid('b')));
        assert!(matches!(tokens[2], Token::Invalid('c')));
        assert!(matches!(tokens[3], Token::Invalid('d')));
        assert_eq!(tokens[4], Token::EndOfInput);
    }
}
