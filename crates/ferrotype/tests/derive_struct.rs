//! Tests for #[derive(TypeScript)] on structs

use ferrotype::{TypeScript, TypeDef};

/// Helper to get the inner definition from a Named TypeDef
fn inner_def(td: TypeDef) -> TypeDef {
    match td {
        TypeDef::Named { def, .. } => *def,
        other => other,
    }
}

// ============================================================================
// NAMED STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct SimpleUser {
    id: String,
    name: String,
    age: i32,
}

#[test]
fn test_named_struct() {
    let td = SimpleUser::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("id: string"));
    assert!(rendered.contains("name: string"));
    assert!(rendered.contains("age: number"));
}

#[test]
fn test_named_struct_name() {
    let td = SimpleUser::typescript();
    // Named types render as just their name
    assert_eq!(td.render(), "SimpleUser");
}

#[derive(TypeScript)]
struct EmptyStruct {}

#[test]
fn test_empty_struct() {
    let td = EmptyStruct::typescript();
    assert_eq!(inner_def(td).render(), "{}");
}

#[derive(TypeScript)]
struct NestedStruct {
    user: SimpleUser,
    active: bool,
}

#[test]
fn test_nested_struct() {
    let td = NestedStruct::typescript();
    let rendered = inner_def(td).render();
    // The user field should reference SimpleUser (which renders as its name)
    assert!(rendered.contains("user: SimpleUser"));
    assert!(rendered.contains("active: boolean"));
}

// ============================================================================
// TUPLE STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct NewtypeString(String);

#[test]
fn test_newtype_struct() {
    // Newtypes should unwrap to their inner type
    let td = NewtypeString::typescript();
    assert_eq!(inner_def(td).render(), "string");
}

#[derive(TypeScript)]
struct TupleTwo(String, i32);

#[test]
fn test_tuple_struct() {
    let td = TupleTwo::typescript();
    assert_eq!(inner_def(td).render(), "[string, number]");
}

#[derive(TypeScript)]
struct TupleThree(String, i32, bool);

#[test]
fn test_tuple_three() {
    let td = TupleThree::typescript();
    assert_eq!(inner_def(td).render(), "[string, number, boolean]");
}

// ============================================================================
// UNIT STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct UnitStruct;

#[test]
fn test_unit_struct() {
    let td = UnitStruct::typescript();
    assert_eq!(inner_def(td).render(), "null");
}

// ============================================================================
// GENERIC STRUCT TESTS
// ============================================================================

#[derive(TypeScript)]
struct Container<T> {
    value: T,
}

#[test]
fn test_generic_struct() {
    let td_string = Container::<String>::typescript();
    assert!(inner_def(td_string).render().contains("value: string"));

    let td_i32 = Container::<i32>::typescript();
    assert!(inner_def(td_i32).render().contains("value: number"));
}

#[derive(TypeScript)]
struct Pair<A, B> {
    first: A,
    second: B,
}

#[test]
fn test_multi_generic_struct() {
    let td = Pair::<String, i32>::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("first: string"));
    assert!(rendered.contains("second: number"));
}

// ============================================================================
// COMPLEX TYPE FIELD TESTS
// ============================================================================

#[derive(TypeScript)]
struct ComplexFields {
    items: Vec<String>,
    maybe_count: Option<i32>,
    result: Result<String, String>,
}

#[test]
fn test_complex_fields() {
    let td = ComplexFields::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("items: string[]"));
    assert!(rendered.contains("maybe_count: number | null"));
    assert!(rendered.contains("result:"));
}

// ============================================================================
// RPC REQUEST/RESPONSE PATTERN TESTS
// ============================================================================

#[derive(TypeScript)]
struct GetUserRequest {
    user_id: String,
}

#[derive(TypeScript)]
struct GetUserResponse {
    id: String,
    username: String,
    email: Option<String>,
}

#[test]
fn test_rpc_request_type() {
    let td = GetUserRequest::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("user_id: string"));
}

#[test]
fn test_rpc_response_type() {
    let td = GetUserResponse::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("id: string"));
    assert!(rendered.contains("username: string"));
    assert!(rendered.contains("email: string | null"));
}

#[derive(TypeScript)]
struct ListUsersRequest {
    page: i32,
    page_size: i32,
}

#[derive(TypeScript)]
struct ListUsersResponse {
    users: Vec<GetUserResponse>,
    total_count: i32,
    has_more: bool,
}

#[test]
fn test_list_request() {
    let td = ListUsersRequest::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("page: number"));
    assert!(rendered.contains("page_size: number"));
}

#[test]
fn test_list_response() {
    let td = ListUsersResponse::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("users:"));
    assert!(rendered.contains("total_count: number"));
    assert!(rendered.contains("has_more: boolean"));
}

// ============================================================================
// RENAME ATTRIBUTE TESTS
// ============================================================================

#[derive(TypeScript)]
#[ts(rename = "RenamedUser")]
struct UserWithRenamedType {
    id: String,
    name: String,
}

