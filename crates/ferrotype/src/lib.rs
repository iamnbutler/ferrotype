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

pub use ferrotype_derive::{rpc_method, TypeScript};

use std::collections::HashMap;

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

/// Trait for types that can be represented as TypeScript types.
///
/// Implementors provide both a type name (for references) and a full
/// type definition (for inline or anonymous types).
pub trait TypeScriptType {
    /// Returns the TypeScript type definition as a string.
    ///
    /// For primitive types, this returns the type directly (e.g., "number", "string").
    /// For complex types, this returns the full inline type definition.
    fn typescript_type() -> String;

    /// Returns the TypeScript type name for use in references.
    ///
    /// This is typically the name used when the type is exported or referenced
    /// elsewhere (e.g., "MyStruct", "UserResponse").
    fn typescript_name() -> &'static str;
}

/// Marker trait for types that can be used as RPC request parameters.
///
/// Types implementing this trait can be serialized and sent as part of
/// an RPC request payload.
pub trait RpcParam: TypeScriptType {}

/// Marker trait for types that can be returned from RPC calls.
///
/// Types implementing this trait can be deserialized from an RPC response.
pub trait RpcReturn: TypeScriptType {}

/// Marker trait for types that represent RPC errors.
///
/// Types implementing this trait can be used as error types in RPC methods.
/// Error types should typically be enums with discriminated variants or
/// structs with error code/message fields.
pub trait RpcErrorType: TypeScriptType {
    /// Generates TypeScript type guards for this error type.
    ///
    /// For enum error types, this generates guards like `isNotFoundError(err)`.
    /// Returns an empty string if type guards are not applicable.
    fn typescript_type_guards() -> String {
        String::new()
    }
}

/// Information about an error variant for TypeScript codegen.
#[derive(Debug, Clone)]
pub struct ErrorVariantInfo {
    /// The variant/error code name
    pub name: &'static str,
    /// TypeScript type for this error variant
    pub typescript_type: String,
}

impl ErrorVariantInfo {
    /// Creates a new ErrorVariantInfo.
    pub fn new(name: &'static str, typescript_type: String) -> Self {
        Self {
            name,
            typescript_type,
        }
    }

    /// Generates a TypeScript type guard function for this error variant.
    pub fn typescript_type_guard(&self, error_type_name: &str) -> String {
        let guard_name = format!("is{}Error", self.name);
        format!(
            r#"function {}(error: {}): error is {{ type: "{}" }} {{
  return (error as any).type === "{}";
}}"#,
            guard_name, error_type_name, self.name, self.name
        )
    }
}

/// Trait for enum error types that can enumerate their variants.
///
/// This trait enables automatic generation of TypeScript type guards
/// for each error variant.
pub trait EnumerableError: RpcErrorType {
    /// Returns information about all error variants.
    fn error_variants() -> Vec<ErrorVariantInfo>;

