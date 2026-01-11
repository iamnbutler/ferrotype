//! Ferrotype Test Fixtures
//!
//! Comprehensive Rust types for testing TypeScript codegen.
//! These fixtures cover the full range of type patterns that
//! ferrotype must handle correctly.

use ferro_type::{Field, Primitive, TypeDef, TypeScript};
use ferro_type_derive::TypeScript as DeriveTypeScript;
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

impl TypeScript for Point {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Point".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("x", TypeDef::Primitive(Primitive::Number)),
                Field::new("y", TypeDef::Primitive(Primitive::Number)),
            ])),
            module: None,
        }
    }
}

/// Struct with multiple field types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

impl TypeScript for User {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "User".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("id", TypeDef::Primitive(Primitive::Number)),
                Field::new("name", TypeDef::Primitive(Primitive::String)),
                Field::new("email", TypeDef::Primitive(Primitive::String)),
                Field::new("active", TypeDef::Primitive(Primitive::Boolean)),
            ])),
            module: None,
        }
    }
}

/// Struct with optional fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

impl TypeScript for Profile {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Profile".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("username", TypeDef::Primitive(Primitive::String)),
                Field::new("display_name", TypeDef::Union(vec![
                    TypeDef::Primitive(Primitive::String),
                    TypeDef::Primitive(Primitive::Null),
                ])),
                Field::new("bio", TypeDef::Union(vec![
                    TypeDef::Primitive(Primitive::String),
                    TypeDef::Primitive(Primitive::Null),
                ])),
                Field::new("avatar_url", TypeDef::Union(vec![
                    TypeDef::Primitive(Primitive::String),
                    TypeDef::Primitive(Primitive::Null),
                ])),
            ])),
            module: None,
        }
    }
}

/// Tuple struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl TypeScript for Rgb {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Rgb".to_string(),
            def: Box::new(TypeDef::Tuple(vec![
                TypeDef::Primitive(Primitive::Number),
                TypeDef::Primitive(Primitive::Number),
                TypeDef::Primitive(Primitive::Number),
            ])),
            module: None,
        }
    }
}

/// Unit struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ping;

impl TypeScript for Ping {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Ping".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::Null)),
            module: None,
        }
    }
}

/// Newtype wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserId(pub u64);

impl TypeScript for UserId {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "UserId".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::Number)),
            module: None,
        }
    }
}

/// Nested struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rectangle {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl TypeScript for Rectangle {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Rectangle".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("top_left", TypeDef::Ref("Point".to_string())),
                Field::new("bottom_right", TypeDef::Ref("Point".to_string())),
            ])),
            module: None,
        }
    }
}

/// Struct with Vec field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Polygon {
    pub vertices: Vec<Point>,
}

impl TypeScript for Polygon {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Polygon".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("vertices", TypeDef::Array(Box::new(TypeDef::Ref("Point".to_string())))),
            ])),
            module: None,
        }
    }
}

/// Struct with HashMap field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl TypeScript for Config {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Config".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("settings", TypeDef::Record {
                    key: Box::new(TypeDef::Primitive(Primitive::String)),
                    value: Box::new(TypeDef::Primitive(Primitive::String)),
                }),
            ])),
            module: None,
        }
    }
}

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

impl TypeScript for Status {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Status".to_string(),
            def: Box::new(TypeDef::Union(vec![
                TypeDef::Literal(ferro_type::Literal::String("Pending".to_string())),
                TypeDef::Literal(ferro_type::Literal::String("Active".to_string())),
                TypeDef::Literal(ferro_type::Literal::String("Completed".to_string())),
                TypeDef::Literal(ferro_type::Literal::String("Failed".to_string())),
            ])),
            module: None,
        }
    }
}

/// Enum with tuple variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Coordinate {
    D2(f64, f64),
    D3(f64, f64, f64),
}

