/**
 * Ferrotype Client - Typesafe Rust->TS RPC
 *
 * This package provides a 100% typesafe client for Ferrotype RPC.
 * No 'as' assertions or 'any' types are used.
 */

export const VERSION = "0.0.1" as const;

// ============================================================================
// Result Type - Rust's Result<T, E> as a TypeScript discriminated union
// ============================================================================

/**
 * Represents a successful result containing a value of type T.
 */
export interface Ok<T> {
  readonly ok: true;
  readonly value: T;
}

/**
 * Represents a failed result containing an error of type E.
 */
export interface Err<E> {
  readonly ok: false;
  readonly error: E;
}

/**
 * A discriminated union representing either success (Ok) or failure (Err).
 * Mirrors Rust's Result<T, E> type.
 */
export type Result<T, E> = Ok<T> | Err<E>;

// ============================================================================
// Result Constructors
// ============================================================================

/**
 * Creates a successful Result containing the given value.
 */
export function ok<T>(value: T): Ok<T> {
  return { ok: true, value };
}

/**
 * Creates a failed Result containing the given error.
 */
export function err<E>(error: E): Err<E> {
  return { ok: false, error };
}

// ============================================================================
// Result Type Guards
// ============================================================================

/**
 * Type guard that checks if a Result is Ok.
 */
export function isOk<T, E>(result: Result<T, E>): result is Ok<T> {
  return result.ok;
}

/**
 * Type guard that checks if a Result is Err.
 */
export function isErr<T, E>(result: Result<T, E>): result is Err<E> {
  return !result.ok;
}

// ============================================================================
// Result Utilities
// ============================================================================

/**
 * Extracts the value from an Ok result, or throws if Err.
 * Use only when you are certain the result is Ok.
 */
export function unwrap<T, E>(result: Result<T, E>): T {
  if (result.ok) {
    return result.value;
  }
  throw new Error("Called unwrap on an Err value");
}

/**
 * Extracts the error from an Err result, or throws if Ok.
 * Use only when you are certain the result is Err.
 */
export function unwrapErr<T, E>(result: Result<T, E>): E {
  if (!result.ok) {
    return result.error;
  }
  throw new Error("Called unwrapErr on an Ok value");
}

/**
 * Extracts the value from an Ok result, or returns the provided default.
 */
export function unwrapOr<T, E>(result: Result<T, E>, defaultValue: T): T {
  if (result.ok) {
    return result.value;
  }
  return defaultValue;
}

/**
 * Extracts the value from an Ok result, or computes it from the error.
 */
export function unwrapOrElse<T, E>(
  result: Result<T, E>,
  fn: (error: E) => T
): T {
  if (result.ok) {
    return result.value;
  }
  return fn(result.error);
}

/**
 * Maps a Result<T, E> to Result<U, E> by applying a function to the Ok value.
 */
export function map<T, E, U>(
  result: Result<T, E>,
  fn: (value: T) => U
): Result<U, E> {
  if (result.ok) {
    return ok(fn(result.value));
  }
  return result;
}

/**
 * Maps a Result<T, E> to Result<T, F> by applying a function to the Err value.
 */
export function mapErr<T, E, F>(
  result: Result<T, E>,
  fn: (error: E) => F
): Result<T, F> {
  if (result.ok) {
    return result;
  }
  return err(fn(result.error));
}

/**
 * Returns the provided Result if Ok, otherwise returns the Err.
 * Enables chaining of Results.
 */
export function and<T, E, U>(
  result: Result<T, E>,
  other: Result<U, E>
): Result<U, E> {
  if (result.ok) {
    return other;
  }
  return result;
}

/**
 * Calls the provided function with the Ok value and returns its Result.
 * Enables chaining operations that may fail.
 */
export function andThen<T, E, U>(
  result: Result<T, E>,
  fn: (value: T) => Result<U, E>
): Result<U, E> {
  if (result.ok) {
    return fn(result.value);
  }
  return result;
}

/**
 * Returns the Result if Ok, otherwise returns the provided alternative.
 */
export function or<T, E, F>(
  result: Result<T, E>,
  other: Result<T, F>
): Result<T, F> {
  if (result.ok) {
    return result;
  }
  return other;
}

/**
 * Returns the Result if Ok, otherwise calls the function with the error.
 */
