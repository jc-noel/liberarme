<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

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

  function validateSteamApiKey(value: string): string | null {
    const v = value.trim();
    if (!v) return "Steam API key is required.";
    if (v.length < 8) return "Steam API key looks too short.";
    return null;
  }

  function validateSteamId64(value: string): string | null {
    const v = value.trim();
    if (!v) return "SteamID64 is required.";
    if (!/^\d{17}$/.test(v))
      return "SteamID64 must be exactly 17 numeric digits.";
    return null;
  }

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
      errorMessage = `Failed to load settings: ${String(error)}`;
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
      errorMessage = `Failed to save API key: ${String(error)}`;
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
      errorMessage = `Failed to save settings: ${String(error)}`;
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
      errorMessage = `Resolution error: ${String(error)}`;
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
    <label for="steam-api-key">Steam API Key</label>
    <div class="input-wrapper">
      <input
        id="steam-api-key"
        type="password"
        bind:value={steamApiKey}
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
      />

      {#if apiKeySaved && !autoSavingApiKey && !apiKeyError}
        <div class="checkmark-wrapper" title="Successfully saved">
          <svg
            class="checkmark"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
        </div>
      {/if}
    </div>
    {#if apiKeyError}
      <p class="field-error">{apiKeyErrorRaw}</p>
    {/if}
    {#if autoSavingApiKey}
      <p class="field-hint">Saving...</p>
    {/if}
  </div>

  <div class="field">
    <label for="steam-id64">SteamID64 Helper</label>
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
    <label for="steam-id64">SteamID64</label>
    <input
      id="steam-id64"
      type="text"
      bind:value={steamId64}
      onblur={() => (steamIdTouched = true)}
      autocomplete="off"
      spellcheck="false"
      placeholder="76561198123456789"
      aria-invalid={steamIdError ? "true" : "false"}
    />
    {#if steamIdError && steamIdTouched}
      <p class="field-error">{steamIdErrorRaw}</p>
    {/if}
  </div>

  <button class="save-btn" onclick={saveSettings} disabled={!canSubmit}>
    {saving ? "Saving..." : "Save Settings"}
  </button>

  {#if statusMessage}
    <p class="success">{statusMessage}</p>
  {/if}

  {#if errorMessage}
    <p class="error">{errorMessage}</p>
  {/if}
</section>

<style>
  .page-header h1 {
    margin: 0;
    font-size: 2.2rem;
  }

  .page-header p {
    margin: 6px 0 0;
    color: #94a3b8;
    font-size: 1.05rem;
  }

  .form-card {
    margin-top: 18px;
    max-width: 620px;
    border: 1px solid #243244;
    border-radius: 14px;
    padding: 16px;
    background: #111a25;
  }

  .field {
    display: grid;
    gap: 8px;
    margin-bottom: 18px;
  }

  label {
    color: #cbd5e1;
    font-weight: 600;
    font-size: 0.95rem;
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
  }

  input {
    background: #0f1722;
    border: 1px solid #2b3a4d;
    color: #e5e7eb;
    border-radius: 10px;
    padding: 10px 12px;
    font: inherit;
    flex: 1;
    min-width: 0;
  }

  input:focus {
    outline: none;
    border-color: #5b7cff;
    box-shadow: 0 0 0 2px rgba(91, 124, 255, 0.2);
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
    color: #34d399;
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

  .helper-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px;
  }

  .helper-btn {
    border: none;
    border-radius: 10px;
    padding: 10px 16px;
    background: rgba(91, 124, 255, 0.2);
    color: #93a9ff;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease;
  }

  .helper-btn:hover:not(:disabled) {
    background: rgba(91, 124, 255, 0.3);
  }

  .helper-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .save-btn {
    border: none;
    border-radius: 12px;
    padding: 10px 14px;
    color: #fff;
    font-weight: 700;
    background: linear-gradient(135deg, #5b7cff, #7c3aed);
    cursor: pointer;
    margin-top: 8px;
  }

  .save-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .muted {
    color: #94a3b8;
    margin-top: 0;
  }

  .success {
    color: #34d399;
    margin-top: 12px;
    margin-bottom: 0;
    font-size: 0.95rem;
  }

  .error {
    color: #f87171;
    margin-top: 12px;
    margin-bottom: 0;
    font-size: 0.95rem;
  }

  .field-error {
    margin: 0;
    color: #fca5a5;
    font-size: 0.9rem;
  }

  input[aria-invalid="true"] {
    border-color: #f87171;
  }
</style>