impl TypeScript for Coordinate {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Coordinate".to_string(),
            def: Box::new(TypeDef::Union(vec![
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("D2".to_string()))),
                    Field::new("value", TypeDef::Tuple(vec![
                        TypeDef::Primitive(Primitive::Number),
                        TypeDef::Primitive(Primitive::Number),
                    ])),
                ]),
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("D3".to_string()))),
                    Field::new("value", TypeDef::Tuple(vec![
                        TypeDef::Primitive(Primitive::Number),
                        TypeDef::Primitive(Primitive::Number),
                        TypeDef::Primitive(Primitive::Number),
                    ])),
                ]),
            ])),
            module: None,
        }
    }
}

/// Enum with struct variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { top_left: Point, width: f64, height: f64 },
    Triangle { a: Point, b: Point, c: Point },
}

/// Mixed variant enum (unit, tuple, and struct variants)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Message {
    Ping,
    Text(String),
    Binary(Vec<u8>),
    Error { code: i32, message: String },
}

impl TypeScript for Message {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "Message".to_string(),
            def: Box::new(TypeDef::Union(vec![
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("Ping".to_string()))),
                ]),
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("Text".to_string()))),
                    Field::new("value", TypeDef::Primitive(Primitive::String)),
                ]),
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("Binary".to_string()))),
                    Field::new("value", TypeDef::Array(Box::new(TypeDef::Primitive(Primitive::Number)))),
                ]),
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("Error".to_string()))),
                    Field::new("code", TypeDef::Primitive(Primitive::Number)),
                    Field::new("message", TypeDef::Primitive(Primitive::String)),
                ]),
            ])),
            module: None,
        }
    }
}

/// Optional enum wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptionalValue<T> {
    None,
    Some(T),
}

// ============================================================================
// RPC REQUEST/RESPONSE FIXTURES
// ============================================================================

/// Typical RPC request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetUserRequest {
    pub user_id: u64,
}

impl TypeScript for GetUserRequest {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "GetUserRequest".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("user_id", TypeDef::Primitive(Primitive::Number)),
            ])),
            module: None,
        }
    }
}

/// Typical RPC response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetUserResponse {
    pub user: Option<User>,
}

impl TypeScript for GetUserResponse {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "GetUserResponse".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("user", TypeDef::Union(vec![
                    TypeDef::Ref("User".to_string()),
                    TypeDef::Primitive(Primitive::Null),
                ])),
            ])),
            module: None,
        }
    }
}

/// List request with pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListUsersRequest {
    pub page: u32,
    pub per_page: u32,
    pub filter: Option<String>,
}

impl TypeScript for ListUsersRequest {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "ListUsersRequest".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("page", TypeDef::Primitive(Primitive::Number)),
                Field::new("per_page", TypeDef::Primitive(Primitive::Number)),
                Field::new("filter", TypeDef::Union(vec![
                    TypeDef::Primitive(Primitive::String),
                    TypeDef::Primitive(Primitive::Null),
                ])),
            ])),
            module: None,
        }
    }
}

/// List response with pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListUsersResponse {
    pub users: Vec<User>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

impl TypeScript for ListUsersResponse {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "ListUsersResponse".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("users", TypeDef::Array(Box::new(TypeDef::Ref("User".to_string())))),
                Field::new("total", TypeDef::Primitive(Primitive::Number)),
                Field::new("page", TypeDef::Primitive(Primitive::Number)),
                Field::new("per_page", TypeDef::Primitive(Primitive::Number)),
            ])),
            module: None,
        }
    }
}

// ============================================================================
// ERROR TYPE FIXTURES
// ============================================================================

/// Simple error type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl TypeScript for ApiError {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "ApiError".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("code", TypeDef::Primitive(Primitive::String)),
                Field::new("message", TypeDef::Primitive(Primitive::String)),
            ])),
            module: None,
        }
    }
}

/// Detailed error with optional fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailedError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub field: Option<String>,
}

