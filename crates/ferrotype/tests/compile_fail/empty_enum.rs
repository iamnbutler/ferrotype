//! Test that TypeScript derive fails on empty enums

use ferrotype::TypeScript;

#[derive(TypeScript)]
enum Empty {}

fn main() {}
