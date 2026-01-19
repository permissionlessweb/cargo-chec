//! Test crate with intentional compilation errors and warnings
//! Used for benchmarking cargo check vs cargo chec output

// ============================================
// WARNINGS: Unused imports (5 warnings)
// ============================================
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;
use std::io::Write;
use std::fmt::Debug;

pub mod tests;

// ============================================
// WARNINGS: Unused variables (10 warnings)
// ============================================
pub fn function_with_unused_variables() {
    let unused_var_1 = 42;
    let unused_var_2 = "hello";
    let unused_var_3 = true;
    let unused_var_4 = 3.14;
    let unused_var_5 = vec![1, 2, 3];
    let unused_var_6 = Some(42);
    let unused_var_7 = None::<i32>;
    let unused_var_8 = (1, 2, 3);
    let unused_var_9 = [1, 2, 3, 4, 5];
    let unused_var_10 = String::from("unused");
}

// ============================================
// WARNINGS: Dead code - unused functions (3 warnings)
// ============================================
fn unused_helper_function_1() -> i32 {
    42
}

fn unused_helper_function_2() -> String {
    String::from("never called")
}

fn unused_helper_function_3(x: i32, y: i32) -> i32 {
    x + y
}

// ============================================
// WARNINGS: Unused mut (2 warnings)
// ============================================
pub fn function_with_unused_mut() {
    let mut unnecessary_mut_1 = 10;
    let mut unnecessary_mut_2 = String::new();
    println!("{} {}", unnecessary_mut_1, unnecessary_mut_2);
}

// ============================================
// WARNINGS: Non-snake case names (2 warnings)
// ============================================
pub fn BadlyNamedFunction() {
    let BadlyNamedVariable = 42;
    println!("{}", BadlyNamedVariable);
}

// ============================================
// WARNINGS: Non-camel case type (2 warnings)
// ============================================
pub struct badly_named_struct {
    pub field: i32,
}

pub enum badly_named_enum {
    Variant1,
    Variant2,
}

// ============================================
// ERROR 1: Type mismatch - assigning String to i32
// ============================================
pub fn type_mismatch_error_1() -> i32 {
    let x: i32 = String::from("not a number");
    x
}

// ============================================
// ERROR 2: Type mismatch - assigning bool to String
// ============================================
pub fn type_mismatch_error_2() -> String {
    let s: String = true;
    s
}

// ============================================
// ERROR 3: Unresolved import
// ============================================
use nonexistent_crate::NonexistentType;

// ============================================
// ERROR 4: Missing trait implementation
// ============================================
pub struct NoDisplayTrait {
    data: Vec<u8>,
}

pub fn missing_trait_impl() {
    let obj = NoDisplayTrait { data: vec![] };
    println!("{}", obj);
}

// ============================================
// ERROR 5: Method not found
// ============================================
pub fn method_not_found() {
    let s = String::from("hello");
    s.nonexistent_method();
}

// ============================================
// ERROR 6: Missing required field in struct
// ============================================
pub struct RequiredFields {
    pub field1: i32,
    pub field2: String,
    pub field3: bool,
}

pub fn missing_field_error() -> RequiredFields {
    RequiredFields {
        field1: 42,
        // missing field2 and field3
    }
}

// ============================================
// ERROR 7: Wrong number of arguments
// ============================================
pub fn takes_three_args(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

pub fn wrong_arg_count() -> i32 {
    takes_three_args(1, 2)
}

// ============================================
// ERROR 8: Incompatible types in binary operation
// ============================================
pub fn incompatible_binary_op() -> i32 {
    let a = 42;
    let b = "string";
    a + b
}

// ============================================
// ERROR 9: Unresolved variable
// ============================================
pub fn unresolved_variable() -> i32 {
    completely_undefined_variable
}

// ============================================
// ERROR 10: Return type mismatch
// ============================================
pub fn return_type_mismatch() -> bool {
    42
}

// ============================================
// ERROR 11: Mismatched types in match arms
// ============================================
pub fn mismatched_match_arms(x: i32) -> i32 {
    match x {
        0 => 0,
        1 => "one",
        _ => 2,
    }
}

// ============================================
// ERROR 12: Cannot find type
// ============================================
pub fn uses_undefined_type() -> UndefinedType {
    UndefinedType::new()
}

// ============================================
// Additional warnings: Unreachable code (2 warnings)
// ============================================
pub fn unreachable_code_warning() -> i32 {
    return 42;
    let unreachable = 100;
    unreachable
}

pub fn another_unreachable() {
    panic!("always panics");
    println!("this will never print");
}
