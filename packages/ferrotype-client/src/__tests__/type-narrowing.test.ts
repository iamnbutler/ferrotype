/**
 * Type narrowing tests for discriminated unions.
 *
 * These tests verify that TypeScript's control flow analysis correctly
 * narrows our discriminated union types. The tests are designed to fail
 * at compile time if type narrowing doesn't work correctly.
 */

import { describe, it, expect, expectTypeOf } from "vitest";
import {
  type Result,
  isOk,
  isErr,
  unwrap,
  unwrapErr,
  Ok,
  Err,
} from "../index.js";

describe("Result type narrowing", () => {
  describe("direct property check narrowing", () => {
    it("narrows to Ok branch when ok is true", () => {
      const result: Result<number, string> = Ok(42);

      if (result.ok) {
        // TypeScript should narrow to { ok: true; value: number }
        expectTypeOf(result.value).toBeNumber();
        expect(result.value).toBe(42);

        // @ts-expect-error - error should not exist on Ok branch
        result.error;
      }
    });

    it("narrows to Err branch when ok is false", () => {
      const result: Result<number, string> = Err("failed");

      if (!result.ok) {
        // TypeScript should narrow to { ok: false; error: string }
        expectTypeOf(result.error).toBeString();
        expect(result.error).toBe("failed");

        // @ts-expect-error - value should not exist on Err branch
        result.value;
      }
    });

    it("narrows in else branch", () => {
      function processResult(result: Result<number, string>): number | string {
        if (result.ok) {
          expectTypeOf(result.value).toBeNumber();
          return result.value;
        } else {
          // Should narrow to Err in else branch
          expectTypeOf(result.error).toBeString();
          return result.error;
        }
      }

      expect(processResult(Ok(42))).toBe(42);
      expect(processResult(Err("failed"))).toBe("failed");
    });
  });

  describe("type guard narrowing", () => {
    it("narrows with isOk type guard", () => {
      const result: Result<number, string> = Ok(42);

      if (isOk(result)) {
        expectTypeOf(result.value).toBeNumber();
        expect(result.value).toBe(42);

        // @ts-expect-error - error should not exist after isOk narrowing
        result.error;
      }
    });

    it("narrows with isErr type guard", () => {
      const result: Result<number, string> = Err("failed");

      if (isErr(result)) {
        expectTypeOf(result.error).toBeString();
        expect(result.error).toBe("failed");

        // @ts-expect-error - value should not exist after isErr narrowing
        result.value;
      }
    });

    it("narrows in else branch with type guards", () => {
      function processResult(result: Result<number, string>): number | string {
        if (isErr(result)) {
          expectTypeOf(result.error).toBeString();
          return result.error;
        } else {
          // Should narrow to Ok in else branch
          expectTypeOf(result.value).toBeNumber();
          return result.value;
        }
      }

      expect(processResult(Ok(42))).toBe(42);
      expect(processResult(Err("failed"))).toBe("failed");
    });
  });

  describe("exhaustive narrowing", () => {
    it("handles both branches exhaustively", () => {
      function getOutput(result: Result<number, string>): number {
        return result.ok ? result.value : result.error.length;
      }

      expect(getOutput(Ok(42))).toBe(42);
      expect(getOutput(Err("fail"))).toBe(4);
    });

    it("works with switch on discriminant", () => {
      function getOutput(result: Result<number, string>): number {
        switch (result.ok) {
          case true:
            return result.value;
          case false:
            return result.error.length;
        }
      }

      expect(getOutput(Ok(42))).toBe(42);
      expect(getOutput(Err("fail"))).toBe(4);
    });
  });

  describe("nested Result narrowing", () => {
    it("narrows nested Results", () => {
      const outer: Result<Result<number, string>, string> = Ok(Ok(42));

      if (outer.ok) {
        const inner = outer.value;
        if (inner.ok) {
          expectTypeOf(inner.value).toBeNumber();
          expect(inner.value).toBe(42);
        }
      }
    });

    it("narrows Result with complex value type", () => {
      type User = { id: number; name: string };
      const result: Result<User, string> = Ok({ id: 1, name: "Alice" });

      if (result.ok) {
        expectTypeOf(result.value.id).toBeNumber();
        expectTypeOf(result.value.name).toBeString();
        expect(result.value.name).toBe("Alice");
      }
    });
  });

  describe("unwrap functions", () => {
    it("unwrap returns correct type", () => {
      const result: Result<number, string> = Ok(42);
      const value = unwrap(result);

      expectTypeOf(value).toBeNumber();
      expect(value).toBe(42);
    });

    it("unwrap throws on Err", () => {
      const result: Result<number, string> = Err("failed");

      expect(() => unwrap(result)).toThrow("Called unwrap on Err: failed");
    });

    it("unwrapErr returns correct type", () => {
      const result: Result<number, string> = Err("failed");
      const error = unwrapErr(result);

      expectTypeOf(error).toBeString();
      expect(error).toBe("failed");
    });

    it("unwrapErr throws on Ok", () => {
      const result: Result<number, string> = Ok(42);

      expect(() => unwrapErr(result)).toThrow("Called unwrapErr on Ok");
    });
  });

  describe("array methods with narrowing", () => {
    it("filter narrows Result array", () => {
      const results: Result<number, string>[] = [
        Ok(1),
        Err("a"),
        Ok(2),
        Err("b"),
        Ok(3),
      ];

      const successes = results.filter(isOk);

      // Each item should be narrowed to Ok type
      for (const success of successes) {
        expectTypeOf(success.value).toBeNumber();
      }

      expect(successes.map((r) => r.value)).toEqual([1, 2, 3]);
    });

    it("filter narrows to Err types", () => {
      const results: Result<number, string>[] = [
        Ok(1),
        Err("a"),
        Ok(2),
        Err("b"),
      ];

      const failures = results.filter(isErr);

      for (const failure of failures) {
        expectTypeOf(failure.error).toBeString();
      }

      expect(failures.map((r) => r.error)).toEqual(["a", "b"]);
    });
  });

  describe("generic Result narrowing", () => {
    function processResult<T, E>(result: Result<T, E>): T | E {
      if (result.ok) {
        return result.value;
      } else {
        return result.error;
      }
    }

    it("works with generic functions", () => {
      const okResult: Result<number, string> = Ok(42);
      const errResult: Result<number, string> = Err("failed");

      expect(processResult(okResult)).toBe(42);
      expect(processResult(errResult)).toBe("failed");
    });
  });
});

describe("constructor type inference", () => {
  it("Ok infers value type", () => {
    const result = Ok(42);
    expectTypeOf(result).toEqualTypeOf<{ ok: true; value: number }>();
  });

  it("Err infers error type", () => {
    const result = Err("failed");
    expectTypeOf(result).toEqualTypeOf<{ ok: false; error: string }>();
  });

  it("Ok and Err are assignable to Result", () => {
    const ok: Result<number, string> = Ok(42);
    const err: Result<number, string> = Err("failed");

    expect(ok.ok).toBe(true);
    expect(err.ok).toBe(false);
  });
});
