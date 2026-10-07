<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";

  type GameSummary = {
    id: string;
    steam_app_id: number;
    title: string;
  };

  type EvidenceRecord = {
    id: number;
    game_id: string;
    source_type: string;
    source_name: string;
    source_url: string | null;
    claim_type: string;
    claim: string;
    confidence: string | null;
    captured_at: number;
    metadata_json: string | null;
  };

  type PcgwSyncResult = {
    evidence: EvidenceRecord[];
    from_cache: boolean;
  };

  export let game: GameSummary;
  export let onclose: () => void;

  let evidence: EvidenceRecord[] = [];
  let loading = false;
  let refreshing = false;
  let error = "";
  let refreshMessage = "";
  let loadedGameId = "";

  $: if (game?.id && game.id !== loadedGameId) {
    loadedGameId = game.id;
    void loadEvidence();
  }

  $: conflicts = findConflictingClaimTypes(evidence);
  $: hasPcgwEvidence = evidence.some((item) => item.source_name === "PCGamingWiki");

  async function loadEvidence() {
    loading = true;
    error = "";
    try {
      evidence = await invoke<EvidenceRecord[]>("get_game_evidence", {
        gameId: game.id,
      });
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function refreshPcGamingWiki(forceRefresh: boolean) {
    refreshing = true;
    error = "";
    refreshMessage = "";

    try {
      const result = await invoke<PcgwSyncResult>("refresh_pcgamingwiki_evidence", {
        gameId: game.id,
        steamAppId: game.steam_app_id,
        forceRefresh,
      });

      refreshMessage = result.from_cache
        ? "Using recent PCGamingWiki evidence already stored locally."
        : `Stored ${result.evidence.length} new PCGamingWiki evidence item${result.evidence.length === 1 ? "" : "s"}.`;

      await loadEvidence();
    } catch (err) {
      error = String(err);
    } finally {
      refreshing = false;
    }
  }

  function findConflictingClaimTypes(items: EvidenceRecord[]): string[] {
    const claims = new Map<string, Set<string>>();

    for (const item of items) {
      if (
        item.claim_type === "source_lookup" ||
        item.claim_type === "source_incomplete"
      ) {
        continue;
      }

      const values = claims.get(item.claim_type) ?? new Set<string>();
      values.add(item.claim.trim().toLowerCase());
      claims.set(item.claim_type, values);
    }

    return [...claims.entries()]
      .filter(([, values]) => values.size > 1)
      .map(([claimType]) => claimType);
  }

  function formatCapturedAt(timestamp: number): string {
    if (!timestamp) return "Unknown time";
    return new Date(timestamp * 1000).toLocaleString();
  }

  function formatClaimType(value: string): string {
    return value
      .split("_")
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(" ");
  }

  function formatSourceType(value: string): string {
    if (value === "local_verification") return "Local verification";
    if (value === "reference_lookup") return "Reference lookup";
    if (value === "reference") return "External reference";
    return formatClaimType(value);
  }

  function confidenceLabel(value: string | null): string {
    if (!value) return "No confidence assigned";
    return `${value.charAt(0).toUpperCase() + value.slice(1)} confidence`;
  }

  async function openSource(url: string) {
    try {
      await openUrl(url);
    } catch (err) {
      error = `Could not open source: ${String(err)}`;
    }
  }
</script>

<button class="backdrop" onclick={onclose} aria-label="Close evidence panel"></button>

<aside class="evidence-panel" role="dialog" aria-modal="true" aria-labelledby="evidence-title">
  <header class="panel-header">
    <div>
      <p class="eyebrow">Preservation evidence</p>
      <h2 id="evidence-title">{game.title}</h2>
      <p class="app-id">Steam AppID {game.steam_app_id}</p>
    </div>
    <button class="close-btn" onclick={onclose} aria-label="Close evidence panel">×</button>
  </header>

  <section class="assessment-card">
    <div class="assessment-heading">
      <span>Preservation assessment</span>
      <strong>Unknown</strong>
    </div>
    <p>
      Liberarme has not calculated a preservation status for this game yet.
      Evidence below is source material, not proof of launcher independence.
    </p>
  </section>

  <section class="toolbar">
    <div>
      <h3>Evidence</h3>
      <p>{evidence.length} stored item{evidence.length === 1 ? "" : "s"}</p>
    </div>
    <button
      class="refresh-btn"
      disabled={refreshing}
      onclick={() => refreshPcGamingWiki(hasPcgwEvidence)}
    >
      {refreshing
        ? "Checking..."
        : hasPcgwEvidence
          ? "Refresh PCGamingWiki"
          : "Check PCGamingWiki"}
    </button>
  </section>

  {#if refreshMessage}
    <p class="refresh-message">{refreshMessage}</p>
  {/if}

  {#if error}
    <p class="panel-error">{error}</p>
  {/if}

  {#if conflicts.length > 0}
    <div class="conflict-note">
      <strong>Conflicting evidence is present.</strong>
      <span>
        Liberarme is keeping every claim visible rather than choosing one automatically.
      </span>
    </div>
  {/if}

  {#if loading}
    <p class="empty-copy">Loading evidence...</p>
  {:else if evidence.length === 0}
    <div class="empty-evidence">
      <strong>No evidence stored yet.</strong>
      <p>
        Check PCGamingWiki to collect external reference evidence. Local verification
        history will also appear here when that workflow is added.
      </p>
    </div>
  {:else}
    <div class="evidence-list">
      {#each [...evidence].reverse() as item (item.id)}
        <article
          class="evidence-item"
          class:conflicting={conflicts.includes(item.claim_type)}
        >
          <div class="evidence-meta">
            <span class="source-type">{formatSourceType(item.source_type)}</span>
            <span>{formatCapturedAt(item.captured_at)}</span>
          </div>

          <h4>{formatClaimType(item.claim_type)}</h4>
          <p class="claim">{item.claim}</p>

          <div class="evidence-footer">
            <div>
              <span class="source-name">{item.source_name}</span>
              <span class="confidence">{confidenceLabel(item.confidence)}</span>
            </div>

            {#if item.source_url}
              <button class="source-link" onclick={() => item.source_url && openSource(item.source_url)}>
                Open source
              </button>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  {/if}
</aside>

<style>
  .backdrop {
    position: fixed;
    border: 0;
    padding: 0;
    inset: 0;
    background: rgba(3, 7, 12, 0.62);
    z-index: 40;
  }

  .evidence-panel {
    position: fixed;
    top: 0;
    right: 0;
    z-index: 41;
    width: min(560px, calc(100vw - 32px));
    height: 100vh;
    box-sizing: border-box;
    overflow-y: auto;
    padding: 24px;
    background: var(--archive-raised);
    border-left: 1px solid var(--border-line);
    box-shadow: -18px 0 48px rgba(0, 0, 0, 0.35);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    gap: 18px;
    align-items: flex-start;
    margin-bottom: 20px;
  }

  .eyebrow {
    margin: 0 0 5px;
    color: var(--accent-text-soft);
    text-transform: uppercase;
    letter-spacing: 0.09em;
    font-size: 0.76rem;
    font-weight: 700;
  }

  h2 {
    margin: 0;
    font-size: 1.65rem;
  }

  .app-id {
    margin: 6px 0 0;
    color: var(--slate-ash);
    font-size: 0.88rem;
  }

  .close-btn {
    border: 1px solid var(--border-line);
    border-radius: 9px;
    width: 36px;
    height: 36px;
    background: var(--archive-card);
    color: var(--paper-white);
    font-size: 1.35rem;
    cursor: pointer;
  }

  .assessment-card {
    margin-bottom: 22px;
    padding: 16px;
    border: 1px solid var(--border-line);
    border-radius: 12px;
    background: var(--archive-well);
  }

  .assessment-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 9px;
    color: var(--slate-ash-bright);
    font-size: 0.9rem;
  }

  .assessment-heading strong {
    padding: 4px 9px;
    border-radius: 999px;
    background: rgba(148, 163, 184, 0.12);
    color: var(--paper-white);
  }

  .assessment-card p {
    margin: 0;
    color: var(--slate-ash);
    line-height: 1.5;
    font-size: 0.92rem;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 12px;
  }

  .toolbar h3 {
    margin: 0;
    font-size: 1.05rem;
  }

  .toolbar p {
    margin: 3px 0 0;
    color: var(--slate-ash);
    font-size: 0.84rem;
  }

  .refresh-btn,
  .source-link {
    border: 1px solid var(--case-file-indigo);
    border-radius: 9px;
    padding: 8px 11px;
    background: rgba(var(--case-file-indigo-rgb), 0.12);
    color: var(--accent-text-soft);
    font-weight: 600;
    cursor: pointer;
  }

  .refresh-btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .refresh-message {
    margin: 0 0 12px;
    color: var(--success);
    font-size: 0.88rem;
  }

  .panel-error {
    margin: 0 0 12px;
    color: var(--danger);
    font-size: 0.88rem;
  }

  .conflict-note {
    display: grid;
    gap: 4px;
    margin-bottom: 14px;
    padding: 12px;
    border: 1px solid #8b5e26;
    border-radius: 10px;
    background: rgba(139, 94, 38, 0.11);
  }

  .conflict-note strong {
    color: #f5c16c;
    font-size: 0.9rem;
  }

  .conflict-note span {
    color: var(--slate-ash-bright);
    font-size: 0.84rem;
    line-height: 1.4;
  }

  .empty-copy,
  .empty-evidence {
    color: var(--slate-ash);
  }

  .empty-evidence {
    padding: 28px 18px;
    text-align: center;
    border: 1px dashed var(--border-dashed);
    border-radius: 12px;
  }

  .empty-evidence strong {
    color: var(--paper-white);
  }

  .empty-evidence p {
    margin: 8px auto 0;
    max-width: 400px;
    line-height: 1.5;
  }

  .evidence-list {
    display: grid;
    gap: 12px;
    padding-bottom: 32px;
  }

  .evidence-item {
    padding: 14px;
    border: 1px solid var(--border-line);
    border-radius: 12px;
    background: var(--archive-card);
  }

  .evidence-item.conflicting {
    border-color: #8b5e26;
  }

  .evidence-meta,
  .evidence-footer,
  .evidence-footer > div {
    display: flex;
    align-items: center;
    gap: 9px;
  }

  .evidence-meta {
    justify-content: space-between;
    color: var(--slate-ash);
    font-size: 0.78rem;
  }

  .source-type {
    color: var(--accent-text-soft);
    font-weight: 600;
  }

  .evidence-item h4 {
    margin: 10px 0 5px;
    font-size: 0.92rem;
  }

  .claim {
    margin: 0;
    line-height: 1.5;
    color: var(--slate-ash-bright);
  }

  .evidence-footer {
    justify-content: space-between;
    margin-top: 13px;
    padding-top: 11px;
    border-top: 1px solid var(--border-hairline);
  }

  .evidence-footer > div {
    flex-wrap: wrap;
  }

  .source-name {
    color: var(--paper-white);
    font-size: 0.82rem;
    font-weight: 600;
  }

  .confidence {
    color: var(--slate-ash);
    font-size: 0.78rem;
  }

  .source-link {
    flex-shrink: 0;
    border-color: var(--border-line-soft);
    background: transparent;
    color: var(--slate-ash-bright);
    font-size: 0.78rem;
  }

  @media (max-width: 620px) {
    .evidence-panel {
      width: 100vw;
      padding: 18px;
    }

    .toolbar,
    .evidence-footer {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
