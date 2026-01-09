/**
 * Tests for the RpcClient base class.
 */

import { describe, it, expect, vi } from "vitest";
import {
  RpcClient,
  RpcClientConfig,
  RpcClientError,
  RpcTransport,
  ok,
  err,
  isOk,
  isErr,
  networkError,
  timeoutError,
  parseError,
  httpError,
  isNetworkError,
  isTimeoutError,
  isParseError,
  isHttpError,
  isServerError,
  type Result,
} from "../src/index.js";

/**
 * Mock transport for testing without actual network calls.
 */
class MockTransport implements RpcTransport {
  private response: Result<string, RpcClientError> = ok('{"ok":true,"value":null}');
  private lastRequest: { url: string; body: string; headers: Record<string, string>; timeout: number } | null = null;

  setResponse(response: Result<string, RpcClientError>): void {
    this.response = response;
  }

  setSuccessResponse<T>(value: T): void {
    this.response = ok(JSON.stringify({ ok: true, value }));
  }

  setErrorResponse<E>(error: E): void {
    this.response = ok(JSON.stringify({ ok: false, error }));
  }

  getLastRequest() {
    return this.lastRequest;
  }

  async send(
    url: string,
    body: string,
    headers: Record<string, string>,
    timeout: number
  ): Promise<Result<string, RpcClientError>> {
    this.lastRequest = { url, body, headers, timeout };
    return this.response;
  }
}

