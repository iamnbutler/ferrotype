import { describe, it, expect } from "vitest";
import {
  HelloResponse,
  isHelloResponse,
  parseHelloResponse,
  HelloClient,
} from "../src/index.js";

describe("HelloResponse", () => {
  describe("isHelloResponse", () => {
    it("returns true for valid HelloResponse", () => {
      expect(isHelloResponse({ message: "Hello" })).toBe(true);
      expect(isHelloResponse({ message: "" })).toBe(true);
      expect(isHelloResponse({ message: "Hello, World!" })).toBe(true);
    });

    it("returns false for invalid values", () => {
      expect(isHelloResponse(null)).toBe(false);
      expect(isHelloResponse(undefined)).toBe(false);
      expect(isHelloResponse(42)).toBe(false);
      expect(isHelloResponse("string")).toBe(false);
      expect(isHelloResponse({})).toBe(false);
      expect(isHelloResponse({ message: 42 })).toBe(false);
      expect(isHelloResponse({ msg: "wrong field" })).toBe(false);
    });
  });

  describe("parseHelloResponse", () => {
    it("parses valid JSON", () => {
      const json = '{"message":"Hello, World!"}';
      const response = parseHelloResponse(json);
      expect(response.message).toBe("Hello, World!");
    });

    it("parses JSON with extra fields (structural typing)", () => {
      const json = '{"message":"Hello","extra":"ignored"}';
      const response = parseHelloResponse(json);
      expect(response.message).toBe("Hello");
    });

    it("throws for invalid JSON structure", () => {
      expect(() => parseHelloResponse("{}")).toThrow("Invalid HelloResponse");
      expect(() => parseHelloResponse('{"msg":"wrong"}')).toThrow(
        "Invalid HelloResponse"
      );
      expect(() => parseHelloResponse('{"message":42}')).toThrow(
        "Invalid HelloResponse"
      );
    });

    it("throws for invalid JSON syntax", () => {
      expect(() => parseHelloResponse("not json")).toThrow();
    });
  });
});

describe("HelloClient", () => {
  it("returns greeting from simulated server", async () => {
    const client = new HelloClient();
    const response = await client.hello();
    expect(response.message).toBe("Hello, World!");
  });
});

describe("Rust-TypeScript roundtrip", () => {
  it("TypeScript can parse JSON produced by Rust server", () => {
    // This is the exact JSON output from the Rust hello() function:
    // HelloResponse { message: "Hello, World!".to_string() }
    // serialized via serde_json::to_string()
    const rustServerOutput = '{"message":"Hello, World!"}';

    const response = parseHelloResponse(rustServerOutput);
    expect(response.message).toBe("Hello, World!");
  });

  it("type structure matches Rust definition", () => {
    // Verify the TypeScript type matches what Rust generates:
    // Rust: { message: string }
    // TypeScript: { message: string }
    const response: HelloResponse = { message: "test" };

    // TypeScript compiler enforces this at compile time
    // Runtime check confirms the structure
    expect(isHelloResponse(response)).toBe(true);
  });
});
