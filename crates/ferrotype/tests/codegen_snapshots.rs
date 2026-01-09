//! Snapshot tests for TypeScript codegen
//!
//! These tests verify that generated TypeScript types match expected output.
//! Run `cargo insta review` to update snapshots after intentional changes.

use ferrotype::TypeScriptType;

// Primitive type codegen tests

#[test]
fn test_unit_type() {
    insta::assert_snapshot!("unit_type", <()>::typescript_type());
}

#[test]
fn test_bool_type() {
    insta::assert_snapshot!("bool_type", bool::typescript_type());
}

#[test]
fn test_string_type() {
    insta::assert_snapshot!("string_type", String::typescript_type());
}

#[test]
fn test_str_type() {
    insta::assert_snapshot!("str_type", <&str>::typescript_type());
}

// Numeric type codegen tests

#[test]
fn test_i32_type() {
    insta::assert_snapshot!("i32_type", i32::typescript_type());
}

#[test]
fn test_i64_type() {
    insta::assert_snapshot!("i64_type", i64::typescript_type());
}

#[test]
fn test_u32_type() {
    insta::assert_snapshot!("u32_type", u32::typescript_type());
}

#[test]
fn test_u64_type() {
    insta::assert_snapshot!("u64_type", u64::typescript_type());
}

#[test]
fn test_f32_type() {
    insta::assert_snapshot!("f32_type", f32::typescript_type());
}

#[test]
fn test_f64_type() {
    insta::assert_snapshot!("f64_type", f64::typescript_type());
}

#[test]
fn test_usize_type() {
    insta::assert_snapshot!("usize_type", usize::typescript_type());
}

// Generic type codegen tests

#[test]
fn test_option_string() {
    insta::assert_snapshot!("option_string", <Option<String>>::typescript_type());
}

#[test]
fn test_option_i32() {
    insta::assert_snapshot!("option_i32", <Option<i32>>::typescript_type());
}

#[test]
fn test_option_bool() {
    insta::assert_snapshot!("option_bool", <Option<bool>>::typescript_type());
}

#[test]
fn test_vec_string() {
    insta::assert_snapshot!("vec_string", <Vec<String>>::typescript_type());
}

#[test]
fn test_vec_i32() {
    insta::assert_snapshot!("vec_i32", <Vec<i32>>::typescript_type());
}

#[test]
fn test_vec_bool() {
    insta::assert_snapshot!("vec_bool", <Vec<bool>>::typescript_type());
}

// Nested generic types

#[test]
fn test_option_vec_string() {
    insta::assert_snapshot!("option_vec_string", <Option<Vec<String>>>::typescript_type());
}

#[test]
fn test_vec_option_i32() {
    insta::assert_snapshot!("vec_option_i32", <Vec<Option<i32>>>::typescript_type());
}

// Result type codegen tests

#[test]
fn test_result_string_string() {
    insta::assert_snapshot!(
        "result_string_string",
        <Result<String, String>>::typescript_type()
    );
}

#[test]
fn test_result_i32_string() {
    insta::assert_snapshot!("result_i32_string", <Result<i32, String>>::typescript_type());
}

#[test]
fn test_result_vec_string() {
    insta::assert_snapshot!(
        "result_vec_string",
        <Result<Vec<String>, String>>::typescript_type()
    );
}

// Tuple type codegen tests

#[test]
fn test_tuple1_string() {
    insta::assert_snapshot!("tuple1_string", <(String,)>::typescript_type());
}

#[test]
fn test_tuple2_string_i32() {
    insta::assert_snapshot!("tuple2_string_i32", <(String, i32)>::typescript_type());
}

#[test]
fn test_tuple3_string_i32_bool() {
    insta::assert_snapshot!(
        "tuple3_string_i32_bool",
        <(String, i32, bool)>::typescript_type()
    );
}

// Complex nested types

#[test]
fn test_nested_result_option() {
    insta::assert_snapshot!(
        "nested_result_option",
        <Result<Option<String>, String>>::typescript_type()
    );
}

#[test]
fn test_nested_vec_tuple() {
    insta::assert_snapshot!(
        "nested_vec_tuple",
        <Vec<(String, i32)>>::typescript_type()
    );
}

#[test]
fn test_deeply_nested() {
    insta::assert_snapshot!(
        "deeply_nested",
        <Result<Vec<Option<(String, i32)>>, String>>::typescript_type()
    );
}

// Type name tests (for export declarations)

#[test]
fn test_type_names() {
    let names = vec![
        ("unit", <()>::typescript_name()),
        ("bool", bool::typescript_name()),
        ("string", String::typescript_name()),
        ("i32", i32::typescript_name()),
        ("Option<String>", <Option<String>>::typescript_name()),
        ("Vec<i32>", <Vec<i32>>::typescript_name()),
        ("Result<String, String>", <Result<String, String>>::typescript_name()),
        ("(String, i32)", <(String, i32)>::typescript_name()),
    ];

    let output = names
        .iter()
        .map(|(rust, ts)| format!("{} -> {}", rust, ts))
        .collect::<Vec<_>>()
        .join("\n");

    insta::assert_snapshot!("type_names", output);
}