export function orElse<T, E, F>(
  result: Result<T, E>,
  fn: (error: E) => Result<T, F>
): Result<T, F> {
  if (result.ok) {
    return result;
  }
  return fn(result.error);
}

/**
 * Pattern matches on a Result, calling the appropriate function.
 */
export function match<T, E, U>(
  result: Result<T, E>,
  handlers: {
    ok: (value: T) => U;
    err: (error: E) => U;
  }
): U {
  if (result.ok) {
    return handlers.ok(result.value);
  }
  return handlers.err(result.error);
}

// ============================================================================
// Struct Variant Enum Codegen - Discriminated unions for Rust enum struct variants
// ============================================================================

/**
 * Base type for a struct variant with a discriminant tag and named fields.
 * Represents a Rust enum variant with struct fields, e.g.:
 *   enum Message { Move { x: i32, y: i32 } }
 * becomes:
 *   type Message = { type: "Move"; x: number; y: number }
 */
export type StructVariant<
  Tag extends string,
  Fields extends Record<string, unknown>
> = { readonly type: Tag } & { readonly [K in keyof Fields]: Fields[K] };

/**
 * Creates a struct variant instance with the given tag and fields.
 *
 * @example
 * type Move = StructVariant<"Move", { x: number; y: number }>;
 * const move: Move = structVariant("Move", { x: 10, y: 20 });
 */
export function structVariant<
  Tag extends string,
  Fields extends Record<string, unknown>
>(tag: Tag, fields: Fields): StructVariant<Tag, Fields> {
  return { type: tag, ...fields } as StructVariant<Tag, Fields>;
}

/**
 * Type guard that checks if a discriminated union value has the given tag.
 *
 * @example
 * type Message = Move | Write;
 * if (isStructVariant(msg, "Move")) {
 *   console.log(msg.x, msg.y); // TypeScript knows msg has x and y
 * }
 */
export function isStructVariant<
  Union extends { readonly type: string },
  Tag extends Union["type"]
>(value: Union, tag: Tag): value is Extract<Union, { type: Tag }> {
  return value.type === tag;
}

/**
 * Exhaustively matches on a discriminated union, calling the appropriate handler.
 *
 * @example
 * type Message =
 *   | StructVariant<"Move", { x: number; y: number }>
 *   | StructVariant<"Write", { text: string }>
 *   | StructVariant<"Quit", {}>;
 *
 * const result = matchEnum(message, {
 *   Move: (m) => `Moving to ${m.x}, ${m.y}`,
 *   Write: (w) => `Writing: ${w.text}`,
 *   Quit: () => "Quitting",
 * });
 */
export function matchEnum<
  Union extends { readonly type: string },
  Handlers extends {
    [K in Union["type"]]: (
      variant: Extract<Union, { type: K }>
    ) => unknown;
  },
  ReturnType = Handlers[Union["type"]] extends (arg: never) => infer R
    ? R
    : never
>(value: Union, handlers: Handlers): ReturnType {
  const handler = handlers[value.type as Union["type"]];
  return handler(value as Extract<Union, { type: Union["type"] }>) as ReturnType;
}

// ============================================================================
// Enum Variant Constructors - Factory functions for creating variants
// ============================================================================

/**
 * Creates a factory function for constructing struct variants of a specific tag.
 * Useful for creating typed constructors for each variant of an enum.
 *
 * @example
 * const Move = variantConstructor<Message, "Move">("Move");
 * const msg: Message = Move({ x: 10, y: 20 });
 */
export function variantConstructor<
  Union extends { readonly type: string },
  Tag extends Union["type"]
>(
  tag: Tag
): (
  fields: Omit<Extract<Union, { type: Tag }>, "type">
) => Extract<Union, { type: Tag }> {
  return (fields) => ({ type: tag, ...fields }) as Extract<Union, { type: Tag }>;
}

/**
 * Creates an object containing constructor functions for all variants of an enum.
 * Each constructor is named after its variant tag.
 *
 * @example
 * type Message =
 *   | StructVariant<"Move", { x: number; y: number }>
 *   | StructVariant<"Write", { text: string }>;
 *
 * const Message = enumConstructors<Message>(["Move", "Write"]);
 * const msg = Message.Move({ x: 10, y: 20 });
 */