impl TypeScript for DetailedError {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "DetailedError".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("code", TypeDef::Primitive(Primitive::String)),
                Field::new("message", TypeDef::Primitive(Primitive::String)),
                Field::new("details", TypeDef::Union(vec![
                    TypeDef::Primitive(Primitive::String),
                    TypeDef::Primitive(Primitive::Null),
                ])),
                Field::new("field", TypeDef::Union(vec![
                    TypeDef::Primitive(Primitive::String),
                    TypeDef::Primitive(Primitive::Null),
                ])),
            ])),
            module: None,
        }
    }
}

/// Error enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RpcError {
    NotFound { resource: String },
    Unauthorized,
    Forbidden { reason: String },
    BadRequest { field: String, message: String },
    Internal,
}

impl TypeScript for RpcError {
    fn typescript() -> TypeDef {
        TypeDef::Named {
            name: "RpcError".to_string(),
            def: Box::new(TypeDef::Union(vec![
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("NotFound".to_string()))),
                    Field::new("resource", TypeDef::Primitive(Primitive::String)),
                ]),
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("Unauthorized".to_string()))),
                ]),
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("Forbidden".to_string()))),
                    Field::new("reason", TypeDef::Primitive(Primitive::String)),
                ]),
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("BadRequest".to_string()))),
                    Field::new("field", TypeDef::Primitive(Primitive::String)),
                    Field::new("message", TypeDef::Primitive(Primitive::String)),
                ]),
                TypeDef::Object(vec![
                    Field::new("type", TypeDef::Literal(ferro_type::Literal::String("Internal".to_string()))),
                ]),
            ])),
            module: None,
        }
    }
}

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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_typescript() {
        let td = Point::typescript();
        assert_eq!(td.render(), "Point");
    }

    #[test]
    fn test_status_typescript() {
        let td = Status::typescript();
        assert_eq!(td.render(), "Status");
    }

    // ========================================================================
    // ROUNDTRIP SERIALIZATION TESTS
    // ========================================================================

    mod roundtrip {
        use super::*;

        /// Helper to verify roundtrip: Rust -> JSON -> Rust
        fn assert_roundtrip<T>(value: T)
        where
            T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
        {
            let json = serde_json::to_string(&value).expect("serialize");
            let parsed: T = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(value, parsed, "roundtrip failed for JSON: {}", json);
        }

        /// Helper to verify JSON format matches expected structure
        fn assert_json_format<T: Serialize>(value: &T, expected: &str) {
            let json = serde_json::to_string(value).expect("serialize");
            let actual: serde_json::Value = serde_json::from_str(&json).unwrap();
            let expected: serde_json::Value = serde_json::from_str(expected).unwrap();
            assert_eq!(actual, expected, "JSON format mismatch");
        }

        // --------------------------------------------------------------------
        // STRUCT ROUNDTRIP TESTS
        // --------------------------------------------------------------------

        #[test]
        fn test_point_roundtrip() {
            assert_roundtrip(Point { x: 0.0, y: 0.0 });
            assert_roundtrip(Point { x: -1.5, y: 2.5 });
            assert_roundtrip(Point { x: f64::MAX, y: f64::MIN });
            assert_json_format(&Point { x: 1.0, y: 2.0 }, r#"{"x":1.0,"y":2.0}"#);
        }

        #[test]
        fn test_user_roundtrip() {
            let user = User {
                id: 12345,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                active: true,
            };
            assert_roundtrip(user.clone());
            assert_json_format(
                &user,
                r#"{"id":12345,"name":"Alice","email":"alice@example.com","active":true}"#,
            );

            // Test with special characters
            assert_roundtrip(User {
                id: 0,
                name: "名前".to_string(),
                email: "test+tag@example.com".to_string(),
                active: false,
            });
        }

        #[test]
        fn test_profile_roundtrip() {
            // All fields present
            assert_roundtrip(Profile {
                username: "alice".to_string(),
                display_name: Some("Alice Smith".to_string()),
                bio: Some("Hello world".to_string()),
                avatar_url: Some("https://example.com/avatar.png".to_string()),
            });

            // All optional fields null
            let minimal = Profile {
                username: "bob".to_string(),
                display_name: None,
                bio: None,
                avatar_url: None,
            };
            assert_roundtrip(minimal.clone());
            assert_json_format(
                &minimal,
                r#"{"username":"bob","display_name":null,"bio":null,"avatar_url":null}"#,
            );

            // Mixed
            assert_roundtrip(Profile {
                username: "charlie".to_string(),
                display_name: Some("Charlie".to_string()),
                bio: None,
                avatar_url: Some("https://example.com/c.png".to_string()),
            });
        }

        #[test]
        fn test_rgb_roundtrip() {
            assert_roundtrip(Rgb(0, 0, 0));
            assert_roundtrip(Rgb(255, 255, 255));
            assert_roundtrip(Rgb(128, 64, 32));
            assert_json_format(&Rgb(255, 128, 0), "[255,128,0]");
        }

        #[test]
        fn test_ping_roundtrip() {
            assert_roundtrip(Ping);
            assert_json_format(&Ping, "null");
        }

        #[test]
        fn test_user_id_roundtrip() {
            assert_roundtrip(UserId(0));
            assert_roundtrip(UserId(u64::MAX));
            assert_json_format(&UserId(42), "42");
        }

        #[test]
        fn test_rectangle_roundtrip() {
            let rect = Rectangle {
                top_left: Point { x: 0.0, y: 10.0 },
                bottom_right: Point { x: 10.0, y: 0.0 },
            };
            assert_roundtrip(rect.clone());
            assert_json_format(
                &rect,
                r#"{"top_left":{"x":0.0,"y":10.0},"bottom_right":{"x":10.0,"y":0.0}}"#,
            );
        }

        #[test]
        fn test_polygon_roundtrip() {
            // Empty
            assert_roundtrip(Polygon { vertices: vec![] });

            // Single point
            assert_roundtrip(Polygon {
                vertices: vec![Point { x: 0.0, y: 0.0 }],
            });

            // Triangle
            let triangle = Polygon {
                vertices: vec![
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 1.0, y: 0.0 },
                    Point { x: 0.5, y: 1.0 },
                ],
            };
            assert_roundtrip(triangle);
        }

        #[test]
        fn test_config_roundtrip() {
            // Empty
            assert_roundtrip(Config {
                settings: HashMap::new(),
            });

            // With values
            let mut settings = HashMap::new();
            settings.insert("theme".to_string(), "dark".to_string());
            settings.insert("language".to_string(), "en".to_string());
            assert_roundtrip(Config { settings });
        }

        // --------------------------------------------------------------------
        // ENUM ROUNDTRIP TESTS
        // --------------------------------------------------------------------

        #[test]
        fn test_status_roundtrip() {
            assert_roundtrip(Status::Pending);
            assert_roundtrip(Status::Active);
            assert_roundtrip(Status::Completed);
            assert_roundtrip(Status::Failed);

            // Verify JSON format (serde default for unit variants)
            assert_json_format(&Status::Active, r#""Active""#);
        }

        #[test]
        fn test_coordinate_roundtrip() {
            assert_roundtrip(Coordinate::D2(1.0, 2.0));
            assert_roundtrip(Coordinate::D3(1.0, 2.0, 3.0));

            // Verify JSON format (serde externally tagged by default)
            assert_json_format(&Coordinate::D2(1.0, 2.0), r#"{"D2":[1.0,2.0]}"#);
            assert_json_format(&Coordinate::D3(1.0, 2.0, 3.0), r#"{"D3":[1.0,2.0,3.0]}"#);
        }

        #[test]
        fn test_shape_roundtrip() {
            assert_roundtrip(Shape::Circle {
                center: Point { x: 0.0, y: 0.0 },
                radius: 5.0,
            });

            assert_roundtrip(Shape::Rectangle {
                top_left: Point { x: 0.0, y: 10.0 },
                width: 10.0,
                height: 10.0,
            });

            assert_roundtrip(Shape::Triangle {
                a: Point { x: 0.0, y: 0.0 },
                b: Point { x: 1.0, y: 0.0 },
                c: Point { x: 0.5, y: 1.0 },
            });
        }

        #[test]
        fn test_message_roundtrip() {
            assert_roundtrip(Message::Ping);
            assert_roundtrip(Message::Text("Hello, world!".to_string()));
            assert_roundtrip(Message::Binary(vec![0, 1, 2, 255]));
            assert_roundtrip(Message::Error {
                code: 500,
                message: "Internal error".to_string(),
            });

            // Empty cases
            assert_roundtrip(Message::Text(String::new()));
            assert_roundtrip(Message::Binary(vec![]));
        }

        #[test]
        fn test_optional_value_roundtrip() {
            assert_roundtrip(OptionalValue::<String>::None);
            assert_roundtrip(OptionalValue::Some("value".to_string()));
            assert_roundtrip(OptionalValue::Some(42i32));
            assert_roundtrip(OptionalValue::Some(Point { x: 1.0, y: 2.0 }));
        }

        // --------------------------------------------------------------------
        // RPC TYPE ROUNDTRIP TESTS
        // --------------------------------------------------------------------

        #[test]
        fn test_get_user_request_roundtrip() {
            assert_roundtrip(GetUserRequest { user_id: 123 });
            assert_roundtrip(GetUserRequest { user_id: 0 });
            assert_roundtrip(GetUserRequest { user_id: u64::MAX });
        }

        #[test]
        fn test_get_user_response_roundtrip() {
            // User present
            assert_roundtrip(GetUserResponse {
                user: Some(User {
                    id: 1,
                    name: "Test".to_string(),
                    email: "test@example.com".to_string(),
                    active: true,
                }),
            });

            // User absent
            assert_roundtrip(GetUserResponse { user: None });
        }

        #[test]
        fn test_list_users_request_roundtrip() {
            assert_roundtrip(ListUsersRequest {
                page: 1,
                per_page: 20,
                filter: None,
            });

            assert_roundtrip(ListUsersRequest {
                page: 5,
                per_page: 100,
                filter: Some("active".to_string()),
            });
        }

        #[test]
        fn test_list_users_response_roundtrip() {
            // Empty
            assert_roundtrip(ListUsersResponse {
                users: vec![],
                total: 0,
                page: 1,
                per_page: 20,
            });

            // With users
            assert_roundtrip(ListUsersResponse {
                users: vec![
                    User {
                        id: 1,
                        name: "Alice".to_string(),
                        email: "alice@example.com".to_string(),
                        active: true,
                    },
                    User {
                        id: 2,
                        name: "Bob".to_string(),
                        email: "bob@example.com".to_string(),
                        active: false,
                    },
                ],
                total: 100,
                page: 1,
                per_page: 2,
            });
        }

        // --------------------------------------------------------------------
        // ERROR TYPE ROUNDTRIP TESTS
        // --------------------------------------------------------------------

        #[test]
        fn test_api_error_roundtrip() {
            assert_roundtrip(ApiError {
                code: "NOT_FOUND".to_string(),
                message: "Resource not found".to_string(),
            });
        }

        #[test]
        fn test_detailed_error_roundtrip() {
            assert_roundtrip(DetailedError {
                code: "VALIDATION_ERROR".to_string(),
                message: "Invalid input".to_string(),
                details: Some("Field 'email' is invalid".to_string()),
                field: Some("email".to_string()),
            });

            assert_roundtrip(DetailedError {
                code: "UNKNOWN".to_string(),
                message: "Unknown error".to_string(),
                details: None,
                field: None,
            });
        }

        #[test]
        fn test_rpc_error_roundtrip() {
            assert_roundtrip(RpcError::NotFound {
                resource: "user/123".to_string(),
            });
            assert_roundtrip(RpcError::Unauthorized);
            assert_roundtrip(RpcError::Forbidden {
                reason: "Insufficient permissions".to_string(),
            });
            assert_roundtrip(RpcError::BadRequest {
                field: "email".to_string(),
                message: "Invalid format".to_string(),
            });
            assert_roundtrip(RpcError::Internal);
        }

        // --------------------------------------------------------------------
        // COMPLEX TYPE ROUNDTRIP TESTS
        // --------------------------------------------------------------------

        #[test]
        fn test_workspace_roundtrip() {
            let mut settings = HashMap::new();
            settings.insert("visibility".to_string(), "private".to_string());

            assert_roundtrip(Workspace {
                id: 1,
                name: "My Workspace".to_string(),
                owner: User {
                    id: 1,
                    name: "Owner".to_string(),
                    email: "owner@example.com".to_string(),
                    active: true,
                },
                members: vec![
                    User {
                        id: 2,
                        name: "Member".to_string(),
                        email: "member@example.com".to_string(),
                        active: true,
                    },
                ],
                settings: Config { settings },
                status: Status::Active,
            });
        }

        #[test]
        fn test_complete_example_roundtrip() {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), 100);
            map.insert("key2".to_string(), 200);

            assert_roundtrip(CompleteExample {
                primitive: 42,
                string: "test".to_string(),
                optional: Some("present".to_string()),
                list: vec![1, 2, 3],
                map,
                nested: User {
                    id: 1,
                    name: "Test".to_string(),
                    email: "test@example.com".to_string(),
                    active: true,
                },
                nested_list: vec![User {
                    id: 2,
                    name: "Another".to_string(),
                    email: "another@example.com".to_string(),
                    active: false,
                }],
                optional_nested: None,
                status: Status::Pending,
            });
        }

        // --------------------------------------------------------------------
        // EDGE CASE TESTS
        // --------------------------------------------------------------------

        #[test]
        fn test_special_characters() {
            // Unicode
            assert_roundtrip(User {
                id: 1,
                name: "日本語 🎉".to_string(),
                email: "test@例え.jp".to_string(),
                active: true,
            });

            // Escape sequences
            assert_roundtrip(User {
                id: 1,
                name: "Tab\tNewline\nQuote\"".to_string(),
                email: "test@example.com".to_string(),
                active: true,
            });
        }

        #[test]
        fn test_numeric_edge_cases() {
            // Zero
            assert_roundtrip(Point { x: 0.0, y: -0.0 });

            // Very small/large
            assert_roundtrip(Point {
                x: f64::MIN_POSITIVE,
                y: f64::MAX,
            });

            // Integer limits
            assert_roundtrip(User {
                id: u64::MAX,
                name: "Max".to_string(),
                email: "max@example.com".to_string(),
                active: true,
            });
        }

        #[test]
        fn test_empty_collections() {
            assert_roundtrip(Polygon { vertices: vec![] });
            assert_roundtrip(Config {
                settings: HashMap::new(),
            });
            assert_roundtrip(ListUsersResponse {
                users: vec![],
                total: 0,
                page: 1,
                per_page: 20,
            });
        }

        #[test]
        fn test_large_collections() {
            // Large vec
            let vertices: Vec<Point> = (0..1000)
                .map(|i| Point {
                    x: i as f64,
                    y: (i * 2) as f64,
                })
                .collect();
            assert_roundtrip(Polygon { vertices });

            // Large hashmap
            let settings: HashMap<String, String> = (0..100)
                .map(|i| (format!("key{}", i), format!("value{}", i)))
                .collect();
            assert_roundtrip(Config { settings });
        }
    }
}

