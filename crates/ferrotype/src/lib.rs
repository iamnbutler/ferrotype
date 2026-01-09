//! Ferrotype: Rust-to-TypeScript type mapping for Zed RPC
//!
//! This crate provides traits for generating TypeScript type definitions
//! from Rust types, enabling type-safe RPC communication.

pub use ferrotype_derive::{rpc_method, TypeScript};

use std::collections::HashMap;

/// Trait for types that can be represented as TypeScript types.
///
/// Implementors provide both a type name (for references) and a full
/// type definition (for inline or anonymous types).
pub trait TypeScriptType {
    /// Returns the TypeScript type definition as a string.
    ///
    /// For primitive types, this returns the type directly (e.g., "number", "string").
    /// For complex types, this returns the full inline type definition.
    fn typescript_type() -> String;

    /// Returns the TypeScript type name for use in references.
    ///
    /// This is typically the name used when the type is exported or referenced
    /// elsewhere (e.g., "MyStruct", "UserResponse").
    fn typescript_name() -> &'static str;
}

/// Marker trait for types that can be used as RPC request parameters.
///
/// Types implementing this trait can be serialized and sent as part of
/// an RPC request payload.
pub trait RpcParam: TypeScriptType {}

/// Marker trait for types that can be returned from RPC calls.
///
/// Types implementing this trait can be deserialized from an RPC response.
pub trait RpcReturn: TypeScriptType {}

/// Marker trait for types that represent RPC errors.
///
/// Types implementing this trait can be used as error types in RPC methods.
/// Error types should typically be enums with discriminated variants or
/// structs with error code/message fields.
pub trait RpcErrorType: TypeScriptType {
    /// Generates TypeScript type guards for this error type.
    ///
    /// For enum error types, this generates guards like `isNotFoundError(err)`.
    /// Returns an empty string if type guards are not applicable.
    fn typescript_type_guards() -> String {
        String::new()
    }
}

/// Information about an error variant for TypeScript codegen.
#[derive(Debug, Clone)]
pub struct ErrorVariantInfo {
    /// The variant/error code name
    pub name: &'static str,
    /// TypeScript type for this error variant
    pub typescript_type: String,
}

impl ErrorVariantInfo {
    /// Creates a new ErrorVariantInfo.
    pub fn new(name: &'static str, typescript_type: String) -> Self {
        Self {
            name,
            typescript_type,
        }
    }

    /// Generates a TypeScript type guard function for this error variant.
    pub fn typescript_type_guard(&self, error_type_name: &str) -> String {
        let guard_name = format!("is{}Error", self.name);
        format!(
            r#"function {}(error: {}): error is {{ type: "{}" }} {{
  return (error as any).type === "{}";
}}"#,
            guard_name, error_type_name, self.name, self.name
        )
    }
}

/// Trait for enum error types that can enumerate their variants.
///
/// This trait enables automatic generation of TypeScript type guards
/// for each error variant.
pub trait EnumerableError: RpcErrorType {
    /// Returns information about all error variants.
    fn error_variants() -> Vec<ErrorVariantInfo>;

    /// Generates TypeScript type guards for all error variants.
    fn generate_all_type_guards() -> String {
        let type_name = Self::typescript_name();
        Self::error_variants()
            .iter()
            .map(|v| v.typescript_type_guard(type_name))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

// ============================================================================
// RPC SERVICE TRAIT
// ============================================================================

/// Information about an RPC method including request/response types.
///
/// This struct captures the metadata needed to generate TypeScript interfaces
/// for RPC methods, including the method name and its parameter/return types.
#[derive(Debug, Clone)]
pub struct RpcMethodInfo {
    /// The method name as it appears in the RPC interface.
    pub name: &'static str,
    /// TypeScript type representation of the request parameters.
    pub request_type: String,
    /// TypeScript type representation of the response.
    pub response_type: String,
}

impl RpcMethodInfo {
    /// Creates a new RpcMethodInfo with the given name and types.
    pub fn new(name: &'static str, request_type: String, response_type: String) -> Self {
        Self {
            name,
            request_type,
            response_type,
        }
    }
}

/// Trait for RPC services that can have methods registered.
///
/// This trait provides the foundation for defining RPC services that can
/// generate TypeScript interfaces. Implementations define the service name
/// and its available methods, enabling automatic TypeScript client generation.
///
/// # Examples
///
/// ```ignore
/// struct UserService;
///
/// impl RpcService for UserService {
///     fn service_name() -> &'static str {
///         "UserService"
///     }
///
///     fn methods() -> Vec<RpcMethodInfo> {
///         vec![
///             RpcMethodInfo {
///                 name: "getUser",
///                 request_type: "{ id: string }".to_string(),
///                 response_type: "User".to_string(),
///             },
///         ]
///     }
/// }
/// ```
pub trait RpcService {
    /// Returns the service name used in TypeScript interface generation.
    fn service_name() -> &'static str;

    /// Returns all registered RPC methods with their type signatures.
    fn methods() -> Vec<RpcMethodInfo>;

    /// Generates the TypeScript interface definition for this service.
    ///
    /// The generated interface includes all methods with their request/response
    /// types wrapped in Promise for async operation.
    fn typescript_interface() -> String {
        let methods: Vec<String> = Self::methods()
            .iter()
            .map(|m| {
                format!(
                    "  {}(request: {}): Promise<{}>",
                    m.name, m.request_type, m.response_type
                )
            })
            .collect();

        format!(
            "interface {} {{\n{}\n}}",
            Self::service_name(),
            methods.join(";\n")
        )
    }
}

// ============================================================================
// PRIMITIVE IMPLEMENTATIONS
// ============================================================================

impl TypeScriptType for () {
    fn typescript_type() -> String {
        "void".to_string()
    }

    fn typescript_name() -> &'static str {
        "void"
    }
}

impl TypeScriptType for bool {
    fn typescript_type() -> String {
        "boolean".to_string()
    }