describe("RpcClient", () => {
  describe("construction", () => {
    it("creates client with required config", () => {
      const transport = new MockTransport();
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);
      const config = client.getConfig();
      expect(config.baseUrl).toBe("http://localhost:8080/rpc");
      expect(config.headers).toEqual({});
      expect(config.timeout).toBe(30000);
    });

    it("creates client with full config", () => {
      const transport = new MockTransport();
      const client = new RpcClient(
        {
          baseUrl: "http://localhost:8080/rpc",
          headers: { Authorization: "Bearer token" },
          timeout: 5000,
        },
        transport
      );
      const config = client.getConfig();
      expect(config.baseUrl).toBe("http://localhost:8080/rpc");
      expect(config.headers).toEqual({ Authorization: "Bearer token" });
      expect(config.timeout).toBe(5000);
    });
  });

  describe("call", () => {
    it("sends correct request format", async () => {
      const transport = new MockTransport();
      transport.setSuccessResponse({ id: 1, name: "Test" });
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

      await client.call("get_user", { user_id: 123 });

      const lastRequest = transport.getLastRequest();
      expect(lastRequest).not.toBeNull();
      expect(lastRequest!.url).toBe("http://localhost:8080/rpc");
      expect(JSON.parse(lastRequest!.body)).toEqual({
        method: "get_user",
        params: { user_id: 123 },
      });
      expect(lastRequest!.headers).toEqual({});
      expect(lastRequest!.timeout).toBe(30000);
    });

    it("includes custom headers", async () => {
      const transport = new MockTransport();
      transport.setSuccessResponse(null);
      const client = new RpcClient(
        {
          baseUrl: "http://localhost:8080/rpc",
          headers: { Authorization: "Bearer token123" },
        },
        transport
      );

      await client.call("ping", {});

      const lastRequest = transport.getLastRequest();
      expect(lastRequest!.headers).toEqual({ Authorization: "Bearer token123" });
    });

    it("returns Ok result on success", async () => {
      const transport = new MockTransport();
      transport.setSuccessResponse({ id: 1, name: "Alice" });
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

      const result = await client.call<{ id: number; name: string }>("get_user", { user_id: 1 });

      expect(isOk(result)).toBe(true);
      if (isOk(result)) {
        expect(result.value).toEqual({ id: 1, name: "Alice" });
      }
    });

    it("returns server error on server-side failure", async () => {
      const transport = new MockTransport();
      transport.setErrorResponse({ code: "NOT_FOUND", message: "User not found" });
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

      const result = await client.call<unknown, { code: string; message: string }>("get_user", { user_id: 999 });

      expect(isErr(result)).toBe(true);
      if (isErr(result)) {
        expect(result.error).toEqual({ code: "NOT_FOUND", message: "User not found" });
      }
    });

    it("returns network error on transport failure", async () => {
      const transport = new MockTransport();
      transport.setResponse(err(networkError("Connection refused")));
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

      const result = await client.call("ping", {});

      expect(isErr(result)).toBe(true);
      if (isErr(result) && isNetworkError(result.error as RpcClientError)) {
        expect((result.error as { type: "NetworkError"; message: string }).message).toBe("Connection refused");
      }
    });

    it("returns timeout error on timeout", async () => {
      const transport = new MockTransport();
      transport.setResponse(err(timeoutError("Request timed out after 5000ms")));
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc", timeout: 5000 }, transport);

      const result = await client.call("slow_method", {});

      expect(isErr(result)).toBe(true);
      if (isErr(result) && isTimeoutError(result.error as RpcClientError)) {
        expect((result.error as { type: "TimeoutError"; message: string }).message).toContain("timed out");
      }
    });

    it("returns parse error on invalid JSON response", async () => {
      const transport = new MockTransport();
      transport.setResponse(ok("not valid json"));
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

      const result = await client.call("broken_endpoint", {});

      expect(isErr(result)).toBe(true);
      if (isErr(result) && isParseError(result.error as RpcClientError)) {
        const parseErr = result.error as { type: "ParseError"; message: string; body: string };
        expect(parseErr.body).toBe("not valid json");
      }
    });

    it("returns HTTP error on non-2xx status", async () => {
      const transport = new MockTransport();
      transport.setResponse(err(httpError(500, "Internal Server Error", "Server crashed")));
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

      const result = await client.call("crashing_method", {});

      expect(isErr(result)).toBe(true);
      if (isErr(result) && isHttpError(result.error as RpcClientError)) {
        const httpErr = result.error as { type: "HttpError"; status: number; statusText: string; body: string };
        expect(httpErr.status).toBe(500);
        expect(httpErr.statusText).toBe("Internal Server Error");
      }
    });
  });

  describe("withHeaders", () => {
    it("creates new client with merged headers", async () => {
      const transport = new MockTransport();
      transport.setSuccessResponse(null);
      const client = new RpcClient(
        {
          baseUrl: "http://localhost:8080/rpc",
          headers: { "X-Api-Key": "key123" },
        },
        transport
      );

      const clientWithAuth = client.withHeaders({ Authorization: "Bearer token" });

      await clientWithAuth.call("protected_method", {});

      const lastRequest = transport.getLastRequest();
      expect(lastRequest!.headers).toEqual({
        "X-Api-Key": "key123",
        Authorization: "Bearer token",
      });
    });

    it("overrides existing headers", async () => {
      const transport = new MockTransport();
      transport.setSuccessResponse(null);
      const client = new RpcClient(
        {
          baseUrl: "http://localhost:8080/rpc",
          headers: { Authorization: "Bearer old" },
        },
        transport
      );

      const clientWithNewAuth = client.withHeaders({ Authorization: "Bearer new" });

      await clientWithNewAuth.call("protected_method", {});

      const lastRequest = transport.getLastRequest();
      expect(lastRequest!.headers).toEqual({ Authorization: "Bearer new" });
    });

    it("does not modify original client", async () => {
      const transport = new MockTransport();
      transport.setSuccessResponse(null);
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

      client.withHeaders({ Authorization: "Bearer token" });

      await client.call("method", {});

      const lastRequest = transport.getLastRequest();
      expect(lastRequest!.headers).toEqual({});
    });
  });

  describe("withTimeout", () => {
    it("creates new client with different timeout", async () => {
      const transport = new MockTransport();
      transport.setSuccessResponse(null);
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

      const clientWithTimeout = client.withTimeout(60000);

      await clientWithTimeout.call("slow_method", {});

      const lastRequest = transport.getLastRequest();
      expect(lastRequest!.timeout).toBe(60000);
    });

    it("does not modify original client", async () => {
      const transport = new MockTransport();
      transport.setSuccessResponse(null);
      const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

      client.withTimeout(60000);

      await client.call("method", {});

      const lastRequest = transport.getLastRequest();
      expect(lastRequest!.timeout).toBe(30000);
    });
  });
});