// ============================================================================
// DERIVED ENUM FIXTURES (using #[derive(TypeScript)])
// ============================================================================

/// Unit variant enum - derived
#[derive(Debug, Clone, DeriveTypeScript)]
pub enum DerivedStatus {
    Pending,
    Active,
    Completed,
    Failed,
}

/// Tuple variant enum - derived
#[derive(Debug, Clone, DeriveTypeScript)]
pub enum DerivedCoordinate {
    D2(f64, f64),
    D3(f64, f64, f64),
}

/// Struct variant enum - derived
#[derive(Debug, Clone, DeriveTypeScript)]
pub enum DerivedShape {
    Circle { center: Point, radius: f64 },
    Rectangle { top_left: Point, width: f64, height: f64 },
    Triangle { a: Point, b: Point, c: Point },
}

/// Mixed variant enum - derived
#[derive(Debug, Clone, DeriveTypeScript)]
pub enum DerivedMessage {
    Ping,
    Text(String),
    Binary(Vec<u8>),
    Error { code: i32, message: String },
}

/// Generic enum - derived
#[derive(Debug, Clone, DeriveTypeScript)]
pub enum DerivedOptionalValue<T: TypeScript> {
    None,
    Some(T),
}

/// Error enum - derived
#[derive(Debug, Clone, DeriveTypeScript)]
pub enum DerivedRpcError {
    NotFound { resource: String },
    Unauthorized,
    Forbidden { reason: String },
    BadRequest { field: String, message: String },
    Internal,
}

