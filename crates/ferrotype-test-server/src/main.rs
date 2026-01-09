//! Ferrotype E2E Test Server
//!
//! A simple HTTP server that implements RPC endpoints for testing
//! the generated TypeScript clients against real Rust handlers.

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use ferrotype_fixtures::{
    GetUserRequest, GetUserResponse, ListUsersRequest, ListUsersResponse, User,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let app = create_router();

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Ferrotype test server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Create the router with all RPC endpoints
pub fn create_router() -> Router {
    // CORS layer for browser testing
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // UserService endpoints
        .route("/UserService/getUser", post(get_user))
        .route("/UserService/listUsers", post(list_users))
        // Generic RPC endpoint for dynamic routing
        .route("/:service/:method", post(generic_rpc_handler))
        // Health check
        .route("/health", post(health_check))
        .layer(cors)
}

/// Handle getUser RPC request
async fn get_user(Json(request): Json<GetUserRequest>) -> Json<GetUserResponse> {
    // Return a mock user based on the request
    let user = if request.user_id > 0 && request.user_id < 1000 {
        Some(User {
            id: request.user_id,
            name: format!("User {}", request.user_id),
            email: format!("user{}@example.com", request.user_id),
            active: true,
        })
    } else {
        None
    };

    Json(GetUserResponse { user })
}

/// Handle listUsers RPC request
async fn list_users(Json(request): Json<ListUsersRequest>) -> Json<ListUsersResponse> {
    let per_page = request.per_page.min(100).max(1);
    let page = request.page.max(1);

    // Generate mock users for the requested page
    let start_id = ((page - 1) * per_page) as u64 + 1;
    let users: Vec<User> = (0..per_page)
        .map(|i| {
            let id = start_id + i as u64;
            User {
                id,
                name: format!("User {}", id),
                email: format!("user{}@example.com", id),
                active: id % 2 == 0,
            }
        })
        .filter(|u| {
            // Apply filter if provided
            request
                .filter
                .as_ref()
                .map(|f| {
                    if f == "active" {
                        u.active
                    } else if f == "inactive" {
                        !u.active
                    } else {
                        true
                    }
                })
                .unwrap_or(true)
        })
        .collect();

    Json(ListUsersResponse {
        users,
        total: 1000, // Mock total
        page,
        per_page,
    })
}

/// Generic RPC handler for testing dynamic routing
async fn generic_rpc_handler(
    Path((service, method)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Echo back the request with service/method info
    Ok(Json(json!({
        "service": service,
        "method": method,
        "request": payload,
        "response": {
            "status": "ok",
            "message": format!("Handled {}/{}", service, method)
        }
    })))
}

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "server": "ferrotype-test-server",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/health")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_user_found() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/UserService/getUser")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"user_id": 42}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: GetUserResponse = serde_json::from_slice(&body).unwrap();

        assert!(json.user.is_some());
        let user = json.user.unwrap();
        assert_eq!(user.id, 42);
        assert_eq!(user.name, "User 42");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/UserService/getUser")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"user_id": 9999}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: GetUserResponse = serde_json::from_slice(&body).unwrap();

        assert!(json.user.is_none());
    }

    #[tokio::test]
    async fn test_list_users() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/UserService/listUsers")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"page": 1, "per_page": 5, "filter": null}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ListUsersResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.users.len(), 5);
        assert_eq!(json.page, 1);
        assert_eq!(json.per_page, 5);
    }

    #[tokio::test]
    async fn test_generic_rpc() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/TestService/testMethod")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"foo": "bar"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["service"], "TestService");
        assert_eq!(json["method"], "testMethod");
        assert_eq!(json["request"]["foo"], "bar");
    }
}
