//! Test that indexed access with an invalid field fails to compile.

use ferro_type::TS;

struct Profile {
    login: String,
    email: String,
}

// This should fail to compile because Profile doesn't have a field named `nonexistent`
#[derive(TS)]
struct InvalidIndexedAccess {
    #[ts(index = Profile, key = nonexistent)]
    username: String,
}

fn main() {}
