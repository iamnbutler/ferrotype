//! TypeScript file generation for ferro-type
//!
//! This crate provides utilities for generating TypeScript definition files
//! from Rust types that implement the `TS` trait.
//!
//! # Example
//!
//! ```ignore
//! use ferro_type::TS;
//! use ferro_type_gen::{Config, Generator, ExportStyle};
//!
//! #[derive(TS)]
//! struct User {
//!     id: String,
//!     name: String,
//! }
//!
//! let mut generator = Generator::new(
//!     Config::new()
//!         .output("types.ts")
//!         .export_style(ExportStyle::Named)
//! );
//!
//! generator.register::<User>();
//! generator.write().expect("Failed to write TypeScript");
//! ```
//!
//! # build.rs Integration
//!
//! ```ignore
//! // build.rs
//! use ferro_type_gen::{Config, Generator};
//!
//! fn main() {
//!     let mut generator = Generator::new(
//!         Config::new().output("../frontend/src/types/api.ts")
//!     );
//!
//!     generator.register::<api::User>()
//!        .register::<api::Post>();
//!
//!     generator.write_if_changed()
//!         .expect("TypeScript generation failed");
//! }
//! ```

use ferro_type::{TypeDef, TypeRegistry, TS};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================================
// UTILITY TYPES
// ============================================================================

/// The Prettify utility type flattens intersection types for better readability.
///
/// Example usage in Rust:
/// ```ignore
/// #[derive(TypeScript)]
/// #[ts(wrapper = "Prettify")]
/// struct User {
///     pub id: String,
///     pub name: String,
/// }
/// ```
///
/// Generates:
/// ```typescript
/// type User = Prettify<{ id: string; name: string }>;
/// ```
pub const PRETTIFY_TYPE: &str = "type Prettify<T> = { [K in keyof T]: T[K] } & {};";

/// Exported version of the Prettify utility type (with export keyword)
pub const PRETTIFY_TYPE_EXPORTED: &str = "export type Prettify<T> = { [K in keyof T]: T[K] } & {};";

/// How to export types in the generated file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportStyle {
    /// No export keyword: `type Foo = ...`
    None,
    /// Named exports: `export type Foo = ...` (default)
    #[default]
    Named,
    /// Export as grouped object at end: `export { Foo, Bar }`
    Grouped,
}

/// Configuration for TypeScript generation
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Output file path (required for file-based generation)
    pub output: Option<PathBuf>,

    /// Export style for generated types
    pub export_style: ExportStyle,

    /// Whether to generate .d.ts (declarations only) vs .ts
    pub declaration_only: bool,

    /// Custom header comment to prepend
    pub header: Option<String>,

    /// Whether to add ESM-style .js extensions to imports
    /// (for future multi-file mode)
    pub esm_extensions: bool,

    /// Include common utility types (Prettify, etc.) in the output
    pub include_utilities: bool,
}

impl Config {
    /// Create a new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the output file path
    pub fn output(mut self, path: impl AsRef<Path>) -> Self {
        self.output = Some(path.as_ref().to_owned());
        self
    }

    /// Set the export style
    pub fn export_style(mut self, style: ExportStyle) -> Self {
        self.export_style = style;
        self
    }

    /// Generate .d.ts declaration file instead of .ts
    pub fn declaration_only(mut self) -> Self {
        self.declaration_only = true;
        self
    }

    /// Set a custom header comment
    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Enable ESM-style .js extensions in imports (for future multi-file mode)
    pub fn esm_extensions(mut self) -> Self {
        self.esm_extensions = true;
        self
    }

    /// Include common utility types (Prettify, etc.) in the generated output
    pub fn include_utilities(mut self) -> Self {
        self.include_utilities = true;
        self
    }
}

/// TypeScript file generator
///
/// Collects types and generates TypeScript definition files.
#[derive(Debug)]
pub struct Generator {
    config: Config,
    registry: TypeRegistry,
}

impl Generator {
    /// Create a new generator with the given config
    pub fn new(config: Config) -> Self {
        Self {
            config,
            registry: TypeRegistry::new(),
        }
    }

    /// Create a new generator with default config
    pub fn with_defaults() -> Self {
        Self::new(Config::default())
    }

