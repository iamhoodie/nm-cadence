<script>
  import { people, folders, screen, selectedSlug } from "../stores.js";
  import { createPerson } from "../api.js";
  import PersonCard from "./PersonCard.svelte";

  function open(slug) {
    selectedSlug.set(slug);
    screen.set("person");
  }

  const grouped = $derived(() => {
    const folderNames = [...new Set([...$folders, ...$people.map((person) => person.group).filter(Boolean)])]
      .sort((a, b) => a.localeCompare(b));
    const groups = folderNames.map((name) => ({
      key: name,
      label: name.toUpperCase(),
      items: $people.filter((person) => person.group === name),
    }));
    groups.push({
      key: "__ungrouped",
      label: "UNGROUPED",
      items: $people.filter((person) => !person.group || !folderNames.includes(person.group)),
    });
    return groups.filter((group) => group.items.length);
  });

  const COLORS = ["#6b7d9c", "#a86b5a", "#8d6480", "#a8824f", "#5d8a8a", "#7a8b5a", "#6f8f72", "#9c7b6b"];

  let adding = $state(false);
  let newName = $state("");
  let newRole = $state("");
  let newColor = $state("#6b7d9c");
  let addError = $state("");

  function openAdd() {
    adding = true;
    newName = "";
    newRole = "";
    newColor = "#6b7d9c";
    addError = "";
  }

  async function submitAdd() {
    if (!newName.trim()) {
      addError = "Name is required.";
      return;
    }
    const person = await createPerson({
      name: newName.trim(),
      role: newRole.trim(),
      cadence_weeks: 2,
      color: newColor,
    });
    people.update((list) => [...list, person].sort((a, b) => a.name.localeCompare(b.name)));
    adding = false;
  }
</script>

<header>
  <div>
    <h1>People</h1>
    <p>{$people.length} people across {grouped().length || 1} groups</p>
  </div>
  <button class="ghost-btn" onclick={openAdd}>+ Add person</button>
</header>

<div class="scroll body">
  {#each grouped() as group}
    <section class="group">
      <div class="group-head">
        <span class="mono-label">{group.label}</span>
        <span class="group-count">{group.items.length}</span>
      </div>
      <div class="grid">
        {#each group.items as person (person.slug)}
          <PersonCard {person} onOpen={open} />
        {/each}
      </div>
    </section>
  {/each}

  {#if !$people.length}
    <div class="empty-state">
      <div class="empty-title">No people yet</div>
      <div class="empty-sub">Add your direct reports and teammates to start tracking 1:1s.</div>
      <button class="ghost-btn" onclick={openAdd}>+ Add your first person</button>
    </div>
  {/if}
</div>

{#if adding}
  <div class="overlay">
    <div class="modal">
      <div class="modal-head">Add person</div>

      <label class="field">
        <span>NAME</span>
        <input bind:value={newName} placeholder="Full name" />
      </label>
      <label class="field">
        <span>ROLE</span>
        <input bind:value={newRole} placeholder="Title or team" />
      </label>
      <label class="field">
        <span>COLOR</span>
        <div class="color-picker">
          <div class="color-swatch-grid">
            {#each COLORS as color}
              <button
                type="button"
                class="color-swatch"
                class:selected={newColor === color}
                style={`--swatch:${color}`}
                onclick={() => (newColor = color)}
                aria-label={`Select ${color}`}
              ></button>
            {/each}
            <label class="color-custom" aria-label="Choose custom color">
              <input type="color" bind:value={newColor} class="color-wheel" />
              <span class="color-custom-face" style={`--swatch:${newColor}`}></span>
            </label>
          </div>
          <div class="color-preview-card">
            <span class="avatar-preview" style="background:{newColor}">{newName ? newName.split(' ').map(w => w[0]).slice(0, 2).join('').toUpperCase() : '?'}</span>
            <div>
              <div class="color-preview-name">{newName || "New person"}</div>
              <div class="color-preview-value">{newColor}</div>
            </div>
          </div>
        </div>
      </label>

      {#if addError}<div class="error">{addError}</div>{/if}

      <div class="modal-foot">
        <button class="text-btn" onclick={() => (adding = false)}>Cancel</button>
        <button class="solid-btn" onclick={submitAdd}>Add person</button>
      </div>
    </div>
  </div>
{/if}

<style>
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 22px 32px 18px;
    border-bottom: 1px solid var(--line);
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
  }
  .body {
    flex: 1;
    padding: 24px 32px 40px;
  }
  .group {
    margin-bottom: 32px;
  }
  .group-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 14px;
  }
  .group-count {
    font-size: 11px;
    color: #a39c8c;
    background: #ece6da;
    padding: 2px 8px;
    border-radius: 20px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 14px;
  }
  @media (max-width: 1180px) {
    .grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  @media (max-width: 760px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
  .empty-state {
    text-align: center;
    padding: 80px 40px;
  }
  .empty-title {
    font-family: var(--serif);
    font-size: 22px;
    margin-bottom: 8px;
  }
  .empty-sub {
    font-size: 14px;
    color: var(--muted);
    margin-bottom: 22px;
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(44, 42, 38, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: var(--paper);
    border-radius: 16px;
    padding: 28px;
    width: 400px;
    max-width: calc(100vw - 48px);
    display: flex;
    flex-direction: column;
    gap: 15px;
    box-shadow: 0 12px 40px rgba(44, 42, 38, 0.18);
  }
  .modal-head {
    font-family: var(--serif);
    font-size: 22px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-family: var(--mono);
    font-size: 11px;
    letter-spacing: 0.12em;
    color: var(--faint);
  }
  .field input {
    border: 1px solid var(--line-2);
    border-radius: 8px;
    padding: 9px 11px;
    font-size: 14px;
    background: var(--card);
    color: var(--ink);
  }
  .field input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .color-picker {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .color-swatch-grid {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 10px;
  }
  .color-swatch,
  .color-custom {
    width: 100%;
    aspect-ratio: 1;
    border-radius: 14px;
    border: 1px solid #dfd7ca;
    background: #f5efe5;
    position: relative;
    cursor: pointer;
    padding: 0;
  }
  .color-swatch::before,
  .color-custom-face {
    content: "";
    position: absolute;
    inset: 7px;
    border-radius: 10px;
    background: var(--swatch);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.25);
  }
  .color-swatch.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(180, 141, 78, 0.14);
  }
  .color-custom {
    overflow: hidden;
  }
  .color-wheel {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    border: none;
    border-radius: inherit;
    padding: 0;
    cursor: pointer;
    opacity: 0;
    background: transparent;
  }
  .color-wheel::-webkit-color-swatch-wrapper { padding: 0; border-radius: inherit; }
  .color-wheel::-webkit-color-swatch { border: none; border-radius: inherit; }
  .color-preview-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: #f8f3ea;
  }
  .avatar-preview {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: white;
    font-size: 13px;
    font-weight: 600;
  }
  .color-preview-name {
    color: var(--ink);
    font-size: 13px;
    font-family: var(--sans);
  }
  .color-preview-value {
    margin-top: 2px;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--faint);
    text-transform: uppercase;
  }
  .error {
    font-size: 13px;
    color: var(--over);
  }
  .modal-foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
