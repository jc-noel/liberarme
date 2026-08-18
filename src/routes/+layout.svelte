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
        <h1>Liberarme</h1>
        <p>audit/backup your games</p>
      </div>
    </div>

    <nav class="nav">
      <a href="/" class:active={$page.url.pathname === "/"}>Library</a>
      <a href="/settings" class:active={$page.url.pathname === "/settings"}>Settings</a>
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
  :global(body) {
    margin: 0;
    background: #0b1118;
    color: #e5e7eb;
    font-family: Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
  }

  .app-shell {
    min-height: 100vh;
    display: grid;
    grid-template-columns: 280px 1fr;
  }

  .sidebar {
    border-right: 1px solid #1f2937;
    padding: 20px 18px;
    background: #0d141c;
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
    background: linear-gradient(135deg, #5b7cff, #7c3aed);
    display: grid;
    place-items: center;
    font-weight: 700;
  }

  .brand h1 {
    margin: 0;
    font-size: 1.8rem;
    line-height: 1.1;
  }

  .brand p {
    margin: 4px 0 0;
    color: #94a3b8;
    font-size: 0.95rem;
  }

  .nav {
    display: grid;
    gap: 8px;
    margin: 16px 0 24px;
  }

  .nav a {
    color: #cbd5e1;
    text-decoration: none;
    padding: 10px 12px;
    border-radius: 10px;
    transition: background 0.15s ease;
  }

  .nav a:hover {
    background: #182230;
  }

  .nav a.active {
    background: #1b2736;
    color: #ffffff;
    font-weight: 600;
  }

  .status-card {
    border: 1px solid #243244;
    border-radius: 14px;
    padding: 12px;
    background: #111a25;
  }

  .status-card h2 {
    margin: 0 0 6px;
    font-size: 0.82rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #94a3b8;
  }

  .status-card p {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 600;
  }

  .content {
    padding: 28px 30px;
  }
</style>