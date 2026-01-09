/**
 * TypeScript Strict Compilation Tests
 *
 * Verifies that generated code patterns compile without errors
 * under TypeScript's strictest settings.
 */

import { describe, it, expect } from "vitest";
import * as ts from "typescript";

/**
 * Compile TypeScript code with strict mode and return diagnostics.
 * Uses the same strict settings as our tsconfig.json.
 */
function compileStrict(code: string): ts.Diagnostic[] {
  const compilerOptions: ts.CompilerOptions = {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    // Let TS use default lib for target (no lib option = ES2022 defaults)
    strict: true,
    noImplicitAny: true,
    strictNullChecks: true,
    strictFunctionTypes: true,
    strictBindCallApply: true,
    strictPropertyInitialization: true,
    noImplicitThis: true,
    useUnknownInCatchVariables: true,
    alwaysStrict: true,
    noUncheckedIndexedAccess: true,
    noImplicitReturns: true,
    noFallthroughCasesInSwitch: true,
    noUnusedLocals: false, // Allow unused in test snippets
    noUnusedParameters: false, // Allow unused in test snippets
    exactOptionalPropertyTypes: true,
    noImplicitOverride: true,
    noPropertyAccessFromIndexSignature: true,
    noEmit: true,
  };

  // Use the default compiler host but override getSourceFile for our test code
  const defaultHost = ts.createCompilerHost(compilerOptions);
  const testFileName = "/virtual/test.ts";

  const host: ts.CompilerHost = {
    ...defaultHost,
    getSourceFile: (fileName, languageVersion, onError, shouldCreate) => {
      if (fileName === testFileName) {
        return ts.createSourceFile(fileName, code, languageVersion, true);
      }
      return defaultHost.getSourceFile(
        fileName,
        languageVersion,
        onError,
        shouldCreate
      );
    },
    fileExists: (fileName) => {
      if (fileName === testFileName) return true;
      return defaultHost.fileExists(fileName);
    },
    readFile: (fileName) => {
      if (fileName === testFileName) return code;
      return defaultHost.readFile(fileName);
    },
  };

  const program = ts.createProgram([testFileName], compilerOptions, host);
  return ts.getPreEmitDiagnostics(program);
}

/**
 * Assert that code compiles without errors under strict mode.
 */
function expectCompiles(code: string): void {
  const diagnostics = compileStrict(code);
  const errors = diagnostics.filter(
    (d) => d.category === ts.DiagnosticCategory.Error
  );

  if (errors.length > 0) {
    const messages = errors.map((d) =>
      ts.flattenDiagnosticMessageText(d.messageText, "\n")
    );
    throw new Error(
      `TypeScript compilation failed:\n${messages.join("\n")}\n\nCode:\n${code}`
    );
  }
}

/**
 * Assert that code does NOT compile (has type errors).
 * Used to verify that strict mode catches expected issues.
 */
function expectDoesNotCompile(code: string): void {
  const diagnostics = compileStrict(code);
  const errors = diagnostics.filter(
    (d) => d.category === ts.DiagnosticCategory.Error
  );

  if (errors.length === 0) {
    throw new Error(
      `Expected TypeScript compilation to fail, but it succeeded.\n\nCode:\n${code}`
    );
  }
}

