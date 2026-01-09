//! Derive macros for ferrotype TypeScript type generation
//!
//! This crate provides `#[derive(TypeScript)]` for generating TypeScript type
//! definitions from Rust enums as discriminated unions.

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
        Data::Struct(_) => {
            Err(syn::Error::new_spanned(
                input,
                "TypeScript derive for structs is not yet implemented. Use ft-a4r task.",
            ))
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
