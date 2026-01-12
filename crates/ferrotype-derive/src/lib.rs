//! Derive macros for ferrotype TypeScript type generation
//!
//! This crate provides:
//! - `#[derive(TS)]` for generating TypeScript type definitions from Rust types
//! - `#[derive(TypeScript)]` (deprecated alias for `TS`)

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, GenericParam, Generics, Ident, Type,
};

// ============================================================================
// ATTRIBUTE PARSING
// ============================================================================

/// Case conversion strategies for rename_all
#[derive(Debug, Clone, Copy, PartialEq)]
enum RenameAll {
    /// camelCase
    CamelCase,
    /// PascalCase
    PascalCase,
    /// snake_case
    SnakeCase,
    /// SCREAMING_SNAKE_CASE
    ScreamingSnakeCase,
    /// kebab-case
    KebabCase,
    /// SCREAMING-KEBAB-CASE
    ScreamingKebabCase,
}

impl RenameAll {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "camelCase" => Some(RenameAll::CamelCase),
            "PascalCase" => Some(RenameAll::PascalCase),
            "snake_case" => Some(RenameAll::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(RenameAll::ScreamingSnakeCase),
            "kebab-case" => Some(RenameAll::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(RenameAll::ScreamingKebabCase),
            _ => None,
        }
    }

    fn apply(&self, name: &str) -> String {
        match self {
            RenameAll::CamelCase => to_camel_case(name),
            RenameAll::PascalCase => to_pascal_case(name),
            RenameAll::SnakeCase => to_snake_case(name),
            RenameAll::ScreamingSnakeCase => to_snake_case(name).to_uppercase(),
            RenameAll::KebabCase => to_snake_case(name).replace('_', "-"),
            RenameAll::ScreamingKebabCase => to_snake_case(name).replace('_', "-").to_uppercase(),
        }
    }
}

/// Convert to camelCase
fn to_camel_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in name.chars().enumerate() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert to PascalCase
fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert to snake_case (from PascalCase or camelCase)
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();

    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }

    result
}

/// Container-level attributes (on struct/enum)
#[derive(Default)]
struct ContainerAttrs {
    /// Rename the type itself
    rename: Option<String>,
    /// Rename all fields/variants
    rename_all: Option<RenameAll>,
    /// Make newtype structs transparent (use inner type directly)
    transparent: bool,
    /// Custom tag field name for enums (default: "type")
    tag: Option<String>,
    /// Content field name for adjacently tagged enums
    content: Option<String>,
    /// Generate untagged union (no discriminant)
    untagged: bool,
    /// Template literal pattern for branded ID types (e.g., "vm-${string}")
    pattern: Option<String>,
    /// Namespace path for the type (e.g., "VM::Git" or "VM.Git")
    namespace: Vec<String>,
    /// Type to extend via intersection (e.g., "Claude.Todo" generates `type X = Claude.Todo & { ... }`)
    extends: Option<String>,
    /// Utility type wrapper (e.g., "Prettify" or "Prettify<Required<")
    wrapper: Option<String>,
}

impl ContainerAttrs {
    fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = ContainerAttrs::default();

        for attr in attrs {
            if !attr.path().is_ident("ts") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.rename = Some(value.value());
                } else if meta.path.is_ident("rename_all") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    let s = value.value();
                    result.rename_all = RenameAll::from_str(&s);
                    if result.rename_all.is_none() {
                        return Err(syn::Error::new_spanned(
                            value,
                            format!(
                                "unknown rename_all value: '{}'. Expected one of: \
                                camelCase, PascalCase, snake_case, \
                                SCREAMING_SNAKE_CASE, kebab-case, SCREAMING-KEBAB-CASE",
                                s
                            ),
                        ));
                    }
                } else if meta.path.is_ident("transparent") {
                    result.transparent = true;
                } else if meta.path.is_ident("tag") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.tag = Some(value.value());
                } else if meta.path.is_ident("content") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.content = Some(value.value());
                } else if meta.path.is_ident("untagged") {
                    result.untagged = true;
                } else if meta.path.is_ident("pattern") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.pattern = Some(value.value());
                } else if meta.path.is_ident("namespace") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    // Parse namespace path - supports both "::" and "." as separators
                    let ns_str = value.value();
                    result.namespace = ns_str
                        .split(|c| c == ':' || c == '.')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                } else if meta.path.is_ident("extends") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.extends = Some(value.value());
                } else if meta.path.is_ident("wrapper") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.wrapper = Some(value.value());
                }
                Ok(())
            })?;
        }

        Ok(result)
    }
}

