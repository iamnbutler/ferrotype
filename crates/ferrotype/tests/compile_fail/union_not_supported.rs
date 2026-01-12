//! Test that TS derive fails on union types

use ferro_type::TS;

#[derive(TS)]
union MyUnion {
    i: i32,
    f: f32,
}

fn main() {}
