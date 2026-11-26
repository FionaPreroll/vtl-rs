#[cfg(test)]
mod lexer_tests {
    use vtl_rs::lexer::lex;
    use vtl_rs::types::Token;

    #[test]
    fn test_lexer_string_literal() {
        let tokens = lex("?=\"HELLO\"").unwrap();
        assert_eq!(tokens.len(), 4); // ? = "HELLO" EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('?'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::StringLiteral(String::from("HELLO")));
        assert_eq!(tokens[3], Token::EndOfInput);
    }

    #[test]
    fn test_lexer_unclosed_string_literal() {
        let result = lex("?=\"HELLO");
        assert!(
            result.is_err(),
            "Expected error for unclosed string literal"
        );
        assert_eq!(result.unwrap_err(), "Unclosed string literal");
    }
}
