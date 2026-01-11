//! Ferrotype: Rust-to-TypeScript type mapping for Zed RPC
//!
//! This crate provides traits for generating TypeScript type definitions
//! from Rust types, enabling type-safe RPC communication.
//!
//! # Core Design
//!
//! The core abstraction is the [`TypeScript`] trait, modeled after serde's `Serialize`.
//! Rather than returning strings directly, it returns a [`TypeDef`] intermediate
//! representation that can be:
//! - Rendered to TypeScript syntax
//! - Analyzed for type dependencies
//! - Deduplicated for cleaner output
//! - Extended for additional targets

pub use ferro_type_derive::TypeScript;
pub use linkme;

use std::collections::HashMap;

// ============================================================================
// AUTO-REGISTRATION VIA DISTRIBUTED SLICE
// ============================================================================

/// Distributed slice for auto-registration of TypeScript types.
///
/// Types that derive `TypeScript` are automatically registered in this slice
/// when the `auto_register` feature is enabled (default). This allows collecting
/// all types without manual registration.
///
/// # Usage
///
/// ```ignore
/// // Types are automatically registered when you derive TypeScript
/// #[derive(TypeScript)]
/// struct User { name: String }
///
/// // Collect all registered types
/// let registry = TypeRegistry::from_distributed();
/// ```
#[linkme::distributed_slice]
pub static TYPESCRIPT_TYPES: [fn() -> TypeDef];

// ============================================================================
// CORE TRAIT AND IR (TypeScript + TypeDef)
// ============================================================================

/// The core trait for types that can be represented as TypeScript.
///
/// This is the foundation trait for ferrotype, similar to how `Serialize` is
/// the foundation of serde. Implementations return a [`TypeDef`] that describes
/// the TypeScript representation of the type.
///
/// # Design Philosophy
///
/// Unlike string-based approaches, returning a structured [`TypeDef`] enables:
/// - **Deduplication**: Named types can be collected and emitted once
/// - **Analysis**: Dependencies between types can be tracked
/// - **Flexibility**: The IR can be rendered in different styles
/// - **Composition**: Complex types build from simpler TypeDefs
///
/// # Example
///
/// ```ignore
/// use ferrotype::{TypeScript, TypeDef, Primitive};
///
/// struct UserId(String);
///
/// impl TypeScript for UserId {
///     fn typescript() -> TypeDef {
///         TypeDef::Named {
///             name: "UserId".into(),
///             def: Box::new(TypeDef::Primitive(Primitive::String)),
///         }
///     }
/// }
/// ```
pub trait TypeScript {
    /// Returns the TypeScript type definition for this type.
    fn typescript() -> TypeDef;
}

/// Intermediate representation for TypeScript types.
///
/// This enum represents all TypeScript types that ferrotype can generate.
/// It serves as the IR between Rust types and TypeScript output, enabling
/// analysis and transformation before rendering.
///
/// # Type Categories
///
/// - **Primitives**: `string`, `number`, `boolean`, `null`, etc.
/// - **Compounds**: Arrays, tuples, objects, unions, intersections
/// - **References**: Named types and type references
/// - **Literals**: Specific string, number, or boolean values
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDef {
    /// A primitive TypeScript type.
    Primitive(Primitive),

    /// An array type: `T[]`
    Array(Box<TypeDef>),

    /// A tuple type: `[T1, T2, ...]`
    Tuple(Vec<TypeDef>),

    /// An object type with named fields: `{ field1: T1; field2?: T2; }`
    Object(Vec<Field>),

    /// A union type: `T1 | T2 | ...`
    Union(Vec<TypeDef>),

    /// An intersection type: `T1 & T2 & ...`
    Intersection(Vec<TypeDef>),

    /// A record/dictionary type: `Record<K, V>` or `{ [key: K]: V }`
    Record {
        key: Box<TypeDef>,
        value: Box<TypeDef>,
    },

    /// A named type definition that should be emitted as a separate declaration.
    /// This is the primary mechanism for type deduplication.
    Named {
        name: String,
        def: Box<TypeDef>,
    },

    /// A reference to a named type. Used to avoid infinite recursion and
    /// to generate cleaner output by referencing previously-defined types.
    Ref(String),

    /// A literal type with a specific value.
    Literal(Literal),

    /// A function type: `(arg1: T1, arg2: T2) => R`
    Function {
        params: Vec<Field>,
        return_type: Box<TypeDef>,
    },

    /// A generic type application: `Generic<T1, T2>`
    Generic {
        base: String,
        args: Vec<TypeDef>,
    },
}

/// Primitive TypeScript types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    /// The `string` type.
    String,
    /// The `number` type.
    Number,
    /// The `boolean` type.
    Boolean,
    /// The `null` type.
    Null,
    /// The `undefined` type.
    Undefined,
    /// The `void` type (for functions that don't return a value).
    Void,
    /// The `never` type (for functions that never return).
    Never,
    /// The `any` type (escape hatch, use sparingly).
    Any,
    /// The `unknown` type (type-safe alternative to any).
    Unknown,
    /// The `bigint` type.
    BigInt,
}

