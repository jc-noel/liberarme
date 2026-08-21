// Pure helper functions for the Library page's reconciled game view.
// Extracted from +page.svelte so they can be unit tested directly.

export type GameStatusFlags = {
  is_installed: boolean;
  is_owned: boolean;
};

// Computes a human-readable status label from the two boolean flags on a
// game row. This is the core of the "reconciled" view: a row can be
// installed, owned-but-not-installed, or (rarely) neither.
export function getStatusLabel(game: GameStatusFlags): string {
  if (game.is_installed) {
    return "Installed";
  }
  if (game.is_owned) {
    return "Owned (not installed)";
  }
  return "Unknown";
}

// Detects the specific "not configured" errors from sync_owned_games so
// the UI can show a soft first-run hint instead of a red error.
export function isMissingCredentialsError(message: string): boolean {
  return message.includes("not configured");
}