/**
 * Hello World Client - TypeScript types matching the Rust server
 *
 * This demonstrates ferrotype's type mapping with real HTTP:
 * - Rust `HelloResponse` struct maps to TypeScript interface
 * - Real fetch calls to the Axum server
 * - JSON is the interchange format
 */

/**
 * Response from the hello RPC method.
 * Maps to Rust: `pub struct HelloResponse { pub message: String }`
 * TypeScript type: `{ message: string }`
 */
export interface HelloResponse {
  readonly message: string;
}

/**
 * Type guard for HelloResponse.
 * Validates that a parsed JSON value conforms to the expected type.
 */
export function isHelloResponse(value: unknown): value is HelloResponse {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const obj = value as Record<string, unknown>;
  return typeof obj.message === "string";
}

/**
 * Parse a JSON string into a HelloResponse.
 * Throws if the JSON doesn't match the expected shape.
 */
export function parseHelloResponse(json: string): HelloResponse {
  const parsed: unknown = JSON.parse(json);
  if (!isHelloResponse(parsed)) {
    throw new Error("Invalid HelloResponse: expected { message: string }");
  }
  return parsed;
}

/**
 * RPC client for the hello method.
 * Makes real HTTP requests to the Rust Axum server.
 */
export class HelloClient {
  private readonly baseUrl: string;

  /**
   * Create a new HelloClient.
   * @param baseUrl The base URL of the Rust server (e.g., "http://localhost:3000")
   */
  constructor(baseUrl: string = "http://localhost:3000") {
    this.baseUrl = baseUrl;
  }

  /**
   * Call the hello RPC method.
   * Makes a real HTTP GET request to /rpc/hello.
   * Returns a greeting message from the server.
   */
  async hello(): Promise<HelloResponse> {
    const response = await fetch(`${this.baseUrl}/rpc/hello`);

    if (!response.ok) {
      throw new Error(`HTTP error: ${response.status} ${response.statusText}`);
    }

    const json = await response.text();
    return parseHelloResponse(json);
  }
}
