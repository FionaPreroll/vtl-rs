#[cfg(test)]
mod lexer_tests {
    use vtl_rs::lexer::lex;
    use vtl_rs::types::Token;

    #[test]
    fn test_lexer_colon_subscript_operator() {
        let tokens = lex(":2)=5").unwrap();
        assert_eq!(tokens.len(), 6); // : 2 ) = 5 EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Colon);
        assert_eq!(tokens[1], Token::Number(2));
        assert_eq!(tokens[2], Token::RightParen);
        assert_eq!(tokens[3], Token::Assign);
        assert_eq!(tokens[4], Token::Number(5));
        assert_eq!(tokens[5], Token::EndOfInput);
    }

    #[test]
    fn test_lexer_colon_with_variable_subscript() {
        let tokens = lex(":A)=").unwrap();
        assert_eq!(tokens.len(), 5); // : A ) = EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Colon);
        assert_eq!(tokens[1], Token::Variable('A'));
        assert_eq!(tokens[2], Token::RightParen);
        assert_eq!(tokens[3], Token::Assign);
        assert_eq!(tokens[4], Token::EndOfInput);
    }
}
