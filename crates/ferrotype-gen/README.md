# ferro-type-gen

TypeScript file generation utilities for ferro-type. Provides `Generator` and `Config` for writing TypeScript definition files from Rust types.

## Installation

```toml
[dependencies]
ferro-type-gen = "0.1.2"
ferro-type = "0.1.2"
```

## Usage

### Basic Generation

```rust
use ferrotype::TypeScript;
use ferro_type_gen::{Config, Generator, ExportStyle};

#[derive(TypeScript)]
struct User {
    id: String,
    name: String,
}

let mut generator = Generator::new(
    Config::new()
        .output("types.ts")
        .export_style(ExportStyle::Named)
);

generator.register::<User>();
generator.write().expect("Failed to write TypeScript");
```

### build.rs Integration

```rust
// build.rs
use ferro_type_gen::{Config, Generator};

fn main() {
    let mut generator = Generator::new(
        Config::new().output("../frontend/src/types/api.ts")
    );

    generator
        .register::<api::User>()
        .register::<api::Post>();

    // Only writes if content changed (avoids unnecessary rebuilds)
    generator.write_if_changed()
        .expect("TypeScript generation failed");
}
```

## Configuration

| Option | Description |
|--------|-------------|
| `output(path)` | Output file path |
| `export_style(style)` | `None`, `Named` (default), or `Grouped` |
| `header(text)` | Custom header comment |
| `declaration_only()` | Generate `.d.ts` instead of `.ts` |
| `esm_extensions()` | Add `.js` extensions to imports |

## Export Styles

- **`ExportStyle::None`**: `type Foo = ...`
- **`ExportStyle::Named`**: `export type Foo = ...` (default)
- **`ExportStyle::Grouped`**: Types without export, `export { Foo, Bar }` at end

## Multi-File Generation

```rust
// Write types to separate files based on module paths
generator.write_multi_file("./types/")?;

// my_crate::models::user -> types/models/user.ts
// my_crate::api -> types/api.ts
```

## Documentation

See the [main repository README](https://github.com/iamnbutler/ferrotype) for full documentation.

## License

MIT
