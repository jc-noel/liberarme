// Pure validation helpers for Steam credential input fields.
// Extracted from settings/+page.svelte so they can be unit tested directly
// without needing to mount the Svelte component.

export function validateSteamApiKey(value: string): string | null {
  const v = value.trim();
  if (!v) return "Steam API key is required.";
  if (v.length < 8) return "Steam API key looks too short.";
  return null;
}

export function validateSteamId64(value: string): string | null {
  const v = value.trim();
  if (!v) return "SteamID64 is required.";
  if (!/^\d{17}$/.test(v)) return "SteamID64 must be exactly 17 numeric digits.";
  return null;
}