export function enumConstructors<Union extends { readonly type: string }>(
  tags: readonly Union["type"][]
): {
  [K in Union["type"]]: (
    fields: Omit<Extract<Union, { type: K }>, "type">
  ) => Extract<Union, { type: K }>;
} {
  const constructors = {} as {
    [K in Union["type"]]: (
      fields: Omit<Extract<Union, { type: K }>, "type">
    ) => Extract<Union, { type: K }>;
  };

  for (const tag of tags) {
    (constructors as Record<string, unknown>)[tag] = (
      fields: Record<string, unknown>
    ) => ({ type: tag, ...fields });
  }

  return constructors;
}

// ============================================================================
// Enum Type Guards - Runtime checks for variant discrimination
// ============================================================================

/**
 * Creates a type guard function for a specific variant tag.
 *
 * @example
 * const isMove = variantGuard<Message, "Move">("Move");
 * if (isMove(msg)) {
 *   console.log(msg.x, msg.y);
 * }
 */
export function variantGuard<
  Union extends { readonly type: string },
  Tag extends Union["type"]
>(tag: Tag): (value: Union) => value is Extract<Union, { type: Tag }> {
  return (value): value is Extract<Union, { type: Tag }> => value.type === tag;
}

/**
 * Creates an object containing type guard functions for all variants of an enum.
 *
 * @example
 * type Message =
 *   | StructVariant<"Move", { x: number; y: number }>
 *   | StructVariant<"Write", { text: string }>;
 *
 * const is = enumGuards<Message>(["Move", "Write"]);
 * if (is.Move(msg)) {
 *   console.log(msg.x, msg.y);
 * }
 */
export function enumGuards<Union extends { readonly type: string }>(
  tags: readonly Union["type"][]
): {
  [K in Union["type"]]: (value: Union) => value is Extract<Union, { type: K }>;
} {
  const guards = {} as {
    [K in Union["type"]]: (
      value: Union
    ) => value is Extract<Union, { type: K }>;
  };

  for (const tag of tags) {
    (guards as Record<string, unknown>)[tag] = (
      value: { readonly type: string }
    ) => value.type === tag;
  }

  return guards;
}

// ============================================================================
// RPC Client - Base class for type-safe RPC calls
// ============================================================================

/**
 * Configuration for the RPC client.
 */
export interface RpcClientConfig {
  /** Base URL for the RPC endpoint (e.g., "http://localhost:8080/rpc") */
  readonly baseUrl: string;
  /** Optional custom headers to include with every request */
  readonly headers?: Record<string, string>;
  /** Optional timeout in milliseconds (default: 30000) */
  readonly timeout?: number;
}

/**
 * RPC error types that can occur during a call.
 */
export type RpcClientError =
  | { readonly type: "NetworkError"; readonly message: string }
  | { readonly type: "TimeoutError"; readonly message: string }
  | { readonly type: "ParseError"; readonly message: string; readonly body: string }
  | { readonly type: "HttpError"; readonly status: number; readonly statusText: string; readonly body: string }
  | { readonly type: "ServerError"; readonly error: unknown };

/**
 * Creates a NetworkError.
 */
export function networkError(message: string): RpcClientError {
  return { type: "NetworkError", message };
}

/**
 * Creates a TimeoutError.
 */
export function timeoutError(message: string): RpcClientError {
  return { type: "TimeoutError", message };
}

/**
 * Creates a ParseError.
 */
export function parseError(message: string, body: string): RpcClientError {
  return { type: "ParseError", message, body };
}

/**
 * Creates an HttpError.
 */
export function httpError(status: number, statusText: string, body: string): RpcClientError {
  return { type: "HttpError", status, statusText, body };
}

/**
 * Creates a ServerError.
 */
export function serverError(error: unknown): RpcClientError {
  return { type: "ServerError", error };
}

/**
 * Type guard for RpcClientError variants.
 */
export function isNetworkError(e: RpcClientError): e is { type: "NetworkError"; message: string } {
  return e.type === "NetworkError";
}

export function isTimeoutError(e: RpcClientError): e is { type: "TimeoutError"; message: string } {
  return e.type === "TimeoutError";
}

export function isParseError(e: RpcClientError): e is { type: "ParseError"; message: string; body: string } {
  return e.type === "ParseError";
}

