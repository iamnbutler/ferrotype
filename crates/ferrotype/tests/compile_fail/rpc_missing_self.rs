//! Test that rpc_method fails when first parameter is not self

use ferrotype::rpc_method;

struct Service;

impl Service {
    #[rpc_method]
    fn method(other: i32, request: String) -> String {
        format!("{}: {}", other, request)
    }
}

fn main() {}
