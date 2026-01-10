//! Test that rpc_method fails with unknown attributes

use ferrotype::rpc_method;

struct Service;

impl Service {
    #[rpc_method(unknown = "value")]
    fn method(&self, request: String) -> String {
        request
    }
}

fn main() {}