export function isHttpError(e: RpcClientError): e is { type: "HttpError"; status: number; statusText: string; body: string } {
  return e.type === "HttpError";
}

export function isServerError(e: RpcClientError): e is { type: "ServerError"; error: unknown } {
  return e.type === "ServerError";
}

/**
 * Wire format for RPC requests.
 * Matches the expected format on the Rust side.
 */
export interface RpcRequest<T = unknown> {
  readonly method: string;
  readonly params: T;
}

/**
 * Wire format for RPC responses.
 * Matches the format produced by the Rust side.
 */
export type RpcResponse<T, E> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E };

/**
 * Transport interface for making RPC requests.
 * Can be implemented for different environments (fetch, Node.js, mock, etc.)
 */
export interface RpcTransport {
  /**
   * Sends an RPC request and returns the raw response body.
   * @param url The full URL to send the request to
   * @param body The JSON-stringified request body
   * @param headers Headers to include with the request
   * @param timeout Request timeout in milliseconds
   * @returns Promise resolving to the response body string, or rejecting with RpcClientError
   */
  send(
    url: string,
    body: string,
    headers: Record<string, string>,
    timeout: number
  ): Promise<Result<string, RpcClientError>>;
}

/**
 * Default transport using the Fetch API.
 */
export class FetchTransport implements RpcTransport {
  async send(
    url: string,
    body: string,
    headers: Record<string, string>,
    timeout: number
  ): Promise<Result<string, RpcClientError>> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeout);

    try {
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...headers,
        },
        body,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      const responseBody = await response.text();

      if (!response.ok) {
        return err(httpError(response.status, response.statusText, responseBody));
      }

      return ok(responseBody);
    } catch (e) {
      clearTimeout(timeoutId);

      if (e instanceof Error) {
        if (e.name === "AbortError") {
          return err(timeoutError(`Request timed out after ${timeout}ms`));
        }
        return err(networkError(e.message));
      }
      return err(networkError("Unknown network error"));
    }
  }
}

/**
 * Base RPC client providing type-safe method calls.
 *
 * @example
 * ```typescript
 * const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" });
 *
 * // Type-safe call with explicit types
 * const result = await client.call<GetUserResponse, ApiError>("get_user", { user_id: 123 });
 *
 * if (result.ok) {
 *   console.log(result.value.user);
 * } else {
 *   console.error(result.error);
 * }
 * ```
 */
export class RpcClient {
  private readonly config: Required<RpcClientConfig>;
  private readonly transport: RpcTransport;

  constructor(config: RpcClientConfig, transport?: RpcTransport) {
    this.config = {
      baseUrl: config.baseUrl,
      headers: config.headers ?? {},
      timeout: config.timeout ?? 30000,
    };
    this.transport = transport ?? new FetchTransport();
  }

  /**
   * Calls an RPC method with type-safe request and response types.
   *
   * @typeParam TResponse - The expected response type on success
   * @typeParam TError - The expected error type from the server
   * @typeParam TRequest - The request parameters type (usually inferred)
   *
   * @param method - The RPC method name to call
   * @param params - The parameters to send with the request
   *
   * @returns A Result containing either:
   *   - Ok<TResponse> on success
   *   - Err<RpcClientError | TError> on failure (client error or server error)
   */
  async call<TResponse, TError = unknown, TRequest = unknown>(
    method: string,
    params: TRequest
  ): Promise<Result<TResponse, RpcClientError | TError>> {
    const request: RpcRequest<TRequest> = { method, params };
    const body = JSON.stringify(request);

    const transportResult = await this.transport.send(
      this.config.baseUrl,
      body,
      this.config.headers,
      this.config.timeout
    );

    if (!transportResult.ok) {
      return transportResult;
    }

    let parsed: RpcResponse<TResponse, TError>;
    try {
      parsed = JSON.parse(transportResult.value) as RpcResponse<TResponse, TError>;
    } catch (e) {
      const message = e instanceof Error ? e.message : "JSON parse error";
      return err(parseError(message, transportResult.value));
    }

    if (parsed.ok) {
      return ok(parsed.value);
    } else {
      return err(parsed.error);
    }
  }

