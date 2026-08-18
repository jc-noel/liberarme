<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let steamApiKey = "";
  let steamId64 = "";

  let loading = false;
  let saving = false;
  let statusMessage = "";
  let errorMessage = "";

  type SteamSettings = {
    api_key: string | null;
    steam_id64: string | null;
  };

  async function loadSettings() {
    loading = true;
    statusMessage = "";
    errorMessage = "";

    try {
      const result = await invoke<SteamSettings>("get_steam_settings");
      steamApiKey = result.api_key ?? "";
      steamId64 = result.steam_id64 ?? "";
    } catch (error) {
      errorMessage = `Failed to load settings: ${String(error)}`;
    } finally {
      loading = false;
    }
  }

  async function saveSettings() {
    saving = true;
    statusMessage = "";
    errorMessage = "";

    try {
      await invoke("set_steam_settings", {
        apiKey: steamApiKey,
        steamId64
      });
      statusMessage = "Settings saved locally.";
    } catch (error) {
      errorMessage = `Failed to save settings: ${String(error)}`;
    } finally {
      saving = false;
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
    <input
      id="steam-api-key"
      type="password"
      bind:value={steamApiKey}
      autocomplete="off"
      spellcheck="false"
      placeholder="Enter your Steam Web API key"
    />
  </div>

  <div class="field">
    <label for="steam-id64">SteamID64</label>
    <input
      id="steam-id64"
      type="text"
      bind:value={steamId64}
      autocomplete="off"
      spellcheck="false"
      placeholder="7656119..."
    />
  </div>

  <button class="save-btn" on:click={saveSettings} disabled={saving || loading}>
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
    gap: 6px;
    margin-bottom: 14px;
  }

  label {
    color: #cbd5e1;
    font-weight: 600;
  }

  input {
    background: #0f1722;
    border: 1px solid #2b3a4d;
    color: #e5e7eb;
    border-radius: 10px;
    padding: 10px 12px;
    font: inherit;
  }

  input:focus {
    outline: none;
    border-color: #5b7cff;
    box-shadow: 0 0 0 2px rgba(91, 124, 255, 0.2);
  }

  .save-btn {
    border: none;
    border-radius: 12px;
    padding: 10px 14px;
    color: #fff;
    font-weight: 700;
    background: linear-gradient(135deg, #5b7cff, #7c3aed);
    cursor: pointer;
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
  }

  .error {
    color: #f87171;
    margin-top: 12px;
    margin-bottom: 0;
  }
</style>