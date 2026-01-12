//! Convert swc AST to ferrotype TypeDef
//!
//! This module transforms TypeScript AST nodes into ferrotype's intermediate
//! representation (TypeDef), enabling subsequent Rust code generation.

use ferro_type::{Field, Literal, Primitive, TypeDef, TypeParam};
use swc_core::ecma::ast::*;

use crate::TsTypeInfo;

/// Convert a parsed TypeScript module to a list of type definitions.
///
/// Extracts all interface declarations and type aliases from the module.
pub fn convert_module(module: &Module) -> Vec<TsTypeInfo> {
    let mut types = Vec::new();

    for item in &module.body {
        match item {
            ModuleItem::Stmt(Stmt::Decl(Decl::TsInterface(iface))) => {
                if let Some(info) = convert_interface(iface) {
                    types.push(info);
                }
            }
            ModuleItem::Stmt(Stmt::Decl(Decl::TsTypeAlias(alias))) => {
                if let Some(info) = convert_type_alias(alias) {
                    types.push(info);
                }
            }
            ModuleItem::Stmt(Stmt::Decl(Decl::TsEnum(ts_enum))) => {
                if let Some(info) = convert_ts_enum(ts_enum) {
                    types.push(info);
                }
            }
            _ => {}
        }
    }

    types
}

/// Convert a TypeScript interface declaration to TsTypeInfo.
fn convert_interface(iface: &TsInterfaceDecl) -> Option<TsTypeInfo> {
    let name = iface.id.sym.to_string();
    let fields = convert_interface_body(&iface.body);

    // Handle generic type parameters
    let typedef = if let Some(ref type_params) = iface.type_params {
        let params: Vec<TypeParam> = type_params
            .params
            .iter()
            .map(convert_type_param)
            .collect();

        TypeDef::GenericDef {
            name: name.clone(),
            type_params: params,
            def: Box::new(TypeDef::Object(fields)),
        }
    } else {
        TypeDef::Named {
            namespace: vec![],
            name: name.clone(),
            def: Box::new(TypeDef::Object(fields)),
            module: None,
            wrapper: None,
        }
    };

    Some(TsTypeInfo {
        name,
        typedef,
        is_interface: true,
    })
}

/// Convert a TypeScript type alias declaration to TsTypeInfo.
fn convert_type_alias(alias: &TsTypeAliasDecl) -> Option<TsTypeInfo> {
    let name = alias.id.sym.to_string();
    let inner_type = convert_ts_type(&alias.type_ann);

    // Handle generic type parameters
    let typedef = if let Some(ref type_params) = alias.type_params {
        let params: Vec<TypeParam> = type_params
            .params
            .iter()
            .map(convert_type_param)
            .collect();

        TypeDef::GenericDef {
            name: name.clone(),
            type_params: params,
            def: Box::new(inner_type),
        }
    } else {
        TypeDef::Named {
            namespace: vec![],
            name: name.clone(),
            def: Box::new(inner_type),
            module: None,
            wrapper: None,
        }
    };

    Some(TsTypeInfo {
        name,
        typedef,
        is_interface: false,
    })
}

/// Convert a TypeScript enum declaration to TsTypeInfo.
///
/// TypeScript enums are converted to a union of literals, which codegen
/// then renders as a Rust enum.
fn convert_ts_enum(ts_enum: &TsEnumDecl) -> Option<TsTypeInfo> {
    let name = ts_enum.id.sym.to_string();

    // Collect enum variants
    let variants: Vec<TypeDef> = ts_enum
        .members
        .iter()
        .filter_map(|member| {
            // Get the member name
            let member_name = match &member.id {
                TsEnumMemberId::Ident(ident) => ident.sym.to_string(),
                TsEnumMemberId::Str(s) => s.value.as_str().unwrap_or("").to_string(),
            };

            // Check if there's an explicit initializer
            if let Some(init) = &member.init {
                match init.as_ref() {
                    Expr::Lit(Lit::Str(s)) => {
                        Some(TypeDef::Literal(Literal::String(s.value.as_str().unwrap_or("").to_string())))
                    }
                    Expr::Lit(Lit::Num(n)) => Some(TypeDef::Literal(Literal::Number(n.value))),
                    _ => {
                        // Use member name as the literal value for computed initializers
                        Some(TypeDef::Literal(Literal::String(member_name)))
                    }
                }
            } else {
                // No initializer - use member name as string literal
                Some(TypeDef::Literal(Literal::String(member_name)))
            }
        })
        .collect();

    let typedef = TypeDef::Named {
        namespace: vec![],
        name: name.clone(),
        def: Box::new(TypeDef::Union(variants)),
        module: None,
        wrapper: None,
    };

    Some(TsTypeInfo {
        name,
        typedef,
        is_interface: false,
    })
}

