<script>
  import { onMount } from "svelte";
  import { ChevronLeft, ChevronRight } from "lucide-svelte";
  import { folders, people, screen, tasks, vaultPath, sidebarCollapsed, toggleSidebar, spellCheck } from "../stores.js";
  import {
    createVaultBackup,
    getAppSettings,
    getVaultPath,
    listFolders,
    listPeople,
    listTasks,
    pickVault,
    saveAppSettings,
    setVaultPath,
  } from "../api.js";

  let changingVault = $state(false);
  let creatingBackup = $state(false);
  let savingArchive = $state(false);
  let message = $state("");
  let error = $state("");
  let autoArchiveDone = $state(true);
  let autoArchiveDays = $state(7);
  let stale1on1Days = $state(14);
  let spellCheckEnabled = $state(true);
  let savingSpellCheck = $state(false);

  async function refreshVaultData() {
    people.set(await listPeople());
    tasks.set(await listTasks());
    folders.set(await listFolders());
    vaultPath.set(await getVaultPath());
  }

  async function chooseVault() {
    changingVault = true;
    error = "";
    message = "";
    try {
      const selected = await pickVault();
      if (!selected) return;
      await setVaultPath(selected);
      await refreshVaultData();
      const settings = await getAppSettings();
      autoArchiveDone = settings.auto_archive_done;
      autoArchiveDays = settings.auto_archive_days;
      stale1on1Days = settings.stale_1on1_days;
      spellCheckEnabled = settings.spell_check ?? true;
      message = "Vault location updated.";
    } catch (err) {
      error = err?.message || String(err);
    } finally {
      changingVault = false;
    }
  }

  async function backupVault() {
    creatingBackup = true;
    error = "";
    message = "";
    try {
      const result = await createVaultBackup();
      message = `Backup created at ${result.path}`;
    } catch (err) {
      error = err?.message || String(err);
    } finally {
      creatingBackup = false;
    }
  }

  async function saveArchiveSettings() {
    savingArchive = true;
    error = "";
    message = "";
    try {
      const saved = await saveAppSettings({
        auto_archive_done: autoArchiveDone,
        auto_archive_days: Math.max(1, Number(autoArchiveDays) || 7),
        stale_1on1_days: Math.max(1, Number(stale1on1Days) || 14),
        spell_check: spellCheckEnabled,
      });
      autoArchiveDone = saved.auto_archive_done;
      autoArchiveDays = saved.auto_archive_days;
      stale1on1Days = saved.stale_1on1_days;
      spellCheckEnabled = saved.spell_check ?? true;
      tasks.set(await listTasks());
      message = "Settings updated.";
    } catch (err) {
      error = err?.message || String(err);
    } finally {
      savingArchive = false;
    }
  }

  async function saveSpellCheck() {
    savingSpellCheck = true;
    error = "";
    message = "";
    try {
      const saved = await saveAppSettings({
        auto_archive_done: autoArchiveDone,
        auto_archive_days: Math.max(1, Number(autoArchiveDays) || 7),
        stale_1on1_days: Math.max(1, Number(stale1on1Days) || 14),
        spell_check: spellCheckEnabled,
      });
      spellCheckEnabled = saved.spell_check ?? true;
      spellCheck.set(spellCheckEnabled);
      message = "Settings updated.";
    } catch (err) {
      error = err?.message || String(err);
    } finally {
      savingSpellCheck = false;
    }
  }

  onMount(async () => {
    try {
      const settings = await getAppSettings();
      autoArchiveDone = settings.auto_archive_done;
      autoArchiveDays = settings.auto_archive_days;
      stale1on1Days = settings.stale_1on1_days;
      spellCheckEnabled = settings.spell_check ?? true;
    } catch (err) {
      error = err?.message || String(err);
    }
  });
</script>

