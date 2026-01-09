import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { spawn, ChildProcess } from "child_process";
import { setTimeout } from "timers/promises";
import {
  HelloResponse,
  isHelloResponse,
  parseHelloResponse,
  HelloClient,
} from "../src/index.js";

// Unit tests for type guards and parsing (no server needed)
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

// Integration tests with real Rust server
describe("HelloClient (real HTTP)", () => {
  let serverProcess: ChildProcess | null = null;
  const TEST_PORT = 13000 + Math.floor(Math.random() * 1000);
  const BASE_URL = `http://localhost:${TEST_PORT}`;

  beforeAll(async () => {
    // Start the Rust server
    serverProcess = spawn("cargo", ["run", "-p", "hello-world"], {
      env: { ...process.env, PORT: String(TEST_PORT) },
      cwd: "../../..",
      stdio: ["ignore", "pipe", "pipe"],
    });

    // Wait for server to be ready by polling the endpoint
    const maxAttempts = 50;
    for (let i = 0; i < maxAttempts; i++) {
      try {
        const response = await fetch(`${BASE_URL}/rpc/hello`);
        if (response.ok) {
          return; // Server is ready
        }
      } catch {
        // Server not ready yet, wait and retry
      }
      await setTimeout(100);
    }
    throw new Error(`Server failed to start on port ${TEST_PORT}`);
  }, 30000); // 30 second timeout for server startup

  afterAll(() => {
    if (serverProcess) {
      serverProcess.kill("SIGTERM");
      serverProcess = null;
    }
  });

  it("makes real HTTP request to Rust server", async () => {
    const client = new HelloClient(BASE_URL);
    const response = await client.hello();
    expect(response.message).toBe("Hello, World!");
  });

  it("receives correct content-type header", async () => {
    const response = await fetch(`${BASE_URL}/rpc/hello`);
    expect(response.headers.get("content-type")).toContain("application/json");
  });

  it("handles CORS headers", async () => {
    const response = await fetch(`${BASE_URL}/rpc/hello`, {
      headers: { Origin: "http://localhost:5173" },
    });
    expect(response.headers.get("access-control-allow-origin")).toBe("*");
  });
});

// Type compatibility tests (no server needed)
describe("Rust-TypeScript type compatibility", () => {
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
