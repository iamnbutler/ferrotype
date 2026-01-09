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