/// Convert a type parameter (e.g., `T`, `T extends string`).
fn convert_type_param(param: &TsTypeParam) -> TypeParam {
    let name = param.name.sym.to_string();
    let mut type_param = TypeParam::new(name);

    if let Some(ref constraint) = param.constraint {
        type_param = type_param.with_constraint(convert_ts_type(constraint));
    }

    if let Some(ref default) = param.default {
        type_param = type_param.with_default(convert_ts_type(default));
    }

    type_param
}

/// Convert interface body members to Field vec.
fn convert_interface_body(body: &TsInterfaceBody) -> Vec<Field> {
    body.body
        .iter()
        .filter_map(|member| {
            match member {
                TsTypeElement::TsPropertySignature(prop) => convert_property_signature(prop),
                _ => None, // Skip methods, index signatures for now
            }
        })
        .collect()
}

/// Convert a property signature to a Field.
fn convert_property_signature(prop: &TsPropertySignature) -> Option<Field> {
    // Extract property name
    let name = match prop.key.as_ref() {
        Expr::Ident(ident) => ident.sym.to_string(),
        _ => return None, // Skip computed properties
    };

    // Extract type annotation
    let ty = prop
        .type_ann
        .as_ref()
        .map(|ann| convert_ts_type(&ann.type_ann))
        .unwrap_or(TypeDef::Primitive(Primitive::Any));

    let field = if prop.optional {
        Field::optional(name, ty)
    } else {
        Field::new(name, ty)
    };

    // Handle readonly
    let field = if prop.readonly {
        field.readonly()
    } else {
        field
    };

    Some(field)
}

/// Convert a TypeScript type to ferrotype TypeDef.
fn convert_ts_type(ts_type: &TsType) -> TypeDef {
    match ts_type {
        TsType::TsKeywordType(kw) => convert_keyword_type(kw),
        TsType::TsArrayType(arr) => {
            TypeDef::Array(Box::new(convert_ts_type(&arr.elem_type)))
        }
        TsType::TsUnionOrIntersectionType(union_or_inter) => {
            convert_union_or_intersection(union_or_inter)
        }
        TsType::TsTypeRef(type_ref) => convert_type_ref(type_ref),
        TsType::TsTypeLit(lit) => convert_type_literal(lit),
        TsType::TsLitType(lit) => convert_literal_type(lit),
        TsType::TsTupleType(tuple) => convert_tuple_type(tuple),
        TsType::TsParenthesizedType(paren) => convert_ts_type(&paren.type_ann),
        TsType::TsOptionalType(opt) => {
            // T? becomes T | null
            TypeDef::Union(vec![
                convert_ts_type(&opt.type_ann),
                TypeDef::Primitive(Primitive::Null),
            ])
        }
        TsType::TsFnOrConstructorType(fn_type) => convert_function_type(fn_type),
        TsType::TsIndexedAccessType(indexed) => convert_indexed_access(indexed),
        TsType::TsTypeQuery(_) => TypeDef::Primitive(Primitive::Any), // typeof expressions
        TsType::TsMappedType(_) => TypeDef::Primitive(Primitive::Any), // Mapped types - complex
        TsType::TsConditionalType(_) => TypeDef::Primitive(Primitive::Any), // Conditional types
        TsType::TsInferType(_) => TypeDef::Primitive(Primitive::Any), // infer keyword
        TsType::TsThisType(_) => TypeDef::Primitive(Primitive::Any), // this type
        TsType::TsTypeOperator(_) => TypeDef::Primitive(Primitive::Any), // keyof, readonly, unique
        TsType::TsRestType(rest) => TypeDef::Array(Box::new(convert_ts_type(&rest.type_ann))),
        TsType::TsTypePredicate(_) => TypeDef::Primitive(Primitive::Boolean), // Type predicates
        TsType::TsImportType(_) => TypeDef::Primitive(Primitive::Any), // import("...").Type
    }
}