    /// Register a type for generation
    ///
    /// The type must implement the `TS` trait (usually via derive).
    /// Returns `&mut Self` for method chaining.
    pub fn register<T: TS>(&mut self) -> &mut Self {
        self.registry.register::<T>();
        self
    }

    /// Add a TypeDef directly to the registry
    ///
    /// Useful when you have a TypeDef from another source.
    pub fn add(&mut self, typedef: TypeDef) -> &mut Self {
        self.registry.add_typedef(typedef);
        self
    }

    /// Get a reference to the internal registry
    pub fn registry(&self) -> &TypeRegistry {
        &self.registry
    }

    /// Get a mutable reference to the internal registry
    pub fn registry_mut(&mut self) -> &mut TypeRegistry {
        &mut self.registry
    }

    /// Generate TypeScript and return as string
    pub fn generate(&self) -> String {
        let mut output = String::new();

        // Header comment
        if let Some(ref header) = self.config.header {
            output.push_str("// ");
            output.push_str(header);
            output.push('\n');
        } else {
            output.push_str("// Generated by ferro-type-gen\n");
            output.push_str("// Do not edit manually\n");
        }
        output.push('\n');

        // Utility types (if configured)
        if self.config.include_utilities {
            match self.config.export_style {
                ExportStyle::None => {
                    output.push_str(PRETTIFY_TYPE);
                }
                ExportStyle::Named | ExportStyle::Grouped => {
                    output.push_str(PRETTIFY_TYPE_EXPORTED);
                }
            }
            output.push_str("\n\n");
        }

        // Types in dependency order
        match self.config.export_style {
            ExportStyle::None => {
                output.push_str(&self.registry.render());
            }
            ExportStyle::Named => {
                output.push_str(&self.registry.render_exported());
            }
            ExportStyle::Grouped => {
                // Render without exports
                output.push_str(&self.registry.render());
                // Add grouped export at end
                let names: Vec<_> = self.registry.sorted_types().into_iter().collect();
                if !names.is_empty() {
                    output.push_str("\nexport { ");
                    output.push_str(&names.join(", "));
                    output.push_str(" };\n");
                }
            }
        }

        output
    }

