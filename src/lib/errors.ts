// Raw JS/runtime exceptions (e.g. the Tauri bridge being unreachable) should never
// reach the user verbatim; translate them to a plain-language fallback instead.
const TECHNICAL_ERROR_PATTERN = /^(TypeError|ReferenceError|SyntaxError|RangeError):|Cannot read propert/;

export function toUserMessage(error: unknown, fallback: string): string {
  const raw = error instanceof Error ? error.message : String(error);

  if (TECHNICAL_ERROR_PATTERN.test(raw)) {
    console.error(error);
    return fallback;
  }

  return raw;
}
