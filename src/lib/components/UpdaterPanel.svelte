<script>
  import { onMount, onDestroy } from "svelte";
  import { screen } from "../stores.js";
  import { checkForUpdate, getUpdaterStatus, installUpdate } from "../api.js";
  import { Settings, Info } from "lucide-svelte";

  let { compact = false } = $props();
  let loading = $state(true);
  let checking = $state(false);
  let installing = $state(false);
  let modalOpen = $state(false);
  let status = $state({ currentVersion: "", configured: false, endpoint: null });
  let updateInfo = $state(null);
  let message = $state("");
  let error = $state("");

  async function refreshStatus({ autoCheck = false } = {}) {
    loading = true;
    error = "";
    try {
      status = await getUpdaterStatus();
      if (autoCheck && status.configured) {
        await handleCheck(true);
      }
    } catch (err) {
      error = err?.message || String(err);
    } finally {
      loading = false;
    }
  }

  async function handleCheck(silent = false) {
    checking = true;
    if (!silent) {
      error = "";
      message = "";
      updateInfo = null;
    }

    try {
      const result = await checkForUpdate();
      updateInfo = result;
      if (result) {
        message = `Update available: v${result.version}`;
      } else if (!silent) {
        message = "You're on the latest version.";
      }
    } catch (err) {
      if (!silent) error = err?.message || String(err);
    } finally {
      checking = false;
    }
  }

  async function handleInstall() {
    if (!updateInfo || installing) return;
    if (!confirm(`Install version ${updateInfo.version} now? The app will restart when the update finishes.`)) {
      return;
    }

    installing = true;
    error = "";
    message = "Downloading update…";
    try {
      await installUpdate();
    } catch (err) {
      error = err?.message || String(err);
      message = "";
      installing = false;
    }
  }

  const FOUR_HOURS = 4 * 60 * 60 * 1000;
  let interval;

  onMount(() => {
    refreshStatus({ autoCheck: true });
    interval = setInterval(() => handleCheck(true), FOUR_HOURS);
  });

  onDestroy(() => clearInterval(interval));
</script>

