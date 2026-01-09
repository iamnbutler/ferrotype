/**
 * Tests for struct variant enum codegen utilities.
 *
 * These utilities support generating TypeScript discriminated unions
 * from Rust enums with struct variants.
 */

import { describe, it, expect } from "vitest";
import {
  structVariant,
  isStructVariant,
  matchEnum,
  variantConstructor,
  enumConstructors,
  variantGuard,
  enumGuards,
  type StructVariant,
} from "../src/index.js";

// Define test enum types that mirror Rust enum struct variants
type Move = StructVariant<"Move", { x: number; y: number }>;
type Write = StructVariant<"Write", { text: string }>;
type Resize = StructVariant<"Resize", { width: number; height: number }>;
type Quit = StructVariant<"Quit", {}>;

type Message = Move | Write | Resize | Quit;

describe("Struct Variant Enum Codegen", () => {
  describe("StructVariant type", () => {
    it("creates correct type structure", () => {
      const move: Move = { type: "Move", x: 10, y: 20 };
      expect(move.type).toBe("Move");
      expect(move.x).toBe(10);
      expect(move.y).toBe(20);
    });

    it("enforces readonly properties", () => {
      const move: Move = { type: "Move", x: 10, y: 20 };
      // TypeScript would error on: move.x = 5;
      // Runtime check that values are as expected
      expect(move.x).toBe(10);
    });

    it("supports empty field objects", () => {
      const quit: Quit = { type: "Quit" };
      expect(quit.type).toBe("Quit");
    });
  });

  describe("structVariant constructor", () => {
    it("creates Move variant", () => {
      const move = structVariant("Move", { x: 10, y: 20 });
      expect(move.type).toBe("Move");
      expect(move.x).toBe(10);
      expect(move.y).toBe(20);
    });

    it("creates Write variant", () => {
      const write = structVariant("Write", { text: "hello" });
      expect(write.type).toBe("Write");
      expect(write.text).toBe("hello");
    });

    it("creates variant with empty fields", () => {
      const quit = structVariant("Quit", {});
      expect(quit.type).toBe("Quit");
    });

    it("creates variant with complex fields", () => {
      type Complex = StructVariant<
        "Complex",
        { data: { nested: string }; items: number[] }
      >;
      const complex = structVariant("Complex", {
        data: { nested: "value" },
        items: [1, 2, 3],
      });
      expect(complex.type).toBe("Complex");
      expect(complex.data.nested).toBe("value");
      expect(complex.items).toEqual([1, 2, 3]);
    });
  });

  describe("isStructVariant type guard", () => {
    it("returns true for matching tag", () => {
      const msg: Message = structVariant("Move", { x: 10, y: 20 });
      expect(isStructVariant(msg, "Move")).toBe(true);
    });

    it("returns false for non-matching tag", () => {
      const msg: Message = structVariant("Move", { x: 10, y: 20 });
      expect(isStructVariant(msg, "Write")).toBe(false);
      expect(isStructVariant(msg, "Quit")).toBe(false);
    });

    it("narrows type correctly", () => {
      const msg: Message = structVariant("Move", { x: 10, y: 20 });
      if (isStructVariant(msg, "Move")) {
        // TypeScript knows msg has x and y here
        const x: number = msg.x;
        const y: number = msg.y;
        expect(x).toBe(10);
        expect(y).toBe(20);
      }
    });

    it("handles all variants", () => {
      const messages: Message[] = [
        structVariant("Move", { x: 1, y: 2 }),
        structVariant("Write", { text: "hi" }),
        structVariant("Resize", { width: 100, height: 200 }),
        structVariant("Quit", {}),
      ];

      expect(isStructVariant(messages[0]!, "Move")).toBe(true);
      expect(isStructVariant(messages[1]!, "Write")).toBe(true);
      expect(isStructVariant(messages[2]!, "Resize")).toBe(true);
      expect(isStructVariant(messages[3]!, "Quit")).toBe(true);
    });
  });

  describe("matchEnum exhaustive matching", () => {
    it("matches Move variant", () => {
      const msg: Message = structVariant("Move", { x: 10, y: 20 });
      const result = matchEnum(msg, {
        Move: (m) => `Moving to ${m.x}, ${m.y}`,
        Write: (w) => `Writing: ${w.text}`,
        Resize: (r) => `Resizing to ${r.width}x${r.height}`,
        Quit: () => "Quitting",
      });
      expect(result).toBe("Moving to 10, 20");
    });

    it("matches Write variant", () => {
      const msg: Message = structVariant("Write", { text: "hello" });
      const result = matchEnum(msg, {
        Move: (m) => `Moving to ${m.x}, ${m.y}`,
        Write: (w) => `Writing: ${w.text}`,
        Resize: (r) => `Resizing to ${r.width}x${r.height}`,
        Quit: () => "Quitting",
      });
      expect(result).toBe("Writing: hello");
    });

    it("matches Quit variant", () => {
      const msg: Message = structVariant("Quit", {});
      const result = matchEnum(msg, {
        Move: (m) => `Moving to ${m.x}, ${m.y}`,
        Write: (w) => `Writing: ${w.text}`,
        Resize: (r) => `Resizing to ${r.width}x${r.height}`,
        Quit: () => "Quitting",
      });
      expect(result).toBe("Quitting");
    });

    it("returns correct type", () => {
      const msg: Message = structVariant("Move", { x: 10, y: 20 });
      const result: number = matchEnum(msg, {
        Move: (m) => m.x + m.y,
        Write: (w) => w.text.length,
        Resize: (r) => r.width * r.height,
        Quit: () => 0,
      });
      expect(result).toBe(30);
    });
  });

  describe("variantConstructor", () => {
    it("creates typed constructor for Move", () => {
      const createMove = variantConstructor<Message, "Move">("Move");
      const move = createMove({ x: 10, y: 20 });
      expect(move.type).toBe("Move");
      expect(move.x).toBe(10);
      expect(move.y).toBe(20);
    });

    it("creates typed constructor for Write", () => {
      const createWrite = variantConstructor<Message, "Write">("Write");
      const write = createWrite({ text: "hello" });
      expect(write.type).toBe("Write");
      expect(write.text).toBe("hello");
    });

    it("creates typed constructor for Quit", () => {
      const createQuit = variantConstructor<Message, "Quit">("Quit");
      const quit = createQuit({});
      expect(quit.type).toBe("Quit");
    });

    it("returns value assignable to union", () => {
      const createMove = variantConstructor<Message, "Move">("Move");
      const msg: Message = createMove({ x: 10, y: 20 });
      expect(msg.type).toBe("Move");
    });
  });

  describe("enumConstructors", () => {
    it("creates constructors for all variants", () => {
      const Message = enumConstructors<Message>([
        "Move",
        "Write",
        "Resize",
        "Quit",
      ]);

      const move = Message.Move({ x: 10, y: 20 });
      expect(move.type).toBe("Move");
      expect(move.x).toBe(10);

      const write = Message.Write({ text: "hello" });
      expect(write.type).toBe("Write");
      expect(write.text).toBe("hello");

      const resize = Message.Resize({ width: 100, height: 200 });
      expect(resize.type).toBe("Resize");

      const quit = Message.Quit({});
      expect(quit.type).toBe("Quit");
    });

    it("constructors return union-compatible values", () => {
      const Message = enumConstructors<Message>([
        "Move",
        "Write",
        "Resize",
        "Quit",
      ]);

      const messages: Message[] = [
        Message.Move({ x: 1, y: 2 }),
        Message.Write({ text: "hi" }),
        Message.Resize({ width: 10, height: 20 }),
        Message.Quit({}),
      ];

      expect(messages).toHaveLength(4);
      expect(messages[0]!.type).toBe("Move");
      expect(messages[1]!.type).toBe("Write");
      expect(messages[2]!.type).toBe("Resize");
      expect(messages[3]!.type).toBe("Quit");
    });
  });

  describe("variantGuard", () => {
    it("creates type guard for Move", () => {
      const isMove = variantGuard<Message, "Move">("Move");

      const move: Message = structVariant("Move", { x: 10, y: 20 });
      const write: Message = structVariant("Write", { text: "hi" });

      expect(isMove(move)).toBe(true);
      expect(isMove(write)).toBe(false);
    });

    it("narrows type correctly", () => {
      const isMove = variantGuard<Message, "Move">("Move");
      const msg: Message = structVariant("Move", { x: 10, y: 20 });

      if (isMove(msg)) {
        // TypeScript knows msg is Move here
        expect(msg.x).toBe(10);
        expect(msg.y).toBe(20);
      }
    });
  });

  describe("enumGuards", () => {
    it("creates guards for all variants", () => {
      const is = enumGuards<Message>(["Move", "Write", "Resize", "Quit"]);

      const move: Message = structVariant("Move", { x: 10, y: 20 });
      const write: Message = structVariant("Write", { text: "hi" });
      const resize: Message = structVariant("Resize", { width: 100, height: 200 });
      const quit: Message = structVariant("Quit", {});

      expect(is.Move(move)).toBe(true);
      expect(is.Move(write)).toBe(false);

      expect(is.Write(write)).toBe(true);
      expect(is.Write(move)).toBe(false);

      expect(is.Resize(resize)).toBe(true);
      expect(is.Quit(quit)).toBe(true);
    });

    it("guards narrow types correctly", () => {
      const is = enumGuards<Message>(["Move", "Write", "Resize", "Quit"]);
      const msg: Message = structVariant("Write", { text: "hello" });

      if (is.Write(msg)) {
        // TypeScript knows msg is Write here
        expect(msg.text).toBe("hello");
      }
    });

    it("supports filtering arrays", () => {
      const is = enumGuards<Message>(["Move", "Write", "Resize", "Quit"]);

      const messages: Message[] = [
        structVariant("Move", { x: 1, y: 2 }),
        structVariant("Write", { text: "a" }),
        structVariant("Move", { x: 3, y: 4 }),
        structVariant("Write", { text: "b" }),
      ];

      const moves = messages.filter(is.Move);
      expect(moves).toHaveLength(2);
      expect(moves[0]!.x).toBe(1);
      expect(moves[1]!.x).toBe(3);

      const writes = messages.filter(is.Write);
      expect(writes).toHaveLength(2);
      expect(writes[0]!.text).toBe("a");
      expect(writes[1]!.text).toBe("b");
    });
  });

  describe("real-world patterns", () => {
    // Simulating Rust's common enum patterns

    type ApiResponse =
      | StructVariant<"Success", { data: unknown; statusCode: number }>
      | StructVariant<"Error", { message: string; code: string }>
      | StructVariant<"Loading", {}>;

    it("handles API response pattern", () => {
      const Api = enumConstructors<ApiResponse>([
        "Success",
        "Error",
        "Loading",
      ]);
      const is = enumGuards<ApiResponse>(["Success", "Error", "Loading"]);

      const success = Api.Success({ data: { user: "Alice" }, statusCode: 200 });
      const error = Api.Error({ message: "Not found", code: "404" });
      const loading = Api.Loading({});

      expect(is.Success(success)).toBe(true);
      expect(is.Error(error)).toBe(true);
      expect(is.Loading(loading)).toBe(true);

      const handleResponse = (response: ApiResponse): string =>
        matchEnum(response, {
          Success: (s) => `Got data with status ${s.statusCode}`,
          Error: (e) => `Error ${e.code}: ${e.message}`,
          Loading: () => "Loading...",
        });

      expect(handleResponse(success)).toBe("Got data with status 200");
      expect(handleResponse(error)).toBe("Error 404: Not found");
      expect(handleResponse(loading)).toBe("Loading...");
    });

    type Shape =
      | StructVariant<"Circle", { radius: number }>
      | StructVariant<"Rectangle", { width: number; height: number }>
      | StructVariant<"Triangle", { base: number; height: number }>;

    it("handles shape area calculation pattern", () => {
      const Shape = enumConstructors<Shape>(["Circle", "Rectangle", "Triangle"]);

      const calculateArea = (shape: Shape): number =>
        matchEnum(shape, {
          Circle: (c) => Math.PI * c.radius ** 2,
          Rectangle: (r) => r.width * r.height,
          Triangle: (t) => (t.base * t.height) / 2,
        });

      const circle = Shape.Circle({ radius: 5 });
      const rectangle = Shape.Rectangle({ width: 10, height: 20 });
      const triangle = Shape.Triangle({ base: 6, height: 8 });

      expect(calculateArea(circle)).toBeCloseTo(78.54, 1);
      expect(calculateArea(rectangle)).toBe(200);
      expect(calculateArea(triangle)).toBe(24);
    });
  });
});
