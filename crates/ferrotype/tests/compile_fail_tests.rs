//! Compile-fail tests for error paths in ferrotype derive macros
//!
//! These tests verify that the derive macros produce clear,
//! helpful error messages when used incorrectly.

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