/// Convert TypeScript keyword types to primitives.
fn convert_keyword_type(kw: &TsKeywordType) -> TypeDef {
    match kw.kind {
        TsKeywordTypeKind::TsStringKeyword => TypeDef::Primitive(Primitive::String),
        TsKeywordTypeKind::TsNumberKeyword => TypeDef::Primitive(Primitive::Number),
        TsKeywordTypeKind::TsBooleanKeyword => TypeDef::Primitive(Primitive::Boolean),
        TsKeywordTypeKind::TsNullKeyword => TypeDef::Primitive(Primitive::Null),
        TsKeywordTypeKind::TsUndefinedKeyword => TypeDef::Primitive(Primitive::Undefined),
        TsKeywordTypeKind::TsVoidKeyword => TypeDef::Primitive(Primitive::Void),
        TsKeywordTypeKind::TsNeverKeyword => TypeDef::Primitive(Primitive::Never),
        TsKeywordTypeKind::TsAnyKeyword => TypeDef::Primitive(Primitive::Any),
        TsKeywordTypeKind::TsUnknownKeyword => TypeDef::Primitive(Primitive::Unknown),
        TsKeywordTypeKind::TsBigIntKeyword => TypeDef::Primitive(Primitive::BigInt),
        TsKeywordTypeKind::TsObjectKeyword => TypeDef::Object(vec![]), // object keyword
        TsKeywordTypeKind::TsSymbolKeyword => TypeDef::Primitive(Primitive::Any), // No symbol in Rust
        TsKeywordTypeKind::TsIntrinsicKeyword => TypeDef::Primitive(Primitive::Any),
    }
}

/// Convert union or intersection types.
fn convert_union_or_intersection(ts_type: &TsUnionOrIntersectionType) -> TypeDef {
    match ts_type {
        TsUnionOrIntersectionType::TsUnionType(union) => {
            let variants: Vec<TypeDef> = union.types.iter().map(|t| convert_ts_type(t)).collect();
            TypeDef::Union(variants)
        }
        TsUnionOrIntersectionType::TsIntersectionType(inter) => {
            let types: Vec<TypeDef> = inter.types.iter().map(|t| convert_ts_type(t)).collect();
            TypeDef::Intersection(types)
        }
    }
}

/// Convert type references (named types, generics).
fn convert_type_ref(type_ref: &TsTypeRef) -> TypeDef {
    let name = match &type_ref.type_name {
        TsEntityName::Ident(ident) => ident.sym.to_string(),
        TsEntityName::TsQualifiedName(qual) => {
            // Handle qualified names like Namespace.Type
            format_qualified_name(qual)
        }
    };

    // Handle built-in generic types
    match name.as_str() {
        "Array" => {
            if let Some(ref type_params) = type_ref.type_params {
                if let Some(first) = type_params.params.first() {
                    return TypeDef::Array(Box::new(convert_ts_type(first)));
                }
            }
            TypeDef::Array(Box::new(TypeDef::Primitive(Primitive::Any)))
        }
        "Record" => {
            if let Some(ref type_params) = type_ref.type_params {
                let params: Vec<_> = type_params.params.iter().collect();
                if params.len() >= 2 {
                    return TypeDef::Record {
                        key: Box::new(convert_ts_type(params[0])),
                        value: Box::new(convert_ts_type(params[1])),
                    };
                }
            }
            TypeDef::Record {
                key: Box::new(TypeDef::Primitive(Primitive::String)),
                value: Box::new(TypeDef::Primitive(Primitive::Any)),
            }
        }
        "Promise" | "Map" | "Set" | "WeakMap" | "WeakSet" => {
            // Handle generic built-ins
            if let Some(ref type_params) = type_ref.type_params {
                let args: Vec<TypeDef> = type_params
                    .params
                    .iter()
                    .map(|t| convert_ts_type(t))
                    .collect();
                TypeDef::Generic { base: name, args }
            } else {
                TypeDef::Generic {
                    base: name,
                    args: vec![],
                }
            }
        }
        _ => {
            // User-defined type reference
            if let Some(ref type_params) = type_ref.type_params {
                let args: Vec<TypeDef> = type_params
                    .params
                    .iter()
                    .map(|t| convert_ts_type(t))
                    .collect();
                if args.is_empty() {
                    TypeDef::Ref(name)
                } else {
                    TypeDef::Generic { base: name, args }
                }
            } else {
                TypeDef::Ref(name)
            }
        }
    }
}

/// Format a qualified name (e.g., Namespace.Type).
fn format_qualified_name(qual: &TsQualifiedName) -> String {
    let left = match &qual.left {
        TsEntityName::Ident(ident) => ident.sym.to_string(),
        TsEntityName::TsQualifiedName(nested) => format_qualified_name(nested),
    };
    format!("{}.{}", left, qual.right.sym)
}

