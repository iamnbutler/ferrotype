//! Tests for #[derive(TypeScript)] on structs

use ferrotype::{TypeScript, TypeScriptType};

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
    let ts = SimpleUser::typescript_type();
    assert!(ts.contains("id: string"));
    assert!(ts.contains("name: string"));
    assert!(ts.contains("age: number"));
}

#[test]
fn test_named_struct_name() {
    assert_eq!(SimpleUser::typescript_name(), "SimpleUser");
}

#[derive(TypeScript)]
struct EmptyStruct {}

#[test]
fn test_empty_struct() {
    assert_eq!(EmptyStruct::typescript_type(), "{}");
}

#[derive(TypeScript)]
struct NestedStruct {
    user: SimpleUser,
    active: bool,
}

#[test]
fn test_nested_struct() {
    let ts = NestedStruct::typescript_type();
    // The user field should reference SimpleUser's typescript type
    assert!(ts.contains("user:"));
    assert!(ts.contains("active: boolean"));
}

// ============================================================================
// TUPLE STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct NewtypeString(String);

#[test]
fn test_newtype_struct() {
    // Newtypes should unwrap to their inner type
    assert_eq!(NewtypeString::typescript_type(), "string");
}

#[derive(TypeScript)]
struct TupleTwo(String, i32);

#[test]
fn test_tuple_struct() {
    assert_eq!(TupleTwo::typescript_type(), "[string, number]");
}

#[derive(TypeScript)]
struct TupleThree(String, i32, bool);

#[test]
fn test_tuple_three() {
    assert_eq!(TupleThree::typescript_type(), "[string, number, boolean]");
}

// ============================================================================
// UNIT STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct UnitStruct;

#[test]
fn test_unit_struct() {
    assert_eq!(UnitStruct::typescript_type(), "null");
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
    assert!(Container::<String>::typescript_type().contains("value: string"));
    assert!(Container::<i32>::typescript_type().contains("value: number"));
}

#[derive(TypeScript)]
struct Pair<A, B> {
    first: A,
    second: B,
}

#[test]
fn test_multi_generic_struct() {
    let ts = Pair::<String, i32>::typescript_type();
    assert!(ts.contains("first: string"));
    assert!(ts.contains("second: number"));
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
    let ts = ComplexFields::typescript_type();
    assert!(ts.contains("items: string[]"));
    assert!(ts.contains("maybe_count: number | null"));
    assert!(ts.contains("result:"));
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
    let ts = GetUserRequest::typescript_type();
    assert!(ts.contains("user_id: string"));
}

#[test]
fn test_rpc_response_type() {
    let ts = GetUserResponse::typescript_type();
    assert!(ts.contains("id: string"));
    assert!(ts.contains("username: string"));
    assert!(ts.contains("email: string | null"));
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
    let ts = ListUsersRequest::typescript_type();
    assert!(ts.contains("page: number"));
    assert!(ts.contains("page_size: number"));
}

#[test]
fn test_list_response() {
    let ts = ListUsersResponse::typescript_type();
    assert!(ts.contains("users:"));
    assert!(ts.contains("total_count: number"));
    assert!(ts.contains("has_more: boolean"));
}
