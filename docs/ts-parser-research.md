# TypeScript Parser Research for ferro-type-import

> Research for bead ft-dnjw: Evaluate TypeScript parsing options

## Executive Summary

**Recommendation: `swc_ecma_parser`**

For ferro-type-import's goal of parsing TypeScript type definitions and generating Rust code, `swc_ecma_parser` is the best choice:
- Full TypeScript AST with dedicated type structs (`TsInterfaceDecl`, `TsTypeAliasDecl`, etc.)
- Battle-tested in production (Next.js, Turbopack, Deno)
- Comprehensive type system support including generics, unions, mapped types
- Well-documented Rust API

## Options Evaluated

### 1. tree-sitter-typescript

**Overview**: Incremental parser that produces Concrete Syntax Trees (CST).

**Pros**:
- Fast incremental parsing (ideal for editors)
- Battle-tested in many editors (Zed, Helix, etc.)
- Language-agnostic query system
- Fixed serialization cost regardless of file size

**Cons**:
- Produces CST, not AST - requires custom queries to extract type information
- No dedicated TypeScript type structs - must navigate tree manually
- Limited semantic understanding of TypeScript types
- No parallel parsing support

**Usage**:
```rust
use tree_sitter::Parser;

let mut parser = Parser::new();
parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())?;
let tree = parser.parse(source_code, None)?;
// Must write custom tree-walking code to extract types
```

**Verdict**: Better suited for editor integrations and syntax highlighting. Too low-level for type extraction - would require significant custom code to navigate the CST and extract type definitions.

### 2. swc_ecma_parser (Recommended)

**Overview**: Full TypeScript/JavaScript parser used by Next.js and Deno.

**Pros**:
- Full AST with dedicated TypeScript type structs
- `TsInterfaceDecl`, `TsTypeAliasDecl`, `TsEnumDecl` directly accessible
- Handles all TypeScript syntax including advanced patterns
- Parallel parsing support
- Passes tc39/test262 test suite
- Active development, production-proven

**Cons**:
- Larger dependency footprint (~300KB source)
- Linear serialization cost with file size
- More complex API than tree-sitter

**Key TypeScript AST Types**:
- `TsInterfaceDecl` - interface declarations
- `TsTypeAliasDecl` - type alias declarations
- `TsEnumDecl` - enum declarations
- `TsUnionType` / `TsIntersectionType` - union/intersection types
- `TsTypeRef` - type references
- `TsPropertySignature` / `TsMethodSignature` - interface members
- `TsConditionalType`, `TsMappedType` - advanced type features

**Usage**: See POC below.

**Verdict**: Best fit for ferro-type-import. Provides direct access to TypeScript type constructs we need to generate Rust code.

### 3. oxc_parser

**Overview**: Newer, highly optimized parser from the OXC project.

**Pros**:
- Extremely fast (arena-based allocation via bumpalo)
- Minimal API: 3 inputs, 1 output
- Full TypeScript support including JSX/TSX
- Smaller memory footprint due to u32 spans

**Cons**:
- Newer/less battle-tested than SWC
- 4GB file size limit (u32 offsets)
- Less documentation
- AST types in separate `oxc_ast` crate

**Usage**:
```rust
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

let allocator = Allocator::default();
let source_type = SourceType::from_path("types.d.ts")?;
let ret = Parser::new(&allocator, source, source_type).parse();
```

**Verdict**: Promising alternative. Worth considering if SWC's dependency size becomes an issue, but SWC's maturity and documentation make it safer for initial implementation.

### 4. Custom Parser

**Overview**: Hand-written parser tailored to our specific needs.

**Pros**:
- Minimal dependencies
- Can be optimized for our exact use case
- Full control over error handling

**Cons**:
- Significant engineering effort
- Must handle TypeScript's complex grammar (generics, conditional types, etc.)
- Edge cases will be discovered in production
- Maintenance burden

**Verdict**: Not recommended. TypeScript's type syntax is complex enough that leveraging an existing, tested parser is the right choice. Would only make sense if we needed an extremely narrow subset.

## Comparison Matrix

| Feature | tree-sitter | swc | oxc | custom |
|---------|-------------|-----|-----|--------|
| TypeScript AST types | CST only | Full AST | Full AST | N/A |
| Type extraction ease | Hard | Easy | Easy | Variable |
| Battle-tested | Yes (editors) | Yes (Next.js) | Growing | No |
| Dependency size | ~100KB | ~300KB | ~200KB | 0 |
| Parallel parsing | No | Yes | Yes | Variable |
| Documentation | Good | Good | Fair | N/A |
| Generic types support | Manual | Built-in | Built-in | Manual |
| Recommended for | Editors | Compilers | Compilers | Narrow use |

## POC: Parsing TypeScript with swc_ecma_parser

### Cargo.toml Dependencies