#[cfg(test)]
mod derive_tests {
    use super::*;

    /// Helper to get the inner definition from a Named TypeDef
    fn inner_def(td: TypeDef) -> TypeDef {
        match td {
            TypeDef::Named { def, .. } => *def,
            other => other,
        }
    }

    #[test]
    fn test_derived_unit_enum() {
        // Pure unit enums should produce simple string unions
        let td = DerivedStatus::typescript();
        assert_eq!(
            inner_def(td.clone()).render(),
            r#""Pending" | "Active" | "Completed" | "Failed""#
        );
        assert_eq!(td.render(), "DerivedStatus");
    }

    #[test]
    fn test_derived_tuple_enum() {
        let td = DerivedCoordinate::typescript();
        assert_eq!(
            inner_def(td.clone()).render(),
            r#"{ type: "D2"; value: [number, number] } | { type: "D3"; value: [number, number, number] }"#
        );
        assert_eq!(td.render(), "DerivedCoordinate");
    }

    #[test]
    fn test_derived_struct_enum() {
        let td = DerivedShape::typescript();
        // Note: Point now renders as "Point" (named type reference)
        assert_eq!(
            inner_def(td.clone()).render(),
            r#"{ type: "Circle"; center: Point; radius: number } | { type: "Rectangle"; top_left: Point; width: number; height: number } | { type: "Triangle"; a: Point; b: Point; c: Point }"#
        );
        assert_eq!(td.render(), "DerivedShape");
    }

