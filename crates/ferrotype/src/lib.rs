//! Ferrotype: Rust-to-TypeScript type mapping for Zed RPC
//!
//! This crate provides traits for generating TypeScript type definitions
//! from Rust types, enabling type-safe RPC communication.

pub use ferrotype_derive::TypeScript;

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
}
