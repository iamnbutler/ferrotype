//! Test that TypeScript derive fails on union types

use ferro_type::TypeScript;

#[derive(TypeScript)]
union MyUnion {
    i: i32,
    f: f32,
}

fn main() {}