/// Indexed access base type specification
#[derive(Clone)]
enum IndexSpec {
    /// Rust type for compile-time validation (e.g., `index = Profile`)
    Type(syn::Type),
    /// String for external TypeScript types - no validation (e.g., `index = "ExternalType"`)
    String(String),
}

/// Indexed access key specification
#[derive(Clone)]
enum KeySpec {
    /// Rust identifier for compile-time validation (e.g., `key = login`)
    Ident(syn::Ident),
    /// String for external TypeScript keys - no validation (e.g., `key = "someKey"`)
    String(String),
}

/// Info needed to generate compile-time validation for indexed access
struct IndexedAccessValidation {
    /// The base type (e.g., Profile)
    index_type: syn::Type,
    /// The field/key identifier (e.g., login)
    key_ident: syn::Ident,
    /// Unique index for naming the validation function
    field_index: usize,
}

/// Field-level attributes
#[derive(Default)]
struct FieldAttrs {
    /// Rename this specific field
    rename: Option<String>,
    /// Skip this field in the generated TypeScript
    skip: bool,
    /// Flatten this field's type into the parent object
    flatten: bool,
    /// Override the TypeScript type with a custom string
    type_override: Option<String>,
    /// Mark this field as optional (with ?) - legacy, use `optional` instead
    default: bool,
    /// Mark this field as optional (with ?), unwrapping Option<T> to T
    /// Unlike `default`, this extracts the inner type from Option<T>
    optional: bool,
    /// Inline the type definition instead of using a reference
    inline: bool,
    /// Base type for indexed access (e.g., Profile in Profile["login"])
    index: Option<IndexSpec>,
    /// Key for indexed access (e.g., login in Profile["login"])
    key: Option<KeySpec>,
    /// Template literal pattern for this field (e.g., "${TOPIC}::${ULID}")
    pattern: Option<String>,
}

impl FieldAttrs {
    fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = FieldAttrs::default();

        for attr in attrs {
            if !attr.path().is_ident("ts") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.rename = Some(value.value());
                } else if meta.path.is_ident("skip") {
                    result.skip = true;
                } else if meta.path.is_ident("flatten") {
                    result.flatten = true;
                } else if meta.path.is_ident("type") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.type_override = Some(value.value());
                } else if meta.path.is_ident("default") {
                    result.default = true;
                } else if meta.path.is_ident("optional") {
                    result.optional = true;
                } else if meta.path.is_ident("inline") {
                    result.inline = true;
                } else if meta.path.is_ident("index") {
                    // Try to parse as Type first (for compile-time validation)
                    // Fall back to string literal (for external TS types)
                    let value_token = meta.value()?;
                    if let Ok(lit_str) = value_token.parse::<syn::LitStr>() {
                        result.index = Some(IndexSpec::String(lit_str.value()));
                    } else {
                        let ty: syn::Type = value_token.parse()?;
                        result.index = Some(IndexSpec::Type(ty));
                    }
                } else if meta.path.is_ident("key") {
                    // Try to parse as Ident first (for compile-time validation)
                    // Fall back to string literal (for external TS types)
                    let value_token = meta.value()?;
                    if let Ok(lit_str) = value_token.parse::<syn::LitStr>() {
                        result.key = Some(KeySpec::String(lit_str.value()));
                    } else {
                        let ident: syn::Ident = value_token.parse()?;
                        result.key = Some(KeySpec::Ident(ident));
                    }
                } else if meta.path.is_ident("pattern") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.pattern = Some(value.value());
                }
                Ok(())
            })?;
        }

        Ok(result)
    }

    /// Returns true if this field uses indexed access type
    fn has_indexed_access(&self) -> bool {
        self.index.is_some() && self.key.is_some()
    }

    /// Returns true if this field uses a template literal pattern
    fn has_pattern(&self) -> bool {
        self.pattern.is_some()
    }
}

/// Get the effective name for a field, applying rename attributes
fn get_field_name(
    original: &str,
    field_attrs: &FieldAttrs,
    container_attrs: &ContainerAttrs,
) -> String {
    // Field-level rename takes precedence
    if let Some(ref renamed) = field_attrs.rename {
        return renamed.clone();
    }

    // Then apply container-level rename_all
    if let Some(rename_all) = container_attrs.rename_all {
        return rename_all.apply(original);
    }

    // Otherwise use original name
    original.to_string()
}

