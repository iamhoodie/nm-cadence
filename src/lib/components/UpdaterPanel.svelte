<script>
  import { onMount } from "svelte";
  import { checkForUpdate, getUpdaterStatus, installUpdate } from "../api.js";

  let loading = $state(true);
  let checking = $state(false);
  let installing = $state(false);
  let status = $state({ currentVersion: "", configured: false, endpoint: null });
  let updateInfo = $state(null);
  let message = $state("");
  let error = $state("");

  async function refreshStatus() {
    loading = true;
    error = "";
    try {
      status = await getUpdaterStatus();
    } catch (err) {
      error = err?.message || String(err);
    } finally {
      loading = false;
    }
  }

  async function handleCheck() {
    checking = true;
    error = "";
    message = "";
    updateInfo = null;
    try {
      const result = await checkForUpdate();
      updateInfo = result;
      if (result) {
        message = `Version ${result.version} is ready to install.`;
      } else {
        message = "You’re on the latest version.";
      }
    } catch (err) {
      error = err?.message || String(err);
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

  onMount(refreshStatus);
</script>

<section class="updates">
  <div class="updates-head">
    <span class="mono-label">UPDATES</span>
    {#if !loading && status.currentVersion}
      <span class="version">v{status.currentVersion}</span>
    {/if}
  </div>

  {#if loading}
    <div class="updates-copy">Loading updater status…</div>
  {:else if error}
    <div class="updates-error">{error}</div>
  {:else if !status.configured}
    <div class="updates-copy">
      Updater isn’t configured in this build yet.
    </div>
    <div class="updates-hint">
      Build with `CADENCE_UPDATER_ENDPOINT` and `CADENCE_UPDATER_PUBKEY`.
    </div>
  {:else}
    <div class="updates-copy">Check for a signed app update and install it in place.</div>
    {#if message}
      <div class="updates-message">{message}</div>
    {/if}
    {#if updateInfo}
      <div class="update-card">
        <div class="update-title">Update available</div>
        <div class="update-version">v{updateInfo.version}</div>
        {#if updateInfo.notes}
          <div class="update-notes">{updateInfo.notes}</div>
        {/if}
      </div>
    {/if}
    <div class="updates-actions">
      <button class="update-btn" onclick={handleCheck} disabled={checking || installing}>
        {checking ? "Checking…" : "Check for updates"}
      </button>
      {#if updateInfo}
        <button class="install-btn" onclick={handleInstall} disabled={installing}>
          {installing ? "Installing…" : "Install update"}
        </button>
      {/if}
    </div>
  {/if}
</section>

<style>
  .updates {
    margin-top: 14px;
    border-top: 1px solid var(--line);
    padding: 14px 8px 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .updates-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .version {
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--faint);
  }
  .updates-copy,
  .updates-hint,
  .updates-message,
  .update-notes {
    font-size: 12px;
    line-height: 1.5;
    color: var(--muted);
  }
  .updates-error {
    font-size: 12px;
    line-height: 1.5;
    color: var(--over);
  }
  .update-card {
    border: 1px solid var(--line);
    background: #f8f3ea;
    border-radius: 10px;
    padding: 10px 12px;
  }
  .update-title {
    font-size: 11px;
    font-family: var(--mono);
    letter-spacing: 0.08em;
    color: var(--faint);
    text-transform: uppercase;
  }
  .update-version {
    margin-top: 5px;
    font-size: 14px;
    color: var(--ink);
    font-weight: 600;
  }
  .update-notes {
    margin-top: 6px;
    white-space: pre-wrap;
  }
  .updates-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .update-btn,
  .install-btn {
    width: 100%;
    border: none;
    border-radius: 8px;
    padding: 9px 11px;
    font-size: 12px;
    cursor: pointer;
  }
  .update-btn {
    background: #e7e1d3;
    color: var(--ink);
  }
  .install-btn {
    background: var(--accent);
    color: white;
  }
  .update-btn:disabled,
  .install-btn:disabled {
    opacity: 0.7;
    cursor: default;
  }
</style>
