<script lang="ts">
  import { page } from "$app/stores";

  // keep this static for now; later wire true dynamic status from store/state.
  const statusText = "Ready to scan";
</script>

<div class="app-shell">
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-logo">L</div>
      <div>
        <p class="brand-name">Liberarme</p>
        <p class="brand-tagline">audit/backup your games</p>
      </div>
    </div>

    <nav class="nav">
      <a href="/" class:active={$page.url.pathname === "/"} aria-current={$page.url.pathname === "/" ? "page" : undefined}>Library</a>
      <a href="/settings" class:active={$page.url.pathname === "/settings"} aria-current={$page.url.pathname === "/settings" ? "page" : undefined}>Settings</a>
    </nav>

    <section class="status-card">
      <h2>Status</h2>
      <p>{statusText}</p>
    </section>
  </aside>

  <main class="content">
    <slot />
  </main>
</div>

<style>
  :global(:root) {
    --archive-bg: #0b1118;
    --archive-raised: #0d141c;
    --archive-card: #111a25;
    --archive-well: #0f1722;
    --case-file-indigo: #5b7cff;
    --case-file-indigo-rgb: 91, 124, 255;
    --case-file-indigo-deep: #3f5fd9;
    --case-file-violet: #7c3aed;
    --paper-white: #e5e7eb;
    --slate-ash: #94a3b8;
    --slate-ash-bright: #cbd5e1;
    --border-hairline: #1f2937;
    --border-line: #243244;
    --border-line-soft: #2b3a4d;
    --border-dashed: #2a3b4f;
    --table-line: #223041;
    --row-hover: #16212e;
    --nav-hover: #182230;
    --nav-active: #1b2736;
    --success: #34d399;
    --danger: #f87171;
    --danger-soft: #fca5a5;
    --on-accent: #ffffff;
    --accent-text-soft: #93a9ff;
  }

  :global(body) {
    margin: 0;
    background: var(--archive-bg);
    color: var(--paper-white);
    font-family: Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
    scrollbar-color: var(--border-line) var(--archive-bg);
    scrollbar-width: thin;
  }

  :global(::selection) {
    background: rgba(var(--case-file-indigo-rgb), 0.35);
    color: var(--on-accent);
  }

  :global(*:focus-visible) {
    outline: 2px solid var(--case-file-indigo);
    outline-offset: 2px;
  }

  :global(::-webkit-scrollbar) {
    width: 10px;
    height: 10px;
  }

  :global(::-webkit-scrollbar-track) {
    background: var(--archive-bg);
  }

  :global(::-webkit-scrollbar-thumb) {
    background: var(--border-line);
    border-radius: 999px;
    border: 2px solid var(--archive-bg);
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: var(--border-line-soft);
  }

  .app-shell {
    min-height: 100vh;
    display: grid;
    grid-template-columns: 280px 1fr;
  }

  .sidebar {
    border-right: 1px solid var(--border-hairline);
    padding: 20px 18px;
    background: var(--archive-raised);
  }

  .brand {
    display: flex;
    gap: 12px;
    align-items: center;
    margin-bottom: 20px;
  }

  .brand-logo {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    background: linear-gradient(135deg, var(--case-file-indigo-deep), var(--case-file-violet));
    color: var(--on-accent);
    display: grid;
    place-items: center;
    font-weight: 700;
  }

  .brand-name {
    margin: 0;
    font-size: 1.8rem;
    line-height: 1.1;
    font-weight: 700;
  }

  .brand-tagline {
    margin: 4px 0 0;
    color: var(--slate-ash);
    font-size: 0.95rem;
  }

  .nav {
    display: grid;
    gap: 8px;
    margin: 16px 0 24px;
  }

  .nav a {
    color: var(--slate-ash-bright);
    text-decoration: none;
    padding: 10px 12px;
    border-radius: 10px;
    transition: background 0.15s ease;
  }

  .nav a:hover {
    background: var(--nav-hover);
  }

  .nav a.active {
    background: var(--nav-active);
    color: var(--on-accent);
    font-weight: 600;
  }

  .status-card {
    border: 1px solid var(--border-line);
    border-radius: 14px;
    padding: 12px;
    background: var(--archive-card);
  }

  .status-card h2 {
    margin: 0 0 6px;
    font-size: 0.82rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--slate-ash);
  }

  .status-card p {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
  }

  .content {
    padding: 28px 30px;
  }

  @media (max-width: 860px) {
    .app-shell {
      grid-template-columns: 1fr;
    }

    .sidebar {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      border-right: none;
      border-bottom: 1px solid var(--border-hairline);
      padding: 14px 18px;
    }

    .brand {
      margin-bottom: 0;
    }

    .brand-tagline,
    .status-card {
      display: none;
    }

    .nav {
      display: flex;
      gap: 8px;
      margin: 0;
    }

    .content {
      padding: 20px;
    }
  }
</style>