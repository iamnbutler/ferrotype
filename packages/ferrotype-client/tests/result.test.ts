/**
 * Tests for the Result discriminated union type.
 */

import { describe, it, expect } from "vitest";
import {
  ok,
  err,
  isOk,
  isErr,
  unwrap,
  unwrapErr,
  unwrapOr,
  unwrapOrElse,
  map,
  mapErr,
  and,
  andThen,
  or,
  orElse,
  match,
  type Result,
} from "../src/index.js";

describe("Result", () => {
  describe("constructors", () => {
    it("ok creates an Ok result", () => {
      const result = ok(42);
      expect(result.ok).toBe(true);
      expect(result.value).toBe(42);
    });

    it("err creates an Err result", () => {
      const result = err("error");
      expect(result.ok).toBe(false);
      expect(result.error).toBe("error");
    });

    it("ok preserves complex types", () => {
      const value = { id: 1, name: "test" };
      const result = ok(value);
      expect(result.value).toEqual(value);
    });

    it("err preserves complex types", () => {
      const error = { code: 404, message: "Not found" };
      const result = err(error);
      expect(result.error).toEqual(error);
    });
  });

  describe("type guards", () => {
    it("isOk returns true for Ok", () => {
      const result: Result<number, string> = ok(42);
      expect(isOk(result)).toBe(true);
    });

    it("isOk returns false for Err", () => {
      const result: Result<number, string> = err("error");
      expect(isOk(result)).toBe(false);
    });

    it("isErr returns true for Err", () => {
      const result: Result<number, string> = err("error");
      expect(isErr(result)).toBe(true);
    });

    it("isErr returns false for Ok", () => {
      const result: Result<number, string> = ok(42);
      expect(isErr(result)).toBe(false);
    });

    it("type narrows correctly after isOk", () => {
      const result: Result<number, string> = ok(42);
      if (isOk(result)) {
        // TypeScript should know result.value exists here
        const value: number = result.value;
        expect(value).toBe(42);
      }
    });

    it("type narrows correctly after isErr", () => {
      const result: Result<number, string> = err("error");
      if (isErr(result)) {
        // TypeScript should know result.error exists here
        const error: string = result.error;
        expect(error).toBe("error");
      }
    });
  });

  describe("unwrap", () => {
    it("unwrap returns value for Ok", () => {
      const result = ok(42);
      expect(unwrap(result)).toBe(42);
    });

    it("unwrap throws for Err", () => {
      const result = err("error");
      expect(() => unwrap(result)).toThrow("Called unwrap on an Err value");
    });

    it("unwrapErr returns error for Err", () => {
      const result = err("error");
      expect(unwrapErr(result)).toBe("error");
    });

    it("unwrapErr throws for Ok", () => {
      const result = ok(42);
      expect(() => unwrapErr(result)).toThrow("Called unwrapErr on an Ok value");
    });

    it("unwrapOr returns value for Ok", () => {
      const result: Result<number, string> = ok(42);
      expect(unwrapOr(result, 0)).toBe(42);
    });

    it("unwrapOr returns default for Err", () => {
      const result: Result<number, string> = err("error");
      expect(unwrapOr(result, 0)).toBe(0);
    });

    it("unwrapOrElse returns value for Ok", () => {
      const result: Result<number, string> = ok(42);
      expect(unwrapOrElse(result, () => 0)).toBe(42);
    });

    it("unwrapOrElse computes default for Err", () => {
      const result: Result<number, string> = err("error");
      expect(unwrapOrElse(result, (e) => e.length)).toBe(5);
    });
  });

  describe("map", () => {
    it("map transforms Ok value", () => {
      const result: Result<number, string> = ok(42);
      const mapped = map(result, (x) => x * 2);
      expect(isOk(mapped)).toBe(true);
      expect(unwrap(mapped)).toBe(84);
    });

    it("map preserves Err", () => {
      const result: Result<number, string> = err("error");
      const mapped = map(result, (x) => x * 2);
      expect(isErr(mapped)).toBe(true);
      expect(unwrapErr(mapped)).toBe("error");
    });

    it("mapErr transforms Err value", () => {
      const result: Result<number, string> = err("error");
      const mapped = mapErr(result, (e) => e.toUpperCase());
      expect(isErr(mapped)).toBe(true);
      expect(unwrapErr(mapped)).toBe("ERROR");
    });

    it("mapErr preserves Ok", () => {
      const result: Result<number, string> = ok(42);
      const mapped = mapErr(result, (e) => e.toUpperCase());
      expect(isOk(mapped)).toBe(true);
      expect(unwrap(mapped)).toBe(42);
    });
  });

  describe("and/andThen", () => {
    it("and returns second Result if first is Ok", () => {
      const first: Result<number, string> = ok(42);
      const second: Result<string, string> = ok("hello");
      const result = and(first, second);
      expect(isOk(result)).toBe(true);
      expect(unwrap(result)).toBe("hello");
    });

    it("and returns first Err if first is Err", () => {
      const first: Result<number, string> = err("first error");
      const second: Result<string, string> = ok("hello");
      const result = and(first, second);
      expect(isErr(result)).toBe(true);
      expect(unwrapErr(result)).toBe("first error");
    });

    it("andThen chains Ok results", () => {
      const result: Result<number, string> = ok(42);
      const chained = andThen(result, (x) => ok(x.toString()));
      expect(isOk(chained)).toBe(true);
      expect(unwrap(chained)).toBe("42");
    });

    it("andThen short-circuits on Err", () => {
      const result: Result<number, string> = err("error");
      let called = false;
      const chained = andThen(result, (x) => {
        called = true;
        return ok(x.toString());
      });
      expect(isErr(chained)).toBe(true);
      expect(called).toBe(false);
    });

    it("andThen propagates inner Err", () => {
      const result: Result<number, string> = ok(42);
      const chained = andThen(result, () => err("inner error"));
      expect(isErr(chained)).toBe(true);
      expect(unwrapErr(chained)).toBe("inner error");
    });
  });

  describe("or/orElse", () => {
    it("or returns first if Ok", () => {
      const first: Result<number, string> = ok(42);
      const second: Result<number, number> = ok(100);
      const result = or(first, second);
      expect(isOk(result)).toBe(true);
      expect(unwrap(result)).toBe(42);
    });

    it("or returns second if first is Err", () => {
      const first: Result<number, string> = err("error");
      const second: Result<number, number> = ok(100);
      const result = or(first, second);
      expect(isOk(result)).toBe(true);
      expect(unwrap(result)).toBe(100);
    });

    it("orElse returns Ok unchanged", () => {
      const result: Result<number, string> = ok(42);
      const recovered = orElse(result, () => ok(0));
      expect(isOk(recovered)).toBe(true);
      expect(unwrap(recovered)).toBe(42);
    });

    it("orElse recovers from Err", () => {
      const result: Result<number, string> = err("error");
      const recovered = orElse(result, (e) => ok(e.length));
      expect(isOk(recovered)).toBe(true);
      expect(unwrap(recovered)).toBe(5);
    });

    it("orElse can return new Err", () => {
      const result: Result<number, string> = err("error");
      const recovered = orElse(result, () => err(404));
      expect(isErr(recovered)).toBe(true);
      expect(unwrapErr(recovered)).toBe(404);
    });
  });

  describe("match", () => {
    it("match calls ok handler for Ok", () => {
      const result: Result<number, string> = ok(42);
      const output = match(result, {
        ok: (v) => `value: ${v}`,
        err: (e) => `error: ${e}`,
      });
      expect(output).toBe("value: 42");
    });

    it("match calls err handler for Err", () => {
      const result: Result<number, string> = err("error");
      const output = match(result, {
        ok: (v) => `value: ${v}`,
        err: (e) => `error: ${e}`,
      });
      expect(output).toBe("error: error");
    });

    it("match enables exhaustive handling", () => {
      const result: Result<number, string> = ok(42);
      // Both branches return the same type
      const num: number = match(result, {
        ok: (v) => v,
        err: () => -1,
      });
      expect(num).toBe(42);
    });
  });

  describe("chaining", () => {
    it("chains multiple operations", () => {
      const parseNumber = (s: string): Result<number, string> => {
        const n = parseInt(s, 10);
        return isNaN(n) ? err("not a number") : ok(n);
      };

      const double = (n: number): Result<number, string> =>
        n > 1000 ? err("too large") : ok(n * 2);

      // Success path
      const success = andThen(parseNumber("42"), double);
      expect(unwrap(success)).toBe(84);

      // Parse failure
      const parseFail = andThen(parseNumber("abc"), double);
      expect(unwrapErr(parseFail)).toBe("not a number");

      // Validation failure
      const validationFail = andThen(parseNumber("1001"), double);
      expect(unwrapErr(validationFail)).toBe("too large");
    });
  });
});
