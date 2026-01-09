/**
 * Roundtrip Serialization Tests
 *
 * These tests verify that TypeScript types correctly match the JSON
 * serialization format produced by Rust's serde.
 */

import { describe, it, expect } from "vitest";
import type {
  Point,
  User,
  Profile,
  Rgb,
  Ping,
  UserId,
  Rectangle,
  Polygon,
  Config,
  Status,
  Coordinate,
  Shape,
  Message,
  OptionalValue,
  GetUserRequest,
  GetUserResponse,
  ListUsersRequest,
  ListUsersResponse,
  ApiError,
  DetailedError,
  RpcError,
  Workspace,
  CompleteExample,
} from "../fixtures.js";
import {
  isStatus,
  isCoordinateD2,
  isCoordinateD3,
  isShapeCircle,
  isShapeRectangle,
  isShapeTriangle,
  isMessagePing,
  isMessageText,
  isMessageBinary,
  isMessageError,
  isOptionalValueNone,
  isOptionalValueSome,
  isRpcErrorNotFound,
  isRpcErrorUnauthorized,
} from "../fixtures.js";

/**
 * Helper to verify JSON roundtrip: parse -> stringify -> parse
 * This simulates receiving JSON from Rust and sending it back.
 */
function assertRoundtrip<T>(json: string, typeCheck?: (value: T) => boolean): T {
  const parsed = JSON.parse(json) as T;
  const reserialized = JSON.stringify(parsed);
  const reparsed = JSON.parse(reserialized) as T;
  expect(reparsed).toEqual(parsed);
  if (typeCheck) {
    expect(typeCheck(parsed)).toBe(true);
  }
  return parsed;
}

describe("Struct Roundtrip", () => {
  it("Point", () => {
    const point = assertRoundtrip<Point>('{"x":1.0,"y":2.0}');
    expect(point.x).toBe(1.0);
    expect(point.y).toBe(2.0);
  });

  it("User", () => {
    const user = assertRoundtrip<User>(
      '{"id":12345,"name":"Alice","email":"alice@example.com","active":true}'
    );
    expect(user.id).toBe(12345);
    expect(user.name).toBe("Alice");
    expect(user.email).toBe("alice@example.com");
    expect(user.active).toBe(true);
  });

  it("Profile with all fields", () => {
    const profile = assertRoundtrip<Profile>(
      '{"username":"alice","display_name":"Alice Smith","bio":"Hello","avatar_url":"https://example.com/a.png"}'
    );
    expect(profile.username).toBe("alice");
    expect(profile.display_name).toBe("Alice Smith");
    expect(profile.bio).toBe("Hello");
    expect(profile.avatar_url).toBe("https://example.com/a.png");
  });

  it("Profile with null fields", () => {
    const profile = assertRoundtrip<Profile>(
      '{"username":"bob","display_name":null,"bio":null,"avatar_url":null}'
    );
    expect(profile.username).toBe("bob");
    expect(profile.display_name).toBeNull();
    expect(profile.bio).toBeNull();
    expect(profile.avatar_url).toBeNull();
  });

  it("Rgb (tuple struct)", () => {
    const rgb = assertRoundtrip<Rgb>("[255,128,0]");
    expect(rgb).toEqual([255, 128, 0]);
    expect(rgb[0]).toBe(255);
    expect(rgb[1]).toBe(128);
    expect(rgb[2]).toBe(0);
  });

  it("Ping (unit struct)", () => {
    const ping = assertRoundtrip<Ping>("null");
    expect(ping).toBeNull();
  });

  it("UserId (newtype)", () => {
    const userId = assertRoundtrip<UserId>("42");
    expect(userId).toBe(42);
  });

  it("Rectangle (nested)", () => {
    const rect = assertRoundtrip<Rectangle>(
      '{"top_left":{"x":0.0,"y":10.0},"bottom_right":{"x":10.0,"y":0.0}}'
    );
    expect(rect.top_left.x).toBe(0.0);
    expect(rect.top_left.y).toBe(10.0);
    expect(rect.bottom_right.x).toBe(10.0);
    expect(rect.bottom_right.y).toBe(0.0);
  });

  it("Polygon (Vec field)", () => {
    const polygon = assertRoundtrip<Polygon>(
      '{"vertices":[{"x":0.0,"y":0.0},{"x":1.0,"y":0.0},{"x":0.5,"y":1.0}]}'
    );
    expect(polygon.vertices).toHaveLength(3);
    expect(polygon.vertices[0]).toEqual({ x: 0.0, y: 0.0 });
  });

  it("Config (HashMap field)", () => {
    const config = assertRoundtrip<Config>(
      '{"settings":{"theme":"dark","language":"en"}}'
    );
    expect(config.settings["theme"]).toBe("dark");
    expect(config.settings["language"]).toBe("en");
  });
});

