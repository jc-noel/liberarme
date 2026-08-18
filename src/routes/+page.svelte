<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type GameRecord = {
    id: string;
    steam_app_id: number;
    title: string;
    normalized_title: string;
    install_path: string;
    install_size: number;
    last_updated: number;
  };

  let games = $state<GameRecord[]>([]);
  let loading = $state(false);
  let error = $state("");
  let hasScanned = $state(false);

  const formatBytes = (bytes: number) => {
    if (!bytes) return "—";

    const units = ["B", "KB", "MB", "GB", "TB"];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex += 1;
    }

    return `${size.toFixed(size >= 10 || unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
  };

  const formatDate = (timestamp: number) => {
    if (!timestamp) return "—";
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  async function scanLibrary() {
    loading = true;
    error = "";

    try {
      const result = await invoke<GameRecord[]>("scan_steam_games");
      games = result;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to scan Steam library.";
    } finally {
      hasScanned = true;
      loading = false;
    }
  }

  async function loadInstalledGames() {
    loading = true;
    error = "";

    try {
      const result = await invoke<GameRecord[]>("get_installed_games");
      games = result;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load installed games.";
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadInstalledGames();
  });
</script>

<section class="page-header">
  <div>
    <h1>Library</h1>
    <p>Scan locally first. Keep everything in one place.</p>
  </div>

  <button class="scan-btn">
    {loading ? "Scanning..." : "Scan Library"}
  </button>
</section>

<section class="stats-grid">
  <article class="stat-card">
    <h2>Games</h2>
    <p>{games.length}</p>
  </article>

  <article class="stat-card">
    <h2>Source</h2>
    <p>Steam</p>
  </article>

  <article class="stat-card">
    <h2>Mode</h2>
    <p>Local scan</p>
  </article>
</section>

{#if error}
  <p class="error">{error}</p>
{/if}

{#if loading}
  <p class="muted">Scanning your Steam folders...</p>
{/if}

{#if !loading && !hasScanned}
  <section class="panel">
    <p>No games scanned yet. Click <strong>Scan Library</strong> to begin.</p>
  </section>
{/if}

{#if !loading && hasScanned && games.length === 0}
  <section class="panel">
    <p>No games found on this machine.</p>
  </section>
{/if}

{#if games.length > 0}
  <section class="panel">
    <table class="games-table">
      <thead>
        <tr>
          <th>Game</th>
          <th>Status</th>
          <th>App ID</th>
          <th>Size</th>
          <th>Last Updated</th>
        </tr>
      </thead>
      <tbody>
        {#each games as game}
          <tr>
            <td>
              <div>{game.title}</div>
              <small>{game.install_path}</small>
            </td>
            <td>Installed · Local</td>
            <td>{game.steam_app_id}</td>
            <td>{formatBytes(game.install_size)}</td>
            <td>{formatDate(game.last_updated)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </section>
{/if}

<style>
  .page-header {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: center;
    margin-bottom: 20px;
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

  .scan-btn {
    border: none;
    border-radius: 14px;
    padding: 11px 18px;
    color: #fff;
    font-weight: 700;
    background: linear-gradient(135deg, #5b7cff, #7c3aed);
    cursor: pointer;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
    margin-bottom: 16px;
  }

  .stat-card {
    border: 1px solid #243244;
    border-radius: 14px;
    padding: 12px 14px;
    background: #111a25;
  }

  .stat-card h2 {
    margin: 0 0 4px;
    font-size: 0.82rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #94a3b8;
  }

  .stat-card p {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 700;
  }

  .panel {
    border: 1px dashed #2a3b4f;
    border-radius: 14px;
    padding: 16px;
    background: #111a25;
    color: #cbd5e1;
  }

  .muted {
    color: #94a3b8;
  }

  .error {
    color: #f87171;
    font-weight: 600;
  }

  .games-table {
    width: 100%;
    border-collapse: collapse;
  }

  .games-table th,
  .games-table td {
    text-align: left;
    border-bottom: 1px solid #223041;
    padding: 10px 8px;
    vertical-align: top;
  }

  .games-table thead th {
    color: #94a3b8;
    font-size: 0.82rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  small {
    color: #94a3b8;
    display: block;
    margin-top: 2px;
  }
</style>