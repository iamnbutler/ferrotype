# ferro-type-derive

Derive macros for ferrotype TypeScript type generation. Provides `#[derive(TypeScript)]` for structs and enums.

## Installation

This crate is typically used through the main `ferro-type` crate, which re-exports the derive macro:

```toml
[dependencies]
ferro-type = "0.1.2"
```

For direct usage:

```toml
[dependencies]
ferro-type-derive = "0.1.2"
```

## Usage

```rust
use ferrotype::TypeScript;

// Structs become TypeScript object types
#[derive(TypeScript)]
struct User {
    id: String,
    name: String,
}
// Renders as: { id: string; name: string }

// Unit enums become string literal unions
#[derive(TypeScript)]
enum Status {
    Pending,
    Active,
    Completed,
}
// Renders as: "Pending" | "Active" | "Completed"

// Data enums become discriminated unions
#[derive(TypeScript)]
enum Message {
    Ping,
    Text(String),
    Error { code: i32, message: String },
}
// Renders as: { type: "Ping" } | { type: "Text"; value: string } | { type: "Error"; code: number; message: string }
```

## Attributes

### Container Attributes

| Attribute | Description |
|-----------|-------------|
| `#[ts(rename = "Name")]` | Rename the type |
| `#[ts(rename_all = "camelCase")]` | Rename all fields/variants |
| `#[ts(tag = "kind")]` | Custom discriminant field name |
| `#[ts(content = "data")]` | Adjacent tagging with content field |
| `#[ts(untagged)]` | Plain union without discriminant |
| `#[ts(transparent)]` | Newtype becomes inner type directly |

### Field Attributes

| Attribute | Description |
|-----------|-------------|
| `#[ts(rename = "name")]` | Rename this field |
| `#[ts(skip)]` | Omit field from output |
| `#[ts(flatten)]` | Inline nested object fields |
| `#[ts(type = "Date")]` | Override TypeScript type |
| `#[ts(default)]` | Mark field as optional (`?`) |
| `#[ts(inline)]` | Inline type definition instead of reference |

## Documentation

See the [main repository README](https://github.com/iamnbutler/ferrotype) for full documentation including type mappings and advanced patterns.

## License

MIT
