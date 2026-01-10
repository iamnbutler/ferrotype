//! Test that rpc_method fails when missing request parameter

use ferrotype::rpc_method;

struct Service;

impl Service {
    #[rpc_method]
    fn method(&self) -> String {
        "no request param".to_string()
    }
}

fn main() {}
