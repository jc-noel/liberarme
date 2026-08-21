import { describe, it, expect } from "vitest";
import { validateSteamApiKey, validateSteamId64 } from "./steamValidation";

describe("validateSteamApiKey", () => {
  it("rejects an empty value", () => {
    expect(validateSteamApiKey("")).toBe("Steam API key is required.");
  });

  it("rejects a value that is only whitespace", () => {
    expect(validateSteamApiKey("   ")).toBe("Steam API key is required.");
  });

  it("rejects a key shorter than 8 characters", () => {
    expect(validateSteamApiKey("short")).toBe("Steam API key looks too short.");
  });

  it("accepts a key of 8 or more characters", () => {
    expect(validateSteamApiKey("12345678")).toBeNull();
  });

  it("trims whitespace before checking length", () => {
    expect(validateSteamApiKey("  12345678  ")).toBeNull();
  });
});

describe("validateSteamId64", () => {
  it("rejects an empty value", () => {
    expect(validateSteamId64("")).toBe("SteamID64 is required.");
  });

  it("rejects a value that is only whitespace", () => {
    expect(validateSteamId64("   ")).toBe("SteamID64 is required.");
  });

  it("rejects a value that is too short", () => {
    expect(validateSteamId64("123")).toBe(
      "SteamID64 must be exactly 17 numeric digits.",
    );
  });

  it("rejects a value that is too long", () => {
    expect(validateSteamId64("765611981234567890")).toBe(
      "SteamID64 must be exactly 17 numeric digits.",
    );
  });

  it("rejects a value with non-numeric characters", () => {
    expect(validateSteamId64("7656119812345678a")).toBe(
      "SteamID64 must be exactly 17 numeric digits.",
    );
  });

  it("accepts a valid 17-digit SteamID64", () => {
    expect(validateSteamId64("76561198123456789")).toBeNull();
  });
});