/// A field in an object type.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// The field name.
    pub name: String,
    /// The field's type.
    pub ty: TypeDef,
    /// Whether the field is optional (`field?: T` vs `field: T`).
    pub optional: bool,
    /// Whether the field is readonly.
    pub readonly: bool,
}

impl Field {
    /// Creates a new required field.
    pub fn new(name: impl Into<String>, ty: TypeDef) -> Self {
        Self {
            name: name.into(),
            ty,
            optional: false,
            readonly: false,
        }
    }

    /// Creates a new optional field.
    pub fn optional(name: impl Into<String>, ty: TypeDef) -> Self {
        Self {
            name: name.into(),
            ty,
            optional: true,
            readonly: false,
        }
    }

    /// Makes this field readonly.
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
}

/// A literal TypeScript type with a specific value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// A string literal: `"foo"`
    String(String),
    /// A number literal: `42`
    Number(f64),
    /// A boolean literal: `true` or `false`
    Boolean(bool),
}

impl TypeDef {
    /// Renders this TypeDef to TypeScript syntax.
    pub fn render(&self) -> String {
        match self {
            TypeDef::Primitive(p) => p.render().to_string(),
            TypeDef::Array(inner) => {
                let inner_str = inner.render();
                // Wrap union types in parens for array syntax
                if matches!(inner.as_ref(), TypeDef::Union(_)) {
                    format!("({})[]", inner_str)
                } else {
                    format!("{}[]", inner_str)
                }
            }
            TypeDef::Tuple(items) => {
                let items_str: Vec<_> = items.iter().map(|t| t.render()).collect();
                format!("[{}]", items_str.join(", "))
            }
            TypeDef::Object(fields) => {
                if fields.is_empty() {
                    "{}".to_string()
                } else {
                    let fields_str: Vec<_> = fields
                        .iter()
                        .map(|f| {
                            let readonly = if f.readonly { "readonly " } else { "" };
                            let opt = if f.optional { "?" } else { "" };
                            format!("{}{}{}: {}", readonly, f.name, opt, f.ty.render())
                        })
                        .collect();
                    format!("{{ {} }}", fields_str.join("; "))
                }
            }
            TypeDef::Union(variants) => {
                let variants_str: Vec<_> = variants.iter().map(|t| t.render()).collect();
                variants_str.join(" | ")
            }
            TypeDef::Intersection(types) => {
                let types_str: Vec<_> = types.iter().map(|t| t.render()).collect();
                types_str.join(" & ")
            }
            TypeDef::Record { key, value } => {
                format!("Record<{}, {}>", key.render(), value.render())
            }
            TypeDef::Named { name, .. } => name.clone(),
            TypeDef::Ref(name) => name.clone(),
            TypeDef::Literal(lit) => lit.render(),
            TypeDef::Function {
                params,
                return_type,
            } => {
                let params_str: Vec<_> = params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.ty.render()))
                    .collect();
                format!("({}) => {}", params_str.join(", "), return_type.render())
            }
            TypeDef::Generic { base, args } => {
                let args_str: Vec<_> = args.iter().map(|t| t.render()).collect();
                format!("{}<{}>", base, args_str.join(", "))
            }
        }
    }

    /// Renders a full type declaration for named types.
    ///
    /// For `Named` types, this returns `type Name = Definition;`
    /// For other types, this just returns the rendered type.
    pub fn render_declaration(&self) -> String {
        match self {
            TypeDef::Named { name, def } => {
                format!("type {} = {};", name, def.render())
            }
            _ => self.render(),
        }
    }
}

impl Primitive {
    /// Renders this primitive to its TypeScript keyword.
    pub fn render(&self) -> &'static str {
        match self {
            Primitive::String => "string",
            Primitive::Number => "number",
            Primitive::Boolean => "boolean",
            Primitive::Null => "null",
            Primitive::Undefined => "undefined",
            Primitive::Void => "void",
            Primitive::Never => "never",
            Primitive::Any => "any",
            Primitive::Unknown => "unknown",
            Primitive::BigInt => "bigint",
        }
    }
}