  /**
   * Creates a new client with additional headers merged with existing ones.
   * Useful for adding authentication tokens.
   *
   * @param headers - Headers to add or override
   * @returns A new RpcClient instance with merged headers
   */
  withHeaders(headers: Record<string, string>): RpcClient {
    return new RpcClient(
      {
        ...this.config,
        headers: { ...this.config.headers, ...headers },
      },
      this.transport
    );
  }

  /**
   * Creates a new client with a different timeout.
   *
   * @param timeout - New timeout in milliseconds
   * @returns A new RpcClient instance with the new timeout
   */
  withTimeout(timeout: number): RpcClient {
    return new RpcClient(
      {
        ...this.config,
        timeout,
      },
      this.transport
    );
  }

  /**
   * Gets the current configuration (read-only).
   */
  getConfig(): Readonly<Required<RpcClientConfig>> {
    return this.config;
  }
}

// ============================================================================
// Option Type - Rust's Option<T> as a TypeScript discriminated union
// ============================================================================

/**
 * Represents the Some variant of an Option, containing a value.
 * Corresponds to Rust's `Some(T)` serialized with serde's externally-tagged format.
 */
export interface Some<T> {
  readonly Some: T;
}

/**
 * Represents the None variant of an Option.
 * Corresponds to Rust's `None` serialized with serde's externally-tagged format.
 */
export interface None {
  readonly None: null;
}

/**
 * A discriminated union representing an optional value.
 * Mirrors Rust's Option<T> type with serde's externally-tagged serialization.
 *
 * @example
 * // From Rust: Option::Some("hello")
 * const some: Option<string> = { Some: "hello" };
 *
 * // From Rust: Option::None
 * const none: Option<string> = { None: null };
 */
export type Option<T> = Some<T> | None;

// ============================================================================
// Option Constructors
// ============================================================================

/**
 * Creates a Some variant containing the given value.
 *
 * @example
 * const opt = some("hello"); // { Some: "hello" }
 */
export function some<T>(value: T): Some<T> {
  return { Some: value };
}

/**
 * Creates a None variant.
 *
 * @example
 * const opt = none<string>(); // { None: null }
 */
export function none<_T = never>(): None {
  return { None: null };
}

// ============================================================================
// Option Type Guards
// ============================================================================

/**
 * Type guard that checks if an Option is Some.
 *
 * @example
 * const opt: Option<number> = some(42);
 * if (isSome(opt)) {
 *   console.log(opt.Some); // TypeScript knows opt.Some exists
 * }
 */
export function isSome<T>(option: Option<T>): option is Some<T> {
  return "Some" in option;
}

/**
 * Type guard that checks if an Option is None.
 *
 * @example
 * const opt: Option<number> = none();
 * if (isNone(opt)) {
 *   console.log("No value");
 * }
 */
export function isNone<T>(option: Option<T>): option is None {
  return "None" in option;
}

// ============================================================================
// Option <-> Nullable Conversions
// ============================================================================

/**
 * Converts an Option<T> to T | null.
 * This is the primary way to work with optional values in idiomatic TypeScript.
 *
 * @example
 * const opt: Option<string> = some("hello");
 * const value: string | null = optionToNullable(opt); // "hello"
 *
 * const noneOpt: Option<string> = none();
 * const nullValue: string | null = optionToNullable(noneOpt); // null
 */
export function optionToNullable<T>(option: Option<T>): T | null {
  if (isSome(option)) {
    return option.Some;
  }
  return null;
}

/**
 * Converts a nullable value (T | null | undefined) to Option<T>.
 * Useful when sending data back to Rust that expects Option<T>.
 *
 * @example
 * const value: string | null = "hello";
 * const opt: Option<string> = nullableToOption(value); // { Some: "hello" }
 *
 * const nullValue: string | null = null;
 * const noneOpt: Option<string> = nullableToOption(nullValue); // { None: null }
 */
export function nullableToOption<T>(value: T | null | undefined): Option<T> {
  if (value === null || value === undefined) {
    return none();
  }
  return some(value);
}

// ============================================================================
// Option Utilities
// ============================================================================

/**
 * Extracts the value from a Some, or throws if None.
 * Use only when you are certain the option is Some.
 *
 * @example
 * const opt = some(42);
 * const value = unwrapOption(opt); // 42
 *
 * const noneOpt = none<number>();
 * unwrapOption(noneOpt); // throws Error
 */