describe("RpcClientError type guards", () => {
  it("isNetworkError", () => {
    const error = networkError("Connection refused");
    expect(isNetworkError(error)).toBe(true);
    expect(isTimeoutError(error)).toBe(false);
    expect(isParseError(error)).toBe(false);
    expect(isHttpError(error)).toBe(false);
    expect(isServerError(error)).toBe(false);
  });

  it("isTimeoutError", () => {
    const error = timeoutError("Timed out");
    expect(isNetworkError(error)).toBe(false);
    expect(isTimeoutError(error)).toBe(true);
    expect(isParseError(error)).toBe(false);
    expect(isHttpError(error)).toBe(false);
    expect(isServerError(error)).toBe(false);
  });

  it("isParseError", () => {
    const error = parseError("Invalid JSON", "not json");
    expect(isNetworkError(error)).toBe(false);
    expect(isTimeoutError(error)).toBe(false);
    expect(isParseError(error)).toBe(true);
    expect(isHttpError(error)).toBe(false);
    expect(isServerError(error)).toBe(false);
  });

  it("isHttpError", () => {
    const error = httpError(404, "Not Found", "Resource not found");
    expect(isNetworkError(error)).toBe(false);
    expect(isTimeoutError(error)).toBe(false);
    expect(isParseError(error)).toBe(false);
    expect(isHttpError(error)).toBe(true);
    expect(isServerError(error)).toBe(false);
  });

  it("type narrowing works correctly", () => {
    const errors: RpcClientError[] = [
      networkError("net"),
      timeoutError("timeout"),
      parseError("parse", "body"),
      httpError(500, "error", "body"),
    ];

    for (const error of errors) {
      if (isNetworkError(error)) {
        const msg: string = error.message;
        expect(msg).toBe("net");
      } else if (isTimeoutError(error)) {
        const msg: string = error.message;
        expect(msg).toBe("timeout");
      } else if (isParseError(error)) {
        const body: string = error.body;
        expect(body).toBe("body");
      } else if (isHttpError(error)) {
        const status: number = error.status;
        expect(status).toBe(500);
      }
    }
  });
});

describe("Type safety", () => {
  it("call method preserves response type", async () => {
    interface UserResponse {
      id: number;
      name: string;
      email: string;
    }

    const transport = new MockTransport();
    transport.setSuccessResponse({ id: 1, name: "Alice", email: "alice@example.com" });
    const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

    const result = await client.call<UserResponse>("get_user", { user_id: 1 });

    if (isOk(result)) {
      // TypeScript should know these properties exist
      const id: number = result.value.id;
      const name: string = result.value.name;
      const email: string = result.value.email;
      expect(id).toBe(1);
      expect(name).toBe("Alice");
      expect(email).toBe("alice@example.com");
    }
  });

  it("call method preserves error type", async () => {
    interface ApiError {
      code: string;
      message: string;
    }

    const transport = new MockTransport();
    transport.setErrorResponse({ code: "NOT_FOUND", message: "User not found" });
    const client = new RpcClient({ baseUrl: "http://localhost:8080/rpc" }, transport);

    const result = await client.call<unknown, ApiError>("get_user", { user_id: 999 });

    if (isErr(result)) {
      // Error could be RpcClientError or ApiError
      const error = result.error;
      if (typeof error === "object" && error !== null && "code" in error) {
        const apiError = error as ApiError;
        expect(apiError.code).toBe("NOT_FOUND");
        expect(apiError.message).toBe("User not found");
      }
    }
  });
});
