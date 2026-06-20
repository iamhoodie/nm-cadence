<script>
  import { screen, selectedSlug, people, folders, initials } from "../stores.js";
  import { createFolder, deleteFolder, updatePerson } from "../api.js";
  import ConfirmModal from "./ConfirmModal.svelte";
  import UpdaterPanel from "./UpdaterPanel.svelte";
  import sideEyeLogo from "../assets/sideeye-logo.png";

  function open(slug) {
    selectedSlug.set(slug);
    screen.set("person");
  }

  const navItems = [
    { key: "people", label: "People", screens: ["people", "person"] },
    { key: "tasks", label: "Tasks", screens: ["tasks"] },
    { key: "conversations", label: "Conversations", screens: ["conversations"] },
  ];

  let searchQuery = $state("");
  let addingFolder = $state(false);
  let newFolderName = $state("");
  let collapsed = $state(new Set());
  let draggedSlug = $state(null);
  let dragMoved = $state(false);
  let activeDropGroup = $state("");
  let confirmState = $state(null);
  let pointerDrag = $state(null);

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
        color: person.color,
        group: "",
      });
      people.update((list) => list.map((item) => (item.slug === updated.slug ? updated : item)));
    }
    await deleteFolder(name);
    folders.update((list) => list.filter((folder) => folder !== name));
  }

  function requestRemoveFolder(name) {
    const affectedCount = $people.filter((person) => person.group === name).length;
    confirmState = {
      title: "Delete folder?",
      message: `Delete "${name}" and move ${affectedCount} ${affectedCount === 1 ? "person" : "people"} out of it? People will be kept, but the folder will be removed.`,
      confirmLabel: "Delete folder",
      action: async () => {
        await removeFolder(name);
      },
    };
  }

  function closeConfirmModal() {
    confirmState = null;
  }

  async function confirmAction() {
    const action = confirmState?.action;
    confirmState = null;
    if (action) await action();
  }

  async function assignGroup(group, slug = draggedSlug) {
    if (!slug) return;
    const person = $people.find((item) => item.slug === slug);
    draggedSlug = null;
    activeDropGroup = "";
    if (!person) return;
    const updated = await updatePerson(person.slug, {
      name: person.name,
      role: person.role,
      bio: person.bio || "",
      color: person.color,
      group: group === "__ungrouped" ? "" : group,
    });
    people.update((list) => list.map((item) => (item.slug === updated.slug ? updated : item)));
  }

  function activateDropGroup(group) {
    activeDropGroup = group;
  }

  function clearDropGroup(group = "") {
    if (!group || activeDropGroup === group) activeDropGroup = "";
  }

  function maybeOpenPerson(slug) {
    if (dragMoved) {
      dragMoved = false;
      return;
    }
    open(slug);
  }

  function updateDropGroupAtPoint(clientX, clientY) {
    const zone = document.elementFromPoint(clientX, clientY)?.closest("[data-drop-group]");
    activeDropGroup = zone?.dataset.dropGroup || "";
    return activeDropGroup;
  }

  function onPointerMove(event) {
    if (!pointerDrag) return;
    const moved =
      Math.abs(event.clientX - pointerDrag.startX) > 6 ||
      Math.abs(event.clientY - pointerDrag.startY) > 6;

    pointerDrag = {
      ...pointerDrag,
      x: event.clientX,
      y: event.clientY,
      active: pointerDrag.active || moved,
    };

    if (pointerDrag.active || moved) {
      dragMoved = true;
      updateDropGroupAtPoint(event.clientX, event.clientY);
      document.body.style.userSelect = "none";
      document.body.style.cursor = "grabbing";
    }
  }

  async function onPointerUp(event) {
    const drag = pointerDrag;
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
    document.body.style.userSelect = "";
    document.body.style.cursor = "";

    pointerDrag = null;
    draggedSlug = null;

    if (!drag?.active) {
      clearDropGroup();
      return;
    }

    const group = updateDropGroupAtPoint(event.clientX, event.clientY);
    clearDropGroup();
    if (group) {
      await assignGroup(group, drag.slug);
    }
  }

  function beginPointerDrag(event, person) {
    if (event.button !== 0) return;
    event.preventDefault();
    draggedSlug = person.slug;
    dragMoved = false;
    pointerDrag = {
      slug: person.slug,
      label: person.name,
      startX: event.clientX,
      startY: event.clientY,
      x: event.clientX,
      y: event.clientY,
      active: false,
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }
</script>

<aside>
  <div class="brand">
    <img class="logo-image" src={sideEyeLogo} alt="SideEye" />
    <span>SideEye</span>
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
        class:folder-block--active={activeDropGroup === group.key}
        data-drop-group={group.key}
        role="group"
      >
        <div class="folder-row">
          <button class="folder-toggle" onclick={() => toggleCollapse(group.key)}>
            <span class="tri" class:open={!collapsed.has(group.key)}>▸</span>
            <span class="folder-name">{group.label}</span>
            <span class="folder-count">{group.people.length}</span>
          </button>
          {#if group.key !== "__ungrouped"}
            <button class="folder-del" onclick={() => requestRemoveFolder(group.key)} title="Remove folder">×</button>
          {/if}
        </div>

        {#if !collapsed.has(group.key)}
          {#each group.people as person (person.slug)}
            <div
              class="row"
              class:active={$selectedSlug === person.slug && $screen === "person"}
              role="button"
              tabindex="0"
              onpointerdown={(event) => beginPointerDrag(event, person)}
              onclick={() => maybeOpenPerson(person.slug)}
              onkeydown={(event) => event.key === "Enter" && maybeOpenPerson(person.slug)}
            >
              <span class="avatar" style="background:{person.color}">{initials(person.name)}</span>
              <span class="who">
                <span class="name">{person.name}</span>
                <span class="role">{person.role}</span>
              </span>
            </div>
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

{#if pointerDrag?.active}
  <div
    class="drag-ghost"
    style={`left:${pointerDrag.x + 14}px;top:${pointerDrag.y + 14}px;`}
  >
    <span class="drag-ghost-avatar">
      {initials($people.find((item) => item.slug === pointerDrag.slug)?.name || pointerDrag.label)}
    </span>
    <div class="drag-ghost-body">
      <div class="drag-ghost-title">{pointerDrag.label}</div>
      <div class="drag-ghost-meta">Move to folder</div>
    </div>
  </div>
{/if}

<ConfirmModal
  open={!!confirmState}
  title={confirmState?.title}
  message={confirmState?.message}
  confirmLabel={confirmState?.confirmLabel}
  onCancel={closeConfirmModal}
  onConfirm={confirmAction}
/>

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
  .logo-image {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    object-fit: cover;
    flex: none;
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
    transition: background 0.14s ease, box-shadow 0.14s ease;
  }
  .folder-block--active {
    background: linear-gradient(180deg, rgba(187, 160, 121, 0.12), rgba(187, 160, 121, 0.04));
    box-shadow: inset 0 0 0 1px rgba(180, 141, 78, 0.24);
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
    cursor: grab;
    user-select: none;
  }
  .row:hover {
    background: #e7e1d3;
  }
  .row:active {
    cursor: grabbing;
  }
  .drag-ghost {
    position: fixed;
    z-index: 160;
    max-width: 220px;
    pointer-events: none;
    background: rgba(252, 249, 243, 0.96);
    border: 1px solid #d9cfbe;
    border-radius: 12px;
    box-shadow: 0 12px 30px rgba(50, 39, 20, 0.14);
    padding: 10px 12px;
    color: var(--ink);
    font-size: 13px;
    line-height: 1.35;
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .drag-ghost-avatar {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: #8b7f69;
    color: white;
    display: grid;
    place-items: center;
    font-size: 11px;
    font-weight: 600;
    flex: none;
  }
  .drag-ghost-body {
    min-width: 0;
  }
  .drag-ghost-title {
    font-weight: 500;
  }
  .drag-ghost-meta {
    margin-top: 3px;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted-2);
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
