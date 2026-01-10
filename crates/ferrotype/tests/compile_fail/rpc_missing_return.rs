//! Test that rpc_method fails when return type is missing

use ferrotype::rpc_method;

struct Service;

impl Service {
    #[rpc_method]
    fn method(&self, request: String) {
        let _ = request;
    }
}

fn main() {}
