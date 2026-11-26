#[cfg(test)]
mod cpu_tests {
    use vtl_rs::cpu::CPU;
    use vtl_rs::interpreter::evaluate_expression;
    use vtl_rs::memory::Memory;

    #[test]
    fn test_cpu_defaults() {
        let cpu = CPU::new();
        assert_eq!(cpu.pemdas, false); // Ensure to default to Altair behavior
    }

    #[test]
    fn test_cpu_accumulator() {
        let mut cpu = CPU::new();
        cpu.set_accumulator(42);
        assert_eq!(cpu.get_accumulator(), 42);
        cpu.set_accumulator(23);
        assert_eq!(cpu.get_accumulator(), 23);
        cpu.set_accumulator(0);
        assert_eq!(cpu.get_accumulator(), 0);
    }

    #[test]
    fn test_cpu_variable_storage() {
        let mut cpu = CPU::new();
        cpu.set_variable('?', 99);
        assert_eq!(cpu.get_variable('?'), Some(99));
        assert_eq!(cpu.get_variable('A'), None);
    }

    #[test]
    fn test_cpu_output_string() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Evaluate ?="HELLO WORLD!"
        let result = evaluate_expression("?=\"HELLO WORLD!\"", &mut cpu, &mut memory);

        // Ensure the result is OK (we expect 0 for string outputs)
        assert_eq!(result, Ok(0));

        // Verify the output in stdout
        assert_eq!(cpu.get_stdout().trim(), "HELLO WORLD!");
    }

    #[test]
    fn test_cpu_output_numeric_expression() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Evaluate ?=2+2
        let result = evaluate_expression("?=2+2", &mut cpu, &mut memory);

        // Ensure the result is OK (we expect the evaluated result, which is 4)
        assert_eq!(result, Ok(4));

        // Verify the output in stdout
        assert_eq!(cpu.get_stdout().trim(), "4");
    }

    #[test]
    fn test_cpu_stdout_flushing() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Evaluate first command ?="HELLO"
        let result1 = evaluate_expression("?=\"HELLO\"", &mut cpu, &mut memory);
        assert_eq!(result1, Ok(0));
        assert_eq!(cpu.get_stdout().trim(), "HELLO");

        // Now flush the stdout
        cpu.clear_stdout();
        assert_eq!(cpu.get_stdout().trim(), ""); // stdout should be empty

        // Evaluate another command ?=2+2
        let result2 = evaluate_expression("?=2+2", &mut cpu, &mut memory);
        assert_eq!(result2, Ok(4));
        assert_eq!(cpu.get_stdout().trim(), "4"); // Only the new result should be there
    }

    #[test]
    fn test_cpu_multiple_commands_without_flushing() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Evaluate ?=2+2
        let result1 = evaluate_expression("?=2+2", &mut cpu, &mut memory);
        assert_eq!(result1, Ok(4));

        // Evaluate ?="HELLO"
        let result2 = evaluate_expression("?=\"HELLO\"", &mut cpu, &mut memory);
        assert_eq!(result2, Ok(0));

        // Verify that both outputs are present, without flushing in between
        assert_eq!(cpu.get_stdout().trim(), "4\nHELLO");
    }

    #[test]
    fn test_cpu_math_and_string_output() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Evaluate a math expression
        let result1 = evaluate_expression("?=5*3", &mut cpu, &mut memory);
        assert_eq!(result1, Ok(15));

        // Evaluate a string output
        let result2 = evaluate_expression("?=\"WORLD\"", &mut cpu, &mut memory);
        assert_eq!(result2, Ok(0));

        // Verify that both outputs are captured in the correct order
        assert_eq!(cpu.get_stdout().trim(), "15\nWORLD");
    }

    #[test]
    fn test_cpu_variable_assignment_output() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Assign 100 to variable A
        let result1 = evaluate_expression("A=100", &mut cpu, &mut memory);
        assert_eq!(result1, Ok(100));

        // Print the value of variable A using ?=A
        let result2 = evaluate_expression("?=A", &mut cpu, &mut memory);
        assert_eq!(result2, Ok(100));

        // Verify that the correct value of A is printed
        assert_eq!(cpu.get_stdout().trim(), "100");
    }

    #[test]
    fn test_cpu_invalid_input() {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);

        // Evaluate an invalid expression
        let result = evaluate_expression("?=A+@", &mut cpu, &mut memory);

        // Expect an error, and ensure no output is produced
        assert!(result.is_err());
        assert_eq!(cpu.get_stdout().trim(), ""); // No output should be present
    }
}