describe("TypeScript Strict Compilation", () => {
  describe("primitive types", () => {
    it("compiles string type", () => {
      expectCompiles(`
        type StringField = string;
        const value: StringField = "hello";
      `);
    });

    it("compiles number type", () => {
      expectCompiles(`
        type NumberField = number;
        const value: NumberField = 42;
      `);
    });

    it("compiles boolean type", () => {
      expectCompiles(`
        type BooleanField = boolean;
        const value: BooleanField = true;
      `);
    });

    it("compiles null type", () => {
      expectCompiles(`
        type NullableString = string | null;
        const value: NullableString = null;
      `);
    });
  });

  describe("struct types (interfaces)", () => {
    it("compiles simple struct", () => {
      expectCompiles(`
        interface User {
          id: number;
          name: string;
        }
        const user: User = { id: 1, name: "Alice" };
      `);
    });

    it("compiles struct with optional field", () => {
      expectCompiles(`
        interface User {
          id: number;
          name: string;
          email?: string | undefined;
        }
        const user: User = { id: 1, name: "Alice" };
        const userWithEmail: User = { id: 2, name: "Bob", email: "bob@example.com" };
      `);
    });

    it("compiles nested struct", () => {
      expectCompiles(`
        interface Address {
          street: string;
          city: string;
        }
        interface User {
          id: number;
          address: Address;
        }
        const user: User = {
          id: 1,
          address: { street: "123 Main", city: "NYC" }
        };
      `);
    });

    it("rejects struct with missing required field", () => {
      expectDoesNotCompile(`
        interface User {
          id: number;
          name: string;
        }
        const user: User = { id: 1 };
      `);
    });
  });

  describe("array types", () => {
    it("compiles array of primitives", () => {
      expectCompiles(`
        type Numbers = number[];
        const values: Numbers = [1, 2, 3];
      `);
    });

    it("compiles array of structs", () => {
      expectCompiles(`
        interface Item {
          id: number;
        }
        type Items = Item[];
        const items: Items = [{ id: 1 }, { id: 2 }];
      `);
    });

    it("compiles readonly array", () => {
      expectCompiles(`
        type Numbers = readonly number[];
        const values: Numbers = [1, 2, 3];
      `);
    });
  });

  describe("discriminated unions (enums)", () => {
    it("compiles unit variant enum", () => {
      expectCompiles(`
        type Status = "pending" | "active" | "completed";
        const status: Status = "active";
      `);
    });

    it("compiles discriminated union with data", () => {
      expectCompiles(`
        type Result<T, E> =
          | { type: "ok"; value: T }
          | { type: "err"; error: E };

        const success: Result<number, string> = { type: "ok", value: 42 };
        const failure: Result<number, string> = { type: "err", error: "failed" };
      `);
    });

    it("compiles exhaustive switch on discriminated union", () => {
      expectCompiles(`
        type Shape =
          | { kind: "circle"; radius: number }
          | { kind: "square"; side: number }
          | { kind: "rectangle"; width: number; height: number };

        function area(shape: Shape): number {
          switch (shape.kind) {
            case "circle":
              return Math.PI * shape.radius ** 2;
            case "square":
              return shape.side ** 2;
            case "rectangle":
              return shape.width * shape.height;
          }
        }
      `);
    });

    it("rejects non-exhaustive switch", () => {
      expectDoesNotCompile(`
        type Shape =
          | { kind: "circle"; radius: number }
          | { kind: "square"; side: number };

        function area(shape: Shape): number {
          switch (shape.kind) {
            case "circle":
              return Math.PI * shape.radius ** 2;
            // Missing "square" case
          }
        }
      `);
    });
  });

  describe("tuple types", () => {
    it("compiles simple tuple", () => {
      expectCompiles(`
        type Point = [number, number];
        const p: Point = [10, 20];
      `);
    });

    it("compiles named tuple elements", () => {
      expectCompiles(`
        type Point = [x: number, y: number];
        const p: Point = [10, 20];
      `);
    });

    it("compiles tuple with mixed types", () => {
      expectCompiles(`
        type Entry = [string, number, boolean];
        const e: Entry = ["key", 42, true];
      `);
    });
  });

  describe("generic types", () => {
    it("compiles generic struct", () => {
      expectCompiles(`
        interface Container<T> {
          value: T;
        }
        const numContainer: Container<number> = { value: 42 };
        const strContainer: Container<string> = { value: "hello" };
      `);
    });

    it("compiles generic with constraint", () => {
      expectCompiles(`
        interface HasId {
          id: number;
        }
        interface Repository<T extends HasId> {
          items: T[];
          getById(id: number): T | undefined;
        }

        interface User extends HasId {
          id: number;
          name: string;
        }

        const repo: Repository<User> = {
          items: [],
          getById(id: number): User | undefined {
            return this.items.find(item => item.id === id);
          }
        };
      `);
    });
  });

  describe("type guards and narrowing", () => {
    it("compiles type guard function", () => {
      expectCompiles(`
        interface Dog {
          kind: "dog";
          bark(): void;
        }
        interface Cat {
          kind: "cat";
          meow(): void;
        }
        type Pet = Dog | Cat;

        function isDog(pet: Pet): pet is Dog {
          return pet.kind === "dog";
        }

        function makeSound(pet: Pet): void {
          if (isDog(pet)) {
            pet.bark();
          } else {
            pet.meow();
          }
        }
      `);
    });

    it("compiles in operator narrowing", () => {
      expectCompiles(`
        type Result =
          | { success: true; data: string }
          | { success: false; error: string };

        function handle(result: Result): string {
          if (result.success) {
            return result.data;
          } else {
            return result.error;
          }
        }
      `);
    });
  });

  describe("mapped and conditional types", () => {
    it("compiles Partial type", () => {
      expectCompiles(`
        interface User {
          id: number;
          name: string;
        }
        type PartialUser = Partial<User>;
        const partial: PartialUser = { name: "Alice" };
      `);
    });

    it("compiles Readonly type", () => {
      expectCompiles(`
        interface User {
          id: number;
          name: string;
        }
        type ReadonlyUser = Readonly<User>;
        const user: ReadonlyUser = { id: 1, name: "Alice" };
      `);
    });

    it("compiles Pick type", () => {
      expectCompiles(`
        interface User {
          id: number;
          name: string;
          email: string;
        }
        type UserPreview = Pick<User, "id" | "name">;
        const preview: UserPreview = { id: 1, name: "Alice" };
      `);
    });
  });

  describe("literal types", () => {
    it("compiles string literal type", () => {
      expectCompiles(`
        type Direction = "north" | "south" | "east" | "west";
        const dir: Direction = "north";
      `);
    });

    it("compiles numeric literal type", () => {
      expectCompiles(`
        type DiceRoll = 1 | 2 | 3 | 4 | 5 | 6;
        const roll: DiceRoll = 4;
      `);
    });

    it("compiles const assertion", () => {
      expectCompiles(`
        const config = {
          endpoint: "/api",
          timeout: 5000,
        } as const;

        type Endpoint = typeof config.endpoint;
      `);
    });
  });

  describe("strict null checks", () => {
    it("requires null check before access", () => {
      expectDoesNotCompile(`
        function getLength(s: string | null): number {
          return s.length;
        }
      `);
    });

    it("compiles with proper null check", () => {
      expectCompiles(`
        function getLength(s: string | null): number {
          if (s === null) {
            return 0;
          }
          return s.length;
        }
      `);
    });

    it("compiles with nullish coalescing", () => {
      expectCompiles(`
        function getLength(s: string | null): number {
          return (s ?? "").length;
        }
      `);
    });
  });
});
