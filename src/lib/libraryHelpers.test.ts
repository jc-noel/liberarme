import { describe, it, expect } from "vitest";
import { getStatusLabel, isMissingCredentialsError } from "./libraryHelpers";

describe("getStatusLabel", () => {
  it("labels an installed game as Installed, regardless of ownership", () => {
    expect(getStatusLabel({ is_installed: true, is_owned: false })).toBe("Installed");
    expect(getStatusLabel({ is_installed: true, is_owned: true })).toBe("Installed");
  });

  it("labels an owned-but-not-installed game correctly", () => {
    expect(getStatusLabel({ is_installed: false, is_owned: true })).toBe(
      "Owned (not installed)",
    );
  });

  it("labels a game with neither flag as Unknown", () => {
    expect(getStatusLabel({ is_installed: false, is_owned: false })).toBe("Unknown");
  });
});

describe("isMissingCredentialsError", () => {
  it("detects the Steam API key not configured message", () => {
    expect(
      isMissingCredentialsError(
        "Steam API key not configured. Please configure settings first.",
      ),
    ).toBe(true);
  });

  it("detects the SteamID64 not configured message", () => {
    expect(
      isMissingCredentialsError(
        "SteamID64 not configured. Please configure settings first.",
      ),
    ).toBe(true);
  });

  it("does not treat an unrelated error as a missing-credentials error", () => {
    expect(isMissingCredentialsError("Network error: connection refused")).toBe(false);
    expect(isMissingCredentialsError("Invalid API key. Check your Steam Web API key")).toBe(
      false,
    );
  });
});