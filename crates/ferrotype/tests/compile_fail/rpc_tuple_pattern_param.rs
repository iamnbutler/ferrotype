//! Test that rpc_method fails when first parameter is a tuple pattern

use ferrotype::rpc_method;

struct Service;

impl Service {
    #[rpc_method]
    fn method((a, b): (i32, i32), request: String) -> String {
        format!("{} {} {}", a, b, request)
    }
}

fn main() {}