impl Literal {
    /// Renders this literal to TypeScript syntax.
    pub fn render(&self) -> String {
        match self {
            Literal::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Literal::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Literal::Boolean(b) => b.to_string(),
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Extracts fields from an Object TypeDef, unwrapping Named if necessary.
///
/// This is used by the derive macro to implement `#[ts(flatten)]`. When a field
/// is marked with flatten, its type's fields are extracted and merged into the
/// containing object.
///
/// # Panics
///
/// Panics if the TypeDef is not an Object (or Named wrapping an Object).
pub fn extract_object_fields(typedef: &TypeDef) -> Vec<Field> {
    match typedef {
        TypeDef::Object(fields) => fields.clone(),
        TypeDef::Named { def, .. } => extract_object_fields(def),
        other => panic!(
            "#[ts(flatten)] can only be used on fields with object types, got: {:?}",
            other
        ),
    }
}

/// Extracts the inner type definition, unwrapping Named if necessary.
///
/// This is used by the derive macro to implement `#[ts(inline)]`. When a field
/// is marked with inline, the full type definition is inlined instead of using
/// a type reference.
pub fn inline_typedef(typedef: TypeDef) -> TypeDef {
    match typedef {
        TypeDef::Named { def, .. } => *def,
        other => other,
    }
}

// ============================================================================
// TYPE REGISTRY
// ============================================================================

use std::collections::{HashSet, VecDeque};

/// A registry for collecting and managing TypeScript type definitions.
///
/// The TypeRegistry collects named types, resolves their dependency order,
/// and renders them to a TypeScript file. This enables:
/// - **Deduplication**: Each named type is emitted once
/// - **Ordering**: Types are emitted in dependency order
/// - **Output**: Generate valid .ts or .d.ts files
///
/// # Example
///
/// ```ignore
/// use ferrotype::{TypeRegistry, TypeScript};
///
/// let mut registry = TypeRegistry::new();
/// registry.register::<User>();
/// registry.register::<Post>();
///
/// let output = registry.render();
/// std::fs::write("types.ts", output)?;
/// ```
#[derive(Debug, Default)]
pub struct TypeRegistry {
    /// Named type definitions, keyed by name
    types: HashMap<String, TypeDef>,
    /// Order in which types were registered (for stable output when no deps)
    registration_order: Vec<String>,
}

impl TypeRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a registry populated with all auto-registered types.
    ///
    /// This collects all types that were registered via the `#[derive(TypeScript)]`
    /// macro using the distributed slice mechanism.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use ferrotype::{TypeRegistry, TypeScript};
    ///
    /// #[derive(TypeScript)]
    /// struct User { name: String, age: u32 }
    ///
    /// #[derive(TypeScript)]
    /// struct Post { title: String, author: User }
    ///
    /// // Collect all types automatically - no manual registration needed!
    /// let registry = TypeRegistry::from_distributed();
    /// println!("{}", registry.render());
    /// ```
    pub fn from_distributed() -> Self {
        let mut registry = Self::new();
        for type_fn in TYPESCRIPT_TYPES {
            let typedef = type_fn();
            registry.add_typedef(typedef);
        }
        registry
    }

    /// Collects all auto-registered types into this registry.
    ///
    /// This is useful when you want to add auto-registered types to an existing
    /// registry that may already have some manually registered types.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut registry = TypeRegistry::new();
    /// // Add some manual types first
    /// registry.register::<SomeManualType>();
    /// // Then collect all auto-registered types
    /// registry.collect_all();
    /// ```
    pub fn collect_all(&mut self) {
        for type_fn in TYPESCRIPT_TYPES {
            let typedef = type_fn();
            self.add_typedef(typedef);
        }
    }

    /// Registers a type that implements TypeScript.
    ///
    /// This extracts all named types from the type definition and adds them
    /// to the registry. Named types are deduplicated by name.
    pub fn register<T: TypeScript>(&mut self) {
        let typedef = T::typescript();
        self.add_typedef(typedef);
    }

    /// Adds a TypeDef to the registry, extracting all named types.
    pub fn add_typedef(&mut self, typedef: TypeDef) {
        self.extract_named_types(&typedef);
    }

    /// Recursively extracts all Named types from a TypeDef.
    fn extract_named_types(&mut self, typedef: &TypeDef) {
        match typedef {
            TypeDef::Named { name, def } => {
                if !self.types.contains_key(name) {
                    self.types.insert(name.clone(), typedef.clone());
                    self.registration_order.push(name.clone());
                    // Also extract from the inner definition
                    self.extract_named_types(def);
                }
            }
            TypeDef::Array(inner) => self.extract_named_types(inner),
            TypeDef::Tuple(items) => {
                for item in items {
                    self.extract_named_types(item);
                }
            }
            TypeDef::Object(fields) => {
                for field in fields {
                    self.extract_named_types(&field.ty);
                }
            }
            TypeDef::Union(items) | TypeDef::Intersection(items) => {
                for item in items {
                    self.extract_named_types(item);
                }
            }
            TypeDef::Record { key, value } => {
                self.extract_named_types(key);
                self.extract_named_types(value);
            }
            TypeDef::Function { params, return_type } => {
                for param in params {
                    self.extract_named_types(&param.ty);
                }
                self.extract_named_types(return_type);
            }
            TypeDef::Generic { args, .. } => {
                for arg in args {
                    self.extract_named_types(arg);
                }
            }
            // Primitives, Refs, and Literals have no nested named types
            TypeDef::Primitive(_) | TypeDef::Ref(_) | TypeDef::Literal(_) => {}
        }
    }

    /// Returns the number of registered types.
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Returns true if no types are registered.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Returns the names of all registered types.
    pub fn type_names(&self) -> impl Iterator<Item = &str> {
        self.types.keys().map(|s| s.as_str())
    }

    /// Gets a type definition by name.
    pub fn get(&self, name: &str) -> Option<&TypeDef> {
        self.types.get(name)
    }

    /// Computes the dependencies for a type (what other named types it references).
    fn get_dependencies(&self, typedef: &TypeDef) -> HashSet<String> {
        let mut deps = HashSet::new();
        self.collect_dependencies(typedef, &mut deps);
        deps
    }

    /// Recursively collects dependencies from a TypeDef.
    fn collect_dependencies(&self, typedef: &TypeDef, deps: &mut HashSet<String>) {
        match typedef {
            TypeDef::Named { def, .. } => {
                // Don't add self as dependency, but check inner def
                self.collect_dependencies(def, deps);
            }
            TypeDef::Ref(name) => {
                if self.types.contains_key(name) {
                    deps.insert(name.clone());
                }
            }
            TypeDef::Array(inner) => self.collect_dependencies(inner, deps),
            TypeDef::Tuple(items) => {
                for item in items {
                    self.collect_dependencies(item, deps);
                }
            }
            TypeDef::Object(fields) => {
                for field in fields {
                    self.collect_dependencies(&field.ty, deps);
                }
            }
            TypeDef::Union(variants) => {
                for v in variants {
                    self.collect_dependencies(v, deps);
                }
            }
            TypeDef::Intersection(types) => {
                for t in types {
                    self.collect_dependencies(t, deps);
                }
            }
            TypeDef::Record { key, value } => {
                self.collect_dependencies(key, deps);
                self.collect_dependencies(value, deps);
            }
            TypeDef::Function { params, return_type } => {
                for param in params {
                    self.collect_dependencies(&param.ty, deps);
                }
                self.collect_dependencies(return_type, deps);
            }
            TypeDef::Generic { args, .. } => {
                for arg in args {
                    self.collect_dependencies(arg, deps);
                }
            }
            TypeDef::Primitive(_) | TypeDef::Literal(_) => {}
        }
    }

    /// Returns types in dependency order (types with no dependencies first).
    ///
    /// Uses Kahn's algorithm for topological sort.
    pub fn sorted_types(&self) -> Vec<&str> {
        // Build dependency graph
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialize
        for name in self.types.keys() {
            in_degree.insert(name.as_str(), 0);
            dependents.insert(name.as_str(), Vec::new());
        }

        // Calculate in-degrees and build reverse graph
        for (name, typedef) in &self.types {
            let deps = self.get_dependencies(typedef);
            for dep in deps {
                if let Some(dep_name) = self.types.get_key_value(&dep) {
                    *in_degree.get_mut(name.as_str()).unwrap() += 1;
                    dependents.get_mut(dep_name.0.as_str()).unwrap().push(name.as_str());
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<&str> = VecDeque::new();
        let mut result: Vec<&str> = Vec::new();

        // Start with types that have no dependencies
        for (name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(name);
            }
        }

        // Sort the initial queue by registration order for stable output
        let mut initial: Vec<_> = queue.drain(..).collect();
        initial.sort_by_key(|name| {
            self.registration_order.iter().position(|n| n == *name).unwrap_or(usize::MAX)
        });
        queue.extend(initial);

        while let Some(name) = queue.pop_front() {
            result.push(name);

            // Get dependents sorted by registration order for stable output
            let mut deps: Vec<_> = dependents.get(name).map(|v| v.as_slice()).unwrap_or(&[]).to_vec();
            deps.sort_by_key(|n| {
                self.registration_order.iter().position(|name| name == *n).unwrap_or(usize::MAX)
            });

            for dependent in deps {
                let degree = in_degree.get_mut(dependent).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(dependent);
                }
            }
        }

        // If result doesn't contain all types, there's a cycle
        // Fall back to registration order for remaining types
        if result.len() < self.types.len() {
            for name in &self.registration_order {
                if !result.contains(&name.as_str()) {
                    result.push(name.as_str());
                }
            }
        }

        result
    }

    /// Renders all registered types to TypeScript declarations.
    ///
    /// Types are emitted in dependency order, with proper formatting.
    pub fn render(&self) -> String {
        let sorted = self.sorted_types();
        let mut output = String::new();

        // Add header comment
        output.push_str("// Generated by ferrotype\n");
        output.push_str("// Do not edit manually\n\n");

        for name in sorted {
            if let Some(typedef) = self.types.get(name) {
                output.push_str(&typedef.render_declaration());
                output.push_str("\n\n");
            }
        }

        // Remove trailing newline
        output.trim_end().to_string() + "\n"
    }

    /// Renders all registered types with `export` keywords.
    pub fn render_exported(&self) -> String {
        let sorted = self.sorted_types();
        let mut output = String::new();

        // Add header comment
        output.push_str("// Generated by ferrotype\n");
        output.push_str("// Do not edit manually\n\n");

        for name in sorted {
            if let Some(typedef) = self.types.get(name) {
                if let TypeDef::Named { name, def } = typedef {
                    output.push_str(&format!("export type {} = {};\n\n", name, def.render()));
                }
            }
        }

        // Remove trailing newline
        output.trim_end().to_string() + "\n"
    }

    /// Clears all registered types.
    pub fn clear(&mut self) {
        self.types.clear();
        self.registration_order.clear();
    }
}

// ============================================================================
// TypeScript TRAIT IMPLEMENTATIONS FOR PRIMITIVES
// ============================================================================

impl TypeScript for () {
    fn typescript() -> TypeDef {
        TypeDef::Primitive(Primitive::Void)
    }
}

impl TypeScript for bool {
    fn typescript() -> TypeDef {
        TypeDef::Primitive(Primitive::Boolean)
    }
}

impl TypeScript for String {
    fn typescript() -> TypeDef {
        TypeDef::Primitive(Primitive::String)
    }
}

impl TypeScript for &str {
    fn typescript() -> TypeDef {
        TypeDef::Primitive(Primitive::String)
    }
}

impl TypeScript for char {
    fn typescript() -> TypeDef {
        TypeDef::Primitive(Primitive::String)
    }
}

macro_rules! impl_typescript_number {
    ($($t:ty),*) => {
        $(
            impl TypeScript for $t {
                fn typescript() -> TypeDef {
                    TypeDef::Primitive(Primitive::Number)
                }
            }
        )*
    };
}

impl_typescript_number!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

// i128/u128 map to bigint in TypeScript
impl TypeScript for i128 {
    fn typescript() -> TypeDef {
        TypeDef::Primitive(Primitive::BigInt)
    }
}

impl TypeScript for u128 {
    fn typescript() -> TypeDef {
        TypeDef::Primitive(Primitive::BigInt)
    }
}

// ============================================================================
// TypeScript TRAIT IMPLEMENTATIONS FOR GENERIC TYPES
// ============================================================================

impl<T: TypeScript> TypeScript for Option<T> {
    fn typescript() -> TypeDef {
        TypeDef::Union(vec![T::typescript(), TypeDef::Primitive(Primitive::Null)])
    }
}

impl<T: TypeScript> TypeScript for Vec<T> {
    fn typescript() -> TypeDef {
        TypeDef::Array(Box::new(T::typescript()))
    }
}

impl<T: TypeScript> TypeScript for Box<T> {
    fn typescript() -> TypeDef {
        T::typescript()
    }
}

impl<T: TypeScript> TypeScript for std::rc::Rc<T> {
    fn typescript() -> TypeDef {
        T::typescript()
    }
}

impl<T: TypeScript> TypeScript for std::sync::Arc<T> {
    fn typescript() -> TypeDef {
        T::typescript()
    }
}

impl<T: TypeScript> TypeScript for std::cell::RefCell<T> {
    fn typescript() -> TypeDef {
        T::typescript()
    }
}

impl<T: TypeScript> TypeScript for std::cell::Cell<T> {
    fn typescript() -> TypeDef {
        T::typescript()
    }
}

impl<K: TypeScript, V: TypeScript> TypeScript for HashMap<K, V> {
    fn typescript() -> TypeDef {
        TypeDef::Record {
            key: Box::new(K::typescript()),
            value: Box::new(V::typescript()),
        }
    }
}

impl<K: TypeScript, V: TypeScript> TypeScript for std::collections::BTreeMap<K, V> {
    fn typescript() -> TypeDef {
        TypeDef::Record {
            key: Box::new(K::typescript()),
            value: Box::new(V::typescript()),
        }
    }
}

impl<T: TypeScript, E: TypeScript> TypeScript for Result<T, E> {
    fn typescript() -> TypeDef {
        TypeDef::Union(vec![
            TypeDef::Object(vec![
                Field::new("ok", TypeDef::Literal(Literal::Boolean(true))),
                Field::new("value", T::typescript()),
            ]),
            TypeDef::Object(vec![
                Field::new("ok", TypeDef::Literal(Literal::Boolean(false))),
                Field::new("error", E::typescript()),
            ]),
        ])
    }
}

// ============================================================================
// TypeScript TRAIT IMPLEMENTATIONS FOR TUPLES
// ============================================================================

impl<A: TypeScript> TypeScript for (A,) {
    fn typescript() -> TypeDef {
        TypeDef::Tuple(vec![A::typescript()])
    }
}

impl<A: TypeScript, B: TypeScript> TypeScript for (A, B) {
    fn typescript() -> TypeDef {
        TypeDef::Tuple(vec![A::typescript(), B::typescript()])
    }
}

impl<A: TypeScript, B: TypeScript, C: TypeScript> TypeScript for (A, B, C) {
    fn typescript() -> TypeDef {
        TypeDef::Tuple(vec![A::typescript(), B::typescript(), C::typescript()])
    }
}

impl<A: TypeScript, B: TypeScript, C: TypeScript, D: TypeScript> TypeScript for (A, B, C, D) {
    fn typescript() -> TypeDef {
        TypeDef::Tuple(vec![
            A::typescript(),
            B::typescript(),
            C::typescript(),
            D::typescript(),
        ])
    }
}

impl<A: TypeScript, B: TypeScript, C: TypeScript, D: TypeScript, E: TypeScript> TypeScript
    for (A, B, C, D, E)
{
    fn typescript() -> TypeDef {
        TypeDef::Tuple(vec![
            A::typescript(),
            B::typescript(),
            C::typescript(),
            D::typescript(),
            E::typescript(),
        ])
    }
}

impl<A: TypeScript, B: TypeScript, C: TypeScript, D: TypeScript, E: TypeScript, F: TypeScript>
    TypeScript for (A, B, C, D, E, F)
{
    fn typescript() -> TypeDef {
        TypeDef::Tuple(vec![
            A::typescript(),
            B::typescript(),
            C::typescript(),
            D::typescript(),
            E::typescript(),
            F::typescript(),
        ])
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TypeScript TRAIT + TypeDef TESTS
    // ========================================================================

    #[test]
    fn test_typedef_primitive_render() {
        assert_eq!(TypeDef::Primitive(Primitive::String).render(), "string");
        assert_eq!(TypeDef::Primitive(Primitive::Number).render(), "number");
        assert_eq!(TypeDef::Primitive(Primitive::Boolean).render(), "boolean");
        assert_eq!(TypeDef::Primitive(Primitive::Null).render(), "null");
        assert_eq!(TypeDef::Primitive(Primitive::Undefined).render(), "undefined");
        assert_eq!(TypeDef::Primitive(Primitive::Void).render(), "void");
        assert_eq!(TypeDef::Primitive(Primitive::Never).render(), "never");
        assert_eq!(TypeDef::Primitive(Primitive::Any).render(), "any");
        assert_eq!(TypeDef::Primitive(Primitive::Unknown).render(), "unknown");
        assert_eq!(TypeDef::Primitive(Primitive::BigInt).render(), "bigint");
    }

    #[test]
    fn test_typedef_array_render() {
        let arr = TypeDef::Array(Box::new(TypeDef::Primitive(Primitive::String)));
        assert_eq!(arr.render(), "string[]");

        // Union in array should be wrapped in parens
        let union_arr = TypeDef::Array(Box::new(TypeDef::Union(vec![
            TypeDef::Primitive(Primitive::String),
            TypeDef::Primitive(Primitive::Number),
        ])));
        assert_eq!(union_arr.render(), "(string | number)[]");
    }

    #[test]
    fn test_typedef_tuple_render() {
        let tuple = TypeDef::Tuple(vec![
            TypeDef::Primitive(Primitive::String),
            TypeDef::Primitive(Primitive::Number),
        ]);
        assert_eq!(tuple.render(), "[string, number]");
    }

    #[test]
    fn test_typedef_object_render() {
        let obj = TypeDef::Object(vec![
            Field::new("name", TypeDef::Primitive(Primitive::String)),
            Field::optional("age", TypeDef::Primitive(Primitive::Number)),
        ]);
        assert_eq!(obj.render(), "{ name: string; age?: number }");

        let empty_obj = TypeDef::Object(vec![]);
        assert_eq!(empty_obj.render(), "{}");
    }

    #[test]
    fn test_typedef_object_readonly_field() {
        let obj = TypeDef::Object(vec![
            Field::new("id", TypeDef::Primitive(Primitive::String)).readonly(),
        ]);
        assert_eq!(obj.render(), "{ readonly id: string }");
    }

    #[test]
    fn test_typedef_union_render() {
        let union = TypeDef::Union(vec![
            TypeDef::Primitive(Primitive::String),
            TypeDef::Primitive(Primitive::Number),
            TypeDef::Primitive(Primitive::Null),
        ]);
        assert_eq!(union.render(), "string | number | null");
    }

    #[test]
    fn test_typedef_intersection_render() {
        let intersection = TypeDef::Intersection(vec![
            TypeDef::Ref("Base".into()),
            TypeDef::Object(vec![
                Field::new("extra", TypeDef::Primitive(Primitive::String)),
            ]),
        ]);
        assert_eq!(intersection.render(), "Base & { extra: string }");
    }

    #[test]
    fn test_typedef_record_render() {
        let record = TypeDef::Record {
            key: Box::new(TypeDef::Primitive(Primitive::String)),
            value: Box::new(TypeDef::Primitive(Primitive::Number)),
        };
        assert_eq!(record.render(), "Record<string, number>");
    }

    #[test]
    fn test_typedef_named_render() {
        let named = TypeDef::Named {
            name: "UserId".into(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
        };
        // Named types render as just their name (for inline use)
        assert_eq!(named.render(), "UserId");
        // Full declaration uses render_declaration
        assert_eq!(named.render_declaration(), "type UserId = string;");
    }

    #[test]
    fn test_typedef_ref_render() {
        let ref_type = TypeDef::Ref("User".into());
        assert_eq!(ref_type.render(), "User");
    }

    #[test]
    fn test_typedef_literal_render() {
        assert_eq!(TypeDef::Literal(Literal::String("foo".into())).render(), "\"foo\"");
        assert_eq!(TypeDef::Literal(Literal::Number(42.0)).render(), "42");
        assert_eq!(TypeDef::Literal(Literal::Number(3.14)).render(), "3.14");
        assert_eq!(TypeDef::Literal(Literal::Boolean(true)).render(), "true");
        assert_eq!(TypeDef::Literal(Literal::Boolean(false)).render(), "false");
    }

    #[test]
    fn test_typedef_literal_escaping() {
        let lit = Literal::String("say \"hello\"".into());
        assert_eq!(lit.render(), "\"say \\\"hello\\\"\"");
    }

    #[test]
    fn test_typedef_function_render() {
        let func = TypeDef::Function {
            params: vec![
                Field::new("name", TypeDef::Primitive(Primitive::String)),
                Field::new("age", TypeDef::Primitive(Primitive::Number)),
            ],
            return_type: Box::new(TypeDef::Primitive(Primitive::Void)),
        };
        assert_eq!(func.render(), "(name: string, age: number) => void");
    }

    #[test]
    fn test_typedef_generic_render() {
        let generic = TypeDef::Generic {
            base: "Promise".into(),
            args: vec![TypeDef::Primitive(Primitive::String)],
        };
        assert_eq!(generic.render(), "Promise<string>");

        let multi_generic = TypeDef::Generic {
            base: "Map".into(),
            args: vec![
                TypeDef::Primitive(Primitive::String),
                TypeDef::Primitive(Primitive::Number),
            ],
        };
        assert_eq!(multi_generic.render(), "Map<string, number>");
    }

    #[test]
    fn test_typescript_trait_primitives() {
        assert_eq!(<()>::typescript().render(), "void");
        assert_eq!(bool::typescript().render(), "boolean");
        assert_eq!(String::typescript().render(), "string");
        assert_eq!(i32::typescript().render(), "number");
        assert_eq!(f64::typescript().render(), "number");
        assert_eq!(i128::typescript().render(), "bigint");
        assert_eq!(u128::typescript().render(), "bigint");
    }

    #[test]
    fn test_typescript_trait_option() {
        let opt = <Option<String>>::typescript();
        assert_eq!(opt.render(), "string | null");
    }

    #[test]
    fn test_typescript_trait_vec() {
        let vec_type = <Vec<i32>>::typescript();
        assert_eq!(vec_type.render(), "number[]");
    }

    #[test]
    fn test_typescript_trait_hashmap() {
        let map = <HashMap<String, i32>>::typescript();
        assert_eq!(map.render(), "Record<string, number>");
    }

    #[test]
    fn test_typescript_trait_result() {
        let result = <Result<String, String>>::typescript();
        assert_eq!(
            result.render(),
            "{ ok: true; value: string } | { ok: false; error: string }"
        );
    }

    #[test]
    fn test_typescript_trait_tuples() {
        assert_eq!(<(String,)>::typescript().render(), "[string]");
        assert_eq!(<(String, i32)>::typescript().render(), "[string, number]");
        assert_eq!(
            <(String, i32, bool)>::typescript().render(),
            "[string, number, boolean]"
        );
    }

    #[test]
    fn test_typescript_trait_box() {
        // Box<T> should be transparent
        assert_eq!(<Box<String>>::typescript().render(), "string");
    }

    #[test]
    fn test_typedef_equality() {
        let a = TypeDef::Primitive(Primitive::String);
        let b = TypeDef::Primitive(Primitive::String);
        let c = TypeDef::Primitive(Primitive::Number);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_field_builder() {
        let field = Field::new("name", TypeDef::Primitive(Primitive::String));
        assert!(!field.optional);
        assert!(!field.readonly);

        let opt_field = Field::optional("name", TypeDef::Primitive(Primitive::String));
        assert!(opt_field.optional);

        let readonly_field = Field::new("id", TypeDef::Primitive(Primitive::String)).readonly();
        assert!(readonly_field.readonly);
    }

    // ========================================================================
    // TYPE REGISTRY TESTS
    // ========================================================================

    #[test]
    fn test_registry_new() {
        let registry = TypeRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_add_typedef() {
        let mut registry = TypeRegistry::new();

        let user_type = TypeDef::Named {
            name: "User".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("id", TypeDef::Primitive(Primitive::String)),
                Field::new("name", TypeDef::Primitive(Primitive::String)),
            ])),
        };

        registry.add_typedef(user_type);

        assert_eq!(registry.len(), 1);
        assert!(registry.get("User").is_some());
    }

    #[test]
    fn test_registry_deduplication() {
        let mut registry = TypeRegistry::new();

        let user_type = TypeDef::Named {
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
        };

        registry.add_typedef(user_type.clone());
        registry.add_typedef(user_type);

        // Should only have one User type
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_extracts_nested_types() {
        let mut registry = TypeRegistry::new();

        // UserId type (no dependencies)
        let user_id = TypeDef::Named {
            name: "UserId".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
        };

        // User type depends on UserId via Ref
        let user = TypeDef::Named {
            name: "User".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("id", TypeDef::Ref("UserId".to_string())),
                Field::new("name", TypeDef::Primitive(Primitive::String)),
            ])),
        };

        // Post type that references User type
        let post_type = TypeDef::Named {
            name: "Post".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("title", TypeDef::Primitive(Primitive::String)),
                Field::new("author", user),
            ])),
        };

