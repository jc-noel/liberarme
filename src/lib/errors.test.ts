import { describe, it, expect, vi } from "vitest";
import { toUserMessage } from "./errors";

describe("toUserMessage", () => {
  it("passes through a specific backend error message unchanged", () => {
    const backendError = "Invalid API key. Check your Steam Web API key";
    expect(toUserMessage(backendError, "fallback")).toBe(backendError);
  });

  it("passes through a privacy-related backend error message unchanged", () => {
    const backendError = "Access denied. Check your Steam privacy settings";
    expect(toUserMessage(backendError, "fallback")).toBe(backendError);
  });

  it("passes through a network error message unchanged", () => {
    const backendError = "Network error: connection refused";
    expect(toUserMessage(backendError, "fallback")).toBe(backendError);
  });

  it("replaces a raw TypeError with the fallback message", () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const jsError = new TypeError("Cannot read properties of undefined");

    expect(toUserMessage(jsError, "Something went wrong.")).toBe(
      "Something went wrong.",
    );
    expect(consoleSpy).toHaveBeenCalledWith(jsError);

    consoleSpy.mockRestore();
  });

  it("replaces a 'Cannot read propert...' style message with the fallback", () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const jsError = "Cannot read property 'foo' of null";

    expect(toUserMessage(jsError, "Something went wrong.")).toBe(
      "Something went wrong.",
    );

    consoleSpy.mockRestore();
  });

  it("handles a plain Error instance by using its message", () => {
    const err = new Error("Custom backend error");
    expect(toUserMessage(err, "fallback")).toBe("Custom backend error");
  });
});