    #[test]
    fn test_derived_mixed_enum() {
        let td = DerivedMessage::typescript();
        assert_eq!(
            inner_def(td.clone()).render(),
            r#"{ type: "Ping" } | { type: "Text"; value: string } | { type: "Binary"; value: number[] } | { type: "Error"; code: number; message: string }"#
        );
        assert_eq!(td.render(), "DerivedMessage");
    }

    #[test]
    fn test_derived_generic_enum() {
        let td = <DerivedOptionalValue<String>>::typescript();
        assert_eq!(
            inner_def(td).render(),
            r#"{ type: "None" } | { type: "Some"; value: string }"#
        );
    }

    #[test]
    fn test_derived_error_enum() {
        let td = DerivedRpcError::typescript();
        assert_eq!(
            inner_def(td).render(),
            r#"{ type: "NotFound"; resource: string } | { type: "Unauthorized" } | { type: "Forbidden"; reason: string } | { type: "BadRequest"; field: string; message: string } | { type: "Internal" }"#
        );
    }

    // Compare derived vs manual implementations
    #[test]
    fn test_derived_matches_manual_status() {
        assert_eq!(
            inner_def(DerivedStatus::typescript()).render(),
            inner_def(Status::typescript()).render()
        );
    }

    #[test]
    fn test_derived_matches_manual_coordinate() {
        assert_eq!(
            inner_def(DerivedCoordinate::typescript()).render(),
            inner_def(Coordinate::typescript()).render()
        );
    }

    #[test]
    fn test_derived_matches_manual_message() {
        assert_eq!(
            inner_def(DerivedMessage::typescript()).render(),
            inner_def(Message::typescript()).render()
        );
    }

    #[test]
    fn test_derived_matches_manual_rpc_error() {
        assert_eq!(
            inner_def(DerivedRpcError::typescript()).render(),
            inner_def(RpcError::typescript()).render()
        );
    }
}
