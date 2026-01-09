//! Ferrotype Test Fixtures
//!
//! Comprehensive Rust types for testing TypeScript codegen.
//! These fixtures cover the full range of type patterns that
//! ferrotype must handle correctly.

use ferrotype::{RpcParam, RpcReturn, TypeScriptType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// STRUCT FIXTURES
// ============================================================================

/// Simple struct with named fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl TypeScriptType for Point {
    fn typescript_type() -> String {
        "{ x: number; y: number }".to_string()
    }
    fn typescript_name() -> &'static str {
        "Point"
    }
}

impl RpcParam for Point {}
impl RpcReturn for Point {}

/// Struct with multiple field types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

impl TypeScriptType for User {
    fn typescript_type() -> String {
        "{ id: number; name: string; email: string; active: boolean }".to_string()
    }
    fn typescript_name() -> &'static str {
        "User"
    }
}

impl RpcParam for User {}
impl RpcReturn for User {}

/// Struct with optional fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

impl TypeScriptType for Profile {
    fn typescript_type() -> String {
        "{ username: string; display_name: string | null; bio: string | null; avatar_url: string | null }".to_string()
    }
    fn typescript_name() -> &'static str {
        "Profile"
    }
}

impl RpcParam for Profile {}
impl RpcReturn for Profile {}

/// Tuple struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl TypeScriptType for Rgb {
    fn typescript_type() -> String {
        "[number, number, number]".to_string()
    }
    fn typescript_name() -> &'static str {
        "Rgb"
    }
}

impl RpcParam for Rgb {}
impl RpcReturn for Rgb {}

/// Unit struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ping;

impl TypeScriptType for Ping {
    fn typescript_type() -> String {
        "null".to_string()
    }
    fn typescript_name() -> &'static str {
        "Ping"
    }
}

impl RpcParam for Ping {}
impl RpcReturn for Ping {}

/// Newtype wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserId(pub u64);

impl TypeScriptType for UserId {
    fn typescript_type() -> String {
        "number".to_string()
    }
    fn typescript_name() -> &'static str {
        "UserId"
    }
}

impl RpcParam for UserId {}
impl RpcReturn for UserId {}

/// Nested struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rectangle {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl TypeScriptType for Rectangle {
    fn typescript_type() -> String {
        "{ top_left: Point; bottom_right: Point }".to_string()
    }
    fn typescript_name() -> &'static str {
        "Rectangle"
    }
}

impl RpcParam for Rectangle {}
impl RpcReturn for Rectangle {}

/// Struct with Vec field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Polygon {
    pub vertices: Vec<Point>,
}

impl TypeScriptType for Polygon {
    fn typescript_type() -> String {
        "{ vertices: Point[] }".to_string()
    }
    fn typescript_name() -> &'static str {
        "Polygon"
    }
}

impl RpcParam for Polygon {}
impl RpcReturn for Polygon {}

/// Struct with HashMap field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl TypeScriptType for Config {
    fn typescript_type() -> String {
        "{ settings: Record<string, string> }".to_string()
    }
    fn typescript_name() -> &'static str {
        "Config"
    }
}

impl RpcParam for Config {}
impl RpcReturn for Config {}

// ============================================================================
// ENUM FIXTURES
// ============================================================================

/// Simple unit-variant enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Pending,
    Active,
    Completed,
    Failed,
}

impl TypeScriptType for Status {
    fn typescript_type() -> String {
        r#""Pending" | "Active" | "Completed" | "Failed""#.to_string()
    }
    fn typescript_name() -> &'static str {
        "Status"
    }
}

impl RpcParam for Status {}
impl RpcReturn for Status {}

/// Enum with tuple variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Coordinate {
    D2(f64, f64),
    D3(f64, f64, f64),
}

impl TypeScriptType for Coordinate {
    fn typescript_type() -> String {
        r#"{ type: "D2"; value: [number, number] } | { type: "D3"; value: [number, number, number] }"#.to_string()
    }
    fn typescript_name() -> &'static str {
        "Coordinate"
    }
}

impl RpcParam for Coordinate {}
impl RpcReturn for Coordinate {}

/// Enum with struct variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { top_left: Point, width: f64, height: f64 },
    Triangle { a: Point, b: Point, c: Point },
}

impl TypeScriptType for Shape {
    fn typescript_type() -> String {
        r#"{ type: "Circle"; center: Point; radius: number } | { type: "Rectangle"; top_left: Point; width: number; height: number } | { type: "Triangle"; a: Point; b: Point; c: Point }"#.to_string()
    }
    fn typescript_name() -> &'static str {
        "Shape"
    }
}

impl RpcParam for Shape {}
impl RpcReturn for Shape {}

/// Mixed variant enum (unit, tuple, and struct variants)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Message {
    Ping,
    Text(String),
    Binary(Vec<u8>),
    Error { code: i32, message: String },
}

impl TypeScriptType for Message {
    fn typescript_type() -> String {
        r#"{ type: "Ping" } | { type: "Text"; value: string } | { type: "Binary"; value: number[] } | { type: "Error"; code: number; message: string }"#.to_string()
    }
    fn typescript_name() -> &'static str {
        "Message"
    }
}

impl RpcParam for Message {}
impl RpcReturn for Message {}

/// Optional enum wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptionalValue<T> {
    None,
    Some(T),
}

impl<T: TypeScriptType> TypeScriptType for OptionalValue<T> {
    fn typescript_type() -> String {
        format!(
            r#"{{ type: "None" }} | {{ type: "Some"; value: {} }}"#,
            T::typescript_type()
        )
    }
    fn typescript_name() -> &'static str {
        "OptionalValue"
    }
}

