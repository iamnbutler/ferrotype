# ferrotype

Rust-to-TypeScript type generation via derive macro. Generates TypeScript type definitions from Rust structs and enums using an intermediate representation for deduplication and dependency ordering.

## Installation

```toml
[dependencies]
ferro-type = "0.1.3"
```

## Features

- `#[derive(TypeScript)]` for structs and enums
- Structured `TypeDef` IR instead of string concatenation
- `TypeRegistry` for collecting types and rendering in dependency order
- Serde-compatible attributes (`rename`, `rename_all`, `skip`, `flatten`, etc.)
- Discriminated union generation for enums
- Support for generics

## Usage

### Basic Derive

```rust
use ferrotype::TypeScript;

#[derive(TypeScript)]
struct User {
    id: String,
    name: String,
    age: i32,
}

// Renders as: { id: string; name: string; age: number }
```

### Enums

```rust
use ferrotype::TypeScript;

// Unit variants become string literal unions
#[derive(TypeScript)]
enum Status {
    Pending,
    Active,
    Completed,
}
// Renders as: "Pending" | "Active" | "Completed"

// Data variants become discriminated unions
#[derive(TypeScript)]
enum Message {
    Ping,
    Text(String),
    Error { code: i32, message: String },
}
// Renders as: { type: "Ping" } | { type: "Text"; value: string } | { type: "Error"; code: number; message: string }
```

### TypeRegistry

```rust
use ferrotype::{TypeScript, TypeRegistry};

#[derive(TypeScript)]
struct User {
    id: String,
    name: String,
}

#[derive(TypeScript)]
struct Post {
    title: String,
    author: User,
}

let mut registry = TypeRegistry::new();
registry.register::<Post>();

// Renders types in dependency order
let output = registry.render();
// type User = { id: string; name: string };
// type Post = { title: string; author: User };
```

## Attributes

### Container Attributes

| Attribute | Description |
|-----------|-------------|
| `#[ts(rename = "Name")]` | Rename the type |
| `#[ts(rename_all = "camelCase")]` | Rename all fields/variants |
| `#[ts(transparent)]` | Newtype becomes inner type directly |
| `#[ts(tag = "kind")]` | Custom discriminant field name (default: `type`) |
| `#[ts(content = "data")]` | Adjacent tagging with content field |
| `#[ts(untagged)]` | Plain union without discriminant |

### Field Attributes

| Attribute | Description |
|-----------|-------------|
| `#[ts(rename = "name")]` | Rename this field |
| `#[ts(skip)]` | Omit field from output |
| `#[ts(flatten)]` | Inline nested object fields |
| `#[ts(type = "Date")]` | Override TypeScript type |
| `#[ts(default)]` | Mark field as optional (`?`) |
| `#[ts(inline)]` | Inline type definition instead of reference |
| `#[ts(pattern = "${A}::${B}")]` | Template literal type |
| `#[ts(index = "T", key = "k")]` | Indexed access type (`T["k"]`) |

### Advanced Features

#### Intersection Types

Extend existing TypeScript types:

```rust
#[derive(TypeScript)]
#[ts(extends = "BaseEntity")]
struct User {
    name: String,
    email: String,
}
// Renders as: type User = BaseEntity & { name: string; email: string }
```

#### Template Literals

Generate branded ID types:

```rust
#[derive(TypeScript)]
struct Order {
    #[ts(pattern = "order-${string}")]
    id: String,
}
// Renders as: id: `order-${string}`
```

#### Indexed Access

Reference nested type properties:

```rust
#[derive(TypeScript)]
struct Comment {
    #[ts(index = "User", key = "id")]
    author_id: String,
}
// Renders as: author_id: User["id"]
```

### Rename Conventions

Supported values for `rename_all`: `camelCase`, `PascalCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE`

## Type Mappings

| Rust | TypeScript |
|------|------------|
| `String`, `&str`, `char` | `string` |
| `i8`..`i64`, `u8`..`u64`, `f32`, `f64` | `number` |
| `i128`, `u128` | `bigint` |
| `bool` | `boolean` |
| `()` | `void` |
| `Option<T>` | `T \| null` |
| `Vec<T>` | `T[]` |
| `HashMap<K, V>` | `Record<K, V>` |
| `Result<T, E>` | `{ ok: true; value: T } \| { ok: false; error: E }` |
| `(A, B, ...)` | `[A, B, ...]` |

## License

MIT