    /// Generates TypeScript type guards for all error variants.
    fn generate_all_type_guards() -> String {
        let type_name = Self::typescript_name();
        Self::error_variants()
            .iter()
            .map(|v| v.typescript_type_guard(type_name))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

// ============================================================================
// RPC SERVICE TRAIT
// ============================================================================

/// Information about an RPC method including request/response types.
///
/// This struct captures the metadata needed to generate TypeScript interfaces
/// for RPC methods, including the method name and its parameter/return types.
#[derive(Debug, Clone)]
pub struct RpcMethodInfo {
    /// The method name as it appears in the RPC interface.
    pub name: &'static str,
    /// TypeScript type representation of the request parameters.
    pub request_type: String,
    /// TypeScript type representation of the response.
    pub response_type: String,
}

impl RpcMethodInfo {
    /// Creates a new RpcMethodInfo with the given name and types.
    pub fn new(name: &'static str, request_type: String, response_type: String) -> Self {
        Self {
            name,
            request_type,
            response_type,
        }
    }
}

/// Trait for RPC services that can have methods registered.
///
/// This trait provides the foundation for defining RPC services that can
/// generate TypeScript interfaces. Implementations define the service name
/// and its available methods, enabling automatic TypeScript client generation.
///
/// # Examples
///
/// ```ignore
/// struct UserService;
///
/// impl RpcService for UserService {
///     fn service_name() -> &'static str {
///         "UserService"
///     }
///
///     fn methods() -> Vec<RpcMethodInfo> {
///         vec![
///             RpcMethodInfo {
///                 name: "getUser",
///                 request_type: "{ id: string }".to_string(),
///                 response_type: "User".to_string(),
///             },
///         ]
///     }
/// }
/// ```
pub trait RpcService {
    /// Returns the service name used in TypeScript interface generation.
    fn service_name() -> &'static str;

    /// Returns all registered RPC methods with their type signatures.
    fn methods() -> Vec<RpcMethodInfo>;

    /// Generates the TypeScript interface definition for this service.
    ///
    /// The generated interface includes all methods with their request/response
    /// types wrapped in Promise for async operation.
    fn typescript_interface() -> String {
        let methods: Vec<String> = Self::methods()
            .iter()
            .map(|m| {
                format!(
                    "  {}(request: {}): Promise<{}>",
                    m.name, m.request_type, m.response_type
                )
            })
            .collect();

        format!(
            "interface {} {{\n{}\n}}",
            Self::service_name(),
            methods.join(";\n")
        )
    }

    /// Generates a TypeScript client class for this service.
    ///
    /// The generated client includes:
    /// - Constructor with baseUrl and optional fetch function
    /// - Type-safe methods for each RPC endpoint
    /// - Automatic JSON serialization/deserialization
    ///
    /// # Example Output
    ///
    /// ```typescript
    /// class UserServiceClient {
    ///   private readonly baseUrl: string;
    ///   private readonly fetch: typeof fetch;
    ///
    ///   constructor(baseUrl: string, fetchFn: typeof fetch = fetch) {
    ///     this.baseUrl = baseUrl;
    ///     this.fetch = fetchFn;
    ///   }
    ///
    ///   async getUser(request: { id: string }): Promise<User> {
    ///     const response = await this.fetch(`${this.baseUrl}/UserService/getUser`, {
    ///       method: 'POST',
    ///       headers: { 'Content-Type': 'application/json' },
    ///       body: JSON.stringify(request),
    ///     });
    ///     if (!response.ok) {
    ///       throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    ///     }
    ///     return response.json();
    ///   }
    /// }
    /// ```
    fn typescript_client() -> String {
        let service_name = Self::service_name();
        let class_name = format!("{}Client", service_name);

        let methods: Vec<String> = Self::methods()
            .iter()
            .map(|m| {
                format!(
                    r#"  async {}(request: {}): Promise<{}> {{
    const response = await this.fetch(`${{this.baseUrl}}/{}/{}`, {{
      method: 'POST',
      headers: {{ 'Content-Type': 'application/json' }},
      body: JSON.stringify(request),
    }});
    if (!response.ok) {{
      throw new Error(`HTTP ${{response.status}}: ${{response.statusText}}`);
    }}
    return response.json();
  }}"#,
                    m.name, m.request_type, m.response_type, service_name, m.name
                )
            })
            .collect();

        format!(
            r#"class {} {{
  private readonly baseUrl: string;
  private readonly fetch: typeof fetch;

  constructor(baseUrl: string, fetchFn: typeof fetch = fetch) {{
    this.baseUrl = baseUrl;
    this.fetch = fetchFn;
  }}

{}
}}"#,
            class_name,
            methods.join("\n\n")
        )
    }

    /// Generates a TypeScript request builder for type-safe request construction.
    ///
    /// The generated code includes:
    /// - Generic `RequestBuilder<TRequest, TResponse>` class
    /// - Fluent API with type inference for setting fields
    /// - Execute method that sends the built request
    ///
    /// # Example Output
    ///
    /// ```typescript
    /// class RequestBuilder<TRequest, TResponse> {
    ///   private data: Partial<TRequest> = {};
    ///
    ///   constructor(
    ///     private readonly client: UserServiceClient,
    ///     private readonly methodName: string,
    ///   ) {}
    ///
    ///   set<K extends keyof TRequest>(key: K, value: TRequest[K]): this {
    ///     this.data[key] = value;
    ///     return this;
    ///   }
    ///
    ///   build(): TRequest {
    ///     return this.data as TRequest;
    ///   }
    ///
    ///   async execute(): Promise<TResponse> {
    ///     return (this.client as any)[this.methodName](this.build());
    ///   }
    /// }
    /// ```
    fn typescript_request_builder() -> String {
        let service_name = Self::service_name();
        let client_class = format!("{}Client", service_name);
        let builder_class = format!("{}RequestBuilder", service_name);

        // Generate builder factory methods for each RPC method
        let factory_methods: Vec<String> = Self::methods()
            .iter()
            .map(|m| {
                format!(
                    r#"  {}(): RequestBuilder<{}, {}> {{
    return new RequestBuilder<{}, {}>(this.client, '{}');
  }}"#,
                    m.name, m.request_type, m.response_type,
                    m.request_type, m.response_type, m.name
                )
            })
            .collect();

        format!(
            r#"/**
 * Generic request builder with type-safe field setting and execution.
 */
class RequestBuilder<TRequest, TResponse> {{
  private data: Partial<TRequest> = {{}};

  constructor(
    private readonly client: {client},
    private readonly methodName: string,
  ) {{}}

  /**
   * Set a field on the request with full type inference.
   */
  set<K extends keyof TRequest>(key: K, value: TRequest[K]): this {{
    this.data[key] = value;
    return this;
  }}

  /**
   * Set multiple fields at once.
   */
  setAll(fields: Partial<TRequest>): this {{
    Object.assign(this.data, fields);
    return this;
  }}

  /**
   * Build the final request object.
   */
  build(): TRequest {{
    return this.data as TRequest;
  }}

  /**
   * Execute the request and return the response.
   */
  async execute(): Promise<TResponse> {{
    return (this.client as any)[this.methodName](this.build());
  }}
}}

/**
 * Factory for creating type-safe request builders for {service}.
 */
class {builder} {{
  constructor(private readonly client: {client}) {{}}

{methods}
}}"#,
            client = client_class,
            service = service_name,
            builder = builder_class,
            methods = factory_methods.join("\n\n")
        )
    }
}

