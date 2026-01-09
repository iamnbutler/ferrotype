//! Hello World RPC Server
//!
//! The simplest possible ferrotype RPC example:
//! - No arguments
//! - Returns a string
//!
//! This demonstrates the type mapping between Rust and TypeScript.

use ferrotype::TypeScriptType;
use serde::{Deserialize, Serialize};

/// Response from the hello RPC method.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HelloResponse {
    pub message: String,
}

impl TypeScriptType for HelloResponse {
    fn typescript_type() -> String {
        "{ message: string }".to_string()
    }

    fn typescript_name() -> &'static str {
        "HelloResponse"
    }
}

/// The hello RPC method - no args, returns a greeting.
pub fn hello() -> HelloResponse {
    HelloResponse {
        message: "Hello, World!".to_string(),
    }
}

fn main() {
    // Simulate an RPC server responding to a "hello" request.
    // In a real server, this would be over HTTP/WebSocket/etc.
    let response = hello();
    let json = serde_json::to_string(&response).expect("serialize response");

    // Output the JSON response (simulating what a real RPC server would send)
    println!("{}", json);

    // Also print the TypeScript type for reference
    eprintln!("TypeScript type: {}", HelloResponse::typescript_type());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_returns_greeting() {
        let response = hello();
        assert_eq!(response.message, "Hello, World!");
    }

    #[test]
    fn test_hello_response_serialization() {
        let response = hello();
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, r#"{"message":"Hello, World!"}"#);
    }

    #[test]
    fn test_hello_response_roundtrip() {
        let original = hello();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: HelloResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_typescript_type() {
        assert_eq!(HelloResponse::typescript_type(), "{ message: string }");
        assert_eq!(HelloResponse::typescript_name(), "HelloResponse");
    }
}
