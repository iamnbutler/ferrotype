//! Derive macros for ferrotype TypeScript type generation
//!
//! This crate provides:
//! - `#[derive(TypeScript)]` for generating TypeScript type definitions from Rust enums
//! - `#[rpc_method]` for marking methods as RPC endpoints

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, GenericParam, Generics, Ident, Type,
};

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
#[proc_macro_derive(TypeScript)]
pub fn derive_typescript(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_derive_typescript(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_derive_typescript(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let name_str = name.to_string();
    let generics = &input.generics;

    match &input.data {
        Data::Enum(data) => {
            let typescript_type = generate_enum_typescript(&data.variants)?;
            generate_impl(name, &name_str, generics, typescript_type)
        }
        Data::Struct(data) => {
            let typescript_type = generate_struct_typescript(&data.fields)?;
            generate_impl(name, &name_str, generics, typescript_type)
        }
        Data::Union(_) => {
            Err(syn::Error::new_spanned(
                input,
                "TypeScript derive is not supported for unions",
            ))
        }
    }
}

fn generate_enum_typescript(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
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
        let variant_strings: Vec<String> = variants
            .iter()
            .map(|v| format!(r#""{}""#, v.ident))
            .collect();
        let joined = variant_strings.join(" | ");
        Ok(quote! { #joined.to_string() })
    } else {
        // Generate discriminated union with type field
        let mut variant_exprs: Vec<TokenStream2> = Vec::new();

        for variant in variants.iter() {
            let variant_name = &variant.ident;
            let variant_name_str = variant_name.to_string();

            match &variant.fields {
                Fields::Unit => {
                    // { type: "VariantName" }
                    let ts = format!(r#"{{ type: "{}" }}"#, variant_name_str);
                    variant_exprs.push(quote! { #ts.to_string() });
                }
                Fields::Unnamed(fields) => {
                    // { type: "VariantName"; value: [T1, T2, ...] } for tuples
                    // { type: "VariantName"; value: T } for newtype
                    if fields.unnamed.len() == 1 {
                        // Newtype variant: { type: "Text"; value: string }
                        let field_type = &fields.unnamed.first().unwrap().ty;
                        let type_expr = type_to_typescript(field_type);
                        variant_exprs.push(quote! {
                            format!(r#"{{ type: "{}"; value: {} }}"#, #variant_name_str, #type_expr)
                        });
                    } else {
                        // Tuple variant: { type: "D2"; value: [number, number] }
                        let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                        let type_exprs: Vec<TokenStream2> = field_types
                            .iter()
                            .map(|t| type_to_typescript(t))
                            .collect();

                        variant_exprs.push(quote! {
                            {
                                let types = vec![#(#type_exprs),*];
                                format!(r#"{{ type: "{}"; value: [{}] }}"#, #variant_name_str, types.join(", "))
                            }
                        });
                    }
                }
                Fields::Named(fields) => {
                    // { type: "Circle"; center: Point; radius: number }
                    let field_parts: Vec<_> = fields.named.iter().map(|f| {
                        let field_name = f.ident.as_ref().unwrap().to_string();
                        let field_type = &f.ty;
                        let type_expr = type_to_typescript(field_type);
                        quote! {
                            format!("{}: {}", #field_name, #type_expr)
                        }
                    }).collect();

                    variant_exprs.push(quote! {
                        {
                            let fields = vec![#(#field_parts),*];
                            format!(r#"{{ type: "{}"; {} }}"#, #variant_name_str, fields.join("; "))
                        }
                    });
                }
            }
        }

        Ok(quote! {
            {
                let variants = vec![#(#variant_exprs),*];
                variants.join(" | ")
            }
        })
    }
}

fn generate_struct_typescript(fields: &syn::Fields) -> syn::Result<TokenStream2> {
    match fields {
        syn::Fields::Named(fields) => {
            // Named struct: { field1: type1; field2: type2 }
            if fields.named.is_empty() {
                // Empty struct becomes empty object
                return Ok(quote! { "{}".to_string() });
            }

            let field_parts: Vec<TokenStream2> = fields
                .named
                .iter()
                .map(|f| {
                    let field_name = f.ident.as_ref().unwrap().to_string();
                    let field_type = &f.ty;
                    let type_expr = type_to_typescript(field_type);
                    quote! {
                        format!("{}: {}", #field_name, #type_expr)
                    }
                })
                .collect();

            Ok(quote! {
                {
                    let fields = vec![#(#field_parts),*];
                    format!("{{ {} }}", fields.join("; "))
                }
            })
        }
        syn::Fields::Unnamed(fields) => {
            // Tuple struct: [type1, type2, ...]
            if fields.unnamed.len() == 1 {
                // Newtype: unwrap to inner type
                let field_type = &fields.unnamed.first().unwrap().ty;
                let type_expr = type_to_typescript(field_type);
                Ok(quote! { #type_expr })
            } else {
                // Tuple: [type1, type2]
                let field_types: Vec<TokenStream2> = fields
                    .unnamed
                    .iter()
                    .map(|f| type_to_typescript(&f.ty))
                    .collect();

                Ok(quote! {
                    {
                        let types = vec![#(#field_types),*];
                        format!("[{}]", types.join(", "))
                    }
                })
            }
        }
        syn::Fields::Unit => {
            // Unit struct becomes null/void
            Ok(quote! { "null".to_string() })
        }
    }
}

/// Convert a Rust type to its TypeScript representation.
/// Uses TypeScriptType trait for types that implement it.
fn type_to_typescript(ty: &Type) -> TokenStream2 {
    quote! { <#ty as ferrotype::TypeScriptType>::typescript_type() }
}

fn generate_impl(
    name: &Ident,
    name_str: &str,
    generics: &Generics,
    typescript_type_expr: TokenStream2,
) -> syn::Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Add TypeScriptType bounds to generic parameters
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
                quote! { #p: ferrotype::TypeScriptType }
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
        impl #impl_generics ferrotype::TypeScriptType for #name #ty_generics #where_clause {
            fn typescript_type() -> String {
                #typescript_type_expr
            }

            fn typescript_name() -> &'static str {
                #name_str
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