// ============================================================================
// PRIMITIVE IMPLEMENTATIONS
// ============================================================================

impl TypeScriptType for () {
    fn typescript_type() -> String {
        "void".to_string()
    }

    fn typescript_name() -> &'static str {
        "void"
    }
}

impl TypeScriptType for bool {
    fn typescript_type() -> String {
        "boolean".to_string()
    }

    fn typescript_name() -> &'static str {
        "boolean"
    }
}

impl TypeScriptType for String {
    fn typescript_type() -> String {
        "string".to_string()
    }

    fn typescript_name() -> &'static str {
        "string"
    }
}

impl TypeScriptType for &str {
    fn typescript_type() -> String {
        "string".to_string()
    }

    fn typescript_name() -> &'static str {
        "string"
    }
}

macro_rules! impl_typescript_number {
    ($($t:ty),*) => {
        $(
            impl TypeScriptType for $t {
                fn typescript_type() -> String {
                    "number".to_string()
                }

                fn typescript_name() -> &'static str {
                    "number"
                }
            }
        )*
    };
}

impl_typescript_number!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

// ============================================================================
// GENERIC TYPE IMPLEMENTATIONS
// ============================================================================

impl<T: TypeScriptType> TypeScriptType for Option<T> {
    fn typescript_type() -> String {
        format!("{} | null", T::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Option"
    }
}

impl<T: TypeScriptType> TypeScriptType for Vec<T> {
    fn typescript_type() -> String {
        format!("{}[]", T::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Array"
    }
}

impl<K: TypeScriptType, V: TypeScriptType> TypeScriptType for HashMap<K, V> {
    fn typescript_type() -> String {
        format!("Record<{}, {}>", K::typescript_type(), V::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Record"
    }
}

impl<T: TypeScriptType, E: TypeScriptType> TypeScriptType for Result<T, E> {
    fn typescript_type() -> String {
        format!(
            "{{ ok: true; value: {} }} | {{ ok: false; error: {} }}",
            T::typescript_type(),
            E::typescript_type()
        )
    }

    fn typescript_name() -> &'static str {
        "Result"
    }
}

impl<T: TypeScriptType> TypeScriptType for Box<T> {
    fn typescript_type() -> String {
        T::typescript_type()
    }

