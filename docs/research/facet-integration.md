# Facet Integration Research

**Issue**: ft-e58g
**Date**: 2026-01-11
**Author**: keeper

## Executive Summary

**Recommendation: SKIP** - Facet is not a good fit for ferrotype's existing functionality. The architectural mismatch (runtime reflection vs compile-time code generation) means facet would add complexity without improving our current approach.

## What Facet Provides

[Facet](https://docs.rs/facet/latest/facet/) is a compile-time reflection crate that gives types a `SHAPE` associated const with:
- Field names, types, and offsets
- Doc comments and attributes
- Enum variants and discriminants
- Virtual tables for type-erased operations

Key characteristic: **Facet generates data, not code.** It uses the lighter `unsynn` parser instead of `syn`.

## Analysis by Focus Area

### 1. Indexed Access Validation

**Current approach**: We generate validation code in const blocks:
```rust
const __VALIDATE_INDEXED_ACCESS_FOO: () = {
    fn _validate(v: &Profile) { let _ = &v.login; }
};
```

**Could facet help?**: No. Facet provides field information at *runtime* via `SHAPE`. Our validation needs to happen at *compile-time*. Our current approach already validates field existence with clear error messages showing available fields.

**Verdict**: No improvement possible.

### 2. Field Iteration in Proc Macro

**Current approach**: Parse struct fields using `syn`:
```rust
for f in fields.named.iter() {
    let field_attrs = FieldAttrs::from_attrs(&f.attrs)?;
    // Process field...
}
```

**Could facet help?**: Not directly. Facet's derive macro generates a `SHAPE` const, but ferrotype needs to *generate code* (TypeDef expressions). We can't use facet's Shape at proc-macro time because it's runtime data.

Facet uses `unsynn` which is lighter than `syn`, but switching parsers would be a significant refactor for marginal compile-time benefit.

**Verdict**: No practical improvement. Different architectural model.

### 3. Type Name Resolution

**Current approach**: Use `stringify!` for type-to-string conversion:
```rust
let index_str = match index_spec {
    IndexSpec::Type(ty) => quote! { stringify!(#ty).to_string() },
    IndexSpec::String(s) => quote! { #s.to_string() },
};
```

**Could facet help?**: Facet provides type names via `Shape::name`, but this is runtime data. Our proc macro needs type names at compile time for code generation.

**Verdict**: No improvement. Different problem domain.

### 4. Generic Handling

**Current approach**: Add TypeScript bounds to generic params:
```rust
let bounds = type_params.iter().map(|p| {
    quote! { #p: ferro_type::TypeScript }
});
```

**Could facet help?**: No. Facet's approach is runtime reflection with type erasure. Our generic handling is compile-time trait bounds - fundamentally different.

**Verdict**: No improvement. Orthogonal concerns.

## Architectural Mismatch

The core issue is **ferrotype and facet solve different problems**:

| Aspect | Ferrotype | Facet |
|--------|-----------|-------|
| Goal | Generate TypeScript types | Runtime reflection |
| Output | Code (TypeDef expressions) | Data (SHAPE const) |
| When | Compile-time code gen | Runtime inspection |
| Use case | Static TS bindings | Serialization, debugging |

Facet excels at runtime reflection scenarios (serialization, pretty-printing, CLI parsing). Ferrotype is a compile-time code generator that produces TypeScript type definitions.

## Compile Time Consideration

One argument for facet is faster compile times (unsynn vs syn). However:
- ferrotype-derive is already a small, focused crate
- Switching to unsynn would require rewriting attribute parsing
- The benefit is marginal for our codebase size

## Conclusion

**Recommendation: SKIP**

Facet is an excellent crate for its intended use case (runtime reflection), but it's architecturally misaligned with ferrotype's needs. Adopting facet would:
- Add a dependency without improving functionality
- Not help with any of the four focus areas
- Potentially complicate the codebase with an unused abstraction

If facet gains compile-time reflection features in the future (there's a Rust project goal for this), we should revisit. For now, our syn-based approach is appropriate.

## Sources

- [facet docs.rs](https://docs.rs/facet/latest/facet/)
- [Introducing facet: Reflection for Rust](https://fasterthanli.me/articles/introducing-facet-reflection-for-rust)
- [Shape struct](https://docs.rs/facet/latest/facet/struct.Shape.html)
