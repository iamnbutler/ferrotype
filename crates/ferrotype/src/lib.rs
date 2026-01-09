//! Ferrotype: Rust-to-TypeScript type mapping for Zed RPC
//!
//! This crate provides traits for generating TypeScript type definitions
//! from Rust types, enabling type-safe RPC communication.

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
