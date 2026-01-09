//! Tests for the #[rpc_method] attribute macro

use ferrotype::{rpc_method, TypeScriptType};

// Request/Response types for testing
struct GetUserRequest {
    id: String,
}

impl TypeScriptType for GetUserRequest {
    fn typescript_type() -> String {
        "{ id: string }".to_string()
    }

    fn typescript_name() -> &'static str {
        "GetUserRequest"
    }
}

struct GetUserResponse {
    name: String,
}

impl TypeScriptType for GetUserResponse {
    fn typescript_type() -> String {
        "{ name: string }".to_string()
    }

    fn typescript_name() -> &'static str {
        "GetUserResponse"
    }
}

struct ListUsersRequest {
    page: i32,
}

impl TypeScriptType for ListUsersRequest {
    fn typescript_type() -> String {
        "{ page: number }".to_string()
    }

    fn typescript_name() -> &'static str {
        "ListUsersRequest"
    }
}

struct ListUsersResponse {
    users: Vec<String>,
}

impl TypeScriptType for ListUsersResponse {
    fn typescript_type() -> String {
        "{ users: string[] }".to_string()
    }

    fn typescript_name() -> &'static str {
        "ListUsersResponse"
    }
}

// Test service with RPC methods
struct TestService;

impl TestService {
    #[rpc_method]
    fn get_user(&self, _request: GetUserRequest) -> GetUserResponse {
        GetUserResponse {
            name: "test".to_string(),
        }
    }

    #[rpc_method(name = "fetchUsers")]
    fn list_users(&self, _request: ListUsersRequest) -> ListUsersResponse {
        ListUsersResponse { users: vec![] }
    }
}

#[test]
fn test_rpc_method_basic() {
    let service = TestService;
    let request = GetUserRequest {
        id: "123".to_string(),
    };
    let response = service.get_user(request);
    assert_eq!(response.name, "test");
}

#[test]
fn test_rpc_method_with_custom_name() {
    let service = TestService;
    let request = ListUsersRequest { page: 1 };
    let response = service.list_users(request);
    assert!(response.users.is_empty());
}

// Test with &self reference
struct RefService;

impl RefService {
    #[rpc_method]
    fn method_with_ref(&self, _request: GetUserRequest) -> GetUserResponse {
        GetUserResponse {
            name: "ref".to_string(),
        }
    }
}

#[test]
fn test_rpc_method_with_self_ref() {
    let service = RefService;
    let request = GetUserRequest {
        id: "456".to_string(),
    };
    let response = service.method_with_ref(request);
    assert_eq!(response.name, "ref");
}

// Test with &mut self reference
struct MutService {
    counter: i32,
}

impl MutService {
    #[rpc_method]
    fn increment(&mut self, _request: GetUserRequest) -> GetUserResponse {
        self.counter += 1;
        GetUserResponse {
            name: format!("count: {}", self.counter),
        }
    }
}

#[test]
fn test_rpc_method_with_mut_self() {
    let mut service = MutService { counter: 0 };
    let request = GetUserRequest {
        id: "789".to_string(),
    };
    let response = service.increment(request);
    assert_eq!(response.name, "count: 1");
    assert_eq!(service.counter, 1);
}
