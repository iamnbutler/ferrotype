# Advanced TypeScript Patterns Research

Research findings for implementing advanced TypeScript patterns in ferrotype.
These patterns enable generating more sophisticated TypeScript from Rust types.

## Overview

This document covers three advanced TypeScript patterns that ferrotype should support:

1. **Template Literal Types** - Pattern-based string types like `vm-${string}`
2. **Nested Type References** - Namespaced types like `VM.Git.State`
3. **Rich Discriminated Unions** - Enhanced union patterns with generic wrappers

## 1. Template Literal Types

Template literal types use JavaScript template literal syntax in type positions.
Introduced in TypeScript 4.1, they enable compile-time string pattern validation.

### Basic Syntax

```typescript
// Simple template literal type
type VmId = `vm-${string}`;

// Valid assignment
const myVm: VmId = "vm-abc123";

// Error: Type '"instance-xyz"' is not assignable to type '`vm-${string}`'
const badVm: VmId = "instance-xyz";
```

### Union Expansion

Template literals expand unions into all possible combinations:

```typescript
type Prefix = "get" | "set";
type Field = "Name" | "Age";
type Methods = `${Prefix}${Field}`;
// Result: "getName" | "setName" | "getAge" | "setAge"
```

### Common ID Patterns

```typescript
// Branded ID types with prefixes
type UserId = `user_${string}`;
type ProductId = `product_${string}`;
type OrderId = `order_${number}`;

// Version patterns
type SemVer = `v${number}.${number}.${number}`;

// Path patterns
type ApiRoute = `/api/${string}`;
type AssetPath = `/assets/${string}.${string}`;
```

### Ferrotype Implementation

**Required TypeDef variant:**

```rust
/// A template literal type: `prefix${Type}suffix`
TemplateLiteral {
    /// String parts interspersed with type placeholders
    /// e.g., "vm-" + ${string} + "" becomes ["vm-", ""]
    strings: Vec<String>,
    /// Types to interpolate (one fewer than strings)
    types: Vec<Box<TypeDef>>,
}
```

**Rendering:**

```rust
fn render_template_literal(strings: &[String], types: &[TypeDef]) -> String {
    let mut result = String::from("`");
    for (i, s) in strings.iter().enumerate() {
        result.push_str(s);
        if i < types.len() {
            result.push_str("${");
            result.push_str(&types[i].render());
            result.push('}');
        }
    }
    result.push('`');
    result
}
// Output: `vm-${string}` or `v${number}.${number}.${number}`
```

**Rust derive usage:**

```rust
#[derive(TypeScript)]
#[ts(pattern = "vm-${string}")]  // New attribute
struct VmId(String);

// Or using a type alias marker:
#[derive(TypeScript)]
#[ts(template_literal)]
struct VmId {
    #[ts(prefix = "vm-")]
    value: String,
}
```

---

## 2. Nested Type References (Namespaced Types)

TypeScript supports accessing types through dot notation, either via namespaces
or through indexed access types.

### Namespace Pattern

```typescript
namespace VM {
    export namespace Git {
        export type State = "clean" | "dirty" | "unknown";
        export interface Config {
            remote: string;
            branch: string;
        }
    }
    export type Id = `vm-${string}`;
}

// Usage
const state: VM.Git.State = "clean";
const config: VM.Git.Config = { remote: "origin", branch: "main" };
```

### Module Pattern (Preferred in Modern TypeScript)

```typescript
// vm.ts
export namespace Git {
    export type State = "clean" | "dirty" | "unknown";
}

// Re-exported as namespace for consumers
export type { Git };
```

### Indexed Access Pattern

```typescript
interface VM {
    Git: {
        State: "clean" | "dirty" | "unknown";
        Config: { remote: string; branch: string };
    };
}

// Access nested type
type GitState = VM["Git"]["State"];
```

### Ferrotype Implementation

**Option A: Extend Ref to support dotted paths:**

```rust
/// A reference to a named type, optionally namespaced
Ref {
    /// Path segments, e.g., ["VM", "Git", "State"]
    path: Vec<String>,
}

// Alternative: keep Ref simple, add NamespacedRef
NamespacedRef(Vec<String>),
```

**Rendering:**

```rust
fn render_namespaced_ref(path: &[String]) -> String {
    path.join(".")
}
// Output: VM.Git.State
```

**Named type with namespace:**

```rust
/// A named type that may be nested in a namespace
Named {
    /// Full path: ["VM", "Git", "State"]
    path: Vec<String>,
    def: Box<TypeDef>,
}
```

**Rust derive usage:**

```rust
#[derive(TypeScript)]
#[ts(namespace = "VM::Git")]  // or "VM.Git"
enum State {
    Clean,
    Dirty,
    Unknown,
}
// Renders: namespace VM { namespace Git { type State = ... } }
```

**Namespace declaration rendering:**

```rust
// For namespace VM.Git { type State = ... }
fn render_namespace_declaration(path: &[String], inner: &str) -> String {
    if path.is_empty() {
        return inner.to_string();
    }

    let mut result = String::new();
    for (i, ns) in path.iter().enumerate() {
        result.push_str("namespace ");
        result.push_str(ns);
        result.push_str(" {\n");
    }
    result.push_str(inner);
    for _ in path.iter() {
        result.push_str("\n}");
    }
    result
}
```

---

## 3. Rich Discriminated Unions with Generic Wrappers

Standard discriminated unions use a literal discriminant:

```typescript
type Message =
    | { type: "text"; content: string }
    | { type: "image"; url: string; alt: string }
    | { type: "video"; url: string; duration: number };
