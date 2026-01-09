//! Tests for #[derive(TypeScript)] on enums
//!
//! These tests verify that the derive macro generates correct TypeScript
//! discriminated union types for various enum patterns.

use ferrotype::{TypeScript, TypeScriptType};

// ============================================================================
// UNIT VARIANT TESTS
// ============================================================================

#[derive(TypeScript)]
enum SimpleStatus {
    Pending,
    Active,
    Completed,
}

#[test]
fn test_unit_variant_enum() {
    assert_eq!(
        SimpleStatus::typescript_type(),
        r#""Pending" | "Active" | "Completed""#
    );
    assert_eq!(SimpleStatus::typescript_name(), "SimpleStatus");
}

#[derive(TypeScript)]
enum SingleVariant {
    Only,
}

#[test]
fn test_single_unit_variant() {
    assert_eq!(SingleVariant::typescript_type(), r#""Only""#);
    assert_eq!(SingleVariant::typescript_name(), "SingleVariant");
}

// ============================================================================
// TUPLE VARIANT TESTS
// ============================================================================

#[derive(TypeScript)]
enum Coordinate {
    D2(f64, f64),
    D3(f64, f64, f64),
}

#[test]
fn test_tuple_variant_enum() {
    assert_eq!(
        Coordinate::typescript_type(),
        r#"{ type: "D2"; value: [number, number] } | { type: "D3"; value: [number, number, number] }"#
    );
    assert_eq!(Coordinate::typescript_name(), "Coordinate");
}

#[derive(TypeScript)]
enum NewtypeWrapper {
    Text(String),
    Number(i32),
}

#[test]
fn test_newtype_variant_enum() {
    assert_eq!(
        NewtypeWrapper::typescript_type(),
        r#"{ type: "Text"; value: string } | { type: "Number"; value: number }"#
    );
}

// ============================================================================
// STRUCT VARIANT TESTS
// ============================================================================

struct Point {
    _x: f64,
    _y: f64,
}

impl TypeScriptType for Point {
    fn typescript_type() -> String {
        "Point".to_string()
    }
    fn typescript_name() -> &'static str {
        "Point"
    }
}

#[derive(TypeScript)]
enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { width: f64, height: f64 },
}

#[test]
fn test_struct_variant_enum() {
    assert_eq!(
        Shape::typescript_type(),
        r#"{ type: "Circle"; center: Point; radius: number } | { type: "Rectangle"; width: number; height: number }"#
    );
    assert_eq!(Shape::typescript_name(), "Shape");
}

// ============================================================================
// MIXED VARIANT TESTS
// ============================================================================

#[derive(TypeScript)]
enum Message {
    Ping,
    Text(String),
    Binary(Vec<u8>),
    Error { code: i32, message: String },
}

#[test]
fn test_mixed_variant_enum() {
    assert_eq!(
        Message::typescript_type(),
        r#"{ type: "Ping" } | { type: "Text"; value: string } | { type: "Binary"; value: number[] } | { type: "Error"; code: number; message: string }"#
    );
    assert_eq!(Message::typescript_name(), "Message");
}

// ============================================================================
// GENERIC ENUM TESTS
// ============================================================================

#[derive(TypeScript)]
enum OptionalValue<T> {
    None,
    Some(T),
}

#[test]
fn test_generic_enum() {
    assert_eq!(
        <OptionalValue<String>>::typescript_type(),
        r#"{ type: "None" } | { type: "Some"; value: string }"#
    );
    assert_eq!(<OptionalValue<String>>::typescript_name(), "OptionalValue");

    assert_eq!(
        <OptionalValue<i32>>::typescript_type(),
        r#"{ type: "None" } | { type: "Some"; value: number }"#
    );
}

#[derive(TypeScript)]
enum ResultLike<T, E> {
    Ok(T),
    Err(E),
}

#[test]
fn test_multi_generic_enum() {
    assert_eq!(
        <ResultLike<String, i32>>::typescript_type(),
        r#"{ type: "Ok"; value: string } | { type: "Err"; value: number }"#
    );
}

// ============================================================================
// COMPLEX NESTED TYPE TESTS
// ============================================================================

#[derive(TypeScript)]
enum ComplexVariants {
    Empty,
    Simple(String),
    Nested(Vec<Option<String>>),
    Struct { items: Vec<i32>, count: u64 },
}

#[test]
fn test_complex_nested_types() {
    let ts = ComplexVariants::typescript_type();
    assert!(ts.contains(r#"{ type: "Empty" }"#));
    assert!(ts.contains(r#"{ type: "Simple"; value: string }"#));
    // Vec<Option<String>> becomes string | null[] (note: parentheses not added for union types)
    assert!(ts.contains(r#"{ type: "Nested"; value: string | null[] }"#));
    assert!(ts.contains(r#"{ type: "Struct"; items: number[]; count: number }"#));
}

// ============================================================================
// SNAPSHOT TESTS
// ============================================================================

#[test]
fn test_derive_enum_snapshots() {
    insta::assert_snapshot!("derive_unit_enum", SimpleStatus::typescript_type());
    insta::assert_snapshot!("derive_tuple_enum", Coordinate::typescript_type());
    insta::assert_snapshot!("derive_struct_enum", Shape::typescript_type());
    insta::assert_snapshot!("derive_mixed_enum", Message::typescript_type());
    insta::assert_snapshot!("derive_generic_enum_string", <OptionalValue<String>>::typescript_type());
    insta::assert_snapshot!("derive_generic_enum_i32", <OptionalValue<i32>>::typescript_type());
}
