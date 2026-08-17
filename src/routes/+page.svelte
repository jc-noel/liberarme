<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

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
</script>

<main class="shell">
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark">L</div>
      <div>
        <div class="brand-name">Liberarme</div>
        <div class="brand-subtitle">Local-first library auditor</div>
      </div>
    </div>

    <nav class="nav">
      <a class="nav-item active" href="/">Library</a>
      <a class="nav-item" href="/">Settings</a>
    </nav>

    <div class="sidebar-card">
      <div class="sidebar-card-label">Status</div>
      <div class="sidebar-card-value">
        {#if loading}
          Scanning...
        {:else if games.length > 0}
          {games.length} games found
        {:else if hasScanned}
          No games found
        {:else}
          Ready to scan
        {/if}
      </div>
    </div>
  </aside>

  <section class="content">
    <header class="topbar">
      <div>
        <h1>Library</h1>
        <p>Scan locally first. Keep everything in one place.</p>
      </div>

      <button class="primary-button" onclick={scanLibrary} disabled={loading}>
        {loading ? "Scanning..." : "Scan Library"}
      </button>
    </header>

    <section class="summary-grid">
      <article class="summary-card">
        <div class="summary-label">Games</div>
        <div class="summary-value">{games.length}</div>
      </article>

      <article class="summary-card">
        <div class="summary-label">Source</div>
        <div class="summary-value">Steam</div>
      </article>

      <article class="summary-card">
        <div class="summary-label">Mode</div>
        <div class="summary-value">Local scan</div>
      </article>
    </section>

    {#if error}
      <div class="message error">{error}</div>
    {/if}

    {#if loading}
      <div class="message muted">Scanning your Steam folders...</div>
    {/if}

    {#if !loading && !hasScanned}
      <div class="empty-state">
        No games scanned yet. Click <strong>Scan Library</strong> to begin.
      </div>
    {/if}

    {#if !loading && hasScanned && games.length === 0}
      <div class="empty-state">
        No games found on this machine.
      </div>
    {/if}

    {#if games.length > 0}
      <div class="table-shell">
        <table>
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
                  <div class="game-title">{game.title}</div>
                  <div class="game-subtitle">{game.install_path}</div>
                </td>
                <td>
                  <div class="badges">
                    <span class="badge badge-installed">Installed</span>
                    <span class="badge badge-local">Local</span>
                  </div>
                </td>
                <td class="mono">{game.steam_app_id}</td>
                <td class="mono">{formatBytes(game.install_size)}</td>
                <td class="mono">{formatDate(game.last_updated)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </section>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family:
      Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
      sans-serif;
    background:
      radial-gradient(circle at top, rgba(109, 94, 252, 0.08), transparent 28%),
      #0b0f14;
    color: #e6eaf0;
  }

  :global(a) {
    color: inherit;
    text-decoration: none;
  }

  .shell {
    min-height: 100vh;
    display: grid;
    grid-template-columns: 280px 1fr;
  }

  .sidebar {
    padding: 24px 18px;
    border-right: 1px solid #1b2430;
    background: rgba(13, 18, 24, 0.88);
    backdrop-filter: blur(12px);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 28px;
  }

  .brand-mark {
    width: 42px;
    height: 42px;
    border-radius: 14px;
    display: grid;
    place-items: center;
    background: #6d5efc;
    color: white;
    font-weight: 700;
    box-shadow: 0 8px 24px rgba(109, 94, 252, 0.25);
  }

  .brand-name {
    font-weight: 700;
    font-size: 1rem;
  }

  .brand-subtitle {
    font-size: 0.85rem;
    color: #91a0b6;
  }

  .nav {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 20px;
  }

  .nav-item {
    padding: 10px 12px;
    border-radius: 10px;
    color: #aab4c3;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .nav-item:hover,
  .nav-item.active {
    background: #16202c;
    color: #f5f7fb;
  }

  .sidebar-card {
    margin-top: 18px;
    padding: 14px;
    border: 1px solid #1b2430;
    border-radius: 14px;
    background: #0f151d;
  }

  .sidebar-card-label {
    color: #91a0b6;
    font-size: 0.8rem;
    margin-bottom: 6px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .sidebar-card-value {
    font-weight: 600;
  }

  .content {
    padding: 32px;
  }

  .topbar {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 20px;
    margin-bottom: 20px;
  }

  h1 {
    margin: 0 0 8px 0;
    font-size: 2rem;
    line-height: 1.1;
  }

  .topbar p {
    margin: 0;
    color: #91a0b6;
  }

  .primary-button {
    border: 1px solid rgba(109, 94, 252, 0.45);
    border-radius: 12px;
    padding: 12px 16px;
    background: linear-gradient(180deg, #7868ff, #6354f4);
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: transform 0.12s ease, filter 0.12s ease;
    box-shadow: 0 10px 24px rgba(99, 84, 244, 0.18);
  }

  .primary-button:hover:not(:disabled) {
    transform: translateY(-1px);
    filter: brightness(1.05);
  }

  .primary-button:disabled {
    cursor: not-allowed;
    opacity: 0.75;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
    margin-bottom: 18px;
  }

  .summary-card {
    padding: 14px 16px;
    border: 1px solid #1b2430;
    border-radius: 14px;
    background: #0f151d;
  }

  .summary-label {
    color: #91a0b6;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: 8px;
  }

  .summary-value {
    font-size: 1.1rem;
    font-weight: 700;
  }

  .message {
    margin-bottom: 16px;
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid #1b2430;
  }

  .message.error {
    background: rgba(184, 74, 74, 0.12);
    border-color: rgba(184, 74, 74, 0.35);
    color: #ffb7b7;
  }

  .message.muted {
    background: #111820;
    color: #91a0b6;
  }

  .empty-state {
    padding: 24px;
    border: 1px dashed #263241;
    border-radius: 16px;
    background: #0f151d;
    color: #91a0b6;
  }

  .table-shell {
    border: 1px solid #1b2430;
    border-radius: 16px;
    overflow: hidden;
    background: #0f151d;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  thead {
    background: #101823;
  }

  th,
  td {
    padding: 14px 16px;
    text-align: left;
    border-bottom: 1px solid #1b2430;
    vertical-align: top;
  }

  th {
    color: #91a0b6;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-weight: 600;
  }

  tbody tr:hover {
    background: rgba(255, 255, 255, 0.02);
  }

  .game-title {
    font-weight: 700;
    margin-bottom: 4px;
  }

  .game-subtitle {
    color: #91a0b6;
    font-size: 0.9rem;
    word-break: break-all;
  }

  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    border-radius: 999px;
    padding: 5px 10px;
    font-size: 0.78rem;
    font-weight: 600;
    border: 1px solid transparent;
  }

  .badge-installed {
    background: rgba(85, 197, 122, 0.12);
    color: #8ee39f;
    border-color: rgba(85, 197, 122, 0.2);
  }

  .badge-local {
    background: rgba(109, 94, 252, 0.12);
    color: #b2a8ff;
    border-color: rgba(109, 94, 252, 0.2);
  }

  .mono {
    font-variant-numeric: tabular-nums;
    color: #d6dbe3;
    white-space: nowrap;
  }

  @media (max-width: 1000px) {
    .shell {
      grid-template-columns: 1fr;
    }

    .sidebar {
      border-right: 0;
      border-bottom: 1px solid #1b2430;
    }

    .summary-grid {
      grid-template-columns: 1fr;
    }

    .topbar {
      flex-direction: column;
      align-items: stretch;
    }
  }

  @media (max-width: 720px) {
    .content {
      padding: 20px;
    }

    .table-shell {
      overflow-x: auto;
    }

    table {
      min-width: 760px;
    }
  }
</style>