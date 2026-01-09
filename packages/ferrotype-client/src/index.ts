/**
 * Ferrotype Client - Typesafe Rust->TS RPC
 *
 * This package provides a 100% typesafe client for Ferrotype RPC.
 * No 'as' assertions or 'any' types are used.
 */

export const VERSION = "0.0.1" as const;

/**
 * Result type matching Rust's Result<T, E>.
 * Uses discriminated union with 'ok' as the discriminant.
 */
export type Result<T, E> =
  | { ok: true; value: T }
  | { ok: false; error: E };

/**
 * Type guard for successful Result.
 */
export function isOk<T, E>(result: Result<T, E>): result is { ok: true; value: T } {
  return result.ok;
}

/**
 * Type guard for failed Result.
 */
export function isErr<T, E>(result: Result<T, E>): result is { ok: false; error: E } {
  return !result.ok;
}

/**
 * Unwrap a Result, throwing if it's an error.
 */
export function unwrap<T, E>(result: Result<T, E>): T {
  if (result.ok) {
    return result.value;
  }
  throw new Error(`Called unwrap on Err: ${String(result.error)}`);
}

/**
 * Unwrap a Result's error, throwing if it's Ok.
 */
export function unwrapErr<T, E>(result: Result<T, E>): E {
  if (!result.ok) {
    return result.error;
  }
  throw new Error(`Called unwrapErr on Ok`);
}

/**
 * Create an Ok Result.
 */
export function Ok<T>(value: T): { ok: true; value: T } {
  return { ok: true, value };
}

/**
 * Create an Err Result.
 */
export function Err<E>(error: E): { ok: false; error: E } {
  return { ok: false, error };
}