        registry.add_typedef(post_type);
        registry.add_typedef(user_id);

        // Should have Post, User, and UserId
        assert_eq!(registry.len(), 3);
        assert!(registry.get("Post").is_some());
        assert!(registry.get("User").is_some());
        assert!(registry.get("UserId").is_some());
    }

    #[test]
    fn test_registry_render() {
        let mut registry = TypeRegistry::new();

        let user_type = TypeDef::Named {
            name: "User".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("id", TypeDef::Primitive(Primitive::String)),
                Field::new("name", TypeDef::Primitive(Primitive::String)),
            ])),
        };

        registry.add_typedef(user_type);

        let output = registry.render();
        assert!(output.contains("// Generated by ferrotype"));
        assert!(output.contains("type User = { id: string; name: string };"));
    }

    #[test]
    fn test_registry_render_exported() {
        let mut registry = TypeRegistry::new();

        let user_type = TypeDef::Named {
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
        };

        registry.add_typedef(user_type);

        let output = registry.render_exported();
        assert!(output.contains("export type User = string;"));
    }

    #[test]
    fn test_registry_dependency_order() {
        let mut registry = TypeRegistry::new();

        // UserId type (no dependencies)
        let user_id = TypeDef::Named {
            name: "UserId".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
        };

        // User type depends on UserId via Ref
        let user = TypeDef::Named {
            name: "User".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("id", TypeDef::Ref("UserId".to_string())),
                Field::new("name", TypeDef::Primitive(Primitive::String)),
            ])),
        };

        // Add in reverse order (User before UserId)
        registry.add_typedef(user);
        registry.add_typedef(user_id);

        let sorted = registry.sorted_types();

        // UserId should come before User
        let user_id_pos = sorted.iter().position(|&n| n == "UserId").unwrap();
        let user_pos = sorted.iter().position(|&n| n == "User").unwrap();
        assert!(user_id_pos < user_pos, "UserId should come before User");
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = TypeRegistry::new();

        let user_type = TypeDef::Named {
            name: "User".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
        };

        registry.add_typedef(user_type);
        assert_eq!(registry.len(), 1);

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_type_names() {
        let mut registry = TypeRegistry::new();

        registry.add_typedef(TypeDef::Named {
            name: "Alpha".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
        });
        registry.add_typedef(TypeDef::Named {
            name: "Beta".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::Number)),
        });

        let names: Vec<_> = registry.type_names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Alpha"));
        assert!(names.contains(&"Beta"));
    }

    #[test]
    fn test_registry_complex_dependencies() {
        let mut registry = TypeRegistry::new();

        // A -> B -> C (C should come first)
        let c = TypeDef::Named {
            name: "C".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
        };

        let b = TypeDef::Named {
            name: "B".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("c", TypeDef::Ref("C".to_string())),
            ])),
        };

        let a = TypeDef::Named {
            name: "A".to_string(),
            def: Box::new(TypeDef::Object(vec![
                Field::new("b", TypeDef::Ref("B".to_string())),
            ])),
        };

        // Add in wrong order
        registry.add_typedef(a);
        registry.add_typedef(b);
        registry.add_typedef(c);

        let sorted = registry.sorted_types();

        let c_pos = sorted.iter().position(|&n| n == "C").unwrap();
        let b_pos = sorted.iter().position(|&n| n == "B").unwrap();
        let a_pos = sorted.iter().position(|&n| n == "A").unwrap();

        assert!(c_pos < b_pos, "C should come before B");
        assert!(b_pos < a_pos, "B should come before A");
    }

    // ========================================================================
    // AUTO-REGISTRATION TESTS
    // ========================================================================

    // Test types for auto-registration
    #[derive(Debug)]
    struct AutoRegTestUser {
        name: String,
        age: u32,
    }

    impl TypeScript for AutoRegTestUser {
        fn typescript() -> TypeDef {
            TypeDef::Named {
                name: "AutoRegTestUser".to_string(),
                def: Box::new(TypeDef::Object(vec![
                    Field::new("name", TypeDef::Primitive(Primitive::String)),
                    Field::new("age", TypeDef::Primitive(Primitive::Number)),
                ])),
            }
        }
    }

    // Register manually for this test (the derive macro does this automatically)
    #[linkme::distributed_slice(TYPESCRIPT_TYPES)]
    static __TEST_REGISTER_USER: fn() -> TypeDef = || AutoRegTestUser::typescript();

    #[test]
    fn test_from_distributed_collects_types() {
        let registry = TypeRegistry::from_distributed();

        // The registry should contain our test type
        assert!(registry.get("AutoRegTestUser").is_some(),
            "Registry should contain AutoRegTestUser");
    }

    #[test]
    fn test_collect_all_adds_to_existing() {
        let mut registry = TypeRegistry::new();

        // Add a manual type first
        let manual_type = TypeDef::Named {
            name: "ManualType".to_string(),
            def: Box::new(TypeDef::Primitive(Primitive::String)),
        };
        registry.add_typedef(manual_type);

        // Then collect all auto-registered types
        registry.collect_all();

        // Should have both the manual type and auto-registered types
        assert!(registry.get("ManualType").is_some(),
            "Registry should contain ManualType");
        assert!(registry.get("AutoRegTestUser").is_some(),
            "Registry should contain AutoRegTestUser from distributed slice");
    }

    #[test]
    fn test_distributed_slice_is_accessible() {
        // Verify the distributed slice can be iterated
        let count = TYPESCRIPT_TYPES.len();
        // At minimum, we have our test type registered
        assert!(count >= 1, "TYPESCRIPT_TYPES should have at least 1 entry");
    }
}
