//! Tests for #[derive(TypeScript)] on structs

use ferrotype::{TypeScript, TypeDef};

/// Helper to get the inner definition from a Named TypeDef
fn inner_def(td: TypeDef) -> TypeDef {
    match td {
        TypeDef::Named { def, .. } => *def,
        other => other,
    }
}

// ============================================================================
// NAMED STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct SimpleUser {
    id: String,
    name: String,
    age: i32,
}

#[test]
fn test_named_struct() {
    let td = SimpleUser::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("id: string"));
    assert!(rendered.contains("name: string"));
    assert!(rendered.contains("age: number"));
}

#[test]
fn test_named_struct_name() {
    let td = SimpleUser::typescript();
    // Named types render as just their name
    assert_eq!(td.render(), "SimpleUser");
}

#[derive(TypeScript)]
struct EmptyStruct {}

#[test]
fn test_empty_struct() {
    let td = EmptyStruct::typescript();
    assert_eq!(inner_def(td).render(), "{}");
}

#[derive(TypeScript)]
struct NestedStruct {
    user: SimpleUser,
    active: bool,
}

#[test]
fn test_nested_struct() {
    let td = NestedStruct::typescript();
    let rendered = inner_def(td).render();
    // The user field should reference SimpleUser (which renders as its name)
    assert!(rendered.contains("user: SimpleUser"));
    assert!(rendered.contains("active: boolean"));
}

// ============================================================================
// TUPLE STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct NewtypeString(String);

#[test]
fn test_newtype_struct() {
    // Newtypes should unwrap to their inner type
    let td = NewtypeString::typescript();
    assert_eq!(inner_def(td).render(), "string");
}

#[derive(TypeScript)]
struct TupleTwo(String, i32);

#[test]
fn test_tuple_struct() {
    let td = TupleTwo::typescript();
    assert_eq!(inner_def(td).render(), "[string, number]");
}

#[derive(TypeScript)]
struct TupleThree(String, i32, bool);

#[test]
fn test_tuple_three() {
    let td = TupleThree::typescript();
    assert_eq!(inner_def(td).render(), "[string, number, boolean]");
}

// ============================================================================
// UNIT STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct UnitStruct;

#[test]
fn test_unit_struct() {
    let td = UnitStruct::typescript();
    assert_eq!(inner_def(td).render(), "null");
}

// ============================================================================
// GENERIC STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct Container<T> {
    value: T,
}

#[test]
fn test_generic_struct() {
    let td_string = Container::<String>::typescript();
    assert!(inner_def(td_string).render().contains("value: string"));

    let td_i32 = Container::<i32>::typescript();
    assert!(inner_def(td_i32).render().contains("value: number"));
}

#[derive(TypeScript)]
struct Pair<A, B> {
    first: A,
    second: B,
}

#[test]
fn test_multi_generic_struct() {
    let td = Pair::<String, i32>::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("first: string"));
    assert!(rendered.contains("second: number"));
}

// ============================================================================
// COMPLEX TYPE FIELD TESTS
// ============================================================================

#[derive(TypeScript)]
struct ComplexFields {
    items: Vec<String>,
    maybe_count: Option<i32>,
    result: Result<String, String>,
}

#[test]
fn test_complex_fields() {
    let td = ComplexFields::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("items: string[]"));
    assert!(rendered.contains("maybe_count: number | null"));
    assert!(rendered.contains("result:"));
}

// ============================================================================
// RPC REQUEST/RESPONSE PATTERN TESTS
// ============================================================================

#[derive(TypeScript)]
struct GetUserRequest {
    user_id: String,
}

#[derive(TypeScript)]
struct GetUserResponse {
    id: String,
    username: String,
    email: Option<String>,
}

#[test]
fn test_rpc_request_type() {
    let td = GetUserRequest::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("user_id: string"));
}

#[test]
fn test_rpc_response_type() {
    let td = GetUserResponse::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("id: string"));
    assert!(rendered.contains("username: string"));
    assert!(rendered.contains("email: string | null"));
}

#[derive(TypeScript)]
struct ListUsersRequest {
    page: i32,
    page_size: i32,
}

#[derive(TypeScript)]
struct ListUsersResponse {
    users: Vec<GetUserResponse>,
    total_count: i32,
    has_more: bool,
}

#[test]
fn test_list_request() {
    let td = ListUsersRequest::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("page: number"));
    assert!(rendered.contains("page_size: number"));
}

#[test]
fn test_list_response() {
    let td = ListUsersResponse::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("users:"));
    assert!(rendered.contains("total_count: number"));
    assert!(rendered.contains("has_more: boolean"));
}
