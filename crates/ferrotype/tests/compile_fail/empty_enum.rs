//! Test that TypeScript derive fails on empty enums

use ferro_type::TypeScript;

#[derive(TypeScript)]
enum Empty {}

fn main() {}
