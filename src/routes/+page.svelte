<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let games: Array<{
    id: string;
    steam_app_id: number;
    title: string;
    install_path: string;
    install_size: number;
    last_updated: number | null;
    synced_at: number;
  }> = [];

  let loading = false;
  let syncing = false;  // ADD THIS - track sync state
  let hasScanned = false;
  let error = "";
  let statusMessage = "";  // ADD THIS - for sync success messages

  type OwnedGame = {
    appid: number;
    name: string;
    playtime_forever: number;
  };

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  }

  function formatDate(timestamp: number): string {
    if (!timestamp) return "Never";
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  async function scanLocalLibrary() {
    loading = true;
    error = "";
    statusMessage = "";

    try {
      games = await invoke("scan_steam_games");
      hasScanned = true;
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  // ADD THIS FUNCTION - sync owned games from Steam API
  async function syncOwnedGames() {
    syncing = true;
    error = "";
    statusMessage = "";

    try {
      const ownedGames = await invoke<OwnedGame[]>("sync_owned_games");
      statusMessage = `Successfully synced ${ownedGames.length} owned games from Steam.`;
      // Optionally reload the local game list to show any new matches
      // games = await invoke("get_installed_games");
    } catch (err) {
      error = String(err);
    } finally {
      syncing = false;
    }
  }

  onMount(async () => {
    try {
      games = await invoke("get_installed_games");
      if (games.length > 0) {
        hasScanned = true;
      }
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
  <button class="scan-btn" on:click={scanLocalLibrary} disabled={loading}>
    {loading ? "Scanning..." : "Scan Library"}
  </button>

  <!-- ADD THIS BUTTON - sync owned games -->
  <button class="sync-btn" on:click={syncOwnedGames} disabled={syncing}>
    {syncing ? "Syncing..." : "Sync Owned Games"}
  </button>
</section>

<section class="status-cards">
  <div class="status-card">
    <h2>Games</h2>
    <p>{games.length}</p>
  </div>

  <div class="status-card">
    <h2>Source</h2>
    <p>Steam</p>
  </div>

  <div class="status-card">
    <h2>Mode</h2>
    <p>Local scan</p>
  </div>
</section>

{#if error}
  <p class="error">{error}</p>
{/if}

{#if statusMessage}
  <p class="success">{statusMessage}</p>
{/if}

{#if loading}
  <p class="muted">Scanning your Steam folders...</p>
{/if}

{#if !loading && !hasScanned && games.length === 0}
  <div class="empty-state">
    <p>No games scanned yet. Click <strong>Scan Library</strong> to begin.</p>
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
            <td>installed</td>
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

  .scan-btn,
  .sync-btn {
    border: none;
    border-radius: 12px;
    padding: 10px 18px;
    color: #fff;
    font-weight: 700;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .scan-btn {
    background: linear-gradient(135deg, #5b7cff, #7c3aed);
  }

  .sync-btn {
    background: linear-gradient(135deg, #34d399, #10b981);
  }

  .scan-btn:hover:not(:disabled),
  .sync-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .scan-btn:disabled,
  .sync-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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