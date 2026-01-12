//! Test that TS derive fails on empty enums

use ferro_type::TS;

#[derive(TS)]
enum Empty {}

fn main() {}
