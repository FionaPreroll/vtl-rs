#[cfg(test)]
mod cpu_evaluator_tests {
    use vtl_rs::cpu::CPU;
    use vtl_rs::interpreter::evaluate_expression;
    use vtl_rs::memory::Memory;

    #[test]
    fn test_evaluator_addition() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);
        let result = evaluate_expression("?=2+2", &mut cpu, &mut memory).unwrap();
        assert_eq!(result, 4);
    }

    #[test]
    fn test_evaluator_large_numbers() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        let result = evaluate_expression("?=999999999999999999999999", &mut cpu, &mut memory);
        assert!(result.is_err(), "Expected overflow for large numbers");
    }

    #[test]
    fn test_division_by_zero() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Division by zero should not panic and should set the division error flag
        let result = evaluate_expression("?=10/0", &mut cpu, &mut memory);

        // Expect the result to be an error
        assert!(result.is_err(), "Expected a division by zero error");

        // Check the exact error message
        let error_msg = result.unwrap_err();
        assert_eq!(
            error_msg, "CPU Error: Evaluation Error (Divide LTR): Division by zero",
            "Unexpected error message: {}",
            error_msg
        );
    }

    #[test]
    fn test_basic_division() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        let result = evaluate_expression("?=6/3", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Basic division failed: {:?}", result);

        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 2,
            "Expected 6 / 3 to equal 2, but got {}",
            result_value
        );
    }

    #[test]
    fn test_zero_as_numerator() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        let result = evaluate_expression("?=0/3", &mut cpu, &mut memory);
        assert!(
            result.is_ok(),
            "Division with zero numerator failed: {:?}",
            result
        );

        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 0,
            "Expected 0 / 3 to equal 0, but got {}",
            result_value
        );
    }

    #[test]
    fn test_negative_numerator() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        let result = evaluate_expression("?=-6/3", &mut cpu, &mut memory);
        assert!(
            result.is_ok(),
            "Division with negative numerator failed: {:?}",
            result
        );

        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, -2,
            "Expected -6 / 3 to equal -2, but got {}",
            result_value
        );
    }

    #[test]
    fn test_negative_denominator() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        let result = evaluate_expression("?=6/-3", &mut cpu, &mut memory);
        assert!(
            result.is_ok(),
            "Division with negative denominator failed: {:?}",
            result
        );

        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, -2,
            "Expected 6 / -3 to equal -2, but got {}",
            result_value
        );
    }

    #[test]
    fn test_both_negative_numbers() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        let result = evaluate_expression("?=-6/-3", &mut cpu, &mut memory);
        assert!(
            result.is_ok(),
            "Division with both negative numbers failed: {:?}",
            result
        );

        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 2,
            "Expected -6 / -3 to equal 2, but got {}",
            result_value
        );
    }

    #[test]
    fn test_integer_truncation() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        let result = evaluate_expression("?=7/3", &mut cpu, &mut memory);
        assert!(
            result.is_ok(),
            "Division resulting in fraction failed: {:?}",
            result
        );

        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 2,
            "Expected 7 / 3 to equal 2 (integer truncation), but got {}",
            result_value
        );
    }

    #[test]
    fn test_remainder_as_variable() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        let result = evaluate_expression("?=7/3", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Division failed: {:?}", result);

        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 2,
            "Expected 7 / 3 to equal 2, but got {}",
            result_value
        );

        // Test remainder retrieval
        let result = evaluate_expression("?=%", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Fetching remainder failed: {:?}", result);

        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 1,
            "Expected remainder 7 % 3 to be 1, but got {}",
            result_value
        );
    }

    // Demo sequence from README
    #[test]
    fn test_mixed_operations() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Test case 1: 22 / 7
        let result = evaluate_expression("?=22/7", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Division failed: {:?}", result);
        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 3,
            "Expected 22 / 7 to equal 3, but got {}",
            result_value
        );

        // Test case 2: %
        let result = evaluate_expression("?=%", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Fetching remainder failed: {:?}", result);
        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 1,
            "Expected remainder 22 % 7 to be 1, but got {}",
            result_value
        );

        // Test case 3: 6 / 1
        let result = evaluate_expression("6/1", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Division failed: {:?}", result);
        assert_eq!(result.unwrap(), 6, "Expected 6 / 1 to equal 6");

        // Test case 4: %
        let result = evaluate_expression("?=%", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Fetching remainder failed: {:?}", result);
        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 0,
            "Expected remainder 6 % 1 to be 0, but got {}",
            result_value
        );

        // Test case 5: 17 / -3
        let result = evaluate_expression("?=17/-3", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Division failed: {:?}", result);
        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, -5,
            "Expected 17 / -3 to equal -5, but got {}",
            result_value
        );

        // Test case 6: %
        let result = evaluate_expression("?=%", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Fetching remainder failed: {:?}", result);
        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 2,
            "Expected remainder 17 % -3 to be 2, but got {}",
            result_value
        );

        // Test case 7: 2 + 2
        let result = evaluate_expression("?=2+2", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Addition failed: {:?}", result);
        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");
        assert_eq!(
            result_value, 4,
            "Expected 2 + 2 to equal 4, but got {}",
            result_value
        );
    }

    #[test]
    fn test_variable_assignment() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Test assigning a value to variable A
        let result = evaluate_expression("A=10", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Assignment to A failed: {:?}", result);
        assert_eq!(cpu.get_variable('A'), Some(10), "Expected A to equal 10");

        // Test assigning a value to variable Z
        let result = evaluate_expression("Z=100", &mut cpu, &mut memory);
        assert!(result.is_ok(), "Assignment to Z failed: {:?}", result);
        assert_eq!(cpu.get_variable('Z'), Some(100), "Expected Z to equal 100");
    }

    #[test]
    fn test_cpu_output_ascii_character() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Output character 'A'
        let result = evaluate_expression("?=\"A\"", &mut cpu, &mut memory);
        assert_eq!(result, Ok(0)); // No numeric value for strings
        assert_eq!(cpu.get_stdout().trim(), "A");
        cpu.clear_stdout();

        // Output ASCII value 65
        let result = evaluate_expression("?=65", &mut cpu, &mut memory);
        assert_eq!(result, Ok(65)); // Numeric output
        assert_eq!(cpu.get_stdout().trim(), "65");
        cpu.clear_stdout();
    }

    #[test]
    fn test_cpu_character_addition() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Assign 'A' to variable A
        let result = evaluate_expression("A=\"A\"", &mut cpu, &mut memory);
        assert_eq!(result, Ok(65)); // ASCII value of 'A'

        // Add 1 to A ('A' + 1 = 'B')
        let result = evaluate_expression("A=A+1", &mut cpu, &mut memory);
        assert_eq!(result, Ok(66)); // ASCII value of 'B'
        assert_eq!(cpu.get_variable('A'), Some(66)); // Stored as 66

        // Add 5 to A ('B' + 5 = 'G')
        let result = evaluate_expression("A=A+5", &mut cpu, &mut memory);
        assert_eq!(result, Ok(71)); // ASCII value of 'G'
        assert_eq!(cpu.get_variable('A'), Some(71)); // Stored as 71
    }

    #[test]
    fn test_cpu_character_subtraction() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Assign 'G' to variable A
        let result = evaluate_expression("A=\"G\"", &mut cpu, &mut memory);
        assert_eq!(result, Ok(71)); // ASCII value of 'G'

        // Subtract 1 from A ('G' - 1 = 'F')
        let result = evaluate_expression("A=A-1", &mut cpu, &mut memory);
        assert_eq!(result, Ok(70)); // ASCII value of 'F'
        assert_eq!(cpu.get_variable('A'), Some(70)); // Stored as 70

        // Subtract 6 from A ('F' - 6 = 'Z')
        let result = evaluate_expression("A=A-6", &mut cpu, &mut memory);
        assert_eq!(result, Ok(64)); // ASCII value of 'z'
        assert_eq!(cpu.get_variable('A'), Some(64)); // Stored as 64
    }

    #[test]
    fn test_cpu_character_multiplication() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Assign 'B' to variable A
        let result = evaluate_expression("A=\"B\"", &mut cpu, &mut memory);
        assert_eq!(result, Ok(66)); // ASCII value of 'B'

        // Multiply A by 2 ('B' * 2 = 132)
        let result = evaluate_expression("A=A*2", &mut cpu, &mut memory);
        assert_eq!(result, Ok(132)); // ASCII result (non-printable)
        assert_eq!(cpu.get_variable('A'), Some(132)); // Stored as 132
    }

    #[test]
    fn test_cpu_character_division() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Assign 'Z' to variable A
        let result = evaluate_expression("A=\"Z\"", &mut cpu, &mut memory);
        assert_eq!(result, Ok(90)); // ASCII value of 'Z'

        // Divide A by 2 ('Z' / 2 = 45)
        let result = evaluate_expression("A=A/2", &mut cpu, &mut memory);
        assert_eq!(result, Ok(45)); // ASCII value of '-'
        assert_eq!(cpu.get_variable('A'), Some(45)); // Stored as 45
    }

    #[test]
    fn test_cpu_character_remainder() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Assign 'H' to variable A
        let result = evaluate_expression("A=\"H\"", &mut cpu, &mut memory);
        assert_eq!(result, Ok(72)); // ASCII value of 'H'

        // Remainder of A divided by 5 ('H' % 5 = 2)
        let result = evaluate_expression("A=A%5", &mut cpu, &mut memory);
        assert_eq!(result, Ok(2)); // Resulting value 2 (non-printable)
        assert_eq!(cpu.get_variable('A'), Some(2)); // Stored as 2
    }

    #[test]
    fn test_random_number_evaluation() {
        let mut cpu = CPU::new_for_tests(Some(42), None); // NOTE that makes it deterministic
        let mut memory = Memory::new(1024);

        // Expression with random number generation and arithmetic
        let result = evaluate_expression("?='", &mut cpu, &mut memory);

        // Ensure the evaluation itself succeeded without errors
        assert!(
            result.is_ok(),
            "Random number evaluation failed: {:?}",
            result
        );

        // Retrieve the value from the variable `?`
        let result_value = cpu
            .get_variable('?')
            .expect("Failed to retrieve '?' variable");

        let expected_value = 8743;
        assert_eq!(
            result_value, expected_value,
            "Expected the random number evaluation to be {}, but got {}",
            expected_value, result_value
        );
    }
}