describe("Enum Roundtrip", () => {
  it("Status (unit variants)", () => {
    assertRoundtrip<Status>('"Pending"', isStatus);
    assertRoundtrip<Status>('"Active"', isStatus);
    assertRoundtrip<Status>('"Completed"', isStatus);
    assertRoundtrip<Status>('"Failed"', isStatus);
  });

  it("Coordinate.D2", () => {
    const coord = assertRoundtrip<Coordinate>('{"D2":[1.0,2.0]}', isCoordinateD2);
    if (isCoordinateD2(coord)) {
      expect(coord.D2[0]).toBe(1.0);
      expect(coord.D2[1]).toBe(2.0);
    }
  });

  it("Coordinate.D3", () => {
    const coord = assertRoundtrip<Coordinate>(
      '{"D3":[1.0,2.0,3.0]}',
      isCoordinateD3
    );
    if (isCoordinateD3(coord)) {
      expect(coord.D3[0]).toBe(1.0);
      expect(coord.D3[1]).toBe(2.0);
      expect(coord.D3[2]).toBe(3.0);
    }
  });

  it("Shape.Circle", () => {
    const shape = assertRoundtrip<Shape>(
      '{"Circle":{"center":{"x":0.0,"y":0.0},"radius":5.0}}',
      isShapeCircle
    );
    if (isShapeCircle(shape)) {
      expect(shape.Circle.center).toEqual({ x: 0.0, y: 0.0 });
      expect(shape.Circle.radius).toBe(5.0);
    }
  });

  it("Shape.Rectangle", () => {
    const shape = assertRoundtrip<Shape>(
      '{"Rectangle":{"top_left":{"x":0.0,"y":10.0},"width":10.0,"height":10.0}}',
      isShapeRectangle
    );
    if (isShapeRectangle(shape)) {
      expect(shape.Rectangle.top_left).toEqual({ x: 0.0, y: 10.0 });
      expect(shape.Rectangle.width).toBe(10.0);
      expect(shape.Rectangle.height).toBe(10.0);
    }
  });

  it("Shape.Triangle", () => {
    const shape = assertRoundtrip<Shape>(
      '{"Triangle":{"a":{"x":0.0,"y":0.0},"b":{"x":1.0,"y":0.0},"c":{"x":0.5,"y":1.0}}}',
      isShapeTriangle
    );
    if (isShapeTriangle(shape)) {
      expect(shape.Triangle.a).toEqual({ x: 0.0, y: 0.0 });
      expect(shape.Triangle.b).toEqual({ x: 1.0, y: 0.0 });
      expect(shape.Triangle.c).toEqual({ x: 0.5, y: 1.0 });
    }
  });

  it("Message.Ping", () => {
    assertRoundtrip<Message>('{"Ping":null}', isMessagePing);
  });

  it("Message.Text", () => {
    const msg = assertRoundtrip<Message>('{"Text":"Hello"}', isMessageText);
    if (isMessageText(msg)) {
      expect(msg.Text).toBe("Hello");
    }
  });

  it("Message.Binary", () => {
    const msg = assertRoundtrip<Message>('{"Binary":[0,1,2,255]}', isMessageBinary);
    if (isMessageBinary(msg)) {
      expect(msg.Binary).toEqual([0, 1, 2, 255]);
    }
  });

  it("Message.Error", () => {
    const msg = assertRoundtrip<Message>(
      '{"Error":{"code":500,"message":"Internal error"}}',
      isMessageError
    );
    if (isMessageError(msg)) {
      expect(msg.Error.code).toBe(500);
      expect(msg.Error.message).toBe("Internal error");
    }
  });

  it("OptionalValue.None", () => {
    assertRoundtrip<OptionalValue<string>>('{"None":null}', isOptionalValueNone);
  });

  it("OptionalValue.Some", () => {
    const opt = assertRoundtrip<OptionalValue<string>>(
      '{"Some":"value"}',
      isOptionalValueSome
    );
    if (isOptionalValueSome(opt)) {
      expect(opt.Some).toBe("value");
    }
  });
});