<header>
  <div class="header-left">
    <button class="sidebar-toggle-btn" onclick={toggleSidebar} title={$sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"} aria-label={$sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}>
      <span class="sidebar-toggle-mark" class:sidebar-toggle-mark--collapsed={$sidebarCollapsed}>
        <ChevronLeft size={16} strokeWidth={1.8} />
      </span>
    </button>
    <div>
      <h1>Settings</h1>
      <p>Manage your vault location, sync across computers, and configure app preferences.</p>
    </div>
  </div>
</header>

<div class="scroll body">
  <section class="card">
    <div class="card-head">
      <div>
        <div class="mono-label">VAULT LOCATION</div>
        <div class="card-title">Current vault</div>
      </div>
      <button class="ghost-btn" onclick={chooseVault} disabled={changingVault}>
        {changingVault ? "Choosing…" : "Choose folder"}
      </button>
    </div>
    <div class="path-box">{$vaultPath || "No vault selected yet."}</div>
    <div class="card-copy">
      Your vault is a plain folder of markdown files. To sync across computers, choose a folder inside <strong>Google Drive</strong>, <strong>iCloud Drive</strong>, or <strong>Dropbox</strong>. Install the same sync app on each machine, choose the same folder, and your notes stay in sync automatically.
    </div>
  </section>

  <section class="card">
    <div class="card-head">
      <div>
        <div class="mono-label">ARCHIVE</div>
        <div class="card-title">Done task cleanup</div>
      </div>
      <button class="ghost-btn" onclick={saveArchiveSettings} disabled={savingArchive}>
        {savingArchive ? "Saving…" : "Save setting"}
      </button>
    </div>
    <div class="archive-grid">
      <label class="toggle-row">
        <input type="checkbox" bind:checked={autoArchiveDone} />
        <span>Automatically archive older completed tasks</span>
      </label>
      <label class="days-field">
        <span>Archive after days</span>
        <input type="number" min="1" bind:value={autoArchiveDays} disabled={!autoArchiveDone} />
      </label>
    </div>
    <div class="card-copy">
      When enabled, completed tasks move into the archive automatically once they reach the age you set.
    </div>
  </section>

  <section class="card">
    <div class="card-head">
      <div>
        <div class="mono-label">1:1 CADENCE</div>
        <div class="card-title">Dashboard attention threshold</div>
      </div>
      <button class="ghost-btn" onclick={saveArchiveSettings} disabled={savingArchive}>
        {savingArchive ? "Saving…" : "Save setting"}
      </button>
    </div>
    <div class="archive-grid">
      <label class="days-field">
        <span>Flag people after days without a 1:1</span>
        <input type="number" min="1" bind:value={stale1on1Days} />
      </label>
    </div>
    <div class="card-copy">
      The dashboard will surface people whose last logged 1:1 is older than this threshold, plus anyone with no 1:1 logged yet.
    </div>
  </section>

  <section class="card">
    <div class="card-head">
      <div>
        <div class="mono-label">EDITOR</div>
        <div class="card-title">Text editing</div>
      </div>
      <button class="ghost-btn" onclick={saveSpellCheck} disabled={savingSpellCheck}>
        {savingSpellCheck ? "Saving…" : "Save setting"}
      </button>
    </div>
    <label class="toggle-row">
      <input type="checkbox" bind:checked={spellCheckEnabled} />
      <span>Enable spell check and autocorrect</span>
    </label>
    <div class="card-copy">
      When disabled, the app won't underline or auto-correct your text. Useful if autocorrect is disruptive when typing names and short notes.
    </div>
  </section>

  <section class="card">
    <div class="card-head">
      <div>
        <div class="mono-label">BACKUP</div>
        <div class="card-title">Create a snapshot</div>
      </div>
      <button class="ghost-btn" onclick={backupVault} disabled={creatingBackup}>
        {creatingBackup ? "Creating…" : "Create backup"}
      </button>
    </div>
    <div class="card-copy">
      This creates a timestamped copy of your current vault in a sibling <code>SideEye Backups</code> folder.
    </div>
  </section>

  {#if message}
    <div class="notice notice--ok">{message}</div>
  {/if}

  {#if error}
    <div class="notice notice--error">{error}</div>
  {/if}

  <button class="text-btn back-btn" onclick={() => screen.set("dashboard")}>Back to dashboard</button>
</div>

<style>
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 22px 32px 18px;
    border-bottom: 1px solid var(--line);
  }
  .header-left {
    display: flex;
    align-items: flex-start;
    gap: 14px;
  }
  .sidebar-toggle-btn {
    border: 1px solid var(--line);
    background: rgba(251, 247, 240, 0.94);
    width: 30px;
    height: 30px;
    min-width: 30px;
    min-height: 30px;
    box-sizing: border-box;
    border-radius: 10px;
    color: var(--muted-2);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
    padding: 0;
    cursor: pointer;
    box-shadow: 0 6px 18px rgba(58, 53, 45, 0.08);
  }
  .sidebar-toggle-btn:hover,
  .sidebar-toggle-btn:focus-visible {
    background: #f2eadb;
    color: var(--ink);
    outline: none;
  }
  .sidebar-toggle-mark {
    width: 16px;
    height: 16px;
    display: block;
    transition: transform 0.16s ease;
  }
  .sidebar-toggle-mark--collapsed {
    transform: rotate(180deg);
  }
  .sidebar-toggle-mark :global(svg) {
    width: 100%;
    height: 100%;
    display: block;
  }
  h1 {
    font-family: var(--serif);
    font-size: 28px;
    line-height: 1;
    margin: 0;
    font-weight: 500;
  }
  header p {
    font-size: 13px;
    color: var(--muted-2);
    margin: 4px 0 0;
    max-width: 640px;
  }
  .body {
    flex: 1;
    padding: 24px 32px 40px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 18px;
  }
  .card {
    border: 1px solid var(--line);
    border-radius: 16px;
    background: var(--panel);
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: min(100%, 760px);
  }
  .card-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }
  .card-title {
    margin-top: 6px;
    font-family: var(--serif);
    font-size: 22px;
    color: var(--ink);
  }
  .card-copy {
    font-size: 13px;
    line-height: 1.6;
    color: var(--muted);
    max-width: 620px;
  }
  .path-box {
    padding: 14px 16px;
    border-radius: 12px;
    border: 1px solid var(--line-2);
    background: var(--card);
    color: var(--ink-2);
    font-size: 13px;
    line-height: 1.5;
    word-break: break-word;
  }
  .archive-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 180px;
    gap: 14px;
    align-items: end;
  }
  .toggle-row {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--ink-2);
    font-size: 13px;
  }
  .days-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.12em;
    color: var(--faint);
  }
  .days-field input {
    border: 1px solid var(--line-2);
    border-radius: 8px;
    padding: 10px 11px;
    font-size: 14px;
    background: var(--card);
    color: var(--ink);
  }
  .days-field input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .notice {
    width: min(100%, 760px);
    padding: 12px 14px;
    border-radius: 12px;
    font-size: 13px;
    line-height: 1.5;
  }
  .notice--ok {
    background: #eef3ea;
    color: #46604a;
    border: 1px solid #cfdcca;
  }
  .notice--error {
    background: #fbefed;
    color: #9b4e46;
    border: 1px solid #e6c4be;
  }
  .back-btn {
    align-self: center;
    margin-top: 4px;
  }
  @media (max-width: 760px) {
    .archive-grid,
    .card-head {
      grid-template-columns: 1fr;
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