    fn typescript_name() -> &'static str {
        "boolean"
    }
}

impl TypeScriptType for String {
    fn typescript_type() -> String {
        "string".to_string()
    }

    fn typescript_name() -> &'static str {
        "string"
    }
}

impl TypeScriptType for &str {
    fn typescript_type() -> String {
        "string".to_string()
    }

    fn typescript_name() -> &'static str {
        "string"
    }
}

macro_rules! impl_typescript_number {
    ($($t:ty),*) => {
        $(
            impl TypeScriptType for $t {
                fn typescript_type() -> String {
                    "number".to_string()
                }

                fn typescript_name() -> &'static str {
                    "number"
                }
            }
        )*
    };
}

impl_typescript_number!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

// ============================================================================
// GENERIC TYPE IMPLEMENTATIONS
// ============================================================================

impl<T: TypeScriptType> TypeScriptType for Option<T> {
    fn typescript_type() -> String {
        format!("{} | null", T::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Option"
    }
}

impl<T: TypeScriptType> TypeScriptType for Vec<T> {
    fn typescript_type() -> String {
        format!("{}[]", T::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Array"
    }
}

impl<K: TypeScriptType, V: TypeScriptType> TypeScriptType for HashMap<K, V> {
    fn typescript_type() -> String {
        format!("Record<{}, {}>", K::typescript_type(), V::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Record"
    }
}

impl<T: TypeScriptType, E: TypeScriptType> TypeScriptType for Result<T, E> {
    fn typescript_type() -> String {
        format!(
            "{{ ok: true; value: {} }} | {{ ok: false; error: {} }}",
            T::typescript_type(),
            E::typescript_type()
        )
    }

    fn typescript_name() -> &'static str {
        "Result"
    }
}

impl<T: TypeScriptType> TypeScriptType for Box<T> {
    fn typescript_type() -> String {
        T::typescript_type()
    }

    fn typescript_name() -> &'static str {
        T::typescript_name()
    }
}

// ============================================================================
// TUPLE IMPLEMENTATIONS
// ============================================================================

impl<A: TypeScriptType> TypeScriptType for (A,) {
    fn typescript_type() -> String {
        format!("[{}]", A::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Tuple1"
    }
}

impl<A: TypeScriptType, B: TypeScriptType> TypeScriptType for (A, B) {
    fn typescript_type() -> String {
        format!("[{}, {}]", A::typescript_type(), B::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Tuple2"
    }
}

impl<A: TypeScriptType, B: TypeScriptType, C: TypeScriptType> TypeScriptType for (A, B, C) {
    fn typescript_type() -> String {
        format!(
            "[{}, {}, {}]",
            A::typescript_type(),
            B::typescript_type(),
            C::typescript_type()
        )
    }

    fn typescript_name() -> &'static str {
        "Tuple3"
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        assert_eq!(i32::typescript_type(), "number");
        assert_eq!(String::typescript_type(), "string");
        assert_eq!(bool::typescript_type(), "boolean");
        assert_eq!(<()>::typescript_type(), "void");
    }

    #[test]
    fn test_option_type() {
        assert_eq!(<Option<String>>::typescript_type(), "string | null");
        assert_eq!(<Option<i32>>::typescript_type(), "number | null");
    }

    #[test]
    fn test_vec_type() {
        assert_eq!(<Vec<String>>::typescript_type(), "string[]");
        assert_eq!(<Vec<i32>>::typescript_type(), "number[]");
    }

