use crate::cpu::CPU;
use crate::lexer::lex;
use crate::memory::Memory;
use crate::parser::Parser;

/// Interpret a string of VTL code with the given CPU and Memory.
///
/// # Example
///
/// ```
/// use vtl_rs::cpu::CPU;
/// use vtl_rs::lexer::lex;
/// use vtl_rs::memory::Memory;
/// use vtl_rs::interpreter::evaluate_expression;
///
/// let mut cpu = CPU::new();
/// let mut memory = Memory::new(1024);
/// let expression = String::from("?=2+3");
/// let result = evaluate_expression(&expression, &mut cpu, &mut memory);
/// assert_eq!(result, Ok(5));
/// ```
#[allow(unused_variables)] // TODO use memory
pub fn evaluate_expression(
    expression: &str,
    cpu: &mut CPU,
    memory: &mut Memory,
) -> Result<i64, String> {
    // Step 1: Lexical analysis
    let tokens = match lex(expression) {
        Ok(tokens) => tokens,
        Err(err) => return Err(format!("Lexer Error: {}", err)),
    };
    if tokens.is_empty() {
        return Err("Lexer Error: tokens empty".into());
    }

    // Step 2: Parse the tokens into an AST
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(node) => node,
        Err(err) => return Err(format!("Parser Error: {}", err)),
    };

    // Step 3: Use the existing CPU evaluate method to evaluate the AST
    let result = match cpu.evaluate(&ast) {
        Ok(result) => result,
        Err(err) => return Err(format!("CPU Error: {}", err)),
    };

    Ok(result)
}