    fn typescript_name() -> &'static str {
        T::typescript_name()
    }
}

// ============================================================================
// TUPLE IMPLEMENTATIONS
// ============================================================================

impl<A: TypeScriptType> TypeScriptType for (A,) {
    fn typescript_type() -> String {
        format!("[{}]", A::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Tuple1"
    }
}

impl<A: TypeScriptType, B: TypeScriptType> TypeScriptType for (A, B) {
    fn typescript_type() -> String {
        format!("[{}, {}]", A::typescript_type(), B::typescript_type())
    }

    fn typescript_name() -> &'static str {
        "Tuple2"
    }
}

impl<A: TypeScriptType, B: TypeScriptType, C: TypeScriptType> TypeScriptType for (A, B, C) {
    fn typescript_type() -> String {
        format!(
            "[{}, {}, {}]",
            A::typescript_type(),
            B::typescript_type(),
            C::typescript_type()
        )
    }

    fn typescript_name() -> &'static str {
        "Tuple3"
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // NEW TypeScript TRAIT + TypeDef TESTS
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
    // LEGACY TypeScriptType TRAIT TESTS (kept for backward compatibility)
    // ========================================================================

    #[test]
    fn test_primitive_types() {
        assert_eq!(i32::typescript_type(), "number");
        assert_eq!(String::typescript_type(), "string");
        assert_eq!(bool::typescript_type(), "boolean");
        assert_eq!(<()>::typescript_type(), "void");
    }

    #[test]
    fn test_option_type() {
        assert_eq!(<Option<String>>::typescript_type(), "string | null");
        assert_eq!(<Option<i32>>::typescript_type(), "number | null");
    }

    #[test]
    fn test_vec_type() {
        assert_eq!(<Vec<String>>::typescript_type(), "string[]");
        assert_eq!(<Vec<i32>>::typescript_type(), "number[]");
    }

    #[test]
    fn test_hashmap_type() {
        assert_eq!(
            <HashMap<String, i32>>::typescript_type(),
            "Record<string, number>"
        );
    }

    #[test]
    fn test_result_type() {
        assert_eq!(
            <Result<String, String>>::typescript_type(),
            "{ ok: true; value: string } | { ok: false; error: string }"
        );
    }

    #[test]
    fn test_tuple_types() {
        assert_eq!(<(String,)>::typescript_type(), "[string]");
        assert_eq!(<(String, i32)>::typescript_type(), "[string, number]");
        assert_eq!(<(String, i32, bool)>::typescript_type(), "[string, number, boolean]");
    }

    // RPC Service tests

    struct TestUserService;

    impl RpcService for TestUserService {
        fn service_name() -> &'static str {
            "UserService"
        }

        fn methods() -> Vec<RpcMethodInfo> {
            vec![
                RpcMethodInfo::new(
                    "getUser",
                    "{ id: string }".to_string(),
                    "User".to_string(),
                ),
                RpcMethodInfo::new(
                    "listUsers",
                    "{ page: number }".to_string(),
                    "User[]".to_string(),
                ),
            ]
        }
    }

    #[test]
    fn test_rpc_method_info() {
        let method = RpcMethodInfo::new(
            "testMethod",
            "string".to_string(),
            "number".to_string(),
        );
        assert_eq!(method.name, "testMethod");
        assert_eq!(method.request_type, "string");
        assert_eq!(method.response_type, "number");
    }

    #[test]
    fn test_rpc_service_name() {
        assert_eq!(TestUserService::service_name(), "UserService");
    }

    #[test]
    fn test_rpc_service_methods() {
        let methods = TestUserService::methods();
        assert_eq!(methods.len(), 2);
        assert_eq!(methods[0].name, "getUser");
        assert_eq!(methods[1].name, "listUsers");
    }

    #[test]
    fn test_rpc_service_typescript_interface() {
        let interface = TestUserService::typescript_interface();
        assert!(interface.contains("interface UserService"));
        assert!(interface.contains("getUser(request: { id: string }): Promise<User>"));
        assert!(interface.contains("listUsers(request: { page: number }): Promise<User[]>"));
    }

