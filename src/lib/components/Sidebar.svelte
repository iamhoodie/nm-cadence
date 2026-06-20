<script>
  import { screen, selectedSlug, people, folders, initials } from "../stores.js";
  import { createFolder, deleteFolder, updatePerson } from "../api.js";
  import UpdaterPanel from "./UpdaterPanel.svelte";

  function open(slug) {
    selectedSlug.set(slug);
    screen.set("person");
  }

  const navItems = [
    { key: "people", label: "People", screens: ["people", "person"] },
    { key: "tasks", label: "Tasks", screens: ["tasks"] },
  ];

  let searchQuery = $state("");
  let addingFolder = $state(false);
  let newFolderName = $state("");
  let collapsed = $state(new Set());
  let draggedSlug = $state(null);

  const visiblePeople = $derived(
    searchQuery.trim()
      ? $people.filter((person) =>
          `${person.name} ${person.role}`.toLowerCase().includes(searchQuery.trim().toLowerCase())
        )
      : $people
  );

  const folderNames = $derived(() => {
    const set = new Set($folders);
    for (const person of $people) {
      if (person.group && $folders.includes(person.group)) set.add(person.group);
    }
    return [...set].sort((a, b) => a.localeCompare(b));
  });

  const groups = $derived(() => {
    const mapped = folderNames().map((name) => ({
      key: name,
      label: name.toUpperCase(),
      people: visiblePeople.filter((person) => person.group === name),
    }));
    const ungrouped = visiblePeople.filter(
      (person) => !person.group || !folderNames().includes(person.group)
    );
    mapped.push({ key: "__ungrouped", label: "UNGROUPED", people: ungrouped });
    return mapped;
  });

  function toggleCollapse(name) {
    const next = new Set(collapsed);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    collapsed = next;
  }

  async function submitFolder() {
    const name = newFolderName.trim();
    if (!name) {
      addingFolder = false;
      newFolderName = "";
      return;
    }
    await createFolder(name);
    folders.update((list) => [...new Set([...list, name])].sort((a, b) => a.localeCompare(b)));
    addingFolder = false;
    newFolderName = "";
  }

  async function removeFolder(name) {
    const affected = $people.filter((person) => person.group === name);
    for (const person of affected) {
      const updated = await updatePerson(person.slug, {
        name: person.name,
        role: person.role,
        bio: person.bio || "",
        cadence_weeks: person.cadence_weeks,
        color: person.color,
        next_1on1: person.next_1on1 || "",
        joined: person.joined || "",
        group: "",
      });
      people.update((list) => list.map((item) => (item.slug === updated.slug ? updated : item)));
    }
    await deleteFolder(name);
    folders.update((list) => list.filter((folder) => folder !== name));
  }

  function startDragPerson(event, slug) {
    draggedSlug = slug;
    event.dataTransfer?.setData("text/person-slug", slug);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  async function assignGroup(group, event) {
    event?.preventDefault();
    const slug = event?.dataTransfer?.getData("text/person-slug") || draggedSlug;
    if (!slug) return;
    const person = $people.find((item) => item.slug === slug);
    draggedSlug = null;
    if (!person) return;
    const updated = await updatePerson(person.slug, {
      name: person.name,
      role: person.role,
      bio: person.bio || "",
      cadence_weeks: person.cadence_weeks,
      color: person.color,
      next_1on1: person.next_1on1 || "",
      joined: person.joined || "",
      group: group === "__ungrouped" ? "" : group,
    });
    people.update((list) => list.map((item) => (item.slug === updated.slug ? updated : item)));
  }
</script>

<aside>
  <div class="brand">
    <div class="logo">NM</div>
    <span>Cadence</span>
  </div>

  <nav>
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={item.screens.includes($screen)}
        onclick={() => screen.set(item.key)}
      >
        {item.label}
      </button>
    {/each}
  </nav>

  <div class="people-head">
    <span class="mono-label">YOUR PEOPLE</span>
    <span class="count">{$people.length}</span>
  </div>
  <input class="search" placeholder="Search people" bind:value={searchQuery} />

  <div class="scroll list">
    {#each groups() as group}
      <div
        class="folder-block"
        role="group"
        ondragover={(event) => event.preventDefault()}
        ondrop={(event) => assignGroup(group.key, event)}
      >
        <div class="folder-row">
          <button class="folder-toggle" onclick={() => toggleCollapse(group.key)}>
            <span class="tri" class:open={!collapsed.has(group.key)}>▸</span>
            <span class="folder-name">{group.label}</span>
            <span class="folder-count">{group.people.length}</span>
          </button>
          {#if group.key !== "__ungrouped"}
            <button class="folder-del" onclick={() => removeFolder(group.key)} title="Remove folder">×</button>
          {/if}
        </div>

        {#if !collapsed.has(group.key)}
          {#each group.people as person (person.slug)}
            <button
              class="row"
              class:active={$selectedSlug === person.slug && $screen === "person"}
              draggable="true"
              ondragstart={(event) => startDragPerson(event, person.slug)}
              ondragend={() => (draggedSlug = null)}
              onclick={() => open(person.slug)}
            >
              <span class="avatar" style="background:{person.color}">{initials(person.name)}</span>
              <span class="who">
                <span class="name">{person.name}</span>
                <span class="role">{person.role}</span>
              </span>
            </button>
          {/each}
          {#if group.people.length === 0}
            <div class="folder-empty">Drop a person here</div>
          {/if}
        {/if}
      </div>
    {/each}

    {#if addingFolder}
      <div class="folder-form">
        <input
          class="folder-input"
          placeholder="Folder name"
          bind:value={newFolderName}
          onkeydown={(event) => {
            if (event.key === "Enter") submitFolder();
            if (event.key === "Escape") {
              addingFolder = false;
              newFolderName = "";
            }
          }}
        />
        <div class="folder-form-btns">
          <button class="text-btn-sm" onclick={() => { addingFolder = false; newFolderName = ""; }}>Cancel</button>
          <button class="solid-btn-sm" onclick={submitFolder}>Create</button>
        </div>
      </div>
    {:else}
      <button class="new-folder-btn" onclick={() => { addingFolder = true; newFolderName = ""; }}>
        + New folder
      </button>
    {/if}
  </div>

  <UpdaterPanel />
</aside>

<style>
  aside {
    width: 272px;
    flex: none;
    background: var(--panel);
    border-right: 1px solid var(--line-2);
    padding: 24px 16px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 8px;
    margin-bottom: 24px;
  }
  .logo {
    width: 28px;
    height: 28px;
    border-radius: 10px;
    background: var(--accent);
    color: white;
    display: grid;
    place-items: center;
    font-family: var(--serif);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
  }
  .brand span {
    font-family: var(--serif);
    font-size: 22px;
    color: var(--ink);
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-bottom: 24px;
  }
  .nav-item {
    text-align: left;
    border: none;
    background: transparent;
    padding: 9px 12px;
    border-radius: 7px;
    font-size: 14px;
    color: #6b6557;
  }
  .nav-item.active {
    background: #e2dccd;
    color: var(--ink);
    font-weight: 600;
  }
  .people-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 8px;
    margin-bottom: 8px;
  }
  .count {
    font-size: 11px;
    color: var(--faint);
  }
  .search {
    background: var(--card);
    border: 1px solid var(--line-2);
    border-radius: 8px;
    padding: 8px 11px;
    font-size: 13px;
    color: var(--ink);
    margin-bottom: 8px;
    width: 100%;
  }
  .search:focus,
  .folder-input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .list {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-right: 4px;
  }
  .folder-block {
    border-radius: 9px;
  }
  .folder-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 2px 2px;
  }
  .folder-toggle {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    border: none;
    background: transparent;
    text-align: left;
    padding: 4px 6px;
    border-radius: 6px;
  }
  .folder-toggle:hover {
    background: #e2dccd;
  }
  .tri {
    font-size: 9px;
    color: var(--faint);
    transition: transform 0.15s ease;
  }
  .tri.open {
    transform: rotate(90deg);
  }
  .folder-name {
    flex: 1;
    font-size: 11px;
    letter-spacing: 0.06em;
    color: var(--ink-2);
    font-weight: 700;
  }
  .folder-count {
    font-size: 10px;
    color: var(--faint);
  }
  .folder-del {
    border: none;
    background: transparent;
    color: #c4b9a6;
    font-size: 14px;
    padding: 2px 4px;
    opacity: 0;
  }
  .folder-row:hover .folder-del {
    opacity: 1;
  }
  .folder-del:hover {
    color: var(--over);
  }
  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px 8px 20px;
    border: none;
    background: transparent;
    text-align: left;
    border-radius: 8px;
  }
  .row:hover {
    background: #e7e1d3;
  }
  .row.active {
    background: #e2dccd;
  }
  .avatar {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    color: white;
    font-size: 11px;
    font-weight: 600;
    display: grid;
    place-items: center;
    flex: none;
  }
  .who {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .name {
    font-size: 13px;
    color: var(--ink-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .role {
    font-size: 11px;
    color: var(--muted-2);
  }
  .folder-empty {
    font-size: 11px;
    color: var(--faint);
    padding: 6px 20px 8px;
    font-style: italic;
  }
  .new-folder-btn,
  .text-btn-sm,
  .solid-btn-sm {
    border-radius: 7px;
  }
  .new-folder-btn {
    border: none;
    background: transparent;
    color: var(--faint);
    font-size: 12px;
    padding: 8px 10px;
    text-align: left;
  }
  .new-folder-btn:hover {
    background: #e7e1d3;
    color: var(--ink);
  }
  .folder-form {
    padding: 6px 4px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .folder-input {
    border: 1px solid var(--line-2);
    border-radius: 7px;
    padding: 7px 9px;
    font-size: 13px;
    background: var(--card);
    color: var(--ink);
  }
  .folder-form-btns {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }
  .text-btn-sm {
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: 12px;
    padding: 5px 8px;
  }
  .solid-btn-sm {
    border: none;
    background: var(--accent);
    color: white;
    padding: 5px 12px;
    font-size: 12px;
  }
</style>
