//! Derive macros for ferrotype TypeScript type generation
//!
//! This crate provides:
//! - `#[derive(TypeScript)]` for generating TypeScript type definitions from Rust enums
//! - `#[rpc_method]` for marking methods as RPC endpoints

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
                }
                Ok(())
            })?;
        }

        Ok(result)
    }
}

/// Field-level attributes
#[derive(Default)]
struct FieldAttrs {
    /// Rename this specific field
    rename: Option<String>,
    /// Skip this field in the generated TypeScript
    skip: bool,
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
                }
                Ok(())
            })?;
        }

        Ok(result)
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

/// Derive macro for generating TypeScript type definitions from Rust enums.
///
/// # Examples
///
/// ## Unit variants
/// ```ignore
/// #[derive(TypeScript)]
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
/// #[derive(TypeScript)]
/// enum Coordinate {
///     D2(f64, f64),
///     D3(f64, f64, f64),
/// }
/// // Generates: { type: "D2"; value: [number, number] } | { type: "D3"; value: [number, number, number] }
/// ```
///
/// ## Struct variants
/// ```ignore
/// #[derive(TypeScript)]
/// enum Shape {
///     Circle { center: Point, radius: f64 },
///     Rectangle { x: f64, y: f64, width: f64, height: f64 },
/// }
/// // Generates: { type: "Circle"; center: Point; radius: number } | { type: "Rectangle"; x: number; y: number; width: number; height: number }
/// ```
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
            generate_impl(name, &type_name, generics, typedef)
        }
        Data::Struct(data) => {
            let typedef = generate_struct_typedef(&data.fields, &container_attrs)?;
            generate_impl(name, &type_name, generics, typedef)
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

    if all_unit {
        // Generate string literal union: "Pending" | "Active" | "Completed"
        let mut variant_exprs: Vec<TokenStream2> = Vec::new();
        for v in variants.iter() {
            let variant_attrs = FieldAttrs::from_attrs(&v.attrs)?;
            let name = get_field_name(&v.ident.to_string(), &variant_attrs, container_attrs);
            variant_exprs.push(
                quote! { ferrotype::TypeDef::Literal(ferrotype::Literal::String(#name.to_string())) }
            );
        }

        Ok(quote! {
            ferrotype::TypeDef::Union(vec![#(#variant_exprs),*])
        })
    } else {
        // Generate discriminated union with type field
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
                    // { type: "VariantName" }
                    quote! {
                        ferrotype::TypeDef::Object(vec![
                            ferrotype::Field::new(
                                "type",
                                ferrotype::TypeDef::Literal(ferrotype::Literal::String(#variant_name_str.to_string()))
                            )
                        ])
                    }
                }
                Fields::Unnamed(fields) => {
                    if fields.unnamed.len() == 1 {
                        // Newtype variant: { type: "Text"; value: T }
                        let field_type = &fields.unnamed.first().unwrap().ty;
                        let type_expr = type_to_typedef(field_type);
                        quote! {
                            ferrotype::TypeDef::Object(vec![
                                ferrotype::Field::new(
                                    "type",
                                    ferrotype::TypeDef::Literal(ferrotype::Literal::String(#variant_name_str.to_string()))
                                ),
                                ferrotype::Field::new("value", #type_expr)
                            ])
                        }
                    } else {
                        // Tuple variant: { type: "D2"; value: [T1, T2] }
                        let field_exprs: Vec<TokenStream2> = fields
                            .unnamed
                            .iter()
                            .map(|f| type_to_typedef(&f.ty))
                            .collect();
                        quote! {
                            ferrotype::TypeDef::Object(vec![
                                ferrotype::Field::new(
                                    "type",
                                    ferrotype::TypeDef::Literal(ferrotype::Literal::String(#variant_name_str.to_string()))
                                ),
                                ferrotype::Field::new(
                                    "value",
                                    ferrotype::TypeDef::Tuple(vec![#(#field_exprs),*])
                                )
                            ])
                        }
                    }
                }
                Fields::Named(fields) => {
                    // { type: "Circle"; center: Point; radius: number }
                    // Note: for struct variant fields, we don't apply rename_all from container
                    // (that's for variant names). Field renames are explicit only.
                    let mut field_exprs: Vec<TokenStream2> = Vec::new();
                    for f in fields.named.iter() {
                        let field_attrs = FieldAttrs::from_attrs(&f.attrs)?;
                        // Skip fields marked with #[ts(skip)]
                        if field_attrs.skip {
                            continue;
                        }
                        let original_name = f.ident.as_ref().unwrap().to_string();
                        let field_name = field_attrs.rename.clone().unwrap_or(original_name);
                        let field_type = &f.ty;
                        let type_expr = type_to_typedef(field_type);
                        field_exprs.push(quote! {
                            ferrotype::Field::new(#field_name, #type_expr)
                        });
                    }

                    quote! {
                        ferrotype::TypeDef::Object({
                            let mut fields = vec![
                                ferrotype::Field::new(
                                    "type",
                                    ferrotype::TypeDef::Literal(ferrotype::Literal::String(#variant_name_str.to_string()))
                                )
                            ];
                            fields.extend(vec![#(#field_exprs),*]);
                            fields
                        })
                    }
                }
            };
            variant_exprs.push(expr);
        }

        Ok(quote! {
            ferrotype::TypeDef::Union(vec![#(#variant_exprs),*])
        })
    }
}

fn generate_struct_typedef(
    fields: &syn::Fields,
    container_attrs: &ContainerAttrs,
) -> syn::Result<TokenStream2> {
    match fields {
        syn::Fields::Named(fields) => {
            // Named struct: Object with fields
            if fields.named.is_empty() {
                // Empty struct becomes empty object
                return Ok(quote! { ferrotype::TypeDef::Object(vec![]) });
            }

            let mut field_exprs: Vec<TokenStream2> = Vec::new();
            for f in fields.named.iter() {
                let field_attrs = FieldAttrs::from_attrs(&f.attrs)?;
                // Skip fields marked with #[ts(skip)]
                if field_attrs.skip {
                    continue;
                }
                let original_name = f.ident.as_ref().unwrap().to_string();
                let field_name = get_field_name(&original_name, &field_attrs, container_attrs);
                let field_type = &f.ty;
                let type_expr = type_to_typedef(field_type);
                field_exprs.push(quote! {
                    ferrotype::Field::new(#field_name, #type_expr)
                });
            }

            Ok(quote! {
                ferrotype::TypeDef::Object(vec![#(#field_exprs),*])
            })
        }
        syn::Fields::Unnamed(fields) => {
            // Tuple struct
            if fields.unnamed.len() == 1 {
                // Newtype: unwrap to inner type
                let field_type = &fields.unnamed.first().unwrap().ty;
                let type_expr = type_to_typedef(field_type);
                Ok(quote! { #type_expr })
            } else {
                // Tuple: [type1, type2, ...]
                let field_exprs: Vec<TokenStream2> = fields
                    .unnamed
                    .iter()
                    .map(|f| type_to_typedef(&f.ty))
                    .collect();

                Ok(quote! {
                    ferrotype::TypeDef::Tuple(vec![#(#field_exprs),*])
                })
            }
        }
        syn::Fields::Unit => {
            // Unit struct becomes null
            Ok(quote! { ferrotype::TypeDef::Primitive(ferrotype::Primitive::Null) })
        }
    }
}

/// Convert a Rust type to its TypeScript TypeDef representation.
/// Uses TypeScript trait for types that implement it.
fn type_to_typedef(ty: &Type) -> TokenStream2 {
    quote! { <#ty as ferrotype::TypeScript>::typescript() }
}

fn generate_impl(
    name: &Ident,
    name_str: &str,
    generics: &Generics,
    typedef_expr: TokenStream2,
) -> syn::Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Add TypeScript bounds to generic parameters
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
                quote! { #p: ferrotype::TypeScript }
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
        impl #impl_generics ferrotype::TypeScript for #name #ty_generics #where_clause {
            fn typescript() -> ferrotype::TypeDef {
                ferrotype::TypeDef::Named {
                    name: #name_str.to_string(),
                    def: Box::new(#typedef_expr),
                }
            }
        }
    })
}

// ============================================================================
// RPC METHOD ATTRIBUTE MACRO
// ============================================================================

/// Attribute macro for marking methods as RPC endpoints.
///
/// This macro is used to annotate methods within an RPC service implementation,
/// indicating they should be exposed as RPC methods. The macro validates the
/// method signature and passes it through unchanged.
///
/// # Attributes
///
/// - `name = "custom_name"` - Override the RPC method name (defaults to function name)
///
/// # Examples
///
/// ```ignore
/// impl MyService {
///     #[rpc_method]
///     fn get_user(&self, request: GetUserRequest) -> GetUserResponse {
///         // ...
///     }
///
///     #[rpc_method(name = "fetchUsers")]
///     fn list_users(&self, request: ListUsersRequest) -> ListUsersResponse {
///         // ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn rpc_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = syn::parse_macro_input!(attr as RpcMethodArgs);
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    match expand_rpc_method(&attr_args, &input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Arguments for the #[rpc_method] attribute.
struct RpcMethodArgs {
    name: Option<String>,
}

impl syn::parse::Parse for RpcMethodArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            if ident == "name" {
                input.parse::<syn::Token![=]>()?;
                let lit: syn::LitStr = input.parse()?;
                name = Some(lit.value());
            } else {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unknown attribute, expected `name`",
                ));
            }

            // Parse optional comma between arguments
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(RpcMethodArgs { name })
    }
}

fn expand_rpc_method(
    args: &RpcMethodArgs,
    input: &syn::ItemFn,
) -> syn::Result<TokenStream2> {
    let fn_name = &input.sig.ident;
    let _method_name = args
        .name
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or_else(|| fn_name.to_string().leak());

    // Validate the function signature has at least a self parameter and a request parameter
    if input.sig.inputs.len() < 2 {
        return Err(syn::Error::new_spanned(
            &input.sig,
            "RPC methods must have at least a self parameter and a request parameter",
        ));
    }

    // Validate first parameter is some form of self
    let first_param = input.sig.inputs.first().unwrap();
    match first_param {
        syn::FnArg::Receiver(_) => {}
        syn::FnArg::Typed(pat) => {
            if let syn::Pat::Ident(ident) = pat.pat.as_ref() {
                if ident.ident != "self" {
                    return Err(syn::Error::new_spanned(
                        first_param,
                        "RPC methods must have a self parameter",
                    ));
                }
            } else {
                return Err(syn::Error::new_spanned(
                    first_param,
                    "RPC methods must have a self parameter",
                ));
            }
        }
    }

    // Validate return type exists
    if matches!(input.sig.output, syn::ReturnType::Default) {
        return Err(syn::Error::new_spanned(
            &input.sig,
            "RPC methods must have a return type",
        ));
    }

    // Pass through the function unchanged
    // The macro serves as a marker that can be processed by other tools/macros
    Ok(quote! { #input })
}