impl<T: TypeScriptType> RpcParam for OptionalValue<T> {}
impl<T: TypeScriptType> RpcReturn for OptionalValue<T> {}

// ============================================================================
// RPC REQUEST/RESPONSE FIXTURES
// ============================================================================

/// Typical RPC request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetUserRequest {
    pub user_id: u64,
}

impl TypeScriptType for GetUserRequest {
    fn typescript_type() -> String {
        "{ user_id: number }".to_string()
    }
    fn typescript_name() -> &'static str {
        "GetUserRequest"
    }
}

impl RpcParam for GetUserRequest {}

/// Typical RPC response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetUserResponse {
    pub user: Option<User>,
}

impl TypeScriptType for GetUserResponse {
    fn typescript_type() -> String {
        "{ user: User | null }".to_string()
    }
    fn typescript_name() -> &'static str {
        "GetUserResponse"
    }
}

impl RpcReturn for GetUserResponse {}

/// List request with pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListUsersRequest {
    pub page: u32,
    pub per_page: u32,
    pub filter: Option<String>,
}

impl TypeScriptType for ListUsersRequest {
    fn typescript_type() -> String {
        "{ page: number; per_page: number; filter: string | null }".to_string()
    }
    fn typescript_name() -> &'static str {
        "ListUsersRequest"
    }
}

impl RpcParam for ListUsersRequest {}

/// List response with pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListUsersResponse {
    pub users: Vec<User>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

impl TypeScriptType for ListUsersResponse {
    fn typescript_type() -> String {
        "{ users: User[]; total: number; page: number; per_page: number }".to_string()
    }
    fn typescript_name() -> &'static str {
        "ListUsersResponse"
    }
}

impl RpcReturn for ListUsersResponse {}

// ============================================================================
// ERROR TYPE FIXTURES
// ============================================================================

/// Simple error type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl TypeScriptType for ApiError {
    fn typescript_type() -> String {
        "{ code: string; message: string }".to_string()
    }
    fn typescript_name() -> &'static str {
        "ApiError"
    }
}

impl RpcReturn for ApiError {}

/// Detailed error with optional fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailedError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub field: Option<String>,
}

impl TypeScriptType for DetailedError {
    fn typescript_type() -> String {
        "{ code: string; message: string; details: string | null; field: string | null }".to_string()
    }
    fn typescript_name() -> &'static str {
        "DetailedError"
    }
}

impl RpcReturn for DetailedError {}

/// Error enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RpcError {
    NotFound { resource: String },
    Unauthorized,
    Forbidden { reason: String },
    BadRequest { field: String, message: String },
    Internal,
}

impl TypeScriptType for RpcError {
    fn typescript_type() -> String {
        r#"{ type: "NotFound"; resource: string } | { type: "Unauthorized" } | { type: "Forbidden"; reason: string } | { type: "BadRequest"; field: string; message: string } | { type: "Internal" }"#.to_string()
    }
    fn typescript_name() -> &'static str {
        "RpcError"
    }
}

impl RpcReturn for RpcError {}

// ============================================================================
// COMPLEX NESTED FIXTURES
// ============================================================================

/// Deeply nested type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub id: u64,
    pub name: String,
    pub owner: User,
    pub members: Vec<User>,
    pub settings: Config,
    pub status: Status,
}

impl TypeScriptType for Workspace {
    fn typescript_type() -> String {
        "{ id: number; name: string; owner: User; members: User[]; settings: Config; status: Status }".to_string()
    }
    fn typescript_name() -> &'static str {
        "Workspace"
    }
}

impl RpcParam for Workspace {}
impl RpcReturn for Workspace {}

/// Type with all common patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteExample {
    pub primitive: i32,
    pub string: String,
    pub optional: Option<String>,
    pub list: Vec<i32>,
    pub map: HashMap<String, i32>,
    pub nested: User,
    pub nested_list: Vec<User>,
    pub optional_nested: Option<User>,
    pub status: Status,
}

impl TypeScriptType for CompleteExample {
    fn typescript_type() -> String {
        "{ primitive: number; string: string; optional: string | null; list: number[]; map: Record<string, number>; nested: User; nested_list: User[]; optional_nested: User | null; status: Status }".to_string()
    }
    fn typescript_name() -> &'static str {
        "CompleteExample"
    }
}

impl RpcParam for CompleteExample {}
impl RpcReturn for CompleteExample {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_types() {
        assert_eq!(Point::typescript_type(), "{ x: number; y: number }");
        assert_eq!(Rgb::typescript_type(), "[number, number, number]");
        assert_eq!(Ping::typescript_type(), "null");
        assert_eq!(UserId::typescript_type(), "number");
    }

    #[test]
    fn test_enum_types() {
        assert_eq!(
            Status::typescript_type(),
            r#""Pending" | "Active" | "Completed" | "Failed""#
        );
    }

    #[test]
    fn test_generic_fixture() {
        assert_eq!(
            <OptionalValue<String>>::typescript_type(),
            r#"{ type: "None" } | { type: "Some"; value: string }"#
        );
    }

    #[test]
    fn test_serialization_roundtrip() {
        let point = Point { x: 1.0, y: 2.0 };
        let json = serde_json::to_string(&point).unwrap();
        let parsed: Point = serde_json::from_str(&json).unwrap();
        assert_eq!(point, parsed);

        let user = User {
            id: 1,
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
            active: true,
        };
        let json = serde_json::to_string(&user).unwrap();
        let parsed: User = serde_json::from_str(&json).unwrap();
        assert_eq!(user, parsed);

        let status = Status::Active;
        let json = serde_json::to_string(&status).unwrap();
        let parsed: Status = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }
}
