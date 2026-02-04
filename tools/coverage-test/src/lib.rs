/// Fully covered: simple addition
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Fully covered: simple multiplication
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Only the `true` branch is tested — the `else` block has consecutive uncovered lines.
pub fn branch_example(flag: bool) -> &'static str {
    if flag {
        "true branch"
    } else {
        let _x = 1;
        let _y = 2;
        let _z = 3;
        "false branch"
    }
}

/// Only arms 0 and 1 are tested — remaining arms produce individual uncovered lines.
pub fn match_example(val: i32) -> &'static str {
    match val {
        0 => "zero",
        1 => "one",
        2 => "two",
        3 => "three",
        _ => "other",
    }
}

/// Never called — produces 9+ consecutive uncovered lines to test range grouping.
pub fn never_called() -> i32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
    let i = 9;
    a + b + c + d + e + f + g + h + i
}

/// Another fully uncovered block.
pub fn also_never_called() -> String {
    let s = String::from("hello");
    let t = String::from(" world");
    let u = format!("{}{}", s, t);
    u.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(3, 4), 12);
        assert_eq!(multiply(0, 100), 0);
    }

    #[test]
    fn test_branch_true_only() {
        assert_eq!(branch_example(true), "true branch");
    }

    #[test]
    fn test_match_zero_and_one() {
        assert_eq!(match_example(0), "zero");
        assert_eq!(match_example(1), "one");
    }
}