#[test]
fn test_type_rename() {
    let td = UserWithRenamedType::typescript();
    // The TypeDef should use the renamed type name
    assert_eq!(td.render(), "RenamedUser");
    assert_eq!(td.render_declaration(), "type RenamedUser = { id: string; name: string };");
}

#[derive(TypeScript)]
struct FieldRenameStruct {
    #[ts(rename = "userId")]
    user_id: String,
    #[ts(rename = "userName")]
    user_name: String,
    not_renamed: i32,
}

#[test]
fn test_field_rename() {
    let td = FieldRenameStruct::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("userId: string"));
    assert!(rendered.contains("userName: string"));
    assert!(rendered.contains("not_renamed: number"));
    // Should NOT contain the original snake_case names for renamed fields
    assert!(!rendered.contains("user_id:"));
    assert!(!rendered.contains("user_name:"));
}

#[derive(TypeScript)]
#[ts(rename_all = "camelCase")]
struct CamelCaseStruct {
    user_id: String,
    user_name: String,
    is_active: bool,
}

#[test]
fn test_rename_all_camel_case() {
    let td = CamelCaseStruct::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("userId: string"));
    assert!(rendered.contains("userName: string"));
    assert!(rendered.contains("isActive: boolean"));
}

#[derive(TypeScript)]
#[ts(rename_all = "PascalCase")]
struct PascalCaseStruct {
    user_id: String,
    is_active: bool,
}

#[test]
fn test_rename_all_pascal_case() {
    let td = PascalCaseStruct::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("UserId: string"));
    assert!(rendered.contains("IsActive: boolean"));
}

#[derive(TypeScript)]
#[ts(rename_all = "SCREAMING_SNAKE_CASE")]
struct ScreamingSnakeStruct {
    user_id: String,
    is_active: bool,
}

#[test]
fn test_rename_all_screaming_snake() {
    let td = ScreamingSnakeStruct::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("USER_ID: string"));
    assert!(rendered.contains("IS_ACTIVE: boolean"));
}

#[derive(TypeScript)]
#[ts(rename_all = "kebab-case")]
struct KebabCaseStruct {
    user_id: String,
    is_active: bool,
}

#[test]
fn test_rename_all_kebab_case() {
    let td = KebabCaseStruct::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("user-id: string"));
    assert!(rendered.contains("is-active: boolean"));
}

#[derive(TypeScript)]
#[ts(rename_all = "camelCase")]
struct MixedRenameStruct {
    user_id: String,
    #[ts(rename = "EXPLICIT_NAME")]
    some_field: String,
    another_field: i32,
}

#[test]
fn test_field_rename_overrides_rename_all() {
    let td = MixedRenameStruct::typescript();
    let rendered = inner_def(td).render();
    // rename_all applies to user_id and another_field
    assert!(rendered.contains("userId: string"));
    assert!(rendered.contains("anotherField: number"));
    // Explicit rename overrides rename_all
    assert!(rendered.contains("EXPLICIT_NAME: string"));
    // Should NOT contain the camelCase version of some_field
    assert!(!rendered.contains("someField:"));
}

// ============================================================================
// SKIP ATTRIBUTE TESTS
// ============================================================================

#[derive(TypeScript)]
struct SkipFieldStruct {
    id: String,
    name: String,
    #[ts(skip)]
    internal_state: i32,
    #[ts(skip)]
    cache: Option<String>,
    visible: bool,
}

#[test]
fn test_skip_field() {
    let td = SkipFieldStruct::typescript();
    let rendered = inner_def(td).render();
    // Should include non-skipped fields
    assert!(rendered.contains("id: string"));
    assert!(rendered.contains("name: string"));
    assert!(rendered.contains("visible: boolean"));
    // Should NOT include skipped fields
    assert!(!rendered.contains("internal_state"));
    assert!(!rendered.contains("cache"));
}

#[derive(TypeScript)]
struct AllSkippedStruct {
    #[ts(skip)]
    hidden1: String,
    #[ts(skip)]
    hidden2: i32,
}

#[test]
fn test_all_fields_skipped() {
    let td = AllSkippedStruct::typescript();
    let rendered = inner_def(td).render();
    // Should be an empty object
    assert_eq!(rendered, "{}");
}

#[derive(TypeScript)]
#[ts(rename_all = "camelCase")]
struct SkipWithRenameAll {
    user_id: String,
    #[ts(skip)]
    internal_id: i32,
    user_name: String,
}

#[test]
fn test_skip_with_rename_all() {
    let td = SkipWithRenameAll::typescript();
    let rendered = inner_def(td).render();
    // rename_all should apply to non-skipped fields
    assert!(rendered.contains("userId: string"));
    assert!(rendered.contains("userName: string"));
    // Skipped field should not appear
    assert!(!rendered.contains("internalId"));
    assert!(!rendered.contains("internal_id"));
}

// ============================================================================
// FLATTEN ATTRIBUTE TESTS
// ============================================================================

