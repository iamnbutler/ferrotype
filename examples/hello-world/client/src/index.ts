/**
 * Hello World Client - TypeScript types matching the Rust server
 *
 * This demonstrates ferrotype's type mapping:
 * - Rust `HelloResponse` struct maps to TypeScript interface
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
 * Simulated RPC client for the hello method.
 * In a real implementation, this would make an HTTP/WebSocket call to the Rust server.
 */
export class HelloClient {
  /**
   * Call the hello RPC method.
   * Returns a greeting message from the server.
   */
  async hello(): Promise<HelloResponse> {
    // In a real client, this would be:
    // const response = await fetch('/rpc/hello');
    // const json = await response.text();
    // return parseHelloResponse(json);

    // For this example, we simulate the server response
    const serverResponse = '{"message":"Hello, World!"}';
    return parseHelloResponse(serverResponse);
  }
}
