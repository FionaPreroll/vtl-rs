#[cfg(test)]
mod lexer_tests {
    use vtl_rs::lexer::lex;
    use vtl_rs::types::Token;

    #[test]
    fn test_system_variable_handling() {
        let tokens = lex("#=100").unwrap();
        assert_eq!(tokens.len(), 4); // # = 100 EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('#'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Number(100));
        assert_eq!(tokens[3], Token::EndOfInput);
    }

    #[test]
    fn test_system_variable_multiple_cases() {
        let tokens = lex("&=50!").unwrap();
        assert_eq!(tokens.len(), 5); // & = 50 ! EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('&'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Number(50));
        assert_eq!(tokens[3], Token::Variable('!'));
        assert_eq!(tokens[4], Token::EndOfInput);
    }

    #[test]
    fn test_mixed_system_and_normal_variables() {
        let tokens = lex("A=10#=20").unwrap();
        assert_eq!(tokens.len(), 7); // A = 10 # = 20 EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('A'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Number(10));
        assert_eq!(tokens[3], Token::Variable('#'));
        assert_eq!(tokens[4], Token::Assign);
        assert_eq!(tokens[5], Token::Number(20));
        assert_eq!(tokens[6], Token::EndOfInput);
    }

    #[test]
    fn test_invalid_character() {
        let tokens = lex("A=@").unwrap();
        assert_eq!(tokens.len(), 4); // A = Invalid('@') EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('A'));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Invalid('@'));
        assert_eq!(tokens[3], Token::EndOfInput);
    }

    #[test]
    fn test_valid_variable_but_no_assignment() {
        let tokens = lex("B").unwrap();
        assert_eq!(tokens.len(), 2); // B EndOfInput

        // Check token types
        assert_eq!(tokens[0], Token::Variable('B'));
        assert_eq!(tokens[1], Token::EndOfInput);
    }

    #[test]
    fn test_random_number_token() {
        let input = "'"; // Random number token
        let tokens = lex(input).expect("Failed to lex input");

        assert_eq!(tokens.len(), 2); // One token + EndOfInput token
        assert_eq!(tokens[0], Token::RandomNumber);
        assert_eq!(tokens[1], Token::EndOfInput);
    }
}
