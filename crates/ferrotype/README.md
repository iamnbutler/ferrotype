# ferro-type

Core library for Rust-to-TypeScript type generation. Provides the `TypeScript` trait, `TypeDef` intermediate representation, and `TypeRegistry` for collecting and rendering types in dependency order.

## Installation

```toml
[dependencies]
ferro-type = "0.1.2"
```

## Usage

```rust
use ferrotype::{TypeScript, TypeRegistry};

#[derive(TypeScript)]
struct User {
    id: String,
    name: String,
    age: i32,
}

#[derive(TypeScript)]
struct Post {
    title: String,
    author: User,
}

// Collect types and render in dependency order
let mut registry = TypeRegistry::new();
registry.register::<Post>();

let output = registry.render();
// type User = { id: string; name: string; age: number };
// type Post = { title: string; author: User };
```

## Features

- `TypeScript` trait for defining TypeScript representations
- `TypeDef` IR for structured type definitions (not string concatenation)
- `TypeRegistry` for deduplication and dependency ordering
- Serde-compatible attributes (`rename`, `rename_all`, `skip`, `flatten`, etc.)
- Discriminated union generation for enums
- Support for generics

## Documentation

See the [main repository README](https://github.com/iamnbutler/ferrotype) for full documentation including:
- All supported attributes
- Type mappings (Rust to TypeScript)
- Enum representations
- Advanced usage patterns

## License

MIT
