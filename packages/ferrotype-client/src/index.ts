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