```toml
[dependencies]
swc_common = { version = "5", features = ["tty-emitter"] }
swc_ecma_parser = "22"
swc_ecma_ast = "5"
```

### Example: Extract Interface Definitions

```rust
use swc_common::{
    sync::Lrc,
    FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_ast::*;

fn parse_typescript(source: &str) -> Result<Module, String> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom("input.ts".into())),
        source.into(),
    );

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
    parser.parse_module().map_err(|e| format!("{:?}", e))
}

/// Extract interface declarations from a parsed module
fn extract_interfaces(module: &Module) -> Vec<&TsInterfaceDecl> {
    module.body.iter().filter_map(|item| {
        match item {
            ModuleItem::Stmt(Stmt::Decl(Decl::TsInterface(iface))) => Some(iface.as_ref()),
            _ => None,
        }
    }).collect()
}

/// Extract type alias declarations from a parsed module
fn extract_type_aliases(module: &Module) -> Vec<&TsTypeAliasDecl> {
    module.body.iter().filter_map(|item| {
        match item {
            ModuleItem::Stmt(Stmt::Decl(Decl::TsTypeAlias(alias))) => Some(alias.as_ref()),
            _ => None,
        }
    }).collect()
}

fn main() -> Result<(), String> {
    let ts_source = r#"
        interface User {
            id: string;
            name: string;
            age?: number;
            roles: string[];
        }

        type UserId = string;

        type Result<T> = { ok: true; value: T } | { ok: false; error: string };

        enum Status {
            Active = "active",
            Inactive = "inactive",
        }
    "#;

    let module = parse_typescript(ts_source)?;

    // Extract interfaces
    let interfaces = extract_interfaces(&module);
    for iface in interfaces {
        println!("Interface: {}", iface.id.sym);
        for member in &iface.body.body {
            if let TsTypeElement::TsPropertySignature(prop) = member {
                if let Expr::Ident(ident) = prop.key.as_ref() {
                    let optional = if prop.optional { "?" } else { "" };
                    println!("  {}{}: {:?}", ident.sym, optional, prop.type_ann);
                }
            }
        }
    }

    // Extract type aliases
    let aliases = extract_type_aliases(&module);
    for alias in aliases {
        println!("Type alias: {}", alias.id.sym);
    }

    Ok(())
}
```

### Key AST Navigation Patterns

**Interface members**:
```rust
match &member {
    TsTypeElement::TsPropertySignature(prop) => { /* field */ }
    TsTypeElement::TsMethodSignature(method) => { /* method */ }
    TsTypeElement::TsIndexSignature(index) => { /* [key: K]: V */ }
    _ => {}
}
```

**Type expressions**:
```rust
match type_ann.as_ref() {
    TsType::TsKeywordType(kw) => match kw.kind {
        TsKeywordTypeKind::TsStringKeyword => "String",
        TsKeywordTypeKind::TsNumberKeyword => "f64",
        TsKeywordTypeKind::TsBooleanKeyword => "bool",
        // ...
    },
    TsType::TsArrayType(arr) => { /* Vec<elem_type> */ }
    TsType::TsUnionType(union) => { /* enum or Option */ }
    TsType::TsTypeRef(ref_) => { /* named type reference */ }
    TsType::TsTypeLit(lit) => { /* inline object type */ }
    // ...
}
```

## Implementation Roadmap

1. **Phase 1**: Basic parsing with swc_ecma_parser
   - Parse `.ts` and `.d.ts` files
   - Extract interfaces, type aliases, enums

2. **Phase 2**: Type mapping IR
   - Build intermediate representation mirroring ferrotype's `TypeDef`
   - Handle primitives, arrays, optionals

3. **Phase 3**: Rust code generation
   - Generate structs with serde derives
   - Handle field naming conventions (camelCase → snake_case)

4. **Phase 4**: Advanced patterns
   - Discriminated unions → Rust enums
   - Generic types → Rust generics
   - Indexed access types

## Conclusion

**Use `swc_ecma_parser`** for ferro-type-import. It provides:
- Direct access to TypeScript type definitions via dedicated AST structs
- Production-proven reliability (Next.js, Deno)
- Comprehensive TypeScript support including advanced type features
- Good Rust documentation and examples

The dependency cost (~300KB) is justified by the significant reduction in implementation complexity compared to tree-sitter (which would require custom CST navigation) or a custom parser (which would require implementing TypeScript's complex grammar).

## Sources

- [swc_ecma_parser Documentation](https://rustdoc.swc.rs/swc_ecma_parser/)
- [swc_ecma_ast Documentation](https://rustdoc.swc.rs/swc_ecma_ast/)
- [tree-sitter-typescript](https://docs.rs/tree-sitter-typescript)
- [oxc_parser](https://docs.rs/oxc_parser)
- [TypeScript Parser Benchmarks](https://dev.to/herrington_darkholme/benchmark-typescript-parsers-demystify-rust-tooling-performance-2go8)
