// Test error suite for benchmarking cargo-tes
// Contains various test failures for meaningful benchmarking

#[cfg(test)]
mod tests {
    // Passing tests (should be filtered out)
    #[test]
    fn test_passing_simple() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_passing_complex() {
        let mut vec = vec![1, 2, 3];
        vec.push(4);
        assert_eq!(vec.len(), 4);
    }

    // Failing tests with short messages
    #[test]
    fn test_failing_assert_eq() {
        assert_eq!(2 + 2, 5);
    }

    #[test]
    fn test_failing_assert() {
        assert!(false);
    }

    #[test]
    fn test_failing_panic_short() {
        panic!("short panic");
    }

    // Failing tests with long messages (to test space minimization)
    #[test]
    fn test_failing_panic_long() {
        panic!("This is a very long error message that contains multiple sentences and should be properly minimized by splitting on whitespace and joining back together to reduce the overall character count in the output while preserving readability for editors and AI tools.");
    }

    #[test]
    fn test_failing_assert_long() {
        let expected = "This is a long expected string with lots of words";
        let actual = "This is a long actual string with different words";
        assert_eq!(expected, actual, "Custom message with additional details about why this test failed and what was expected versus what was received in the test execution.");
    }

    // Ignored tests (filtered out as "warnings")
    #[test]
    #[ignore]
    fn test_ignored_simple() {
        assert_eq!(1 + 1, 3);
    }

    #[test]
    #[ignore]
    fn test_ignored_with_reason() {
        // This test is ignored for benchmarking purposes
        panic!("ignored panic");
    }

    // Tests that panic with backtraces
    #[test]
    fn test_failing_backtrace() {
        let vec = vec![1, 2, 3];
        vec[10]; // Index out of bounds
    }

    // Tests with custom messages
    #[test]
    fn test_failing_custom_message() {
        assert!(
            false,
            "Custom failure message for testing output formatting"
        );
    }

    // Additional passing tests for dilution
    #[test]
    fn test_passing_another() {
        assert!(true);
    }

    #[test]
    fn test_passing_third() {
        let x = 42;
        assert_eq!(x * 2, 84);
    }
}

/// ```
/// This doctest will fail intentionally
/// ```
/// panic!("doctest failure");
#[test]
fn doctest_placeholder() {
    // Placeholder to ensure doctests are included
}