    #[test]
    fn test_rpc_service_typescript_client() {
        let client = TestUserService::typescript_client();
        // Check class declaration
        assert!(client.contains("class UserServiceClient"));
        // Check constructor
        assert!(client.contains("constructor(baseUrl: string, fetchFn: typeof fetch = fetch)"));
        assert!(client.contains("this.baseUrl = baseUrl"));
        assert!(client.contains("this.fetch = fetchFn"));
        // Check method signatures
        assert!(client.contains("async getUser(request: { id: string }): Promise<User>"));
        assert!(client.contains("async listUsers(request: { page: number }): Promise<User[]>"));
        // Check fetch calls with correct endpoints
        assert!(client.contains("/UserService/getUser"));
        assert!(client.contains("/UserService/listUsers"));
        // Check error handling
        assert!(client.contains("if (!response.ok)"));
        assert!(client.contains("throw new Error"));
    }

    #[test]
    fn test_rpc_service_request_builder() {
        let builder = TestUserService::typescript_request_builder();
        // Check generic RequestBuilder class
        assert!(builder.contains("class RequestBuilder<TRequest, TResponse>"));
        assert!(builder.contains("private data: Partial<TRequest>"));
        // Check set method with type inference
        assert!(builder.contains("set<K extends keyof TRequest>(key: K, value: TRequest[K]): this"));
        // Check setAll method
        assert!(builder.contains("setAll(fields: Partial<TRequest>): this"));
        // Check build method
        assert!(builder.contains("build(): TRequest"));
        // Check execute method
        assert!(builder.contains("async execute(): Promise<TResponse>"));
        // Check factory class
        assert!(builder.contains("class UserServiceRequestBuilder"));
        assert!(builder.contains("constructor(private readonly client: UserServiceClient)"));
        // Check factory methods
        assert!(builder.contains("getUser(): RequestBuilder<{ id: string }, User>"));
        assert!(builder.contains("listUsers(): RequestBuilder<{ page: number }, User[]>"));
    }

    // Error type tests

    struct ApiError;

    impl TypeScriptType for ApiError {
        fn typescript_type() -> String {
            "{ code: string; message: string }".to_string()
        }

        fn typescript_name() -> &'static str {
            "ApiError"
        }
    }

    impl RpcErrorType for ApiError {}

    struct TestRpcError;

    impl TypeScriptType for TestRpcError {
        fn typescript_type() -> String {
            r#"{ type: "NotFound"; resource: string } | { type: "Unauthorized" } | { type: "BadRequest"; field: string; message: string }"#.to_string()
        }

        fn typescript_name() -> &'static str {
            "RpcError"
        }
    }

    impl RpcErrorType for TestRpcError {}

    impl EnumerableError for TestRpcError {
        fn error_variants() -> Vec<ErrorVariantInfo> {
            vec![
                ErrorVariantInfo::new(
                    "NotFound",
                    r#"{ type: "NotFound"; resource: string }"#.to_string(),
                ),
                ErrorVariantInfo::new(
                    "Unauthorized",
                    r#"{ type: "Unauthorized" }"#.to_string(),
                ),
                ErrorVariantInfo::new(
                    "BadRequest",
                    r#"{ type: "BadRequest"; field: string; message: string }"#.to_string(),
                ),
            ]
        }
    }

    #[test]
    fn test_error_variant_info() {
        let variant = ErrorVariantInfo::new("NotFound", "{ type: \"NotFound\" }".to_string());
        assert_eq!(variant.name, "NotFound");
    }

    #[test]
    fn test_error_type_guard_generation() {
        let variant = ErrorVariantInfo::new("NotFound", "{ type: \"NotFound\" }".to_string());
        let guard = variant.typescript_type_guard("RpcError");
        assert!(guard.contains("function isNotFoundError"));
        assert!(guard.contains("error: RpcError"));
        assert!(guard.contains("error is { type: \"NotFound\" }"));
        assert!(guard.contains("return (error as any).type === \"NotFound\""));
    }

    #[test]
    fn test_enumerable_error_all_guards() {
        let guards = TestRpcError::generate_all_type_guards();
        assert!(guards.contains("isNotFoundError"));
        assert!(guards.contains("isUnauthorizedError"));
        assert!(guards.contains("isBadRequestError"));
    }

    #[test]
    fn test_rpc_error_type_default_guards() {
        // Default implementation returns empty string
        assert_eq!(ApiError::typescript_type_guards(), "");
    }
}
