<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { toUserMessage } from "$lib/errors";
  import { validateSteamApiKey, validateSteamId64 } from "$lib/steamValidation";

  const STEAM_API_KEY_URL = "https://steamcommunity.com/dev/apikey";

  function openSteamApiKeyPage(event: MouseEvent) {
    event.preventDefault();
    openUrl(STEAM_API_KEY_URL).catch(() => {
      window.open(STEAM_API_KEY_URL, "_blank", "noopener,noreferrer");
    });
  }

  let steamApiKey = "";
  let steamId64 = "";
  let steamIdHelper = "";

  let loading = false;
  let saving = false;
  let statusMessage = "";
  let errorMessage = "";

  let apiKeyTouched = false;
  let steamIdTouched = false;
  let submitAttempted = false;
  let helperLoading = false;
  let autoSavingApiKey = false;
  let apiKeySaved = false;

  type SteamSettings = {
    api_key: string | null;
    steam_id64: string | null;
  };

  type ResolveSteamIdResult = {
    success: boolean;
    steam_id64: string | null;
    error: string | null;
  };

  $: apiKeyErrorRaw = validateSteamApiKey(steamApiKey);
  $: steamIdErrorRaw = validateSteamId64(steamId64);

  $: apiKeyError = (apiKeyTouched || submitAttempted) && !!apiKeyErrorRaw;
  $: steamIdError = (steamIdTouched || submitAttempted) && !!steamIdErrorRaw;

  $: canSubmit = !loading && !saving && !apiKeyErrorRaw && !steamIdErrorRaw;

  async function loadSettings() {
    loading = true;
    statusMessage = "";
    errorMessage = "";

    try {
      const result = await invoke<SteamSettings>("get_steam_settings");
      steamApiKey = result.api_key ?? "";
      steamId64 = result.steam_id64 ?? "";

      if (steamApiKey) {
        apiKeySaved = true;
      }
    } catch (error) {
      errorMessage = toUserMessage(
        error,
        "Couldn't load your saved settings. Try reopening Settings.",
      );
    } finally {
      loading = false;
    }
  }

  // auto-save api key (for vanity resolve and app requests)
  async function autoSaveApiKey() {
    if (apiKeyErrorRaw) {
      return; // don't save if validation failed
    }

    autoSavingApiKey = true;
    try {
      await invoke("set_steam_settings", {
        apiKey: steamApiKey,
        steamId64: steamId64 || "", // use existing or empty
      });
      apiKeySaved = true;
    } catch (error) {
      errorMessage = toUserMessage(
        error,
        "Couldn't save your API key. Check your connection and try again.",
      );
      apiKeySaved = false;
    } finally {
      autoSavingApiKey = false;
    }
  }

  async function saveSettings() {
    statusMessage = "";
    errorMessage = "";

    if (apiKeyErrorRaw || steamIdErrorRaw) {
      errorMessage =
        apiKeyErrorRaw ?? steamIdErrorRaw ?? "Please fix validation errors.";
      return;
    }

    saving = true;
    try {
      await invoke("set_steam_settings", {
        apiKey: steamApiKey,
        steamId64,
      });
      statusMessage = "Settings saved locally.";
    } catch (error) {
      errorMessage = toUserMessage(
        error,
        "Couldn't save your settings. Check your connection and try again.",
      );
    } finally {
      saving = false;
    }
  }

  async function resolveVanityId() {
    errorMessage = "";
    statusMessage = "";

    if (!steamIdHelper.trim()) {
      errorMessage = "Please enter a Steam profile URL or SteamID64.";
      return;
    }

    helperLoading = true;
    try {
      const result = await invoke<ResolveSteamIdResult>("resolve_steam_id", {
        input: steamIdHelper,
      });

      if (result.success && result.steam_id64) {
        steamId64 = result.steam_id64;
        steamIdHelper = "";
        statusMessage = "SteamID64 resolved successfully!";
      } else {
        errorMessage = result.error || "Failed to resolve SteamID64.";
      }
    } catch (error) {
      errorMessage = toUserMessage(
        error,
        "Couldn't resolve that profile. Double-check the URL or username and try again.",
      );
    } finally {
      helperLoading = false;
    }
  }

  onMount(loadSettings);
</script>

<section class="page-header">
  <div>
    <h1>Settings</h1>
    <p>Configure Steam credentials (stored locally).</p>
  </div>
</section>

