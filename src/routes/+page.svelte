<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let games: Array<{
    id: string;
    steam_app_id: number;
    title: string;
    install_path: string | null;
    install_size: number | null;
    last_updated: number | null;
    synced_at: number;
    is_installed: boolean;
    is_owned: boolean;
  }> = [];

  let loading = false;
  let loadingLabel = "";
  let syncing = false;
  let hasScanned = false;
  let error = "";
  let statusMessage = "";
  let infoMessage = "";
  let dropdownOpen = false;

  type OwnedGame = {
    appid: number;
    name: string;
    playtime_forever: number;
  };

  function formatBytes(bytes: number | null): string {
    if (!bytes) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  }

  function formatDate(timestamp: number): string {
    if (!timestamp) return "Never";
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  function getStatusLabel(game: {
    is_installed: boolean;
    is_owned: boolean;
  }): string {
    if (game.is_installed) {
      return "Installed";
    }
    if (game.is_owned) {
      return "Owned (not installed)";
    }
    return "Unknown";
  }

  function isMissingCredentialsError(message: string): boolean {
    return message.includes("not configured");
  }

  async function refreshCombinedList() {
    games = await invoke("get_all_games");
    if (games.length > 0) {
      hasScanned = true;
    }
  }

  async function scanLocalOnly() {
    loading = true;
    loadingLabel = "Scanning locally...";
    error = "";
    statusMessage = "";
    infoMessage = "";

    try {
      await invoke("scan_steam_games");
      await refreshCombinedList();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
      loadingLabel = "";
    }
  }

  async function syncOwnedOnly() {
    loading = true;
    loadingLabel = "Syncing owned games...";
    error = "";
    statusMessage = "";
    infoMessage = "";

    try {
      const ownedGames = await invoke<OwnedGame[]>("sync_owned_games");
      statusMessage = `Successfully synced ${ownedGames.length} owned games from Steam.`;
      await refreshCombinedList();
    } catch (err) {
      const message = String(err);
      if (isMissingCredentialsError(message)) {
        infoMessage =
          "Connect your Steam account in Settings to sync owned games.";
      } else {
        error = message;
      }
    } finally {
      loading = false;
      loadingLabel = "";
    }
  }

  async function scanAll() {
    loading = true;
    error = "";
    statusMessage = "";
    infoMessage = "";

    // local scan. If this fails, stop - there's nothing to reconcile.
    loadingLabel = "Scanning locally...";
    try {
      await invoke("scan_steam_games");
    } catch (err) {
      error = String(err);
      loading = false;
      loadingLabel = "";
      return;
    }

    // first-run users who haven't configured Settings yet, so it's shown
    // as a soft hint rather than an error, and local scan results still stand.
    loadingLabel = "Syncing owned games...";
    try {
      const ownedGames = await invoke<OwnedGame[]>("sync_owned_games");
      statusMessage = `Successfully synced ${ownedGames.length} owned games from Steam.`;
    } catch (err) {
      const message = String(err);
      if (isMissingCredentialsError(message)) {
        infoMessage =
          "Connect your Steam account in Settings to also see owned-but-not-installed games.";
      } else {
        infoMessage = `Local scan complete. Owned games sync didn't finish: ${message}`;
      }
    }

    // local scan results should still populate the table.
    try {
      await refreshCombinedList();
    } catch (err) {
      error = String(err);
    }

    loading = false;
    loadingLabel = "";
  }

  function toggleDropdown() {
    dropdownOpen = !dropdownOpen;
  }

  function closeDropdown() {
    dropdownOpen = false;
  }

  async function handleDropdownAction(action: "local" | "owned") {
    closeDropdown();
    if (action === "local") {
      await scanLocalOnly();
    } else {
      await syncOwnedOnly();
    }
  }

  onMount(async () => {
    try {
      await refreshCombinedList();
    } catch (err) {
      error = String(err);
    }
  });
</script>

<section class="page-header">
  <div>
    <h1>Library</h1>
    <p>Scan locally first. Keep everything in one place.</p>
  </div>
</section>

<section class="actions">
  <div class="split-btn" class:open={dropdownOpen}>
    <button class="scan-all-btn" onclick={scanAll} disabled={loading}>
      {loading ? loadingLabel || "Working..." : "Scan All"}
    </button>
    <button
      class="split-caret"
      onclick={toggleDropdown}
      disabled={loading}
      aria-haspopup="true"
      aria-expanded={dropdownOpen}
      aria-label="More scan options"
    >
      ▾
    </button>

    {#if dropdownOpen}
      <div class="dropdown-menu" role="menu">
        <button role="menuitem" onclick={() => handleDropdownAction("local")}>
          Scan Local Only
        </button>
        <button role="menuitem" onclick={() => handleDropdownAction("owned")}>
          Sync Owned Games Only
        </button>
      </div>
    {/if}
  </div>
</section>

<section class="status-cards">
  <div class="status-card">
    <h2>Games</h2>
    <p>{games.length}</p>
  </div>

  <div class="status-card">
    <h2>Source</h2>
    <p>{games.some((g) => g.is_owned) ? "Local + API" : "Local Steam scan"}</p>
  </div>

  <div class="status-card">
    <h2>Mode</h2>
    <p>{loading ? "Scan" : "Review"}</p>
  </div>
</section>

{#if error}
  <p class="error">{error}</p>
{/if}

{#if statusMessage}
  <p class="success">{statusMessage}</p>
{/if}

{#if infoMessage}
  <p class="info">{infoMessage}</p>
{/if}

{#if loading}
  <p class="muted">{loadingLabel}</p>
{/if}

{#if !loading && !hasScanned && games.length === 0}
  <div class="empty-state">
    <p>No games scanned yet. Click <strong>Scan All</strong> to begin.</p>
  </div>
{/if}

{#if !loading && hasScanned && games.length === 0}
  <div class="empty-state">
    <p>No games found on this machine.</p>
  </div>
{/if}

{#if games.length > 0}
  <div class="table-container">
    <table>
      <thead>
        <tr>
          <th>Game</th>
          <th>Status</th>
          <th>App ID</th>
          <th>Size</th>
          <th>Activity</th>
        </tr>
      </thead>
      <tbody>
        {#each games as game (game.id)}
          <tr>
            <td>
              <div class="game-info">
                <div>{game.title}</div>
                <div class="install-path">{game.install_path}</div>
              </div>
            </td>
            <td>{getStatusLabel(game)}</td>
            <td>{game.steam_app_id}</td>
            <td>{formatBytes(game.install_size)}</td>
            <td>
              <div class="activity">
                <div>updated: {formatDate(game.last_updated || 0)}</div>
                <div>synced: {formatDate(game.synced_at)}</div>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .page-header {
    margin-bottom: 24px;
  }

  .page-header h1 {
    margin: 0;
    font-size: 2.2rem;
  }

  .page-header p {
    margin: 6px 0 0;
    color: #94a3b8;
    font-size: 1.05rem;
  }

  .actions {
    display: flex;
    gap: 12px;
    margin-bottom: 24px;
  }

  .split-btn {
    position: relative;
    display: flex;
  }

  .scan-all-btn {
    border: none;
    border-radius: 12px 0 0 12px;
    padding: 10px 18px;
    color: #fff;
    font-weight: 700;
    cursor: pointer;
    background: linear-gradient(135deg, #5b7cff, #7c3aed);
    transition: opacity 0.15s ease;
  }

  .split-caret {
    border: none;
    border-left: 1px solid rgba(255, 255, 255, 0.25);
    border-radius: 0 12px 12px 0;
    padding: 10px 12px;
    color: #fff;
    cursor: pointer;
    background: linear-gradient(135deg, #5b7cff, #7c3aed);
    transition: opacity 0.15s ease;
  }

  .scan-all-btn:hover:not(:disabled),
  .split-caret:hover:not(:disabled) {
    opacity: 0.9;
  }

  .scan-all-btn:disabled,
  .split-caret:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    min-width: 200px;
    display: grid;
    background: #111a25;
    border: 1px solid #243244;
    border-radius: 10px;
    overflow: hidden;
    z-index: 10;
  }

  .dropdown-menu button {
    text-align: left;
    padding: 10px 14px;
    border: none;
    background: transparent;
    color: #e5e7eb;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .dropdown-menu button:hover {
    background: #1a2332;
  }

  .status-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
    margin-bottom: 24px;
  }

  .status-card {
    border: 1px solid #243244;
    border-radius: 12px;
    padding: 16px;
    background: #111a25;
  }

  .status-card h2 {
    margin: 0 0 8px;
    font-size: 0.9rem;
    color: #94a3b8;
    font-weight: 600;
    text-transform: uppercase;
  }

  .status-card p {
    margin: 0;
    font-size: 1.8rem;
    color: #e5e7eb;
    font-weight: 700;
  }

  .error {
    color: #f87171;
    margin-bottom: 16px;
    font-size: 0.95rem;
  }

  .success {
    color: #34d399;
    margin-bottom: 16px;
    font-size: 0.95rem;
  }

  .info {
    color: #7dd3fc;
    margin-bottom: 16px;
    font-size: 0.95rem;
  }

  .muted {
    color: #94a3b8;
    margin-bottom: 16px;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #94a3b8;
  }

  .empty-state p {
    margin: 0;
    font-size: 1.05rem;
  }

  .table-container {
    overflow-x: auto;
    border: 1px solid #243244;
    border-radius: 12px;
    background: #111a25;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  thead {
    background: #0f1722;
    border-bottom: 1px solid #243244;
  }

  th {
    padding: 12px;
    text-align: left;
    color: #cbd5e1;
    font-weight: 600;
    font-size: 0.9rem;
    text-transform: uppercase;
  }

  td {
    padding: 12px;
    border-bottom: 1px solid #243244;
    color: #e5e7eb;
  }

  tbody tr:hover {
    background: #1a2332;
  }

  .game-info {
    display: grid;
    gap: 4px;
  }

  .game-info div:first-child {
    font-weight: 500;
  }

  .install-path {
    color: #94a3b8;
    font-size: 0.85rem;
  }

  .activity {
    display: grid;
    gap: 4px;
    font-size: 0.9rem;
    color: #94a3b8;
  }
</style>