    #[test]
    fn test_hashmap_type() {
        assert_eq!(
            <HashMap<String, i32>>::typescript_type(),
            "Record<string, number>"
        );
    }

    #[test]
    fn test_result_type() {
        assert_eq!(
            <Result<String, String>>::typescript_type(),
            "{ ok: true; value: string } | { ok: false; error: string }"
        );
    }

    #[test]
    fn test_tuple_types() {
        assert_eq!(<(String,)>::typescript_type(), "[string]");
        assert_eq!(<(String, i32)>::typescript_type(), "[string, number]");
        assert_eq!(<(String, i32, bool)>::typescript_type(), "[string, number, boolean]");
    }

    // RPC Service tests

    struct TestUserService;

    impl RpcService for TestUserService {
        fn service_name() -> &'static str {
            "UserService"
        }

        fn methods() -> Vec<RpcMethodInfo> {
            vec![
                RpcMethodInfo::new(
                    "getUser",
                    "{ id: string }".to_string(),
                    "User".to_string(),
                ),
                RpcMethodInfo::new(
                    "listUsers",
                    "{ page: number }".to_string(),
                    "User[]".to_string(),
                ),
            ]
        }
    }

    #[test]
    fn test_rpc_method_info() {
        let method = RpcMethodInfo::new(
            "testMethod",
            "string".to_string(),
            "number".to_string(),
        );
        assert_eq!(method.name, "testMethod");
        assert_eq!(method.request_type, "string");
        assert_eq!(method.response_type, "number");
    }

    #[test]
    fn test_rpc_service_name() {
        assert_eq!(TestUserService::service_name(), "UserService");
    }

    #[test]
    fn test_rpc_service_methods() {
        let methods = TestUserService::methods();
        assert_eq!(methods.len(), 2);
        assert_eq!(methods[0].name, "getUser");
        assert_eq!(methods[1].name, "listUsers");
    }

    #[test]
    fn test_rpc_service_typescript_interface() {
        let interface = TestUserService::typescript_interface();
        assert!(interface.contains("interface UserService"));
        assert!(interface.contains("getUser(request: { id: string }): Promise<User>"));
        assert!(interface.contains("listUsers(request: { page: number }): Promise<User[]>"));
    }

    // Error type tests

    struct ApiError;

    impl TypeScriptType for ApiError {
        fn typescript_type() -> String {
            "{ code: string; message: string }".to_string()
        }

        fn typescript_name() -> &'static str {
            "ApiError"
        }
    }

    impl RpcErrorType for ApiError {}

    struct TestRpcError;

    impl TypeScriptType for TestRpcError {
        fn typescript_type() -> String {
            r#"{ type: "NotFound"; resource: string } | { type: "Unauthorized" } | { type: "BadRequest"; field: string; message: string }"#.to_string()
        }

        fn typescript_name() -> &'static str {
            "RpcError"
        }
    }

    impl RpcErrorType for TestRpcError {}

    impl EnumerableError for TestRpcError {
        fn error_variants() -> Vec<ErrorVariantInfo> {
            vec![
                ErrorVariantInfo::new(
                    "NotFound",
                    r#"{ type: "NotFound"; resource: string }"#.to_string(),
                ),
                ErrorVariantInfo::new(
                    "Unauthorized",
                    r#"{ type: "Unauthorized" }"#.to_string(),
                ),
                ErrorVariantInfo::new(
                    "BadRequest",
                    r#"{ type: "BadRequest"; field: string; message: string }"#.to_string(),
                ),
            ]
        }
    }

    #[test]
    fn test_error_variant_info() {
        let variant = ErrorVariantInfo::new("NotFound", "{ type: \"NotFound\" }".to_string());
        assert_eq!(variant.name, "NotFound");
    }

    #[test]
    fn test_error_type_guard_generation() {
        let variant = ErrorVariantInfo::new("NotFound", "{ type: \"NotFound\" }".to_string());
        let guard = variant.typescript_type_guard("RpcError");
        assert!(guard.contains("function isNotFoundError"));
        assert!(guard.contains("error: RpcError"));
        assert!(guard.contains("error is { type: \"NotFound\" }"));
        assert!(guard.contains("return (error as any).type === \"NotFound\""));
    }

    #[test]
    fn test_enumerable_error_all_guards() {
        let guards = TestRpcError::generate_all_type_guards();
        assert!(guards.contains("isNotFoundError"));
        assert!(guards.contains("isUnauthorizedError"));
        assert!(guards.contains("isBadRequestError"));
    }

    #[test]
    fn test_rpc_error_type_default_guards() {
        // Default implementation returns empty string
        assert_eq!(ApiError::typescript_type_guards(), "");
    }
}
