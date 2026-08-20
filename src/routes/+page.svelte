<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { toUserMessage } from "$lib/errors";

  type GameRecord = {
    id: string;
    steam_app_id: number;
    title: string;
    normalized_title: string;
    install_path: string;
    install_size: number;
    last_updated: number | null;
    synced_at: number;
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

  const formatDate = (timestamp: number | null) => {
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
      error = toUserMessage(
        err,
        "Couldn't scan your Steam library. Check your connection and try again.",
      );
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
      error = toUserMessage(
        err,
        "Couldn't load your installed games. Check your connection and try again.",
      );
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

  <button class="scan-btn" onclick={scanLibrary} disabled={loading}>
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
  <p class="error" role="alert">{error}</p>
{/if}

{#if loading}
  <p class="muted" aria-live="polite">Scanning your Steam folders...</p>
{/if}

{#if !loading && !hasScanned && games.length === 0}
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
          <th class="col-game" scope="col">Game</th>
          <th class="col-status" scope="col">Status</th>
          <th class="col-numeric" scope="col">App ID</th>
          <th class="col-numeric" scope="col">Size</th>
          <th class="col-activity" scope="col">Activity</th>
        </tr>
      </thead>
      <tbody>
        {#each games as game (game.id)}
          <tr>
            <td class="col-game">
              <div class="game-title">{game.title}</div>
              <small class="game-path" title={game.install_path}
                >{game.install_path}</small
              >
            </td>
            <td class="col-status"><span class="badge">installed</span></td>
            <td class="col-numeric">{game.steam_app_id}</td>
            <td class="col-numeric">{formatBytes(game.install_size)}</td>
            <td class="col-activity">
              <small>updated: {formatDate(game.last_updated)}</small>
              <small>synced: {formatDate(game.synced_at)}</small>
            </td>
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
    color: var(--slate-ash);
    font-size: 1.05rem;
  }

  .scan-btn {
    border: none;
    border-radius: 14px;
    padding: 11px 18px;
    color: var(--on-accent);
    font-weight: 700;
    background: linear-gradient(135deg, var(--case-file-indigo-deep), var(--case-file-violet));
    cursor: pointer;
  }

  .scan-btn:disabled {
    opacity: 0.65;
    cursor: not-allowed;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
    margin-bottom: 16px;
  }

  .stat-card {
    border: 1px solid var(--border-line);
    border-radius: 14px;
    padding: 12px 14px;
    background: var(--archive-card);
  }

  .stat-card h2 {
    margin: 0 0 4px;
    font-size: 0.82rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--slate-ash);
  }

  .stat-card p {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 700;
  }

  .panel {
    border: 1px dashed var(--border-dashed);
    border-radius: 14px;
    padding: 16px;
    background: var(--archive-card);
    color: var(--slate-ash-bright);
    overflow-x: auto;
  }

  .muted {
    color: var(--slate-ash);
  }

  .error {
    color: var(--danger);
    font-weight: 600;
  }

  .games-table {
    width: 100%;
    min-width: 640px;
    table-layout: fixed;
    border-collapse: collapse;
  }

  .games-table th,
  .games-table td {
    text-align: left;
    border-bottom: 1px solid var(--table-line);
    padding: 12px 10px;
    line-height: 1.45;
    vertical-align: top;
  }

  .games-table thead th {
    color: var(--slate-ash);
    font-size: 0.82rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .games-table tbody tr:hover {
    background: var(--row-hover);
  }

  .col-game {
    width: 42%;
  }

  .col-status {
    width: 14%;
  }

  .col-numeric {
    width: 14%;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .col-activity {
    width: 22%;
  }

  .game-title {
    font-weight: 600;
  }

  .game-path {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .badge {
    display: inline-block;
    padding: 3px 10px;
    border-radius: 999px;
    background: rgba(var(--case-file-indigo-rgb), 0.16);
    color: var(--accent-text-soft);
    font-size: 0.78rem;
    font-weight: 600;
  }

  small {
    color: var(--slate-ash);
    display: block;
    margin-top: 2px;
  }

  @media (max-width: 640px) {
    .page-header {
      flex-direction: column;
      align-items: flex-start;
    }

    .scan-btn {
      width: 100%;
    }

    .stats-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