/// Parse a template literal pattern into strings and type names.
///
/// For example, `"vm-${string}"` becomes:
/// - strings: ["vm-", ""]
/// - types: ["string"]
///
/// And `"v${number}.${number}.${number}"` becomes:
/// - strings: ["v", ".", ".", ""]
/// - types: ["number", "number", "number"]
fn parse_template_pattern(pattern: &str) -> syn::Result<(Vec<String>, Vec<String>)> {
    let mut strings = Vec::new();
    let mut types = Vec::new();
    let mut current = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            // Found ${...}
            strings.push(std::mem::take(&mut current));
            chars.next(); // consume '{'

            let mut type_name = String::new();
            let mut depth = 1;
            while let Some(tc) = chars.next() {
                if tc == '{' {
                    depth += 1;
                    type_name.push(tc);
                } else if tc == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    type_name.push(tc);
                } else {
                    type_name.push(tc);
                }
            }

            if type_name.is_empty() {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "Empty type placeholder ${} in pattern",
                ));
            }
            types.push(type_name);
        } else {
            current.push(c);
        }
    }

    // Push the remaining string (always one more string than types)
    strings.push(current);

    Ok((strings, types))
}

/// Check if a type is `Option<T>` (or `std::option::Option<T>`)
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(last) = type_path.path.segments.last() {
            return last.ident == "Option";
        }
    }
    false
}

/// Extract the inner type from `Option<T>`, returns `Some(&T)` if successful
fn extract_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(last) = type_path.path.segments.last() {
            if last.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return Some(inner);
                    }
                }
            }
        }
    }
    None
}