<section class="updates">
  {#if updateInfo && !compact}
    <div class="update-banner">
      <div class="update-banner-text">
        <span class="update-banner-label">Update available</span>
        <span class="update-banner-version">v{updateInfo.version}</span>
      </div>
      <button class="update-banner-btn" onclick={handleInstall} disabled={installing}>
        {installing ? "Installing…" : "Install"}
      </button>
    </div>
  {/if}
  <div class="version-row" class:version-row--compact={compact}>
    <button class="settings-link" class:settings-link--compact={compact} onclick={() => screen.set("settings")} title="Settings" aria-label="Settings">
      {#if compact}
        <Settings size={20} strokeWidth={1.8} aria-hidden="true" />
      {:else}
        Settings
      {/if}
    </button>
    <button class="version-trigger" class:version-trigger--compact={compact} onclick={() => (modalOpen = true)} title="App info and updates" aria-label="App info and updates">
      {#if compact}
        <Info size={20} strokeWidth={1.8} aria-hidden="true" />
      {:else}
        <span class="version-text">v{status.currentVersion || "…"}</span>
        <span class="version-mark">i</span>
      {/if}
      {#if updateInfo}
        <span class="update-dot" aria-label="Update available"></span>
      {/if}
    </button>
  </div>

  {#if modalOpen}
    <div class="overlay" role="dialog" aria-modal="true">
      <div class="modal">
        <div class="modal-head-row">
          <div>
            <div class="modal-head">SideEye</div>
            <div class="modal-sub">Version {status.currentVersion || "Unknown"}</div>
          </div>
          <button class="close-btn" onclick={() => (modalOpen = false)} aria-label="Close">×</button>
        </div>

        <div class="modal-copy">
          Track your people, notes, and follow-ups in a local markdown vault.
        </div>

        {#if loading}
          <div class="state-copy">Loading update status…</div>
        {:else if error}
          <div class="state-error">{error}</div>
        {:else if !status.configured}
          <div class="state-copy">Updates are not configured in this build.</div>
        {:else}
          {#if updateInfo}
            <div class="update-card">
              <div class="update-title">Update ready</div>
              <div class="update-version">v{updateInfo.version}</div>
              {#if updateInfo.notes}
                <div class="update-notes">{updateInfo.notes}</div>
              {/if}
            </div>
          {:else if message}
            <div class="state-copy">{message}</div>
          {/if}

          <div class="actions">
            <button class="check-btn" onclick={() => handleCheck(false)} disabled={checking || installing}>
              {checking ? "Checking…" : "Check for updates"}
            </button>
            {#if updateInfo}
              <button class="install-btn" onclick={handleInstall} disabled={installing}>
                {installing ? "Installing…" : "Install update"}
              </button>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .updates {
    margin-top: auto;
    padding: 6px 8px 0;
    border-top: 1px solid var(--line);
  }
  .version-row {
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .version-row--compact {
    flex-direction: row;
    justify-content: center;
    gap: 4px;
    width: 100%;
  }
  .settings-link,
  .version-trigger {
    border: none;
    background: transparent;
    padding: 0;
    color: var(--ink-2);
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    cursor: pointer;
  }
  .settings-link {
    font-family: var(--mono);
    letter-spacing: 0.04em;
    color: var(--muted-2);
  }
  .version-trigger {
    margin-left: auto;
  }
  .settings-link--compact,
  .version-trigger--compact {
    flex: 1;
    padding: 4px;
    border-radius: 14px;
    justify-content: center;
    gap: 0;
    border: none;
    background: transparent;
    color: #6b6557;
    box-sizing: border-box;
    margin-left: 0;
  }
  .version-mark {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--faint);
    flex: none;
  }
  .version-text {
    min-width: 0;
    text-align: left;
    font-family: var(--mono);
    letter-spacing: 0.04em;
    color: var(--muted-2);
  }
  .settings-link:hover,
  .version-trigger:hover,
  .settings-link:focus-visible,
  .version-trigger:focus-visible {
    color: var(--ink);
    outline: none;
  }
  .settings-link--compact:hover,
  .version-trigger--compact:hover,
  .settings-link--compact:focus-visible,
  .version-trigger--compact:focus-visible {
    background: rgba(226, 220, 205, 0.72);
  }
  .update-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    background: #f5eedf;
    border: 1px solid #e0cfa8;
    border-radius: 10px;
    padding: 8px 10px;
    margin-bottom: 6px;
  }
  .update-banner-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .update-banner-label {
    font-family: var(--mono);
    font-size: 9px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .update-banner-version {
    font-size: 13px;
    font-weight: 600;
    color: var(--ink);
  }
  .update-banner-btn {
    border: none;
    background: var(--accent);
    color: white;
    border-radius: 7px;
    padding: 5px 11px;
    font-size: 12px;
    cursor: pointer;
    flex: none;
  }
  .update-banner-btn:disabled {
    opacity: 0.7;
    cursor: default;
  }
  .update-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    flex: none;
    animation: pulse 2s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(0.75); }
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(44, 42, 38, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 140;
  }
  .modal {
    width: 360px;
    max-width: calc(100vw - 48px);
    border-radius: 16px;
    background: var(--paper);
    box-shadow: 0 16px 48px rgba(44, 42, 38, 0.2);
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .modal-head-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }
  .modal-head {
    font-family: var(--serif);
    font-size: 24px;
    color: var(--ink);
  }
  .modal-sub,
  .modal-copy,
  .state-copy,
  .update-notes {
    font-size: 12px;
    line-height: 1.5;
    color: var(--muted);
  }
  .close-btn {
    border: none;
    background: transparent;
    color: var(--faint);
    font-size: 24px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
  }
  .state-error {
    font-size: 12px;
    line-height: 1.5;
    color: var(--over);
  }
  .update-card {
    border: 1px solid var(--line);
    background: #f8f3ea;
    border-radius: 12px;
    padding: 12px;
  }
  .update-title {
    font-size: 10px;
    font-family: var(--mono);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--faint);
  }
  .update-version {
    margin-top: 4px;
    font-size: 15px;
    color: var(--ink);
    font-weight: 600;
  }
  .update-notes {
    margin-top: 8px;
    white-space: pre-wrap;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .check-btn,
  .install-btn {
    border: none;
    border-radius: 8px;
    padding: 9px 12px;
    font-size: 12px;
    cursor: pointer;
  }
  .check-btn {
    background: #e7e1d3;
    color: var(--ink);
  }
  .install-btn {
    background: var(--accent);
    color: white;
  }
  .check-btn:disabled,
  .install-btn:disabled {
    opacity: 0.7;
    cursor: default;
  }
</style>