/// Convert type literals (inline object types).
fn convert_type_literal(lit: &TsTypeLit) -> TypeDef {
    let fields: Vec<Field> = lit
        .members
        .iter()
        .filter_map(|member| match member {
            TsTypeElement::TsPropertySignature(prop) => convert_property_signature(prop),
            TsTypeElement::TsIndexSignature(index) => {
                // { [key: string]: value } -> Record<string, value>
                // For simplicity, we'll skip index signatures for now
                let _ = index;
                None
            }
            _ => None,
        })
        .collect();

    TypeDef::Object(fields)
}

/// Convert literal types (string literals, number literals, etc.).
fn convert_literal_type(lit: &TsLitType) -> TypeDef {
    match &lit.lit {
        TsLit::Str(s) => TypeDef::Literal(Literal::String(s.value.as_str().unwrap_or("").to_string())),
        TsLit::Number(n) => TypeDef::Literal(Literal::Number(n.value)),
        TsLit::Bool(b) => TypeDef::Literal(Literal::Boolean(b.value)),
        TsLit::BigInt(_) => TypeDef::Primitive(Primitive::BigInt),
        TsLit::Tpl(_) => TypeDef::Primitive(Primitive::String), // Template literal types
    }
}

/// Convert tuple types.
fn convert_tuple_type(tuple: &TsTupleType) -> TypeDef {
    let elements: Vec<TypeDef> = tuple
        .elem_types
        .iter()
        .map(|elem| convert_ts_type(&elem.ty))
        .collect();
    TypeDef::Tuple(elements)
}

/// Convert function types.
fn convert_function_type(fn_type: &TsFnOrConstructorType) -> TypeDef {
    match fn_type {
        TsFnOrConstructorType::TsFnType(fn_sig) => {
            let params: Vec<Field> = fn_sig
                .params
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let (name, ty) = extract_param_info(param, i);
                    Field::new(name, ty)
                })
                .collect();

            let return_type = convert_ts_type(&fn_sig.type_ann.type_ann);

            TypeDef::Function {
                params,
                return_type: Box::new(return_type),
            }
        }
        TsFnOrConstructorType::TsConstructorType(_) => {
            // Constructor types are rare in data modeling
            TypeDef::Primitive(Primitive::Any)
        }
    }
}

/// Extract parameter name and type from function parameter.
fn extract_param_info(param: &TsFnParam, index: usize) -> (String, TypeDef) {
    match param {
        TsFnParam::Ident(ident) => {
            let name = ident.sym.as_str().to_string();
            let ty = ident
                .type_ann
                .as_ref()
                .map(|ann| convert_ts_type(&ann.type_ann))
                .unwrap_or(TypeDef::Primitive(Primitive::Any));
            (name, ty)
        }
        TsFnParam::Array(arr) => {
            let ty = arr
                .type_ann
                .as_ref()
                .map(|ann| convert_ts_type(&ann.type_ann))
                .unwrap_or(TypeDef::Primitive(Primitive::Any));
            (format!("arg{}", index), ty)
        }
        TsFnParam::Rest(rest) => {
            let ty = rest
                .type_ann
                .as_ref()
                .map(|ann| convert_ts_type(&ann.type_ann))
                .unwrap_or(TypeDef::Array(Box::new(TypeDef::Primitive(Primitive::Any))));
            (format!("rest{}", index), ty)
        }
        TsFnParam::Object(obj) => {
            let ty = obj
                .type_ann
                .as_ref()
                .map(|ann| convert_ts_type(&ann.type_ann))
                .unwrap_or(TypeDef::Primitive(Primitive::Any));
            (format!("arg{}", index), ty)
        }
    }
}

