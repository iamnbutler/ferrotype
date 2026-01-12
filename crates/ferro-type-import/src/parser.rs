//! TypeScript parsing using swc_ecma_parser
//!
//! This module provides parsing of TypeScript source files into swc's AST.

use swc_core::common::{sync::Lrc, FileName, SourceMap};
use swc_core::ecma::ast::{EsVersion, Module};
use swc_core::ecma::parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

/// Parse TypeScript source code into an AST Module.
///
/// # Arguments
///
/// * `source` - The TypeScript source code to parse
///
/// # Returns
///
/// A parsed `Module` on success, or an error message on failure.
///
/// # Example
///
/// ```ignore
/// use ferro_type_import::parse_typescript;
///
/// let source = "interface User { name: string; }";
/// let module = parse_typescript(source)?;
/// ```
pub fn parse_typescript(source: &str) -> Result<Module, String> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(Lrc::new(FileName::Custom("input.ts".into())), source.to_string());

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: false,
            decorators: true,
            dts: true, // Enable .d.ts mode for declaration files
            no_early_errors: false,
            ..Default::default()
        }),
        EsVersion::latest(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    parser.parse_module().map_err(|e| format!("Parse error: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interface() {
        let source = r#"
            interface User {
                id: string;
                name: string;
            }
        "#;
        let result = parse_typescript(source);
        assert!(result.is_ok(), "Failed to parse interface: {:?}", result.err());
    }

    #[test]
    fn test_parse_type_alias() {
        let source = "type UserId = string;";
        let result = parse_typescript(source);
        assert!(result.is_ok(), "Failed to parse type alias: {:?}", result.err());
    }

    #[test]
    fn test_parse_optional_field() {
        let source = r#"
            interface Config {
                required: string;
                optional?: number;
            }
        "#;
        let result = parse_typescript(source);
        assert!(result.is_ok(), "Failed to parse optional field: {:?}", result.err());
    }

    #[test]
    fn test_parse_array_type() {
        let source = r#"
            interface Container {
                items: string[];
            }
        "#;
        let result = parse_typescript(source);
        assert!(result.is_ok(), "Failed to parse array type: {:?}", result.err());
    }

    #[test]
    fn test_parse_union_type() {
        let source = "type Result = string | number | null;";
        let result = parse_typescript(source);
        assert!(result.is_ok(), "Failed to parse union type: {:?}", result.err());
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let source = "interface { }"; // Missing name
        let result = parse_typescript(source);
        assert!(result.is_err(), "Should fail on invalid syntax");
    }
}