    /// Generate TypeScript to the configured output file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No output path is configured
    /// - The file cannot be written
    pub fn write(&self) -> std::io::Result<()> {
        let output_path = self
            .config
            .output
            .as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "No output path configured"))?;

        // Create parent directories
        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let content = self.generate();
        std::fs::write(output_path, content)
    }

    /// Write only if content has changed
    ///
    /// Returns `Ok(true)` if the file was written, `Ok(false)` if unchanged.
    /// This is useful in build.rs to avoid unnecessary rebuilds.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No output path is configured
    /// - The file cannot be read or written
    pub fn write_if_changed(&self) -> std::io::Result<bool> {
        let output_path = self
            .config
            .output
            .as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "No output path configured"))?;

        let new_content = self.generate();

        // Check if file exists and has same content
        if output_path.exists() {
            let existing = std::fs::read_to_string(output_path)?;
            if existing == new_content {
                return Ok(false); // No change
            }
        }

        // Create parent directories and write
        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(output_path, new_content)?;
        Ok(true) // Changed
    }

    // ========================================================================
    // MULTI-FILE GENERATION
    // ========================================================================

    /// Group types by their module path
    ///
    /// Returns a map from module path to list of type names in that module.
    /// Types without a module path are grouped under "default".
    pub fn types_by_module(&self) -> HashMap<String, Vec<String>> {
        let mut result: HashMap<String, Vec<String>> = HashMap::new();

        for name in self.registry.type_names() {
            if let Some(typedef) = self.registry.get(name) {
                let module = match typedef {
                    TypeDef::Named { module, .. } => {
                        module.clone().unwrap_or_else(|| "default".to_string())
                    }
                    _ => "default".to_string(),
                };
                result.entry(module).or_default().push(name.to_string());
            }
        }

        result
    }

    /// Convert a module path to a file path
    ///
    /// For example:
    /// - `my_crate::models::user` -> `models/user.ts`
    /// - `my_crate::api::requests` -> `api/requests.ts`
    ///
    /// The crate name is stripped from the beginning.
    pub fn module_to_path(module: &str) -> PathBuf {
        // Split by :: and skip the crate name (first segment)
        let parts: Vec<&str> = module.split("::").collect();
        let path_parts = if parts.len() > 1 {
            &parts[1..]
        } else {
            &parts[..]
        };

        let mut path = PathBuf::new();
        for part in path_parts {
            path.push(part);
        }
        path.set_extension("ts");
        path
    }

    /// Generate TypeScript content for a specific module
    ///
    /// Only includes types from the specified module.
    pub fn generate_for_module(&self, module: &str, type_names: &[String]) -> String {
        let mut output = String::new();

        // Header comment
        if let Some(ref header) = self.config.header {
            output.push_str("// ");
            output.push_str(header);
            output.push('\n');
        } else {
            output.push_str("// Generated by ferro-type-gen\n");
            output.push_str("// Do not edit manually\n");
            output.push_str("// Module: ");
            output.push_str(module);
            output.push('\n');
        }
        output.push('\n');

        // Get types for this module in dependency order
        let sorted = self.registry.sorted_types();
        let module_types: Vec<_> = sorted
            .into_iter()
            .filter(|name| type_names.contains(&name.to_string()))
            .collect();

        // TODO: Add import statements for types from other modules

        // Render types
        for name in module_types {
            if let Some(typedef) = self.registry.get(name) {
                if let TypeDef::Named { name, def, .. } = typedef {
                    let export_prefix = match self.config.export_style {
                        ExportStyle::None => "",
                        ExportStyle::Named | ExportStyle::Grouped => "export ",
                    };
                    output.push_str(&format!("{}type {} = {};\n\n", export_prefix, name, def.render()));
                }
            }
        }

        output
    }

    /// Write TypeScript to multiple files, organized by module
    ///
    /// Types are grouped by their module path and written to corresponding files.
    /// For example:
    /// - Types from `my_crate::models::user` go to `<output_dir>/models/user.ts`
    /// - Types from `my_crate::api` go to `<output_dir>/api.ts`
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Base directory for output files
    ///
    /// # Returns
    ///
    /// Returns the number of files written.
    ///
    /// # Errors
    ///
    /// Returns an error if files cannot be written.
    pub fn write_multi_file(&self, output_dir: impl AsRef<Path>) -> std::io::Result<usize> {
        let output_dir = output_dir.as_ref();
        let types_by_module = self.types_by_module();
        let mut count = 0;

        for (module, type_names) in &types_by_module {
            let file_path = if module == "default" {
                output_dir.join("types.ts")
            } else {
                output_dir.join(Self::module_to_path(module))
            };

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            let content = self.generate_for_module(module, type_names);
            std::fs::write(&file_path, content)?;
            count += 1;
        }

        Ok(count)
    }

    /// Write multi-file only if content has changed
    ///
    /// Returns the number of files that were written (changed).
    pub fn write_multi_file_if_changed(&self, output_dir: impl AsRef<Path>) -> std::io::Result<usize> {
        let output_dir = output_dir.as_ref();
        let types_by_module = self.types_by_module();
        let mut count = 0;

        for (module, type_names) in &types_by_module {
            let file_path = if module == "default" {
                output_dir.join("types.ts")
            } else {
                output_dir.join(Self::module_to_path(module))
            };

            let new_content = self.generate_for_module(module, type_names);

            // Check if file exists and has same content
            let should_write = if file_path.exists() {
                let existing = std::fs::read_to_string(&file_path)?;
                existing != new_content
            } else {
                true
            };

            if should_write {
                // Create parent directories
                if let Some(parent) = file_path.parent() {
                    if !parent.as_os_str().is_empty() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                std::fs::write(&file_path, new_content)?;
                count += 1;
            }
        }

        Ok(count)
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/// Generate TypeScript for a single type
///
/// Returns the TypeScript definition as a string.
pub fn generate<T: TS>() -> String {
    let mut generator = Generator::with_defaults();
    generator.register::<T>();
    generator.generate()
}

/// Export types from a registry to a file
///
/// Convenience function for simple use cases.
pub fn export_to_file<P: AsRef<Path>>(path: P, registry: &TypeRegistry) -> std::io::Result<()> {
    let content = registry.render_exported();

    // Create parent directories
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    std::fs::write(path, content)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ferro_type::{Field, Primitive, TypeDef};

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .output("types.ts")
            .export_style(ExportStyle::Named)
            .header("Custom header")
            .declaration_only()
            .esm_extensions();

        assert_eq!(config.output, Some(PathBuf::from("types.ts")));
        assert_eq!(config.export_style, ExportStyle::Named);
        assert_eq!(config.header, Some("Custom header".to_string()));
        assert!(config.declaration_only);
        assert!(config.esm_extensions);
    }

    #[test]
    fn test_generator_register() {
        let mut generator = Generator::with_defaults();

        // Register string type (primitive, no named type added)
        generator.register::<String>();

        // Registry should be empty since String doesn't create a Named type
        assert_eq!(generator.registry().len(), 0);
    }

    #[test]
    fn test_generator_add_typedef() {
        let mut generator = Generator::with_defaults();

        let user_type = TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("id", TypeDef::Primitive(Primitive::String)),
                Field::new("name", TypeDef::Primitive(Primitive::String)),
            ])),
            module: None,
            wrapper: None,
        };

        generator.add(user_type);

        assert_eq!(generator.registry().len(), 1);
        assert!(generator.registry().get("User").is_some());
    }

    #[test]
    fn test_generate_export_none() {
        let mut generator = Generator::new(Config::new().export_style(ExportStyle::None));

        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });

        let output = generator.generate();
        assert!(output.contains("type User = string;"));
        assert!(!output.contains("export type User"));
    }

    #[test]
    fn test_generate_export_named() {
        let mut generator = Generator::new(Config::new().export_style(ExportStyle::Named));

        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });

        let output = generator.generate();
        assert!(output.contains("export type User = string;"));
    }

    #[test]
    fn test_generate_export_grouped() {
        let mut generator = Generator::new(Config::new().export_style(ExportStyle::Grouped));

        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "Post".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });

        let output = generator.generate();
        assert!(output.contains("type User = string;"));
        assert!(output.contains("type Post = string;"));
        assert!(output.contains("export { "));
        assert!(output.contains("User"));
        assert!(output.contains("Post"));
    }

    #[test]
    fn test_generate_custom_header() {
        let generator = Generator::new(Config::new().header("My custom header"));

        let output = generator.generate();
        assert!(output.starts_with("// My custom header\n"));
    }

    #[test]
    fn test_generate_default_header() {
        let generator = Generator::with_defaults();

        let output = generator.generate();
        assert!(output.contains("// Generated by ferro-type-gen"));
        assert!(output.contains("// Do not edit manually"));
    }

    #[test]
    fn test_include_utilities() {
        let generator = Generator::new(Config::new().include_utilities());

        let output = generator.generate();
        assert!(output.contains("export type Prettify<T>"));
        assert!(output.contains("{ [K in keyof T]: T[K] }"));
    }

    #[test]
    fn test_include_utilities_no_export() {
        let generator = Generator::new(
            Config::new()
                .export_style(ExportStyle::None)
                .include_utilities()
        );

        let output = generator.generate();
        assert!(output.contains("type Prettify<T>"));
        assert!(!output.contains("export type Prettify"));
    }

    #[test]
    fn test_no_utilities_by_default() {
        let generator = Generator::with_defaults();

        let output = generator.generate();
        assert!(!output.contains("Prettify"));
    }

    #[test]
    fn test_write_creates_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("nested/dir/types.ts");

        let mut generator = Generator::new(Config::new().output(&output_path));
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });

        generator.write().unwrap();

        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("export type User = string;"));
    }

    #[test]
    fn test_write_if_changed() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("types.ts");

        let mut generator = Generator::new(Config::new().output(&output_path));
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });

        // First write should return true (changed)
        assert!(generator.write_if_changed().unwrap());

        // Second write should return false (unchanged)
        assert!(!generator.write_if_changed().unwrap());

        // Add another type
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "Post".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });

        // Third write should return true (changed)
        assert!(generator.write_if_changed().unwrap());
    }

    #[test]
    fn test_write_no_output_configured() {
        let generator = Generator::with_defaults();
        let result = generator.write();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_convenience_generate() {
        // generate() with a primitive type
        let output = generate::<String>();
        // String is primitive, doesn't produce named types
        assert!(output.contains("// Generated by ferro-type-gen"));
    }

    #[test]
    fn test_convenience_export_to_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("types.ts");

        let mut registry = TypeRegistry::new();
        registry.add_typedef(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });

        export_to_file(&output_path, &registry).unwrap();

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("export type User = string;"));
    }

    // ========================================================================
    // MULTI-FILE TESTS
    // ========================================================================

    #[test]
    fn test_module_to_path() {
        assert_eq!(
            Generator::module_to_path("my_crate::models::user"),
            PathBuf::from("models/user.ts")
        );
        assert_eq!(
            Generator::module_to_path("my_crate::api"),
            PathBuf::from("api.ts")
        );
        assert_eq!(
            Generator::module_to_path("my_crate::nested::deep::module"),
            PathBuf::from("nested/deep/module.ts")
        );
    }

    #[test]
    fn test_types_by_module() {
        let mut generator = Generator::with_defaults();

        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: Some("my_crate::models".to_string()),
            wrapper: None,
        });
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "Post".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: Some("my_crate::models".to_string()),
            wrapper: None,
        });
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "Request".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: Some("my_crate::api".to_string()),
            wrapper: None,
        });
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "Orphan".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });

        let by_module = generator.types_by_module();

        assert_eq!(by_module.len(), 3);
        assert!(by_module.get("my_crate::models").unwrap().contains(&"User".to_string()));
        assert!(by_module.get("my_crate::models").unwrap().contains(&"Post".to_string()));
        assert!(by_module.get("my_crate::api").unwrap().contains(&"Request".to_string()));
        assert!(by_module.get("default").unwrap().contains(&"Orphan".to_string()));
    }

    #[test]
    fn test_generate_for_module() {
        let mut generator = Generator::with_defaults();

        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: Some("my_crate::models".to_string()),
            wrapper: None,
        });
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "Post".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::Number)),
            module: Some("my_crate::models".to_string()),
            wrapper: None,
        });

        let output = generator.generate_for_module("my_crate::models", &["User".to_string(), "Post".to_string()]);

        assert!(output.contains("// Module: my_crate::models"));
        assert!(output.contains("export type User = string;"));
        assert!(output.contains("export type Post = number;"));
    }

    #[test]
    fn test_write_multi_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        let mut generator = Generator::with_defaults();

        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: Some("my_crate::models::user".to_string()),
            wrapper: None,
        });
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "Request".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: Some("my_crate::api".to_string()),
            wrapper: None,
        });

        let count = generator.write_multi_file(temp_dir.path()).unwrap();
        assert_eq!(count, 2);

        // Check files exist
        let user_path = temp_dir.path().join("models/user.ts");
        let api_path = temp_dir.path().join("api.ts");

        assert!(user_path.exists());
        assert!(api_path.exists());

        // Check content
        let user_content = std::fs::read_to_string(&user_path).unwrap();
        assert!(user_content.contains("export type User = string;"));

        let api_content = std::fs::read_to_string(&api_path).unwrap();
        assert!(api_content.contains("export type Request = string;"));
    }

    #[test]
    fn test_write_multi_file_if_changed() {
        let temp_dir = tempfile::tempdir().unwrap();

        let mut generator = Generator::with_defaults();
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: Some("my_crate::models".to_string()),
            wrapper: None,
        });

        // First write should write
        let count1 = generator.write_multi_file_if_changed(temp_dir.path()).unwrap();
        assert_eq!(count1, 1);

        // Second write should not write (unchanged)
        let count2 = generator.write_multi_file_if_changed(temp_dir.path()).unwrap();
        assert_eq!(count2, 0);

        // Add another type
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "Post".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::Number)),
            module: Some("my_crate::models".to_string()),
            wrapper: None,
        });

        // Third write should write (changed)
        let count3 = generator.write_multi_file_if_changed(temp_dir.path()).unwrap();
        assert_eq!(count3, 1);
    }

    #[test]
    fn test_write_multi_file_default_module() {
        let temp_dir = tempfile::tempdir().unwrap();

        let mut generator = Generator::with_defaults();
        generator.add(TypeDef::Named {
            namespace: vec![],
            name: "Orphan".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
            module: None,
            wrapper: None,
        });

        generator.write_multi_file(temp_dir.path()).unwrap();

        // Types without module go to types.ts
        let types_path = temp_dir.path().join("types.ts");
        assert!(types_path.exists());

        let content = std::fs::read_to_string(&types_path).unwrap();
        assert!(content.contains("export type Orphan = string;"));
    }
}