/// Convert indexed access types (e.g., T["key"]).
fn convert_indexed_access(indexed: &TsIndexedAccessType) -> TypeDef {
    // For now, only handle simple cases like Type["property"]
    let base = match indexed.obj_type.as_ref() {
        TsType::TsTypeRef(type_ref) => match &type_ref.type_name {
            TsEntityName::Ident(ident) => ident.sym.to_string(),
            TsEntityName::TsQualifiedName(qual) => format_qualified_name(qual),
        },
        _ => return TypeDef::Primitive(Primitive::Any),
    };

    let key = match indexed.index_type.as_ref() {
        TsType::TsLitType(lit) => match &lit.lit {
            TsLit::Str(s) => s.value.as_str().unwrap_or("").to_string(),
            _ => return TypeDef::Primitive(Primitive::Any),
        },
        _ => return TypeDef::Primitive(Primitive::Any),
    };

    TypeDef::IndexedAccess { base, key }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_typescript;

    #[test]
    fn test_convert_simple_interface() {
        let source = r#"
            interface User {
                id: string;
                name: string;
            }
        "#;
        let module = parse_typescript(source).unwrap();
        let types = convert_module(&module);

        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "User");
        assert!(types[0].is_interface);
    }

    #[test]
    fn test_convert_optional_field() {
        let source = r#"
            interface Config {
                required: string;
                optional?: number;
            }
        "#;
        let module = parse_typescript(source).unwrap();
        let types = convert_module(&module);

        assert_eq!(types.len(), 1);
        if let TypeDef::Named { def, .. } = &types[0].typedef {
            if let TypeDef::Object(fields) = def.as_ref() {
                assert_eq!(fields.len(), 2);
                assert!(!fields[0].optional);
                assert!(fields[1].optional);
            } else {
                panic!("Expected Object typedef");
            }
        } else {
            panic!("Expected Named typedef");
        }
    }

    #[test]
    fn test_convert_array_type() {
        let source = r#"
            interface Container {
                items: string[];
            }
        "#;
        let module = parse_typescript(source).unwrap();
        let types = convert_module(&module);

        assert_eq!(types.len(), 1);
        if let TypeDef::Named { def, .. } = &types[0].typedef {
            if let TypeDef::Object(fields) = def.as_ref() {
                assert!(matches!(fields[0].ty, TypeDef::Array(_)));
            }
        }
    }

    #[test]
    fn test_convert_union_type() {
        let source = "type Result = string | number;";
        let module = parse_typescript(source).unwrap();
        let types = convert_module(&module);

        assert_eq!(types.len(), 1);
        if let TypeDef::Named { def, .. } = &types[0].typedef {
            assert!(matches!(def.as_ref(), TypeDef::Union(_)));
        }
    }

    #[test]
    fn test_convert_type_alias() {
        let source = "type UserId = string;";
        let module = parse_typescript(source).unwrap();
        let types = convert_module(&module);

        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "UserId");
        assert!(!types[0].is_interface);
    }

    #[test]
    fn test_convert_literal_type() {
        let source = r#"type Status = "active" | "inactive";"#;
        let module = parse_typescript(source).unwrap();
        let types = convert_module(&module);

        assert_eq!(types.len(), 1);
        if let TypeDef::Named { def, .. } = &types[0].typedef {
            if let TypeDef::Union(variants) = def.as_ref() {
                assert_eq!(variants.len(), 2);
                assert!(matches!(&variants[0], TypeDef::Literal(Literal::String(s)) if s == "active"));
            }
        }
    }

    #[test]
    fn test_convert_generic_interface() {
        let source = r#"
            interface Container<T> {
                value: T;
            }
        "#;
        let module = parse_typescript(source).unwrap();
        let types = convert_module(&module);

        assert_eq!(types.len(), 1);
        assert!(matches!(&types[0].typedef, TypeDef::GenericDef { .. }));
    }

    #[test]
    fn test_convert_ts_enum() {
        let source = r#"
            enum Status {
                Active,
                Inactive,
                Pending
            }
        "#;
        let module = parse_typescript(source).unwrap();
        let types = convert_module(&module);

        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "Status");
        if let TypeDef::Named { def, .. } = &types[0].typedef {
            if let TypeDef::Union(variants) = def.as_ref() {
                assert_eq!(variants.len(), 3);
                assert!(matches!(&variants[0], TypeDef::Literal(Literal::String(s)) if s == "Active"));
            } else {
                panic!("Expected Union typedef");
            }
        } else {
            panic!("Expected Named typedef");
        }
    }

    #[test]
    fn test_convert_ts_enum_with_string_values() {
        let source = r#"
            enum Direction {
                Up = "UP",
                Down = "DOWN",
                Left = "LEFT",
                Right = "RIGHT"
            }
        "#;
        let module = parse_typescript(source).unwrap();
        let types = convert_module(&module);

        assert_eq!(types.len(), 1);
        if let TypeDef::Named { def, .. } = &types[0].typedef {
            if let TypeDef::Union(variants) = def.as_ref() {
                assert_eq!(variants.len(), 4);
                assert!(matches!(&variants[0], TypeDef::Literal(Literal::String(s)) if s == "UP"));
            }
        }
    }
}
