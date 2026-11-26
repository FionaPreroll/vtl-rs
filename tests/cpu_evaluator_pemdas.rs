#[cfg(test)]
mod cpu_evaluator_tests {
    use vtl_rs::cpu::CPU;
    use vtl_rs::interpreter::evaluate_expression;
    use vtl_rs::memory::Memory;

    // Helper function to evaluate expressions with left_to_right flag
    fn evaluate_with_mode(expression: &str, left_to_right: bool) -> Result<i64, String> {
        let mut cpu = CPU::new();
        let mut memory = Memory::new(1024);
        cpu.pemdas = !left_to_right; // Set the evaluation mode
        let ret = evaluate_expression(expression, &mut cpu, &mut memory);
        assert_eq!(cpu.pemdas, !left_to_right);
        ret
    }

    #[test]
    fn test_evaluator_left_to_right_true() {
        // Strict left-to-right evaluation (no operator precedence)

        // 2 + 3 * 4 should be evaluated as (2 + 3) * 4 = 5 * 4 = 20
        let result = evaluate_with_mode("?=2+3*4", true).unwrap();
        assert_eq!(result, 20);

        // 10 - 2 + 5 should be evaluated as (10 - 2) + 5 = 8 + 5 = 13
        let result = evaluate_with_mode("?=10-2+5", true).unwrap();
        assert_eq!(result, 13);

        // 6 / 2 * 3 should be evaluated as (6 / 2) * 3 = 3 * 3 = 9
        let result = evaluate_with_mode("?=6/2*3", true).unwrap();
        assert_eq!(result, 9);

        // 2 * 5 + 3 should be evaluated as (2 * 5) + 3 = 10 + 3 = 13
        let result = evaluate_with_mode("?=2*5+3", true).unwrap();
        assert_eq!(result, 13);

        // 2 + 5 * 3 should be evaluated as (2 + 5) * 3 = 7 * 3 = 21
        let result = evaluate_with_mode("?=2+5*3", true).unwrap();
        assert_eq!(result, 21);

        // 100 / 10 / 2 should be evaluated as (100 / 10) / 2 = 10 / 2 = 5
        let result = evaluate_with_mode("?=100/10/2", true).unwrap();
        assert_eq!(result, 5);

        // 5 - 2 * 3 should be evaluated as (5 - 2) * 3 = 3 * 3 = 9
        let result = evaluate_with_mode("?=5-2*3", true).unwrap();
        assert_eq!(result, 9);
    }

    #[test]
    fn test_evaluator_left_to_right_false() {
        // Evaluation with operator precedence (PEMDAS)

        // 2 + 3 * 4 should be evaluated as 2 + (3 * 4) = 2 + 12 = 14
        let result = evaluate_with_mode("?=2+3*4", false).unwrap();
        assert_eq!(result, 14);

        // 10 - 2 + 5 should be evaluated as (10 - 2) + 5 = 8 + 5 = 13
        // Since + and - have the same precedence, left-to-right applies
        let result = evaluate_with_mode("?=10-2+5", false).unwrap();
        assert_eq!(result, 13);

        // 6 / 2 * 3 should be evaluated as (6 / 2) * 3 = 3 * 3 = 9
        // Since * and / have the same precedence, left-to-right applies
        let result = evaluate_with_mode("?=6/2*3", false).unwrap();
        assert_eq!(result, 9);

        // 2 * 5 + 3 should be evaluated as (2 * 5) + 3 = 10 + 3 = 13
        let result = evaluate_with_mode("?=2*5+3", false).unwrap();
        assert_eq!(result, 13);

        // 2 + 5 * 3 should be evaluated as 2 + (5 * 3) = 2 + 15 = 17
        let result = evaluate_with_mode("?=2+5*3", false).unwrap();
        assert_eq!(result, 17);

        // 100 / 10 / 2 should be evaluated as (100 / 10) / 2 = 10 / 2 = 5
        // Since * and / have the same precedence, left-to-right applies
        let result = evaluate_with_mode("?=100/10/2", false).unwrap();
        assert_eq!(result, 5);

        // 5 - 2 * 3 should be evaluated as 5 - (2 * 3) = 5 - 6 = -1
        let result = evaluate_with_mode("?=5-2*3", false).unwrap();
        assert_eq!(result, -1);
    }
}