```

### Generic Core<T> Wrapper Pattern

A common advanced pattern wraps discriminated union variants in a generic:

```typescript
// Core wrapper adds common metadata to any type
interface Core<T extends { type: string }> {
    id: string;
    timestamp: Date;
    version: number;
    data: T;
}

// Variant types
interface TextData { type: "text"; content: string }
interface ImageData { type: "image"; url: string }

// Full types include Core wrapper
type TextMessage = Core<TextData>;
type ImageMessage = Core<ImageData>;

// Union of wrapped types
type Message = TextMessage | ImageMessage;
```

### Benefits

1. **Shared metadata**: All variants share `id`, `timestamp`, `version`
2. **Type narrowing**: Can narrow on `data.type` discriminant
3. **Separation of concerns**: Core vs variant-specific data
4. **Extensibility**: Add new fields to Core without changing variants

### Variant: Discriminated Union with Payload<T>

```typescript
// Generic payload wrapper
type Payload<K extends string, T> = {
    kind: K;
    payload: T;
};

// Define variants
type CreateUser = Payload<"create", { name: string; email: string }>;
type DeleteUser = Payload<"delete", { userId: string }>;
type UpdateUser = Payload<"update", { userId: string; changes: Partial<User> }>;

// Union
type UserAction = CreateUser | DeleteUser | UpdateUser;

// Type guard narrows based on kind
function handleAction(action: UserAction) {
    switch (action.kind) {
        case "create":
            // action.payload is { name: string; email: string }
            break;
        case "delete":
            // action.payload is { userId: string }
            break;
    }
}
```

### Ferrotype Implementation

This pattern requires:

1. **Generic type definitions** with type parameters
2. **Generic instantiation** in output

**Required TypeDef changes:**

```rust
/// A generic type definition with type parameters
GenericDef {
    name: String,
    /// Type parameters, e.g., ["T", "K extends string"]
    type_params: Vec<TypeParam>,
    def: Box<TypeDef>,
}

/// A type parameter
struct TypeParam {
    name: String,
    /// Optional constraint, e.g., "string" for "T extends string"
    constraint: Option<Box<TypeDef>>,
    /// Optional default, e.g., "never" for "T = never"
    default: Option<Box<TypeDef>>,
}
```

**Rust derive usage:**

```rust
// Define a generic wrapper
#[derive(TypeScript)]
#[ts(generic = "T: { type: string }")]
struct Core<T> {
    id: String,
    timestamp: u64,
    data: T,
}

// Use the wrapper
#[derive(TypeScript)]
struct TextMessage {
    #[ts(type = "text")]
    r#type: (),  // Literal type marker
    content: String,
}

// Core<TextMessage> becomes:
// { id: string; timestamp: number; data: { type: "text"; content: string } }
```

---

## Implementation Priority

Based on complexity and value:

### Phase 1: Template Literal Types
- **Complexity**: Low
- **Value**: High (branded IDs are common)
- **TypeDef change**: Add `TemplateLiteral` variant
- **Derive change**: Add `#[ts(pattern = "...")]` attribute

### Phase 2: Nested Type References
- **Complexity**: Medium
- **Value**: Medium (namespace organization)
- **TypeDef change**: Extend `Ref` or add `NamespacedRef`
- **Registry change**: Handle namespace declarations in output

### Phase 3: Generic Type Definitions
- **Complexity**: High
- **Value**: High (reusable patterns)
- **TypeDef change**: Add `GenericDef`, `TypeParam`
- **Derive change**: Parse Rust generics, add constraint attributes

---

## References

- [TypeScript Template Literal Types](https://www.typescriptlang.org/docs/handbook/2/template-literal-types.html)
- [TypeScript Namespaces](https://www.typescriptlang.org/docs/handbook/namespaces.html)
- [TypeScript Discriminated Unions](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions)
- [Branded Types in TypeScript](https://www.learningtypescript.com/articles/branded-types)

---

## Note on "ace.types"

The research task mentioned "ace.types" as a reference file. This file was not
found in public repositories or documentation. The patterns documented above
are based on standard TypeScript patterns that align with the task descriptions:

- `Core<T>` pattern → Generic wrapper for discriminated unions
- `vm-${string}` → Template literal types
- `VM.Git.State` → Namespaced type references

If "ace.types" refers to a specific internal file, the patterns above should
still apply as they represent common TypeScript patterns for these use cases.
