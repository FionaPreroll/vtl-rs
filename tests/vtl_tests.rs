#[cfg(test)]
mod tests {
    use vtl_rs::cpu::CPU;
    use vtl_rs::interpreter::evaluate_expression;
    use vtl_rs::memory::Memory;

    #[test]
    fn test_addition_instruction() {
        let mut cpu = CPU::new(); // Initialize the CPU state
        let mut memory = Memory::new(1024); // Initialize memory with size 1024 bytes

        // Try evaluating the expression ?=2+2
        let result = evaluate_expression("?=2+2", &mut cpu, &mut memory);

        // If the result is an error, print it for debugging purposes
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }

        // Assert that the expression evaluation was successful
        assert!(result.is_ok(), "Expression evaluation failed: {:?}", result);

        // Verify the result, in this case, the result of 2 + 2 should be 4 and stored in the '?' variable
        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 4,
            "Expected result of 2 + 2 to be 4, but got {}",
            result_value
        );
    }

    #[test]
    fn test_large_number_overflow() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // A very large number that exceeds i64::MAX
        let expression = "?=10101010100000000000000000000000000022929299292";

        let result = evaluate_expression(expression, &mut cpu, &mut memory);

        assert!(result.is_err(), "Expected an overflow error");
        assert_eq!(result.unwrap_err(), "Lexer Error: Overflow error");
    }
}
