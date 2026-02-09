// Tests demonstrating --nocapture output capture with cargo tes
// These are fixture tests for testing cargo-tes functionality.
// They are ignored by default to prevent normal test runs from failing.
// Run with: cargo test -- --ignored

#[test]
#[ignore = "fixture test for cargo-tes (intentionally fails)"]
fn test_failing_with_stdout() {
    println!("STDOUT: This is normal output from failing test");
    println!("STDOUT: Multiple lines of output");
    println!("STDOUT: Should be captured by --nocapture");
    assert_eq!(2 + 2, 5, "Intentional failure to trigger output");
}

#[test]
#[ignore = "fixture test for cargo-tes (intentionally fails)"]
fn test_failing_with_stderr() {
    eprintln!("STDERR: This is error output from failing test");
    eprintln!("STDERR: Multiple error lines");
    eprintln!("STDERR: Should also be captured");
    panic!("Intentional panic to show stderr capture");
}

#[test]
#[ignore = "fixture test for cargo-tes (intentionally fails)"]
fn test_failing_with_both_streams() {
    println!("STDOUT: Normal output line 1");
    eprintln!("STDERR: Error output line 1");
    println!("STDOUT: Normal output line 2");
    eprintln!("STDERR: Error output line 2");
    assert!(false, "Both stdout and stderr should be captured together");
}

#[test]
fn test_passing_with_output() {
    println!("PASSING: This output only shows with --nocapture or --show-output");
    eprintln!("PASSING: Same for stderr in passing tests");
    assert_eq!(1 + 1, 2);
}
