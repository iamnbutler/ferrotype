//! Hello World RPC Server
//!
//! A real HTTP server demonstrating ferrotype type mapping:
//! - Axum server on an actual port
//! - GET /rpc/hello endpoint
//! - Returns JSON response
//!
//! Run with: cargo run -p hello-world
//! Test with: curl http://localhost:3000/rpc/hello

use axum::{routing::get, Json, Router};
use ferro_type::TS;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

/// Response from the hello RPC method.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub struct HelloResponse {
    pub message: String,
}

/// The hello RPC method - no args, returns a greeting.
pub fn hello() -> HelloResponse {
    HelloResponse {
        message: "Hello, World!".to_string(),
    }
}

/// HTTP handler for GET /rpc/hello
async fn hello_handler() -> Json<HelloResponse> {
    Json(hello())
}

/// Create the router with all RPC endpoints.
pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/rpc/hello", get(hello_handler))
        .layer(cors)
}

#[tokio::main]
async fn main() {
    let app = create_router();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Hello World RPC server listening on http://{}", addr);
    println!("Try: curl http://{}/rpc/hello", addr);
    println!("TypeScript type: {}", HelloResponse::typescript().render());

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
        let td = HelloResponse::typescript();
        // Named types render as their name
        assert_eq!(td.render(), "HelloResponse");
        // The declaration shows the full type
        assert_eq!(
            td.render_declaration(),
            "type HelloResponse = { message: string };"
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Duration;

    /// Start the server on a random available port and return the base URL.
    async fn start_test_server() -> (String, tokio::task::JoinHandle<()>) {
        let app = create_router();

        // Bind to port 0 to get a random available port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        (base_url, handle)
    }

    #[tokio::test]
    async fn test_real_http_hello() {
        let (base_url, _handle) = start_test_server().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/rpc/hello", base_url))
            .send()
            .await
            .expect("request failed");

        assert!(response.status().is_success());

        let hello_response: HelloResponse = response.json().await.expect("parse json failed");
        assert_eq!(hello_response.message, "Hello, World!");
    }

    #[tokio::test]
    async fn test_real_http_json_content_type() {
        let (base_url, _handle) = start_test_server().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/rpc/hello", base_url))
            .send()
            .await
            .expect("request failed");

        let content_type = response
            .headers()
            .get("content-type")
            .expect("no content-type header")
            .to_str()
            .unwrap();

        assert!(content_type.contains("application/json"));
    }

    #[tokio::test]
    async fn test_real_http_cors_headers() {
        let (base_url, _handle) = start_test_server().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/rpc/hello", base_url))
            .header("Origin", "http://localhost:5173")
            .send()
            .await
            .expect("request failed");

        // CORS should allow any origin
        let cors_header = response
            .headers()
            .get("access-control-allow-origin")
            .expect("no CORS header");

        assert_eq!(cors_header.to_str().unwrap(), "*");
    }
}
