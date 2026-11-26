use crate::cpu::CPU;
use crate::types::ASTNode;
use crate::types::Token;

impl CPU {
    pub fn evaluate(&mut self, node: &ASTNode) -> Result<i64, String> {
        match node {
            // Handle number nodes
            ASTNode::Number(value) => Ok(*value),

            // Handle variable retrieval
            ASTNode::Variable(var) => {
                match *var {
                    '%' => Ok(self.remainder), // Special case for `%`, return cached remainder
                    _ => Ok(self.get_variable(*var).unwrap_or(0)),
                }
            }

            // Handle binary operations
            ASTNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                if !self.pemdas {
                    // Evaluate expressions in strict left-to-right order as VTL-2 does

                    // Collect all operands and operators in order
                    let mut operands = Vec::new();
                    let mut operators = Vec::new();

                    // Helper function to flatten the AST
                    fn flatten_ast(
                        node: &ASTNode,
                        operands: &mut Vec<ASTNode>,
                        operators: &mut Vec<Token>,
                    ) {
                        match node {
                            ASTNode::BinaryOp {
                                left,
                                operator,
                                right,
                            } => {
                                flatten_ast(left, operands, operators);
                                operators.push(operator.clone());
                                flatten_ast(right, operands, operators);
                            }
                            _ => {
                                operands.push(node.clone());
                            }
                        }
                    }

                    flatten_ast(node, &mut operands, &mut operators);

                    // Initialize the result with the first operand
                    let mut result = self.evaluate(&operands[0])?;

                    // Iterate over the operators and apply them in order
                    for (i, op) in operators.iter().enumerate() {
                        let right_val = self.evaluate(&operands[i + 1])?;

                        result = match op {
                            Token::Plus => result + right_val,
                            Token::Minus => result - right_val,
                            Token::Multiply => result * right_val,
                            Token::Divide => {
                                if right_val == 0 {
                                    return Err("Evaluation Error (Divide LTR): Division by zero"
                                        .to_string());
                                }
                                self.remainder = result % right_val;
                                result / right_val
                            }
                            Token::Remainder => {
                                if right_val == 0 {
                                    return Err(
                                        "Evaluation Error (Remainder LTR): Division by zero"
                                            .to_string(),
                                    );
                                }
                                result % right_val
                            }
                            _ => return Err("Evaluation Error (LTR): Unknown operator".to_string()),
                        };
                    }

                    Ok(result)
                } else {
                    // Evaluate expressions in PEMDAS (normal) order
                    let left_val = self.evaluate(left)?;
                    let right_val = self.evaluate(right)?;
                    match operator {
                        Token::Plus => Ok(left_val + right_val),
                        Token::Minus => Ok(left_val - right_val),
                        Token::Multiply => Ok(left_val * right_val),
                        Token::Divide => {
                            if right_val == 0 {
                                Err("Evaluation Error (Divide PEMDAS): Division by zero"
                                    .to_string())
                            } else {
                                self.remainder = left_val % right_val;
                                Ok(left_val / right_val)
                            }
                        }
                        Token::Remainder => {
                            if right_val == 0 {
                                Err("Evaluation Error (Remainder PEMDAS): Division by zero"
                                    .to_string())
                            } else {
                                Ok(left_val % right_val)
                            }
                        }
                        _ => {
                            Err("Evaluation Error (PEMDAS): Unknown operator in BinaryOp"
                                .to_string())
                        }
                    }
                }
            }

            // Handle unary operations
            ASTNode::UnaryOp { operator, expr } => {
                let value = self.evaluate(expr)?;
                match operator {
                    Token::Minus => Ok(-value),
                    _ => Err("Unknown operator in UnaryOp".into()),
                }
            }

            // Handle nullary operations
            ASTNode::NullaryOp { operator } => match operator {
                Token::RandomNumber => {
                    let random_value = self.generate_random_number(); // Use
                                                                      // PRNG
                    Ok(random_value)
                }
                _ => Err("Unknown operator in NullaryOp".into()),
            },

            // Handle assignment operations
            ASTNode::Assignment { variable, expr } => {
                if *variable == '?' {
                    // This is an output statement
                    match expr.as_ref() {
                        // Evaluate strings directly
                        ASTNode::String(value) => {
                            self.write_to_stdout(value);
                            Ok(0) // Strings do not have a numeric value
                        }

                        // Others as usual
                        _ => {
                            let value = self.evaluate(expr)?;
                            self.set_variable(*variable, value);
                            self.write_to_stdout(value.to_string().as_str());
                            Ok(value)
                        }
                    }
                } else if *variable == '#' {
                    // Assign to the program counter
                    if let ASTNode::Number(line_number) = **expr {
                        self.set_program_counter(line_number as usize);
                        Ok(line_number)
                    } else {
                        Err("Invalid line number assignment to #".to_string())
                    }
                } else {
                    // Regular variable assignment
                    let value = self.evaluate(expr)?;
                    self.set_variable(*variable, value);
                    Ok(value)
                }
            }

            // Handle string literals: return the ASCII value of the first character
            ASTNode::String(value) => {
                if let Some(ch) = value.chars().next() {
                    Ok(ch as i64) // Return the ASCII value of the first character
                } else {
                    Err("Empty string".to_string()) // Error if string is empty
                }
            }
        }
    }
}
