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

// ============================================================================
// RENAME ATTRIBUTE TESTS
// ============================================================================

#[derive(TypeScript)]
#[ts(rename = "RenamedStatus")]
enum StatusWithRenamedType {
    Active,
    Inactive,
}

#[test]
fn test_enum_type_rename() {
    let td = StatusWithRenamedType::typescript();
    assert_eq!(td.render(), "RenamedStatus");
}

#[derive(TypeScript)]
enum VariantRenameEnum {
    #[ts(rename = "active")]
    Active,
    #[ts(rename = "inactive")]
    Inactive,
}

#[test]
fn test_enum_variant_rename() {
    let td = VariantRenameEnum::typescript();
    let rendered = inner_def(td).render();
    assert_eq!(rendered, r#""active" | "inactive""#);
}

#[derive(TypeScript)]
#[ts(rename_all = "camelCase")]
enum CamelCaseEnum {
    FirstVariant,
    SecondVariant,
    ThirdOption,
}

#[test]
fn test_enum_rename_all_camel_case() {
    let td = CamelCaseEnum::typescript();
    let rendered = inner_def(td).render();
    assert_eq!(rendered, r#""firstVariant" | "secondVariant" | "thirdOption""#);
}

#[derive(TypeScript)]
#[ts(rename_all = "snake_case")]
enum SnakeCaseEnum {
    FirstVariant,
    SecondVariant,
}

#[test]
fn test_enum_rename_all_snake_case() {
    let td = SnakeCaseEnum::typescript();
    let rendered = inner_def(td).render();
    assert_eq!(rendered, r#""first_variant" | "second_variant""#);
}

#[derive(TypeScript)]
#[ts(rename_all = "SCREAMING_SNAKE_CASE")]
enum ScreamingEnum {
    FirstVariant,
    SecondVariant,
}

#[test]
fn test_enum_rename_all_screaming() {
    let td = ScreamingEnum::typescript();
    let rendered = inner_def(td).render();
    assert_eq!(rendered, r#""FIRST_VARIANT" | "SECOND_VARIANT""#);
}

#[derive(TypeScript)]
#[ts(rename_all = "camelCase")]
enum MixedRenameEnum {
    FirstVariant,
    #[ts(rename = "EXPLICIT")]
    SecondVariant,
    ThirdVariant,
}

#[test]
fn test_enum_variant_rename_overrides_rename_all() {
    let td = MixedRenameEnum::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains(r#""firstVariant""#));
    assert!(rendered.contains(r#""EXPLICIT""#));
    assert!(rendered.contains(r#""thirdVariant""#));
    // Should NOT contain the camelCase version of SecondVariant
    assert!(!rendered.contains(r#""secondVariant""#));
}

#[derive(TypeScript)]
#[ts(rename_all = "camelCase")]
enum DataVariantRenameEnum {
    #[ts(rename = "textMessage")]
    Text(String),
    #[ts(rename = "errorInfo")]
    Error { code: i32, message: String },
}

#[test]
fn test_data_variant_rename() {
    let td = DataVariantRenameEnum::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains(r#"type: "textMessage""#));
    assert!(rendered.contains(r#"type: "errorInfo""#));
}

// ============================================================================
// SKIP ATTRIBUTE TESTS
// ============================================================================

#[derive(TypeScript)]
enum EnumWithSkippedField {
    Error {
        code: i32,
        message: String,
        #[ts(skip)]
        internal_trace: String,
    },
}

#[test]
fn test_enum_variant_field_skip() {
    let td = EnumWithSkippedField::typescript();
    let rendered = inner_def(td).render();
    // Should include non-skipped fields
    assert!(rendered.contains("code: number"));
    assert!(rendered.contains("message: string"));
    // Should NOT include skipped field
    assert!(!rendered.contains("internal_trace"));
}

// ============================================================================
// TAG ATTRIBUTE TESTS - #[ts(tag = "...")]
// ============================================================================

#[derive(TypeScript)]
#[ts(tag = "kind")]
enum CustomTagEnum {
    Ping,
    Text(String),
    Error { code: i32, message: String },
}

#[test]
fn test_custom_tag_field_name() {
    let td = CustomTagEnum::typescript();
    let rendered = inner_def(td).render();
    // Should use "kind" instead of "type"
    assert!(rendered.contains(r#"kind: "Ping""#));
    assert!(rendered.contains(r#"kind: "Text""#));
    assert!(rendered.contains(r#"kind: "Error""#));
    // Should NOT use "type"
    assert!(!rendered.contains(r#"type: "#));
}

#[derive(TypeScript)]
#[ts(tag = "t")]
enum ShortTagEnum {
    A,
    B(i32),
}

#[test]
fn test_short_tag_name() {
    let td = ShortTagEnum::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains(r#"t: "A""#));
    assert!(rendered.contains(r#"t: "B""#));
}

// ============================================================================
// CONTENT ATTRIBUTE TESTS - #[ts(content = "...")]
// ============================================================================

#[derive(TypeScript)]
#[ts(tag = "t", content = "c")]
enum AdjacentTaggedEnum {
    Text(String),
    Number(i32),
    Tuple(i32, i32),
}

#[test]
fn test_adjacent_tagging_newtype() {
    let td = AdjacentTaggedEnum::typescript();
    let rendered = inner_def(td).render();
    // Adjacent tagging: { t: "Variant", c: data }
    assert!(rendered.contains(r#"{ t: "Text"; c: string }"#));
    assert!(rendered.contains(r#"{ t: "Number"; c: number }"#));
    assert!(rendered.contains(r#"{ t: "Tuple"; c: [number, number] }"#));
}

#[derive(TypeScript)]
#[ts(tag = "tag", content = "data")]
enum AdjacentStructVariant {
    Error { code: i32, message: String },
    Info { text: String },
}

#[test]
fn test_adjacent_tagging_struct_variant() {
    let td = AdjacentStructVariant::typescript();
    let rendered = inner_def(td).render();
    // Struct variants get nested object in content field
    assert!(rendered.contains(r#"{ tag: "Error"; data: { code: number; message: string } }"#));
    assert!(rendered.contains(r#"{ tag: "Info"; data: { text: string } }"#));
}

#[derive(TypeScript)]
#[ts(tag = "kind", content = "value")]
enum AdjacentMixedVariants {
    Ping,
    Text(String),
    Coords(f64, f64),
    Data { x: i32 },
}

#[test]
fn test_adjacent_tagging_mixed_variants() {
    let td = AdjacentMixedVariants::typescript();
    let rendered = inner_def(td).render();
    // Unit variant: just the tag, no content
    assert!(rendered.contains(r#"{ kind: "Ping" }"#));
    // Newtype variant: content is the inner type
    assert!(rendered.contains(r#"{ kind: "Text"; value: string }"#));
    // Tuple variant: content is a tuple
    assert!(rendered.contains(r#"{ kind: "Coords"; value: [number, number] }"#));
    // Struct variant: content is nested object
    assert!(rendered.contains(r#"{ kind: "Data"; value: { x: number } }"#));
}

// ============================================================================
// UNTAGGED ATTRIBUTE TESTS - #[ts(untagged)]
// ============================================================================

#[derive(TypeScript)]
#[ts(untagged)]
enum UntaggedValue {
    Num(i32),
    Str(String),
    Bool(bool),
}

#[test]
fn test_untagged_newtype_variants() {
    let td = UntaggedValue::typescript();
    let rendered = inner_def(td).render();
    // Untagged: plain union of inner types
    assert_eq!(rendered, "number | string | boolean");
}

#[derive(TypeScript)]
#[ts(untagged)]
enum UntaggedMixed {
    Null,
    Text(String),
    Pair(i32, i32),
    Object { name: String },
}

#[test]
fn test_untagged_mixed_variants() {
    let td = UntaggedMixed::typescript();
    let rendered = inner_def(td).render();
    // Unit variant becomes string literal
    assert!(rendered.contains(r#""Null""#));
    // Newtype unwraps to inner type
    assert!(rendered.contains("string"));
    // Tuple becomes tuple type
    assert!(rendered.contains("[number, number]"));
    // Struct variant becomes object type (no tag)
    assert!(rendered.contains("{ name: string }"));
}

#[derive(TypeScript)]
#[ts(untagged)]
enum UntaggedUnitEnum {
    A,
    B,
    C,
}

#[test]
fn test_untagged_unit_variants() {
    let td = UntaggedUnitEnum::typescript();
    let rendered = inner_def(td).render();
    // Unit variants become string literals
    assert_eq!(rendered, r#""A" | "B" | "C""#);
}

#[derive(TypeScript)]
#[ts(untagged, rename_all = "camelCase")]
enum UntaggedWithRename {
    FirstItem,
    SecondItem(String),
}

#[test]
fn test_untagged_with_rename_all() {
    let td = UntaggedWithRename::typescript();
    let rendered = inner_def(td).render();
    // rename_all applies to unit variant name
    assert!(rendered.contains(r#""firstItem""#));
    // Newtype unwraps, rename doesn't affect type
    assert!(rendered.contains("string"));
}

// ============================================================================
// SNAPSHOT TESTS FOR NEW ATTRIBUTES
// ============================================================================

#[test]
fn test_tag_content_untagged_snapshots() {
    insta::assert_snapshot!("custom_tag_enum", inner_def(CustomTagEnum::typescript()).render());
    insta::assert_snapshot!("adjacent_tagged_enum", inner_def(AdjacentTaggedEnum::typescript()).render());
    insta::assert_snapshot!("adjacent_struct_variant", inner_def(AdjacentStructVariant::typescript()).render());
    insta::assert_snapshot!("untagged_value", inner_def(UntaggedValue::typescript()).render());
    insta::assert_snapshot!("untagged_mixed", inner_def(UntaggedMixed::typescript()).render());
}
