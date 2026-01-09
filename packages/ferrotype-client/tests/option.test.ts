/**
 * Tests for the Option type and utilities.
 */

import { describe, it, expect } from "vitest";
import {
  some,
  none,
  isSome,
  isNone,
  optionToNullable,
  nullableToOption,
  unwrapOption,
  unwrapOptionOr,
  unwrapOptionOrElse,
  mapOption,
  flatMapOption,
  matchOption,
  filterOption,
  optionToResult,
  resultToOption,
  ok,
  err,
  isOk,
  isErr,
  type Option,
  type Some,
  type None,
} from "../src/index.js";

describe("Option", () => {
  describe("constructors", () => {
    it("some creates a Some variant", () => {
      const opt = some(42);
      expect(opt).toEqual({ Some: 42 });
    });

    it("some preserves complex types", () => {
      const value = { id: 1, name: "test" };
      const opt = some(value);
      expect(opt.Some).toEqual(value);
    });

    it("none creates a None variant", () => {
      const opt = none();
      expect(opt).toEqual({ None: null });
    });
  });

  describe("type guards", () => {
    it("isSome returns true for Some", () => {
      const opt: Option<number> = some(42);
      expect(isSome(opt)).toBe(true);
    });

    it("isSome returns false for None", () => {
      const opt: Option<number> = none();
      expect(isSome(opt)).toBe(false);
    });

    it("isNone returns true for None", () => {
      const opt: Option<number> = none();
      expect(isNone(opt)).toBe(true);
    });

    it("isNone returns false for Some", () => {
      const opt: Option<number> = some(42);
      expect(isNone(opt)).toBe(false);
    });

    it("type narrows correctly after isSome", () => {
      const opt: Option<number> = some(42);
      if (isSome(opt)) {
        // TypeScript should know opt.Some exists here
        const value: number = opt.Some;
        expect(value).toBe(42);
      }
    });

    it("type narrows correctly after isNone", () => {
      const opt: Option<number> = none();
      if (isNone(opt)) {
        // TypeScript should know opt.None exists here
        const nullValue: null = opt.None;
        expect(nullValue).toBeNull();
      }
    });
  });

  describe("optionToNullable", () => {
    it("returns value for Some", () => {
      const opt = some("hello");
      expect(optionToNullable(opt)).toBe("hello");
    });

    it("returns null for None", () => {
      const opt = none<string>();
      expect(optionToNullable(opt)).toBeNull();
    });

    it("preserves complex types", () => {
      const value = { id: 1, name: "test" };
      const opt = some(value);
      expect(optionToNullable(opt)).toEqual(value);
    });

    it("works with number 0", () => {
      const opt = some(0);
      expect(optionToNullable(opt)).toBe(0);
    });

    it("works with empty string", () => {
      const opt = some("");
      expect(optionToNullable(opt)).toBe("");
    });

    it("works with false", () => {
      const opt = some(false);
      expect(optionToNullable(opt)).toBe(false);
    });
  });

  describe("nullableToOption", () => {
    it("returns Some for non-null value", () => {
      const opt = nullableToOption("hello");
      expect(isSome(opt)).toBe(true);
      expect(optionToNullable(opt)).toBe("hello");
    });

    it("returns None for null", () => {
      const opt = nullableToOption<string>(null);
      expect(isNone(opt)).toBe(true);
    });

    it("returns None for undefined", () => {
      const opt = nullableToOption<string>(undefined);
      expect(isNone(opt)).toBe(true);
    });

    it("preserves number 0", () => {
      const opt = nullableToOption(0);
      expect(isSome(opt)).toBe(true);
      expect(optionToNullable(opt)).toBe(0);
    });

    it("preserves empty string", () => {
      const opt = nullableToOption("");
      expect(isSome(opt)).toBe(true);
      expect(optionToNullable(opt)).toBe("");
    });

    it("preserves false", () => {
      const opt = nullableToOption(false);
      expect(isSome(opt)).toBe(true);
      expect(optionToNullable(opt)).toBe(false);
    });
  });

  describe("unwrapOption", () => {
    it("returns value for Some", () => {
      const opt = some(42);
      expect(unwrapOption(opt)).toBe(42);
    });

    it("throws for None", () => {
      const opt = none<number>();
      expect(() => unwrapOption(opt)).toThrow("Called unwrapOption on a None value");
    });
  });

  describe("unwrapOptionOr", () => {
    it("returns value for Some", () => {
      const opt: Option<number> = some(42);
      expect(unwrapOptionOr(opt, 0)).toBe(42);
    });

    it("returns default for None", () => {
      const opt: Option<number> = none();
      expect(unwrapOptionOr(opt, 0)).toBe(0);
    });
  });

  describe("unwrapOptionOrElse", () => {
    it("returns value for Some without calling function", () => {
      const opt: Option<number> = some(42);
      let called = false;
      const result = unwrapOptionOrElse(opt, () => {
        called = true;
        return 0;
      });
      expect(result).toBe(42);
      expect(called).toBe(false);
    });

    it("calls function and returns result for None", () => {
      const opt: Option<number> = none();
      const result = unwrapOptionOrElse(opt, () => 100);
      expect(result).toBe(100);
    });
  });

  describe("mapOption", () => {
    it("transforms Some value", () => {
      const opt: Option<number> = some(5);
      const mapped = mapOption(opt, (x) => x * 2);
      expect(isSome(mapped)).toBe(true);
      expect(unwrapOption(mapped)).toBe(10);
    });

    it("preserves None", () => {
      const opt: Option<number> = none();
      let called = false;
      const mapped = mapOption(opt, (x) => {
        called = true;
        return x * 2;
      });
      expect(isNone(mapped)).toBe(true);
      expect(called).toBe(false);
    });

    it("can change type", () => {
      const opt: Option<number> = some(42);
      const mapped = mapOption(opt, (x) => x.toString());
      expect(isSome(mapped)).toBe(true);
      if (isSome(mapped)) {
        const str: string = mapped.Some;
        expect(str).toBe("42");
      }
    });
  });

  describe("flatMapOption", () => {
    it("chains Some values", () => {
      const opt: Option<string> = some("42");
      const parsed = flatMapOption(opt, (s) => {
        const n = parseInt(s, 10);
        return isNaN(n) ? none<number>() : some(n);
      });
      expect(isSome(parsed)).toBe(true);
      expect(unwrapOption(parsed)).toBe(42);
    });

    it("short-circuits on None input", () => {
      const opt: Option<string> = none();
      let called = false;
      const result = flatMapOption(opt, (s) => {
        called = true;
        return some(s.length);
      });
      expect(isNone(result)).toBe(true);
      expect(called).toBe(false);
    });

    it("propagates None from function", () => {
      const opt: Option<string> = some("not a number");
      const parsed = flatMapOption(opt, (s) => {
        const n = parseInt(s, 10);
        return isNaN(n) ? none<number>() : some(n);
      });
      expect(isNone(parsed)).toBe(true);
    });
  });

  describe("matchOption", () => {
    it("calls some handler for Some", () => {
      const opt: Option<number> = some(42);
      const result = matchOption(opt, {
        some: (n) => `value: ${n}`,
        none: () => "nothing",
      });
      expect(result).toBe("value: 42");
    });

    it("calls none handler for None", () => {
      const opt: Option<number> = none();
      const result = matchOption(opt, {
        some: (n) => `value: ${n}`,
        none: () => "nothing",
      });
      expect(result).toBe("nothing");
    });

    it("enables exhaustive handling", () => {
      const opt: Option<number> = some(42);
      const num: number = matchOption(opt, {
        some: (n) => n,
        none: () => -1,
      });
      expect(num).toBe(42);
    });
  });

  describe("filterOption", () => {
    it("preserves Some when predicate is true", () => {
      const opt: Option<number> = some(5);
      const filtered = filterOption(opt, (n) => n > 3);
      expect(isSome(filtered)).toBe(true);
      expect(unwrapOption(filtered)).toBe(5);
    });

    it("returns None when predicate is false", () => {
      const opt: Option<number> = some(5);
      const filtered = filterOption(opt, (n) => n > 10);
      expect(isNone(filtered)).toBe(true);
    });

    it("preserves None", () => {
      const opt: Option<number> = none();
      let called = false;
      const filtered = filterOption(opt, () => {
        called = true;
        return true;
      });
      expect(isNone(filtered)).toBe(true);
      expect(called).toBe(false);
    });
  });

  describe("optionToResult", () => {
    it("converts Some to Ok", () => {
      const opt: Option<number> = some(42);
      const result = optionToResult(opt, "no value");
      expect(isOk(result)).toBe(true);
      if (isOk(result)) {
        expect(result.value).toBe(42);
      }
    });

    it("converts None to Err with provided error", () => {
      const opt: Option<number> = none();
      const result = optionToResult(opt, "no value");
      expect(isErr(result)).toBe(true);
      if (isErr(result)) {
        expect(result.error).toBe("no value");
      }
    });
  });

  describe("resultToOption", () => {
    it("converts Ok to Some", () => {
      const result = ok(42);
      const opt = resultToOption(result);
      expect(isSome(opt)).toBe(true);
      expect(unwrapOption(opt)).toBe(42);
    });

    it("converts Err to None", () => {
      const result = err("error");
      const opt = resultToOption(result);
      expect(isNone(opt)).toBe(true);
    });
  });

  describe("roundtrip with JSON", () => {
    it("Some survives JSON roundtrip", () => {
      const opt = some({ id: 1, name: "test" });
      const json = JSON.stringify(opt);
      const parsed = JSON.parse(json) as Option<{ id: number; name: string }>;
      expect(isSome(parsed)).toBe(true);
      expect(optionToNullable(parsed)).toEqual({ id: 1, name: "test" });
    });

    it("None survives JSON roundtrip", () => {
      const opt = none<string>();
      const json = JSON.stringify(opt);
      const parsed = JSON.parse(json) as Option<string>;
      expect(isNone(parsed)).toBe(true);
    });

    it("matches Rust serde externally-tagged format", () => {
      // Simulate receiving JSON from Rust: Option::Some("hello")
      const rustSomeJson = '{"Some":"hello"}';
      const someOpt = JSON.parse(rustSomeJson) as Option<string>;
      expect(isSome(someOpt)).toBe(true);
      expect(optionToNullable(someOpt)).toBe("hello");

      // Simulate receiving JSON from Rust: Option::None
      const rustNoneJson = '{"None":null}';
      const noneOpt = JSON.parse(rustNoneJson) as Option<string>;
      expect(isNone(noneOpt)).toBe(true);
      expect(optionToNullable(noneOpt)).toBeNull();
    });
  });

  describe("chaining operations", () => {
    it("chains multiple transformations", () => {
      const opt: Option<string> = some("  42  ");

      const result = flatMapOption(
        mapOption(opt, (s) => s.trim()),
        (s) => {
          const n = parseInt(s, 10);
          return isNaN(n) ? none<number>() : some(n);
        }
      );

      expect(isSome(result)).toBe(true);
      expect(unwrapOption(result)).toBe(42);
    });

    it("short-circuits chain on first None", () => {
      const opt: Option<string> = none();

      let mapCalled = false;
      let flatMapCalled = false;

      const result = flatMapOption(
        mapOption(opt, (s) => {
          mapCalled = true;
          return s.trim();
        }),
        (s) => {
          flatMapCalled = true;
          return some(s.length);
        }
      );

      expect(isNone(result)).toBe(true);
      expect(mapCalled).toBe(false);
      expect(flatMapCalled).toBe(false);
    });
  });
});