/// Convert a type name string to a TypeDef expression.
/// Supports: string, number, boolean, bigint, and type references.
fn type_name_to_typedef(name: &str) -> TokenStream2 {
    match name.trim() {
        "string" => quote! { ferro_type::TypeDef::Primitive(ferro_type::Primitive::String) },
        "number" => quote! { ferro_type::TypeDef::Primitive(ferro_type::Primitive::Number) },
        "boolean" => quote! { ferro_type::TypeDef::Primitive(ferro_type::Primitive::Boolean) },
        "bigint" => quote! { ferro_type::TypeDef::Primitive(ferro_type::Primitive::BigInt) },
        "any" => quote! { ferro_type::TypeDef::Primitive(ferro_type::Primitive::Any) },
        "unknown" => quote! { ferro_type::TypeDef::Primitive(ferro_type::Primitive::Unknown) },
        // For other types, treat as a reference
        other => {
            let type_ref = other.trim();
            quote! { ferro_type::TypeDef::Ref(#type_ref.to_string()) }
        }
    }
}

/// Generate a TemplateLiteral TypeDef expression from parsed pattern.
fn generate_template_literal_expr(strings: &[String], types: &[String]) -> TokenStream2 {
    let string_literals: Vec<_> = strings.iter().map(|s| quote! { #s.to_string() }).collect();
    let type_exprs: Vec<_> = types.iter().map(|t| {
        let typedef = type_name_to_typedef(t);
        quote! { Box::new(#typedef) }
    }).collect();

    quote! {
        ferro_type::TypeDef::TemplateLiteral {
            strings: vec![#(#string_literals),*],
            types: vec![#(#type_exprs),*],
        }
    }
}

/// Derive macro for generating TypeScript type definitions from Rust types.
///
/// # Examples
///
/// ## Unit variants
/// ```ignore
/// #[derive(TS)]
/// enum Status {
///     Pending,
///     Active,
///     Completed,
/// }
/// // Generates: "Pending" | "Active" | "Completed"
/// ```
///
/// ## Tuple variants
/// ```ignore
/// #[derive(TS)]
/// enum Coordinate {
///     D2(f64, f64),
///     D3(f64, f64, f64),
/// }
/// // Generates: { type: "D2"; value: [number, number] } | { type: "D3"; value: [number, number, number] }
/// ```
///
/// ## Struct variants
/// ```ignore
/// #[derive(TS)]
/// enum Shape {
///     Circle { center: Point, radius: f64 },
///     Rectangle { x: f64, y: f64, width: f64, height: f64 },
/// }
/// // Generates: { type: "Circle"; center: Point; radius: number } | { type: "Rectangle"; x: number; y: number; width: number; height: number }
/// ```
///
/// ## Structs
/// ```ignore
/// #[derive(TS)]
/// struct User {
///     id: String,
///     name: String,
///     age: i32,
/// }
/// // Generates: { id: string; name: string; age: number }
/// ```
#[proc_macro_derive(TS, attributes(ts))]
pub fn derive_ts(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_derive_typescript(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Deprecated: Use `#[derive(TS)]` instead.
///
/// This is a deprecated alias for backwards compatibility. It will be removed in v0.3.0.
#[deprecated(since = "0.2.0", note = "use `#[derive(TS)]` instead")]
#[proc_macro_derive(TypeScript, attributes(ts))]
pub fn derive_typescript(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_derive_typescript(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_derive_typescript(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let generics = &input.generics;

    // Parse container-level attributes
    let container_attrs = ContainerAttrs::from_attrs(&input.attrs)?;

    // Use renamed type name if specified, otherwise use original
    let type_name = container_attrs
        .rename
        .clone()
        .unwrap_or_else(|| name.to_string());

    match &input.data {
        Data::Enum(data) => {
            let typedef = generate_enum_typedef(&data.variants, &container_attrs)?;
            generate_impl(name, &type_name, &container_attrs.namespace, &container_attrs.wrapper, generics, typedef)
        }
        Data::Struct(data) => {
            // Handle transparent newtypes - they become the inner type directly
            if container_attrs.transparent {
                if let syn::Fields::Unnamed(fields) = &data.fields {
                    if fields.unnamed.len() == 1 {
                        let inner_type = &fields.unnamed.first().unwrap().ty;
                        return generate_transparent_impl(name, inner_type, generics);
                    }
                }
                return Err(syn::Error::new_spanned(
                    input,
                    "#[ts(transparent)] can only be used on newtype structs (single unnamed field)",
                ));
            }

            // Handle template literal patterns - typically for branded ID types
            if let Some(ref pattern) = container_attrs.pattern {
                let (strings, types) = parse_template_pattern(pattern)?;
                let typedef = generate_template_literal_expr(&strings, &types);
                return generate_impl(name, &type_name, &[], &container_attrs.wrapper, generics, typedef);
            }

            let (typedef, validations) = generate_struct_typedef(&data.fields, &container_attrs)?;

            // Handle intersection types via extends attribute
            let typedef = if let Some(ref extends_type) = container_attrs.extends {
                quote! {
                    ferro_type::TypeDef::Intersection(vec![
                        ferro_type::TypeDef::Ref(#extends_type.to_string()),
                        #typedef
                    ])
                }
            } else {
                typedef
            };

            let impl_code = generate_impl(name, &type_name, &container_attrs.namespace, &container_attrs.wrapper, generics, typedef)?;

            // Generate validation code for indexed access with Type/Ident
            let validation_code = generate_indexed_access_validations(name, &validations);

            Ok(quote! {
                #impl_code
                #validation_code
            })
        }
        Data::Union(_) => {
            Err(syn::Error::new_spanned(
                input,
                "TypeScript derive is not supported for unions",
            ))
        }
    }
}

fn generate_enum_typedef(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    container_attrs: &ContainerAttrs,
) -> syn::Result<TokenStream2> {
    if variants.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Cannot derive TypeScript for empty enum",
        ));
    }

    // Check if all variants are unit variants (for string literal union type)
    let all_unit = variants.iter().all(|v| matches!(v.fields, Fields::Unit));

    // Handle untagged enums: generate plain union without discriminant
    if container_attrs.untagged {
        return generate_untagged_enum(variants, container_attrs);
    }

    // Get tag field name (default: "type")
    let tag_name = container_attrs.tag.as_deref().unwrap_or("type");

    // Check if using adjacent tagging (content field specified)
    let content_name = container_attrs.content.as_deref();

    if all_unit {
        // Generate string literal union: "Pending" | "Active" | "Completed"
        let mut variant_exprs: Vec<TokenStream2> = Vec::new();
        for v in variants.iter() {
            let variant_attrs = FieldAttrs::from_attrs(&v.attrs)?;
            let name = get_field_name(&v.ident.to_string(), &variant_attrs, container_attrs);
            variant_exprs.push(
                quote! { ferro_type::TypeDef::Literal(ferro_type::Literal::String(#name.to_string())) }
            );
        }

        Ok(quote! {
            ferro_type::TypeDef::Union(vec![#(#variant_exprs),*])
        })
    } else {
        // Generate discriminated union with tag field
        let mut variant_exprs: Vec<TokenStream2> = Vec::new();

        for variant in variants.iter() {
            let variant_attrs = FieldAttrs::from_attrs(&variant.attrs)?;
            let variant_name_str = get_field_name(
                &variant.ident.to_string(),
                &variant_attrs,
                container_attrs,
            );

            let expr = match &variant.fields {
                Fields::Unit => {
                    // { [tag]: "VariantName" }
                    quote! {
                        ferro_type::TypeDef::Object(vec![
                            ferro_type::Field::new(
                                #tag_name,
                                ferro_type::TypeDef::Literal(ferro_type::Literal::String(#variant_name_str.to_string()))
                            )
                        ])
                    }
                }
                Fields::Unnamed(fields) => {
                    if let Some(content) = content_name {
                        // Adjacent tagging: { [tag]: "Variant", [content]: data }
                        let content_type = if fields.unnamed.len() == 1 {
                            let field_type = &fields.unnamed.first().unwrap().ty;
                            type_to_typedef(field_type)
                        } else {
                            let field_exprs: Vec<TokenStream2> = fields
                                .unnamed
                                .iter()
                                .map(|f| type_to_typedef(&f.ty))
                                .collect();
                            quote! { ferro_type::TypeDef::Tuple(vec![#(#field_exprs),*]) }
                        };
                        quote! {
                            ferro_type::TypeDef::Object(vec![
                                ferro_type::Field::new(
                                    #tag_name,
                                    ferro_type::TypeDef::Literal(ferro_type::Literal::String(#variant_name_str.to_string()))
                                ),
                                ferro_type::Field::new(#content, #content_type)
                            ])
                        }
                    } else if fields.unnamed.len() == 1 {
                        // Newtype variant (internal tagging): { [tag]: "Text"; value: T }
                        let field_type = &fields.unnamed.first().unwrap().ty;
                        let type_expr = type_to_typedef(field_type);
                        quote! {
                            ferro_type::TypeDef::Object(vec![
                                ferro_type::Field::new(
                                    #tag_name,
                                    ferro_type::TypeDef::Literal(ferro_type::Literal::String(#variant_name_str.to_string()))
                                ),
                                ferro_type::Field::new("value", #type_expr)
                            ])
                        }
                    } else {
                        // Tuple variant (internal tagging): { [tag]: "D2"; value: [T1, T2] }
                        let field_exprs: Vec<TokenStream2> = fields
                            .unnamed
                            .iter()
                            .map(|f| type_to_typedef(&f.ty))
                            .collect();
                        quote! {
                            ferro_type::TypeDef::Object(vec![
                                ferro_type::Field::new(
                                    #tag_name,
                                    ferro_type::TypeDef::Literal(ferro_type::Literal::String(#variant_name_str.to_string()))
                                ),
                                ferro_type::Field::new(
                                    "value",
                                    ferro_type::TypeDef::Tuple(vec![#(#field_exprs),*])
                                )
                            ])
                        }
                    }
                }
                Fields::Named(fields) => {
                    let mut field_exprs: Vec<TokenStream2> = Vec::new();
                    for f in fields.named.iter() {
                        let field_attrs = FieldAttrs::from_attrs(&f.attrs)?;
                        if field_attrs.skip {
                            continue;
                        }
                        let original_name = f.ident.as_ref().unwrap().to_string();
                        let field_name = field_attrs.rename.clone().unwrap_or(original_name);
                        let field_type = &f.ty;
                        let type_expr = type_to_typedef(field_type);
                        field_exprs.push(quote! {
                            ferro_type::Field::new(#field_name, #type_expr)
                        });
                    }

                    if let Some(content) = content_name {
                        // Adjacent tagging: { [tag]: "Variant", [content]: { fields... } }
                        quote! {
                            ferro_type::TypeDef::Object(vec![
                                ferro_type::Field::new(
                                    #tag_name,
                                    ferro_type::TypeDef::Literal(ferro_type::Literal::String(#variant_name_str.to_string()))
                                ),
                                ferro_type::Field::new(
                                    #content,
                                    ferro_type::TypeDef::Object(vec![#(#field_exprs),*])
                                )
                            ])
                        }
                    } else {
                        // Internal tagging: { [tag]: "Circle"; center: Point; radius: number }
                        quote! {
                            ferro_type::TypeDef::Object({
                                let mut fields = vec![
                                    ferro_type::Field::new(
                                        #tag_name,
                                        ferro_type::TypeDef::Literal(ferro_type::Literal::String(#variant_name_str.to_string()))
                                    )
                                ];
                                fields.extend(vec![#(#field_exprs),*]);
                                fields
                            })
                        }
                    }
                }
            };
            variant_exprs.push(expr);
        }

        Ok(quote! {
            ferro_type::TypeDef::Union(vec![#(#variant_exprs),*])
        })
    }
}

/// Generate untagged enum: plain union without discriminant fields
fn generate_untagged_enum(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    container_attrs: &ContainerAttrs,
) -> syn::Result<TokenStream2> {
    let mut variant_exprs: Vec<TokenStream2> = Vec::new();

    for variant in variants.iter() {
        let variant_attrs = FieldAttrs::from_attrs(&variant.attrs)?;
        let variant_name_str = get_field_name(
            &variant.ident.to_string(),
            &variant_attrs,
            container_attrs,
        );

        let expr = match &variant.fields {
            Fields::Unit => {
                // Unit variant in untagged enum becomes string literal
                quote! {
                    ferro_type::TypeDef::Literal(ferro_type::Literal::String(#variant_name_str.to_string()))
                }
            }
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    // Newtype: just the inner type
                    let field_type = &fields.unnamed.first().unwrap().ty;
                    type_to_typedef(field_type)
                } else {
                    // Tuple: [T1, T2, ...]
                    let field_exprs: Vec<TokenStream2> = fields
                        .unnamed
                        .iter()
                        .map(|f| type_to_typedef(&f.ty))
                        .collect();
                    quote! {
                        ferro_type::TypeDef::Tuple(vec![#(#field_exprs),*])
                    }
                }
            }
            Fields::Named(fields) => {
                // Struct variant: { field1: T1, field2: T2 }
                let mut field_exprs: Vec<TokenStream2> = Vec::new();
                for f in fields.named.iter() {
                    let field_attrs = FieldAttrs::from_attrs(&f.attrs)?;
                    if field_attrs.skip {
                        continue;
                    }
                    let original_name = f.ident.as_ref().unwrap().to_string();
                    let field_name = field_attrs.rename.clone().unwrap_or(original_name);
                    let field_type = &f.ty;
                    let type_expr = type_to_typedef(field_type);
                    field_exprs.push(quote! {
                        ferro_type::Field::new(#field_name, #type_expr)
                    });
                }
                quote! {
                    ferro_type::TypeDef::Object(vec![#(#field_exprs),*])
                }
            }
        };
        variant_exprs.push(expr);
    }

    Ok(quote! {
        ferro_type::TypeDef::Union(vec![#(#variant_exprs),*])
    })
}

fn generate_struct_typedef(
    fields: &syn::Fields,
    container_attrs: &ContainerAttrs,
) -> syn::Result<(TokenStream2, Vec<IndexedAccessValidation>)> {
    match fields {
        syn::Fields::Named(fields) => {
            // Named struct: Object with fields
            if fields.named.is_empty() {
                // Empty struct becomes empty object
                return Ok((quote! { ferro_type::TypeDef::Object(vec![]) }, vec![]));
            }

            // Separate regular fields from flattened fields
            let mut regular_field_exprs: Vec<TokenStream2> = Vec::new();
            let mut validations: Vec<IndexedAccessValidation> = Vec::new();
            let mut field_index: usize = 0;
            let mut flatten_exprs: Vec<TokenStream2> = Vec::new();

            for f in fields.named.iter() {
                let field_attrs = FieldAttrs::from_attrs(&f.attrs)?;
                // Skip fields marked with #[ts(skip)]
                if field_attrs.skip {
                    continue;
                }

                let field_type = &f.ty;

                // Validate indexed access attributes - both must be present or neither
                if field_attrs.index.is_some() != field_attrs.key.is_some() {
                    return Err(syn::Error::new_spanned(
                        f,
                        "#[ts(index = ...)] and #[ts(key = ...)] must be used together",
                    ));
                }

                if field_attrs.flatten {
                    // For flattened fields, we extract the inner type's fields at runtime
                    flatten_exprs.push(quote! {
                        {
                            let inner_td = <#field_type as ferro_type::TS>::typescript();
                            ferro_type::extract_object_fields(&inner_td)
                        }
                    });
                } else {
                    let original_name = f.ident.as_ref().unwrap().to_string();
                    let field_name = get_field_name(&original_name, &field_attrs, container_attrs);

                    // Determine the type expression
                    let type_expr = if let Some(ref type_override) = field_attrs.type_override {
                        quote! { ferro_type::TypeDef::Ref(#type_override.to_string()) }
                    } else if field_attrs.has_indexed_access() {
                        // Use indexed access type: Profile["login"]
                        let index_spec = field_attrs.index.as_ref().unwrap();
                        let key_spec = field_attrs.key.as_ref().unwrap();

                        // Get string representations for TypeDef
                        let index_str = match index_spec {
                            IndexSpec::Type(ty) => quote! { stringify!(#ty).to_string() },
                            IndexSpec::String(s) => quote! { #s.to_string() },
                        };
                        let key_str = match key_spec {
                            KeySpec::Ident(ident) => quote! { stringify!(#ident).to_string() },
                            KeySpec::String(s) => quote! { #s.to_string() },
                        };

                        // Collect validation info if both are Type/Ident (not strings)
                        if let (IndexSpec::Type(index_type), KeySpec::Ident(key_ident)) =
                            (index_spec, key_spec)
                        {
                            validations.push(IndexedAccessValidation {
                                index_type: index_type.clone(),
                                key_ident: key_ident.clone(),
                                field_index,
                            });
                            field_index += 1;
                        }

                        quote! {
                            ferro_type::TypeDef::IndexedAccess {
                                base: #index_str,
                                key: #key_str,
                            }
                        }
                    } else if field_attrs.has_pattern() {
                        // Use template literal pattern
                        let pattern = field_attrs.pattern.as_ref().unwrap();
                        let (strings, types) = parse_template_pattern(pattern)?;
                        generate_template_literal_expr(&strings, &types)
                    } else if field_attrs.optional && is_option_type(field_type) {
                        // For #[ts(optional)] on Option<T>, unwrap to just T
                        // This generates `field?: T` instead of `field?: T | null`
                        let inner_type = extract_option_inner(field_type).unwrap();
                        let base_expr = type_to_typedef(inner_type);
                        if field_attrs.inline {
                            quote! { ferro_type::inline_typedef(#base_expr) }
                        } else {
                            base_expr
                        }
                    } else {
                        let base_expr = type_to_typedef(field_type);
                        if field_attrs.inline {
                            quote! { ferro_type::inline_typedef(#base_expr) }
                        } else {
                            base_expr
                        }
                    };

                    // Create field (optional if default or optional attribute is set)
                    if field_attrs.default || field_attrs.optional {
                        regular_field_exprs.push(quote! {
                            ferro_type::Field::optional(#field_name, #type_expr)
                        });
                    } else {
                        regular_field_exprs.push(quote! {
                            ferro_type::Field::new(#field_name, #type_expr)
                        });
                    }
                }
            }

            // If there are flattened fields, we need to build the vec dynamically
            if flatten_exprs.is_empty() {
                Ok((quote! {
                    ferro_type::TypeDef::Object(vec![#(#regular_field_exprs),*])
                }, validations))
            } else {
                Ok((quote! {
                    {
                        let mut fields = vec![#(#regular_field_exprs),*];
                        #(fields.extend(#flatten_exprs);)*
                        ferro_type::TypeDef::Object(fields)
                    }
                }, validations))
            }
        }
        syn::Fields::Unnamed(fields) => {
            // Tuple struct - no indexed access possible
            if fields.unnamed.len() == 1 {
                // Newtype: unwrap to inner type
                let field_type = &fields.unnamed.first().unwrap().ty;
                let type_expr = type_to_typedef(field_type);
                Ok((quote! { #type_expr }, vec![]))
            } else {
                // Tuple: [type1, type2, ...]
                let field_exprs: Vec<TokenStream2> = fields
                    .unnamed
                    .iter()
                    .map(|f| type_to_typedef(&f.ty))
                    .collect();

                Ok((quote! {
                    ferro_type::TypeDef::Tuple(vec![#(#field_exprs),*])
                }, vec![]))
            }
        }
        syn::Fields::Unit => {
            // Unit struct becomes null
            Ok((quote! { ferro_type::TypeDef::Primitive(ferro_type::Primitive::Null) }, vec![]))
        }
    }
}

/// Generate compile-time validation code for indexed access fields.
///
/// For each validated indexed access, generates:
/// ```ignore
/// const _: () = {
///     fn _validate_indexed_access_0(p: &Profile) -> &_ { &p.login }
/// };
/// ```
///
/// This ensures that the type has the specified field at compile time.
fn generate_indexed_access_validations(
    struct_name: &Ident,
    validations: &[IndexedAccessValidation],
) -> TokenStream2 {
    if validations.is_empty() {
        return quote! {};
    }

    let validation_fns: Vec<TokenStream2> = validations
        .iter()
        .map(|v| {
            let index_type = &v.index_type;
            let key_ident = &v.key_ident;
            let fn_name = syn::Ident::new(
                &format!("_validate_indexed_access_{}", v.field_index),
                proc_macro2::Span::call_site(),
            );
            // Use a simple field access that will fail to compile if the field doesn't exist
            quote! {
                #[allow(dead_code)]
                fn #fn_name(__val: &#index_type) {
                    let _ = &__val.#key_ident;
                }
            }
        })
        .collect();

    let const_name = syn::Ident::new(
        &format!(
            "__VALIDATE_INDEXED_ACCESS_{}",
            struct_name.to_string().to_uppercase()
        ),
        struct_name.span(),
    );

    quote! {
        #[doc(hidden)]
        const #const_name: () = {
            #(#validation_fns)*
        };
    }
}

/// Convert a Rust type to its TypeScript TypeDef representation.
/// Uses TS trait for types that implement it.
fn type_to_typedef(ty: &Type) -> TokenStream2 {
    quote! { <#ty as ferro_type::TS>::typescript() }
}

/// Generate implementation for a transparent newtype wrapper.
/// The TypeScript representation is just the inner type, not wrapped in Named.
fn generate_transparent_impl(
    name: &Ident,
    inner_type: &Type,
    generics: &Generics,
) -> syn::Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Add TS bounds to generic parameters
    let where_clause = if generics.params.is_empty() {
        where_clause.cloned()
    } else {
        let type_params: Vec<_> = generics.params.iter().filter_map(|p| {
            if let GenericParam::Type(tp) = p {
                Some(&tp.ident)
            } else {
                None
            }
        }).collect();

        if type_params.is_empty() {
            where_clause.cloned()
        } else {
            let bounds = type_params.iter().map(|p| {
                quote! { #p: ferro_type::TS }
            });

            if let Some(existing_where) = where_clause {
                let existing_predicates = &existing_where.predicates;
                Some(syn::parse_quote! { where #(#bounds,)* #existing_predicates })
            } else {
                Some(syn::parse_quote! { where #(#bounds),* })
            }
        }
    };

    Ok(quote! {
        impl #impl_generics ferro_type::TS for #name #ty_generics #where_clause {
            fn typescript() -> ferro_type::TypeDef {
                <#inner_type as ferro_type::TS>::typescript()
            }
        }
    })
}

fn generate_impl(
    name: &Ident,
    name_str: &str,
    namespace: &[String],
    wrapper: &Option<String>,
    generics: &Generics,
    typedef_expr: TokenStream2,
) -> syn::Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Add TS bounds to generic parameters
    let where_clause = if generics.params.is_empty() {
        where_clause.cloned()
    } else {
        let type_params: Vec<_> = generics.params.iter().filter_map(|p| {
            if let GenericParam::Type(tp) = p {
                Some(&tp.ident)
            } else {
                None
            }
        }).collect();

        if type_params.is_empty() {
            where_clause.cloned()
        } else {
            let bounds = type_params.iter().map(|p| {
                quote! { #p: ferro_type::TS }
            });

            if let Some(existing_where) = where_clause {
                let existing_predicates = &existing_where.predicates;
                Some(syn::parse_quote! { where #(#bounds,)* #existing_predicates })
            } else {
                Some(syn::parse_quote! { where #(#bounds),* })
            }
        }
    };

    // Generate auto-registration code only for non-generic types
    // Generic types can't be auto-registered because we need concrete type parameters
    let registration = if generics.params.is_empty() {
        let register_name = syn::Ident::new(
            &format!("__FERRO_TYPE_REGISTER_{}", name.to_string().to_uppercase()),
            name.span(),
        );
        quote! {
            #[ferro_type::linkme::distributed_slice(ferro_type::TYPESCRIPT_TYPES)]
            #[linkme(crate = ferro_type::linkme)]
            static #register_name: fn() -> ferro_type::TypeDef = || <#name as ferro_type::TS>::typescript();
        }
    } else {
        quote! {}
    };

    // Generate namespace vec
    let namespace_expr = if namespace.is_empty() {
        quote! { vec![] }
    } else {
        let ns_strings = namespace.iter().map(|s| quote! { #s.to_string() });
        quote! { vec![#(#ns_strings),*] }
    };

    // Generate wrapper option
    let wrapper_expr = match wrapper {
        Some(w) => quote! { Some(#w.to_string()) },
        None => quote! { None },
    };

    Ok(quote! {
        impl #impl_generics ferro_type::TS for #name #ty_generics #where_clause {
            fn typescript() -> ferro_type::TypeDef {
                ferro_type::TypeDef::Named {
                    namespace: #namespace_expr,
                    name: #name_str.to_string(),
                    def: Box::new(#typedef_expr),
                    module: Some(module_path!().to_string()),
                    wrapper: #wrapper_expr,
                }
            }
        }

        #registration
    })
}
