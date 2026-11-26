#[cfg(test)]
mod parser_tests {
    use vtl_rs::lexer::lex;
    use vtl_rs::parser::Parser;
    use vtl_rs::types::ASTNode;

    #[test]
    fn test_parser_basic_assignment() {
        let tokens = lex("?=2+2").unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        // Check if the AST root is an assignmen
        #[allow(unused_variables)]
        if let ASTNode::Assignment { variable, expr } = ast {
            assert_eq!(variable, '?');
        } else {
            panic!("Expected an assignment node");
        }
    }

    #[test]
    fn test_parser_empty_input() {
        let tokens = lex("").unwrap();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err(), "Expected an error on empty input");
    }

    #[test]
    fn test_parser_invalid_syntax() {
        let tokens = lex("?=2++2").unwrap();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err(), "Expected a syntax error");
    }

    #[test]
    fn test_parser_string_assignment_to_variable() {
        let tokens = lex("A=\"HELLO\"").unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        // Check if the AST root is an assignment node with a string literal
        if let ASTNode::Assignment { variable, expr } = ast {
            assert_eq!(variable, 'A');
            if let ASTNode::String(value) = *expr {
                assert_eq!(value, "HELLO");
            } else {
                panic!("Expected a string literal in the assignment");
            }
        } else {
            panic!("Expected an assignment node");
        }
    }

    #[test]
    fn test_parser_string_output() {
        let tokens = lex("?=\"HELLO\"").unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        // Check if the AST root is an assignment with a string literal
        if let ASTNode::Assignment { variable, expr } = ast {
            assert_eq!(variable, '?');
            if let ASTNode::String(value) = *expr {
                assert_eq!(value, "HELLO");
            } else {
                panic!("Expected a string literal in the output assignment");
            }
        } else {
            panic!("Expected an assignment node");
        }
    }

    #[test]
    fn test_parser_numeric_output() {
        let tokens = lex("?=123").unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        // Check if the AST root is an assignment with a numeric literal
        if let ASTNode::Assignment { variable, expr } = ast {
            assert_eq!(variable, '?');
            if let ASTNode::Number(value) = *expr {
                assert_eq!(value, 123);
            } else {
                panic!("Expected a numeric literal in the output assignment");
            }
        } else {
            panic!("Expected an assignment node");
        }
    }
}