describe("RPC Type Roundtrip", () => {
  it("GetUserRequest", () => {
    const req = assertRoundtrip<GetUserRequest>('{"user_id":123}');
    expect(req.user_id).toBe(123);
  });

  it("GetUserResponse with user", () => {
    const res = assertRoundtrip<GetUserResponse>(
      '{"user":{"id":1,"name":"Test","email":"test@example.com","active":true}}'
    );
    expect(res.user).not.toBeNull();
    expect(res.user?.id).toBe(1);
  });

  it("GetUserResponse without user", () => {
    const res = assertRoundtrip<GetUserResponse>('{"user":null}');
    expect(res.user).toBeNull();
  });

  it("ListUsersRequest", () => {
    const req = assertRoundtrip<ListUsersRequest>(
      '{"page":1,"per_page":20,"filter":null}'
    );
    expect(req.page).toBe(1);
    expect(req.per_page).toBe(20);
    expect(req.filter).toBeNull();
  });

  it("ListUsersResponse", () => {
    const res = assertRoundtrip<ListUsersResponse>(
      '{"users":[{"id":1,"name":"Alice","email":"alice@example.com","active":true}],"total":100,"page":1,"per_page":1}'
    );
    expect(res.users).toHaveLength(1);
    expect(res.total).toBe(100);
  });
});

describe("Error Type Roundtrip", () => {
  it("ApiError", () => {
    const err = assertRoundtrip<ApiError>(
      '{"code":"NOT_FOUND","message":"Resource not found"}'
    );
    expect(err.code).toBe("NOT_FOUND");
    expect(err.message).toBe("Resource not found");
  });

  it("DetailedError", () => {
    const err = assertRoundtrip<DetailedError>(
      '{"code":"VALIDATION","message":"Invalid","details":"Email format","field":"email"}'
    );
    expect(err.code).toBe("VALIDATION");
    expect(err.details).toBe("Email format");
    expect(err.field).toBe("email");
  });

  it("RpcError.NotFound", () => {
    const err = assertRoundtrip<RpcError>(
      '{"NotFound":{"resource":"user/123"}}',
      isRpcErrorNotFound
    );
    if (isRpcErrorNotFound(err)) {
      expect(err.NotFound.resource).toBe("user/123");
    }
  });

  it("RpcError.Unauthorized", () => {
    assertRoundtrip<RpcError>('{"Unauthorized":null}', isRpcErrorUnauthorized);
  });
});

describe("Complex Type Roundtrip", () => {
  it("Workspace", () => {
    const workspace = assertRoundtrip<Workspace>(
      '{"id":1,"name":"My Workspace","owner":{"id":1,"name":"Owner","email":"owner@example.com","active":true},"members":[{"id":2,"name":"Member","email":"member@example.com","active":true}],"settings":{"settings":{"visibility":"private"}},"status":"Active"}'
    );
    expect(workspace.id).toBe(1);
    expect(workspace.name).toBe("My Workspace");
    expect(workspace.owner.name).toBe("Owner");
    expect(workspace.members).toHaveLength(1);
    expect(workspace.settings.settings["visibility"]).toBe("private");
    expect(workspace.status).toBe("Active");
  });

  it("CompleteExample", () => {
    const example = assertRoundtrip<CompleteExample>(
      '{"primitive":42,"string":"test","optional":"present","list":[1,2,3],"map":{"key1":100},"nested":{"id":1,"name":"Test","email":"test@example.com","active":true},"nested_list":[],"optional_nested":null,"status":"Pending"}'
    );
    expect(example.primitive).toBe(42);
    expect(example.string).toBe("test");
    expect(example.optional).toBe("present");
    expect(example.list).toEqual([1, 2, 3]);
    expect(example.map["key1"]).toBe(100);
    expect(example.nested.name).toBe("Test");
    expect(example.optional_nested).toBeNull();
    expect(example.status).toBe("Pending");
  });
});

describe("Edge Cases", () => {
  it("Unicode characters", () => {
    const user = assertRoundtrip<User>(
      '{"id":1,"name":"日本語 🎉","email":"test@example.com","active":true}'
    );
    expect(user.name).toBe("日本語 🎉");
  });

  it("Escaped characters", () => {
    const user = assertRoundtrip<User>(
      '{"id":1,"name":"Tab\\tNewline\\nQuote\\"","email":"test@example.com","active":true}'
    );
    expect(user.name).toBe('Tab\tNewline\nQuote"');
  });

  it("Empty collections", () => {
    const polygon = assertRoundtrip<Polygon>('{"vertices":[]}');
    expect(polygon.vertices).toHaveLength(0);

    const config = assertRoundtrip<Config>('{"settings":{}}');
    expect(Object.keys(config.settings)).toHaveLength(0);
  });

  it("Large numbers", () => {
    // Note: JS can't represent u64::MAX precisely, but this tests reasonable ranges
    const user = assertRoundtrip<User>(
      '{"id":9007199254740991,"name":"Max","email":"max@example.com","active":true}'
    );
    expect(user.id).toBe(Number.MAX_SAFE_INTEGER);
  });
});