<section class="form-card">
  {#if loading}
    <p class="muted">Loading settings...</p>
  {/if}

  <div class="field">
    <label for="steam-api-key">Steam API Key <span class="required-marker">Required</span></label>
    <p class="field-hint">
      Used to fetch your library from Steam's Web API. <br/>
      <a class="inline-link" href={STEAM_API_KEY_URL} onclick={openSteamApiKeyPage}>Get your key from Steam →</a>
    </p>
    <div class="input-wrapper">
      <input
        id="steam-api-key"
        type="password"
        bind:value={steamApiKey}
        oninput={() => {
          apiKeySaved = false;
        }}
        onblur={() => {
          apiKeyTouched = true;
          if (!apiKeyErrorRaw) {
            autoSaveApiKey();
          }
        }}
        autocomplete="off"
        spellcheck="false"
        placeholder="Enter your Steam Web API key"
        aria-invalid={apiKeyError ? "true" : "false"}
        aria-describedby={apiKeyError ? "steam-api-key-error" : undefined}
        aria-required="true"
      />

      {#if apiKeySaved && !autoSavingApiKey && !apiKeyError}
        <div class="checkmark-wrapper" title="Successfully saved">
          <svg
            class="checkmark"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            aria-hidden="true"
          >
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
        </div>
      {/if}
    </div>
    {#if apiKeyError}
      <p class="field-error" id="steam-api-key-error">{apiKeyErrorRaw}</p>
    {/if}
    {#if autoSavingApiKey}
      <p class="field-hint">Saving...</p>
    {/if}
  </div>

  <div class="field">
    <label for="steam-id-helper">SteamID64 Helper</label>
    <p class="field-hint">Paste your Steam profile URL or username below to resolve SteamID64.</p>
    <div class="helper-row">
      <input
        id="steam-id-helper"
        type="text"
        bind:value={steamIdHelper}
        autocomplete="off"
        spellcheck="false"
        placeholder="steamcommunity.com/id/username or username"
        disabled={helperLoading}
      />
      <button
        class="helper-btn"
        onclick={resolveVanityId}
        disabled={helperLoading || !steamIdHelper.trim()}
        title="Resolve Steam profile URL to numeric SteamID64"
      >
        {helperLoading ? "Resolving..." : "Resolve"}
      </button>
    </div>
  </div>

  <div class="field">
    <label for="steam-id64">SteamID64 <span class="required-marker">Required</span></label>
    <p class="field-hint">Must be exactly 17 numeric digits. Use the helper above if you only know your profile URL.</p>
    <input
      id="steam-id64"
      type="text"
      bind:value={steamId64}
      onblur={() => (steamIdTouched = true)}
      autocomplete="off"
      spellcheck="false"
      placeholder="76561198123456789"
      aria-invalid={steamIdError ? "true" : "false"}
      aria-describedby={steamIdError && steamIdTouched ? "steam-id64-error" : undefined}
      aria-required="true"
    />
    {#if steamIdError && steamIdTouched}
      <p class="field-error" id="steam-id64-error">{steamIdErrorRaw}</p>
    {/if}
  </div>

  <button class="save-btn" onclick={saveSettings} disabled={!canSubmit}>
    {saving ? "Saving..." : "Save Settings"}
  </button>

  {#if statusMessage}
    <p class="success" role="status">{statusMessage}</p>
  {/if}

  {#if errorMessage}
    <p class="error" role="alert">{errorMessage}</p>
  {/if}
</section>

<style>
  .page-header h1 {
    margin: 0;
    font-size: 2.2rem;
  }

  .page-header p {
    margin: 6px 0 0;
    color: var(--slate-ash);
    font-size: 1.05rem;
  }

  .form-card {
    margin-top: 18px;
    max-width: 620px;
    border: 1px solid var(--border-line);
    border-radius: 14px;
    padding: 16px;
    background: var(--archive-card);
  }

  .field {
    display: grid;
    gap: 8px;
    margin-bottom: 18px;
  }

  label {
    color: var(--slate-ash-bright);
    font-weight: 600;
    font-size: 0.95rem;
  }

  .required-marker {
    margin-left: 6px;
    font-size: 0.78rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--slate-ash);
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
  }

  .input-wrapper input {
    padding-right: 44px;
  }

  input {
    background: var(--archive-well);
    border: 1px solid var(--border-line-soft);
    color: var(--paper-white);
    border-radius: 10px;
    padding: 10px 12px;
    font: inherit;
    flex: 1;
    min-width: 0;
  }

  input:focus {
    outline: none;
    border-color: var(--case-file-indigo);
    box-shadow: 0 0 0 2px rgba(var(--case-file-indigo-rgb), 0.2);
  }

  input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .checkmark-wrapper {
    position: absolute;
    right: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    color: var(--success);
    pointer-events: none; /* Don't interfere with input */
  }

  .checkmark {
    width: 20px;
    height: 20px;
    animation: checkmarkPulse 0.3s ease-out;
  }

  @keyframes checkmarkPulse {
    0% {
      opacity: 0;
      transform: scale(0.8);
    }
    100% {
      opacity: 1;
      transform: scale(1);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .checkmark {
      animation: none;
    }
  }

  .helper-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px;
  }

  .helper-btn {
    border: none;
    border-radius: 10px;
    padding: 10px 16px;
    background: rgba(var(--case-file-indigo-rgb), 0.2);
    color: var(--accent-text-soft);
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease;
  }

  .helper-btn:hover:not(:disabled) {
    background: rgba(var(--case-file-indigo-rgb), 0.3);
  }

  .helper-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .save-btn {
    border: none;
    border-radius: 12px;
    padding: 10px 14px;
    color: var(--on-accent);
    font-weight: 700;
    background: linear-gradient(135deg, var(--case-file-indigo-deep), var(--case-file-violet));
    cursor: pointer;
    margin-top: 8px;
  }

  .save-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .muted {
    color: var(--slate-ash);
    margin-top: 0;
  }

  .success {
    color: var(--success);
    margin-top: 12px;
    margin-bottom: 0;
    font-size: 0.95rem;
  }

  .error {
    color: var(--danger);
    margin-top: 12px;
    margin-bottom: 0;
    font-size: 0.95rem;
  }

  .field-error {
    margin: 0;
    color: var(--danger-soft);
    font-size: 0.9rem;
  }

  .field-hint {
    margin: 0;
    color: var(--slate-ash);
    font-size: 0.85rem;
  }

  .inline-link {
    color: var(--accent-text-soft);
    font-weight: 600;
    text-decoration: none;
  }

  .inline-link:hover {
    text-decoration: underline;
  }

  input[aria-invalid="true"] {
    border-color: var(--danger);
  }

  @media (max-width: 480px) {
    .helper-row {
      grid-template-columns: 1fr;
    }
  }
</style>
