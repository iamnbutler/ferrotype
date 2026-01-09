/**
 * Ferrotype Test Fixtures - TypeScript Types
 *
 * These types correspond to the Rust fixtures in ferrotype-fixtures.
 * They are used to verify that JSON serialized from Rust can be
 * correctly typed and parsed in TypeScript.
 */

// ============================================================================
// STRUCT TYPES
// ============================================================================

/** Simple struct with named fields */
export interface Point {
  x: number;
  y: number;
}

/** Struct with multiple field types */
export interface User {
  id: number;
  name: string;
  email: string;
  active: boolean;
}

/** Struct with optional fields */
export interface Profile {
  username: string;
  display_name: string | null;
  bio: string | null;
  avatar_url: string | null;
}

/** Tuple struct (serializes as array) */
export type Rgb = [number, number, number];

/** Unit struct (serializes as null) */
export type Ping = null;

/** Newtype wrapper (serializes as inner value) */
export type UserId = number;

/** Nested struct */
export interface Rectangle {
  top_left: Point;
  bottom_right: Point;
}

/** Struct with Vec field */
export interface Polygon {
  vertices: Point[];
}

/** Struct with HashMap field */
export interface Config {
  settings: Record<string, string>;
}

// ============================================================================
// ENUM TYPES
// ============================================================================

/** Simple unit-variant enum (serializes as string) */
export type Status = "Pending" | "Active" | "Completed" | "Failed";

/** Enum with tuple variants (serde externally tagged) */
export type Coordinate =
  | { D2: [number, number] }
  | { D3: [number, number, number] };

/** Enum with struct variants (serde externally tagged) */
export type Shape =
  | { Circle: { center: Point; radius: number } }
  | { Rectangle: { top_left: Point; width: number; height: number } }
  | { Triangle: { a: Point; b: Point; c: Point } };

/** Mixed variant enum (serde externally tagged) */
export type Message =
  | { Ping: null }
  | { Text: string }
  | { Binary: number[] }
  | { Error: { code: number; message: string } };

/** Optional enum wrapper (serde externally tagged) */
export type OptionalValue<T> = { None: null } | { Some: T };

// ============================================================================
// RPC REQUEST/RESPONSE TYPES
// ============================================================================

/** Typical RPC request */
export interface GetUserRequest {
  user_id: number;
}

/** Typical RPC response */
export interface GetUserResponse {
  user: User | null;
}

/** List request with pagination */
export interface ListUsersRequest {
  page: number;
  per_page: number;
  filter: string | null;
}

/** List response with pagination metadata */
export interface ListUsersResponse {
  users: User[];
  total: number;
  page: number;
  per_page: number;
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/** Simple error type */
export interface ApiError {
  code: string;
  message: string;
}

/** Detailed error with optional fields */
export interface DetailedError {
  code: string;
  message: string;
  details: string | null;
  field: string | null;
}

/** Error enum (serde externally tagged) */
export type RpcError =
  | { NotFound: { resource: string } }
  | { Unauthorized: null }
  | { Forbidden: { reason: string } }
  | { BadRequest: { field: string; message: string } }
  | { Internal: null };

// ============================================================================
// COMPLEX NESTED TYPES
// ============================================================================

/** Deeply nested type */
export interface Workspace {
  id: number;
  name: string;
  owner: User;
  members: User[];
  settings: Config;
  status: Status;
}

/** Type with all common patterns */
export interface CompleteExample {
  primitive: number;
  string: string;
  optional: string | null;
  list: number[];
  map: Record<string, number>;
  nested: User;
  nested_list: User[];
  optional_nested: User | null;
  status: Status;
}

// ============================================================================
// TYPE GUARDS
// ============================================================================

/** Type guard for Status enum */
export function isStatus(value: unknown): value is Status {
  return (
    value === "Pending" ||
    value === "Active" ||
    value === "Completed" ||
    value === "Failed"
  );
}

/** Type guard for Coordinate.D2 */
export function isCoordinateD2(
  value: Coordinate
): value is { D2: [number, number] } {
  return "D2" in value;
}

/** Type guard for Coordinate.D3 */
export function isCoordinateD3(
  value: Coordinate
): value is { D3: [number, number, number] } {
  return "D3" in value;
}

/** Type guard for Shape.Circle */
export function isShapeCircle(
  value: Shape
): value is { Circle: { center: Point; radius: number } } {
  return "Circle" in value;
}

/** Type guard for Shape.Rectangle */
export function isShapeRectangle(
  value: Shape
): value is { Rectangle: { top_left: Point; width: number; height: number } } {
  return "Rectangle" in value;
}

/** Type guard for Shape.Triangle */
export function isShapeTriangle(
  value: Shape
): value is { Triangle: { a: Point; b: Point; c: Point } } {
  return "Triangle" in value;
}

/** Type guard for Message variants */
export function isMessagePing(value: Message): value is { Ping: null } {
  return "Ping" in value;
}

export function isMessageText(value: Message): value is { Text: string } {
  return "Text" in value;
}

export function isMessageBinary(value: Message): value is { Binary: number[] } {
  return "Binary" in value;
}

export function isMessageError(
  value: Message
): value is { Error: { code: number; message: string } } {
  return "Error" in value;
}

/** Type guard for OptionalValue */
export function isOptionalValueNone<T>(
  value: OptionalValue<T>
): value is { None: null } {
  return "None" in value;
}

export function isOptionalValueSome<T>(
  value: OptionalValue<T>
): value is { Some: T } {
  return "Some" in value;
}

/** Type guard for RpcError variants */
export function isRpcErrorNotFound(
  value: RpcError
): value is { NotFound: { resource: string } } {
  return "NotFound" in value;
}

export function isRpcErrorUnauthorized(
  value: RpcError
): value is { Unauthorized: null } {
  return "Unauthorized" in value;
}

export function isRpcErrorForbidden(
  value: RpcError
): value is { Forbidden: { reason: string } } {
  return "Forbidden" in value;
}

export function isRpcErrorBadRequest(
  value: RpcError
): value is { BadRequest: { field: string; message: string } } {
  return "BadRequest" in value;
}

export function isRpcErrorInternal(
  value: RpcError
): value is { Internal: null } {
  return "Internal" in value;
}
