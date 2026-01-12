//! ferro-type-import: TypeScript-to-Rust type code generation
//!
//! This crate parses TypeScript type definitions and generates equivalent
//! Rust code with serde derives for serialization compatibility.
//!
//! # Usage
//!
//! ```ignore
//! use ferro_type_import::{parse_typescript, generate_rust};
//!
//! let ts_source = r#"
//!     interface User {
//!         id: string;
//!         name: string;
//!         age?: number;
//!     }
//! "#;
//!
//! let rust_code = generate_rust(ts_source)?;
//! // Outputs:
//! // #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//! // pub struct User {
//! //     pub id: String,
//! //     pub name: String,
//! //     #[serde(skip_serializing_if = "Option::is_none")]
//! //     pub age: Option<f64>,
//! // }
//! ```

mod parser;
mod converter;
mod codegen;

pub use parser::parse_typescript;
pub use converter::convert_module;
pub use codegen::generate_rust_from_types;

use ferro_type::TypeDef;

/// Parse TypeScript source and generate Rust code.
///
/// This is the main entry point for converting TypeScript types to Rust.
pub fn generate_rust(source: &str) -> Result<String, String> {
    let module = parse_typescript(source)?;
    let types = convert_module(&module);
    Ok(generate_rust_from_types(&types))
}

/// Parsed TypeScript type information ready for code generation.
#[derive(Debug, Clone)]
pub struct TsTypeInfo {
    /// The name of the type
    pub name: String,
    /// The ferrotype TypeDef representation
    pub typedef: TypeDef,
    /// Whether this is an interface (struct) or type alias
    pub is_interface: bool,
}