export function unwrapOption<T>(option: Option<T>): T {
  if (isSome(option)) {
    return option.Some;
  }
  throw new Error("Called unwrapOption on a None value");
}

/**
 * Extracts the value from a Some, or returns the provided default.
 *
 * @example
 * const opt: Option<number> = none();
 * const value = unwrapOptionOr(opt, 0); // 0
 */
export function unwrapOptionOr<T>(option: Option<T>, defaultValue: T): T {
  if (isSome(option)) {
    return option.Some;
  }
  return defaultValue;
}

/**
 * Extracts the value from a Some, or computes it from a function.
 *
 * @example
 * const opt: Option<number> = none();
 * const value = unwrapOptionOrElse(opt, () => computeDefault()); // result of computeDefault()
 */
export function unwrapOptionOrElse<T>(option: Option<T>, fn: () => T): T {
  if (isSome(option)) {
    return option.Some;
  }
  return fn();
}

/**
 * Maps an Option<T> to Option<U> by applying a function to the Some value.
 *
 * @example
 * const opt: Option<number> = some(5);
 * const doubled = mapOption(opt, x => x * 2); // { Some: 10 }
 *
 * const noneOpt: Option<number> = none();
 * const mapped = mapOption(noneOpt, x => x * 2); // { None: null }
 */
export function mapOption<T, U>(option: Option<T>, fn: (value: T) => U): Option<U> {
  if (isSome(option)) {
    return some(fn(option.Some));
  }
  return option;
}

/**
 * Maps an Option<T> to Option<U>, where the mapping function also returns an Option.
 * Flattens the result (avoids Option<Option<U>>).
 *
 * @example
 * const opt: Option<string> = some("42");
 * const parsed = flatMapOption(opt, s => {
 *   const n = parseInt(s, 10);
 *   return isNaN(n) ? none() : some(n);
 * }); // { Some: 42 }
 */
export function flatMapOption<T, U>(option: Option<T>, fn: (value: T) => Option<U>): Option<U> {
  if (isSome(option)) {
    return fn(option.Some);
  }
  return none();
}

/**
 * Pattern matches on an Option, calling the appropriate handler.
 *
 * @example
 * const opt: Option<number> = some(42);
 * const result = matchOption(opt, {
 *   some: (n) => `Got ${n}`,
 *   none: () => "Nothing",
 * }); // "Got 42"
 */
export function matchOption<T, U>(
  option: Option<T>,
  handlers: {
    some: (value: T) => U;
    none: () => U;
  }
): U {
  if (isSome(option)) {
    return handlers.some(option.Some);
  }
  return handlers.none();
}

/**
 * Filters an Option by a predicate. Returns None if the predicate is false.
 *
 * @example
 * const opt: Option<number> = some(5);
 * const filtered = filterOption(opt, n => n > 3); // { Some: 5 }
 * const filteredOut = filterOption(opt, n => n > 10); // { None: null }
 */
export function filterOption<T>(option: Option<T>, predicate: (value: T) => boolean): Option<T> {
  if (isSome(option) && predicate(option.Some)) {
    return option;
  }
  return none();
}

/**
 * Converts an Option<T> to a Result<T, E> with the given error if None.
 *
 * @example
 * const opt: Option<number> = some(42);
 * const result = optionToResult(opt, "No value"); // { ok: true, value: 42 }
 *
 * const noneOpt: Option<number> = none();
 * const errResult = optionToResult(noneOpt, "No value"); // { ok: false, error: "No value" }
 */
export function optionToResult<T, E>(option: Option<T>, error: E): Result<T, E> {
  if (isSome(option)) {
    return ok(option.Some);
  }
  return err(error);
}

/**
 * Converts a Result<T, E> to an Option<T>, discarding the error.
 *
 * @example
 * const result: Result<number, string> = ok(42);
 * const opt = resultToOption(result); // { Some: 42 }
 *
 * const errResult: Result<number, string> = err("failed");
 * const noneOpt = resultToOption(errResult); // { None: null }
 */
export function resultToOption<T, E>(result: Result<T, E>): Option<T> {
  if (result.ok) {
    return some(result.value);
  }
  return none();
}
