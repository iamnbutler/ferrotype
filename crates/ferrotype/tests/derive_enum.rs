//! Tests for #[derive(TypeScript)] on enums
//!
//! These tests verify that the derive macro generates correct TypeScript
//! discriminated union types for various enum patterns.

use ferrotype::{TypeScript, TypeDef, Primitive};

/// Helper to get the inner definition from a Named TypeDef
fn inner_def(td: TypeDef) -> TypeDef {
    match td {
        TypeDef::Named { def, .. } => *def,
        other => other,
    }
}

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
    let td = SimpleStatus::typescript();
    let rendered = inner_def(td.clone()).render();
    assert_eq!(rendered, r#""Pending" | "Active" | "Completed""#);
    assert_eq!(td.render(), "SimpleStatus");
}

#[derive(TypeScript)]
enum SingleVariant {
    Only,
}

#[test]
fn test_single_unit_variant() {
    let td = SingleVariant::typescript();
    assert_eq!(inner_def(td.clone()).render(), r#""Only""#);
    assert_eq!(td.render(), "SingleVariant");
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
    let td = Coordinate::typescript();
    let rendered = inner_def(td.clone()).render();
    assert_eq!(
        rendered,
        r#"{ type: "D2"; value: [number, number] } | { type: "D3"; value: [number, number, number] }"#
    );
    assert_eq!(td.render(), "Coordinate");
}

#[derive(TypeScript)]
enum NewtypeWrapper {
    Text(String),
    Number(i32),
}

#[test]
fn test_newtype_variant_enum() {
    let td = NewtypeWrapper::typescript();
    assert_eq!(
        inner_def(td).render(),
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

impl TypeScript for Point {
    fn typescript() -> TypeDef {
        TypeDef::Ref("Point".to_string())
    }
}

#[derive(TypeScript)]
enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { width: f64, height: f64 },
}

#[test]
fn test_struct_variant_enum() {
    let td = Shape::typescript();
    let rendered = inner_def(td.clone()).render();
    assert_eq!(
        rendered,
        r#"{ type: "Circle"; center: Point; radius: number } | { type: "Rectangle"; width: number; height: number }"#
    );
    assert_eq!(td.render(), "Shape");
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
    let td = Message::typescript();
    let rendered = inner_def(td.clone()).render();
    assert_eq!(
        rendered,
        r#"{ type: "Ping" } | { type: "Text"; value: string } | { type: "Binary"; value: number[] } | { type: "Error"; code: number; message: string }"#
    );
    assert_eq!(td.render(), "Message");
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
    let td_string = <OptionalValue<String>>::typescript();
    assert_eq!(
        inner_def(td_string.clone()).render(),
        r#"{ type: "None" } | { type: "Some"; value: string }"#
    );
    assert_eq!(td_string.render(), "OptionalValue");

    let td_i32 = <OptionalValue<i32>>::typescript();
    assert_eq!(
        inner_def(td_i32).render(),
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
    let td = <ResultLike<String, i32>>::typescript();
    assert_eq!(
        inner_def(td).render(),
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
    let td = ComplexVariants::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains(r#"{ type: "Empty" }"#));
    assert!(rendered.contains(r#"{ type: "Simple"; value: string }"#));
    // Vec<Option<String>> becomes (string | null)[] with parens for union in array
    assert!(rendered.contains(r#"{ type: "Nested"; value: (string | null)[] }"#));
    assert!(rendered.contains(r#"{ type: "Struct"; items: number[]; count: number }"#));
}

// ============================================================================
// SNAPSHOT TESTS
// ============================================================================

#[test]
fn test_derive_enum_snapshots() {
    insta::assert_snapshot!("derive_unit_enum", inner_def(SimpleStatus::typescript()).render());
    insta::assert_snapshot!("derive_tuple_enum", inner_def(Coordinate::typescript()).render());
    insta::assert_snapshot!("derive_struct_enum", inner_def(Shape::typescript()).render());
    insta::assert_snapshot!("derive_mixed_enum", inner_def(Message::typescript()).render());
    insta::assert_snapshot!("derive_generic_enum_string", inner_def(<OptionalValue<String>>::typescript()).render());
    insta::assert_snapshot!("derive_generic_enum_i32", inner_def(<OptionalValue<i32>>::typescript()).render());
}