#[derive(TypeScript)]
struct InnerData {
    inner_field: String,
    inner_count: i32,
}

#[derive(TypeScript)]
struct OuterWithFlatten {
    outer_id: String,
    #[ts(flatten)]
    inner: InnerData,
    outer_active: bool,
}

#[test]
fn test_flatten_basic() {
    let td = OuterWithFlatten::typescript();
    let rendered = inner_def(td).render();
    // Should contain all fields from both structs
    assert!(rendered.contains("outer_id: string"));
    assert!(rendered.contains("inner_field: string"));
    assert!(rendered.contains("inner_count: number"));
    assert!(rendered.contains("outer_active: boolean"));
    // Should NOT contain the 'inner' field as a nested object
    assert!(!rendered.contains("inner: InnerData"));
    assert!(!rendered.contains("inner: {"));
}

#[derive(TypeScript)]
struct Metadata {
    created_at: String,
    updated_at: String,
}

#[derive(TypeScript)]
struct Resource {
    id: String,
    #[ts(flatten)]
    meta: Metadata,
}

#[test]
fn test_flatten_with_multiple_fields() {
    let td = Resource::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("id: string"));
    assert!(rendered.contains("created_at: string"));
    assert!(rendered.contains("updated_at: string"));
}

// ============================================================================
// TYPE OVERRIDE ATTRIBUTE TESTS
// ============================================================================

#[derive(TypeScript)]
struct DateFields {
    #[ts(type = "Date")]
    created_at: String,
    #[ts(type = "Date")]
    updated_at: String,
    name: String,
}

#[test]
fn test_type_override() {
    let td = DateFields::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("created_at: Date"));
    assert!(rendered.contains("updated_at: Date"));
    assert!(rendered.contains("name: string"));
}

#[derive(TypeScript)]
struct CustomTypes {
    #[ts(type = "HTMLElement")]
    element: u64,
    #[ts(type = "Map<string, number>")]
    data: String,
}

#[test]
fn test_type_override_complex() {
    let td = CustomTypes::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("element: HTMLElement"));
    assert!(rendered.contains("data: Map<string, number>"));
}

// ============================================================================
// TRANSPARENT ATTRIBUTE TESTS
// ============================================================================

#[derive(TypeScript)]
#[ts(transparent)]
struct UserId(String);

#[test]
fn test_transparent_newtype() {
    let td = UserId::typescript();
    // Transparent types should NOT wrap in Named - they become the inner type directly
    assert_eq!(td.render(), "string");
}

#[derive(TypeScript)]
#[ts(transparent)]
struct Count(i32);

#[test]
fn test_transparent_number() {
    let td = Count::typescript();
    assert_eq!(td.render(), "number");
}

#[derive(TypeScript)]
#[ts(transparent)]
struct Items(Vec<String>);

#[test]
fn test_transparent_complex() {
    let td = Items::typescript();
    assert_eq!(td.render(), "string[]");
}

// ============================================================================
// DEFAULT ATTRIBUTE TESTS
// ============================================================================

#[derive(TypeScript)]
struct ConfigWithDefaults {
    required_field: String,
    #[ts(default)]
    optional_count: i32,
    #[ts(default)]
    optional_name: String,
}

#[test]
fn test_default_makes_optional() {
    let td = ConfigWithDefaults::typescript();
    let rendered = inner_def(td).render();
    // required_field should NOT have ?
    assert!(rendered.contains("required_field: string"));
    assert!(!rendered.contains("required_field?"));
    // optional fields should have ?
    assert!(rendered.contains("optional_count?: number"));
    assert!(rendered.contains("optional_name?: string"));
}

#[derive(TypeScript)]
#[ts(rename_all = "camelCase")]
struct ConfigWithDefaultsAndRename {
    user_id: String,
    #[ts(default)]
    display_name: String,
}

#[test]
fn test_default_with_rename() {
    let td = ConfigWithDefaultsAndRename::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("userId: string"));
    assert!(rendered.contains("displayName?: string"));
}

// ============================================================================
// INLINE ATTRIBUTE TESTS
// ============================================================================

#[derive(TypeScript)]
struct Address {
    street: String,
    city: String,
}

#[derive(TypeScript)]
struct PersonWithInline {
    name: String,
    #[ts(inline)]
    address: Address,
}

#[test]
fn test_inline_type() {
    let td = PersonWithInline::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("name: string"));
    // The address field should be inlined, not a reference to Address
    // It should contain the actual object type definition
    assert!(rendered.contains("address: { street: string; city: string }"));
    assert!(!rendered.contains("address: Address"));
}

#[derive(TypeScript)]
struct NestedType {
    value: i32,
}

#[derive(TypeScript)]
struct ContainerWithInline {
    #[ts(inline)]
    nested: NestedType,
    other: String,
}

#[test]
fn test_inline_simple_type() {
    let td = ContainerWithInline::typescript();
    let rendered = inner_def(td).render();
    assert!(rendered.contains("nested: { value: number }"));
    assert!(rendered.contains("other: string"));
}
