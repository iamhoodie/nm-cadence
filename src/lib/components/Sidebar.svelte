<script>
  import { screen, selectedSlug, people, folders, initials, GROUP_COLORS, colorForPerson, fireAppAction } from "../stores.js";
  import { createFolder, updateFolder, reorderFolders, deleteFolder, updatePerson } from "../api.js";
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
  let folderColor = $state(GROUP_COLORS[0]);
  let editingFolderName = $state("");
  let customFolderColorInput = $state();
  let collapsed = $state(new Set());
  let draggedSlug = $state(null);
  let dragMoved = $state(false);
  let activeDropGroup = $state("");
  let activeFolderDropKey = $state("");
  let confirmState = $state(null);
  let pointerDrag = $state(null);
  let contextMenu = $state(null);

  const visiblePeople = $derived(
    searchQuery.trim()
      ? $people.filter((person) =>
          `${person.name} ${person.role}`.toLowerCase().includes(searchQuery.trim().toLowerCase())
        )
      : $people
  );

  const searchResults = $derived(
    visiblePeople.slice().sort((a, b) => a.name.localeCompare(b.name))
  );

  const folderNames = $derived(() => {
    const set = new Set($folders.map((folder) => folder.name));
    for (const person of $people) {
      if (person.group) set.add(person.group);
    }
    return [...set];
  });

  function folderColorForGroup(name) {
    return $folders.find((folder) => folder.name === name)?.color
      || $people.find((person) => person.group === name)?.color
      || GROUP_COLORS[0];
  }

  const groups = $derived(() => {
    const mapped = folderNames().map((name) => ({
      key: name,
      label: name.toUpperCase(),
      color: folderColorForGroup(name),
      people: visiblePeople.filter((person) => person.group === name),
    }));
    const ungrouped = visiblePeople.filter(
      (person) => !person.group || !folderNames().includes(person.group)
    );
    if (ungrouped.length) {
      mapped.push({ key: "__ungrouped", label: "UNGROUPED", color: GROUP_COLORS[0], people: ungrouped });
    }
    return mapped;
  });

  function isPresetColor(color) {
    return GROUP_COLORS.includes(color);
  }

  function openCustomFolderColorPicker() {
    customFolderColorInput?.click();
  }

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
    if (editingFolderName) {
      await updateFolder(editingFolderName, name, folderColor);
      folders.update((list) =>
        list.map((folder) =>
          folder.name === editingFolderName ? { ...folder, name, color: folderColor } : folder
        )
      );
      people.update((list) =>
        list.map((person) =>
          person.group === editingFolderName ? { ...person, group: name } : person
        )
      );
    } else {
      await createFolder(name, folderColor);
      folders.update((list) => [...list, { name, color: folderColor }]);
    }
    addingFolder = false;
    newFolderName = "";
    folderColor = GROUP_COLORS[0];
    editingFolderName = "";
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
    folders.update((list) => list.filter((folder) => folder.name !== name));
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

  function closeContextMenu() {
    contextMenu = null;
  }

  function openNewFolderModal() {
    addingFolder = true;
    editingFolderName = "";
    newFolderName = "";
    folderColor = GROUP_COLORS[0];
  }

  function openEditFolderModal(name) {
    addingFolder = true;
    editingFolderName = name;
    newFolderName = name;
    folderColor = folderColorForGroup(name);
  }

  async function reorderGroupToTarget(dragKey, targetKey) {
    if (!dragKey || !targetKey || dragKey === targetKey || targetKey === "__ungrouped") return;
    const names = $folders.map((folder) => folder.name);
    const fromIndex = names.indexOf(dragKey);
    const targetIndex = names.indexOf(targetKey);
    if (fromIndex < 0 || targetIndex < 0) return;
    const nextNames = [...names];
    const [moved] = nextNames.splice(fromIndex, 1);
    nextNames.splice(targetIndex, 0, moved);
    await reorderFolders(nextNames);
    folders.update((list) => {
      const map = new Map(list.map((folder) => [folder.name, folder]));
      return nextNames.map((folderName) => map.get(folderName)).filter(Boolean);
    });
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
    closeContextMenu();
    open(slug);
  }

  function openPersonAction(slug, action) {
    selectedSlug.set(slug);
    screen.set("person");
    closeContextMenu();
    setTimeout(() => fireAppAction(action), 0);
  }

  function openPersonContextMenu(event, person) {
    event.preventDefault();
    contextMenu = {
      kind: "person",
      slug: person.slug,
      x: event.clientX,
      y: event.clientY,
    };
  }

  function openGroupContextMenu(event, group) {
    if (group.key === "__ungrouped") return;
    event.preventDefault();
    event.stopPropagation();
    contextMenu = {
      kind: "group",
      groupKey: group.key,
      x: event.clientX,
      y: event.clientY,
    };
  }

  function updateDropGroupAtPoint(clientX, clientY) {
    const zone = document.elementFromPoint(clientX, clientY)?.closest("[data-drop-group]");
    activeDropGroup = zone?.dataset.dropGroup || "";
    return activeDropGroup;
  }

  function updateFolderDropAtPoint(clientX, clientY) {
    const zone = document.elementFromPoint(clientX, clientY)?.closest("[data-folder-key]");
    activeFolderDropKey = zone?.dataset.folderKey || "";
    return activeFolderDropKey;
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
      if (pointerDrag.kind === "person") {
        updateDropGroupAtPoint(event.clientX, event.clientY);
      } else if (pointerDrag.kind === "group") {
        updateFolderDropAtPoint(event.clientX, event.clientY);
      }
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
      activeFolderDropKey = "";
      return;
    }

    if (drag.kind === "person") {
      const group = updateDropGroupAtPoint(event.clientX, event.clientY);
      clearDropGroup();
      if (group) {
        await assignGroup(group, drag.slug);
      }
    } else if (drag.kind === "group") {
      const targetKey = updateFolderDropAtPoint(event.clientX, event.clientY);
      activeFolderDropKey = "";
      await reorderGroupToTarget(drag.groupKey, targetKey);
    }
  }

  function beginPointerDrag(event, person) {
    if (event.button !== 0) return;
    event.preventDefault();
    draggedSlug = person.slug;
    dragMoved = false;
    pointerDrag = {
      kind: "person",
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

  function beginGroupDrag(event, group) {
    if (event.button !== 0 || group.key === "__ungrouped") return;
    event.preventDefault();
    event.stopPropagation();
    pointerDrag = {
      kind: "group",
      groupKey: group.key,
      label: group.label,
      startX: event.clientX,
      startY: event.clientY,
      x: event.clientX,
      y: event.clientY,
      active: false,
    };
    activeFolderDropKey = "";
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }

  $effect(() => {
    if (!contextMenu) return;

    const handleClick = () => closeContextMenu();
    const handleEscape = (event) => {
      if (event.key === "Escape") closeContextMenu();
    };

    window.addEventListener("click", handleClick);
    window.addEventListener("keydown", handleEscape);

    return () => {
      window.removeEventListener("click", handleClick);
      window.removeEventListener("keydown", handleEscape);
    };
  });
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
  <div class="search-wrap">
    <input class="search" placeholder="Search people" bind:value={searchQuery} />
    {#if searchQuery.trim()}
      <button class="search-clear-btn" onclick={() => (searchQuery = "")} aria-label="Clear people search">
        ×
      </button>
    {/if}
  </div>

  {#if searchQuery.trim()}
    <div class="search-results">
      {#if searchResults.length}
        {#each searchResults as person (person.slug)}
          <button
            class="people-suggestion sidebar-search-result"
            onclick={() => maybeOpenPerson(person.slug)}
            oncontextmenu={(event) => openPersonContextMenu(event, person)}
          >
            <span class="people-suggestion-av" style="background:{colorForPerson(person, $folders)}">{initials(person.name)}</span>
            <span class="search-result-copy">
              <span class="people-suggestion-name">{person.name}</span>
              <span class="search-result-role">{person.role || "No role"}</span>
            </span>
          </button>
        {/each}
      {:else}
        <div class="search-empty">No people found.</div>
      {/if}
    </div>
  {/if}

  <div class="scroll list" class:list--hidden={searchQuery.trim()}>
    {#each groups() as group}
      <div
        class="folder-block"
        class:folder-block--active={activeDropGroup === group.key}
        class:folder-block--sorting={activeFolderDropKey === group.key}
        data-drop-group={group.key}
        data-folder-key={group.key}
        role="group"
      >
        <div
          class="folder-row"
          role="presentation"
          onpointerdown={(event) => beginGroupDrag(event, group)}
          oncontextmenu={(event) => openGroupContextMenu(event, group)}
        >
          <button class="folder-toggle" onclick={() => toggleCollapse(group.key)}>
            <span class="tri" class:open={!collapsed.has(group.key)}>▸</span>
            <span class="folder-dot" style={`background:${group.color}`}></span>
            <span class="folder-name">{group.label}</span>
            <span class="folder-count">{group.people.length}</span>
          </button>
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
              oncontextmenu={(event) => openPersonContextMenu(event, person)}
            >
              <span class="avatar" style="background:{colorForPerson(person, $folders)}">{initials(person.name)}</span>
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

    {#if !addingFolder}
      <button class="new-folder-btn" onclick={openNewFolderModal}>
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
    {#if pointerDrag.kind === "person"}
      <span class="drag-ghost-avatar">
        {initials($people.find((item) => item.slug === pointerDrag.slug)?.name || pointerDrag.label)}
      </span>
    {:else}
      <span class="drag-ghost-avatar drag-ghost-avatar--group" style={`background:${folderColorForGroup(pointerDrag.groupKey)}`}>#</span>
    {/if}
    <div class="drag-ghost-body">
      <div class="drag-ghost-title">{pointerDrag.label}</div>
      <div class="drag-ghost-meta">{pointerDrag.kind === "group" ? "Reorder group" : "Move to folder"}</div>
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

{#if contextMenu}
  <div
    class="person-context-menu"
    style={`left:${contextMenu.x}px;top:${contextMenu.y}px;`}
    role="menu"
  >
    {#if contextMenu.kind === "person"}
      <button class="person-context-item" onclick={() => maybeOpenPerson(contextMenu.slug)}>Open</button>
      <button class="person-context-item" onclick={() => openPersonAction(contextMenu.slug, "edit-person")}>Edit</button>
      <button class="person-context-item person-context-item--danger" onclick={() => openPersonAction(contextMenu.slug, "delete-person")}>Delete</button>
    {:else if contextMenu.kind === "group"}
      <button class="person-context-item" onclick={() => { openEditFolderModal(contextMenu.groupKey); closeContextMenu(); }}>Edit</button>
      <button class="person-context-item person-context-item--danger" onclick={() => { requestRemoveFolder(contextMenu.groupKey); closeContextMenu(); }}>Delete</button>
    {/if}
  </div>
{/if}

{#if addingFolder}
  <div class="group-modal-overlay">
    <div class="group-modal">
      <div class="group-modal-head">{editingFolderName ? "Edit group" : "New group"}</div>
      <label class="group-modal-field">
        <span>NAME</span>
        <input
          class="folder-input"
          placeholder="Group name"
          bind:value={newFolderName}
          onkeydown={(event) => {
            if (event.key === "Enter") submitFolder();
            if (event.key === "Escape") {
              addingFolder = false;
              newFolderName = "";
              editingFolderName = "";
            }
          }}
        />
      </label>
      <div class="group-modal-field">
        <span>COLOR</span>
        <div class="folder-color-grid">
          {#each GROUP_COLORS as color}
            <button
              class="folder-color-swatch"
              class:folder-color-swatch--active={folderColor === color}
              style={`--swatch:${color}`}
              onclick={() => (folderColor = color)}
              aria-label={`Select ${color}`}
            ></button>
          {/each}
          <button
            class="folder-custom-color-btn"
            class:folder-custom-color-btn--active={!isPresetColor(folderColor)}
            onclick={openCustomFolderColorPicker}
          >
            Custom
          </button>
        </div>
        <input bind:this={customFolderColorInput} class="hidden-color-input" type="color" bind:value={folderColor} />
      </div>
      <div class="group-modal-field">
        <span>PREVIEW</span>
        <div class="group-preview-card">
          <div class="group-preview-row">
            <span class="group-preview-tri">▸</span>
            <span class="group-preview-dot" style={`background:${folderColor}`}></span>
            <span class="group-preview-name">{newFolderName || "Untitled group"}</span>
            <span class="group-preview-count">0</span>
          </div>
          <div class="group-preview-copy">This is how the group color will appear in the sidebar.</div>
        </div>
      </div>
      <div class="group-modal-foot">
        <div class="folder-form-btns">
          <button class="text-btn-sm" onclick={() => { addingFolder = false; newFolderName = ""; editingFolderName = ""; }}>Cancel</button>
          <button class="solid-btn-sm" onclick={submitFolder} disabled={!newFolderName.trim()}>{editingFolderName ? "Save" : "Create"}</button>
        </div>
      </div>
    </div>
  </div>
{/if}

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
  .search-wrap {
    position: relative;
    margin-bottom: 8px;
  }
  .search {
    background: var(--card);
    border: 1px solid var(--line-2);
    border-radius: 8px;
    padding: 8px 34px 8px 11px;
    font-size: 13px;
    color: var(--ink);
    width: 100%;
  }
  .search-clear-btn {
    position: absolute;
    top: 50%;
    right: 8px;
    transform: translateY(-50%);
    width: 20px;
    height: 20px;
    border: none;
    background: transparent;
    color: var(--faint);
    border-radius: 999px;
    display: grid;
    place-items: center;
    font-size: 16px;
    line-height: 1;
    padding: 0;
  }
  .search-clear-btn:hover {
    background: #ebe3d5;
    color: var(--ink);
  }
  .search:focus,
  .folder-input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .search-results {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 8px;
  }
  .sidebar-search-result {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    border: 1px solid var(--line);
    background: rgba(251, 247, 240, 0.9);
    border-radius: 10px;
    padding: 8px 10px;
    text-align: left;
    color: var(--ink);
  }
  .sidebar-search-result:hover {
    background: #f3ede2;
  }
  .people-suggestion-av {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: white;
    font-size: 9px;
    font-weight: 700;
    flex: none;
  }
  .search-result-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .people-suggestion-name {
    font-size: 13px;
  }
  .search-result-role,
  .search-empty {
    font-size: 11px;
    color: var(--muted-2);
  }
  .list {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-right: 4px;
  }
  .list--hidden {
    display: none;
  }
  .folder-block {
    border-radius: 9px;
    transition: background 0.14s ease, box-shadow 0.14s ease;
  }
  .folder-block--active {
    background: linear-gradient(180deg, rgba(187, 160, 121, 0.12), rgba(187, 160, 121, 0.04));
    box-shadow: inset 0 0 0 1px rgba(180, 141, 78, 0.24);
  }
  .folder-block--sorting {
    background: linear-gradient(180deg, rgba(107, 125, 156, 0.14), rgba(107, 125, 156, 0.05));
    box-shadow: inset 0 0 0 1px rgba(107, 125, 156, 0.28);
  }
  .folder-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 2px 2px;
    border-radius: 8px;
    transition: background 0.14s ease;
    user-select: none;
  }
  .folder-row:hover {
    background: #e2dccd;
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
  .folder-row:hover .folder-toggle {
    background: transparent;
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
  .folder-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    flex: none;
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
    user-select: none;
  }
  .row:hover {
    background: #e7e1d3;
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
  .drag-ghost-avatar--group {
    background: #8b7f69;
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
  .group-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(44, 42, 38, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 180;
  }
  .group-modal {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 360px;
    max-width: calc(100vw - 48px);
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--paper);
    padding: 20px;
    box-shadow: 0 12px 40px rgba(44, 42, 38, 0.18);
  }
  .group-modal-head {
    font-family: var(--serif);
    font-size: 18px;
    color: var(--ink);
  }
  .group-modal-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.12em;
    color: var(--faint);
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
  .group-modal-foot {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
  }
  .group-preview-card {
    border: 1px solid var(--line);
    border-radius: 12px;
    background: rgba(251, 247, 240, 0.9);
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .group-preview-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 2px;
    border-radius: 8px;
    background: #e2dccd;
  }
  .group-preview-tri {
    font-size: 9px;
    color: var(--faint);
    margin-left: 4px;
  }
  .group-preview-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    flex: none;
  }
  .group-preview-name {
    flex: 1;
    font-size: 11px;
    letter-spacing: 0.06em;
    color: var(--ink-2);
    font-weight: 700;
    text-transform: uppercase;
  }
  .group-preview-count {
    font-size: 10px;
    color: var(--faint);
    margin-right: 4px;
  }
  .group-preview-copy {
    font-size: 11px;
    letter-spacing: normal;
    color: var(--muted-2);
    font-family: var(--sans, inherit);
  }
  .folder-color-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 8px;
  }
  .folder-color-swatch {
    width: 100%;
    aspect-ratio: 1;
    border-radius: 12px;
    border: 1px solid #dfd7ca;
    background: #f5efe5;
    position: relative;
    padding: 0;
  }
  .folder-color-swatch::before {
    content: "";
    position: absolute;
    inset: 6px;
    border-radius: 9px;
    background: var(--swatch);
  }
  .folder-color-swatch--active {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(180, 141, 78, 0.14);
  }
  .folder-custom-color-btn {
    border: none;
    background: transparent;
    color: var(--muted);
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    cursor: pointer;
    padding: 0 6px;
    align-self: center;
    white-space: nowrap;
  }
  .folder-custom-color-btn--active {
    color: var(--ink);
    text-decoration: underline;
    text-underline-offset: 0.28em;
  }
  .hidden-color-input {
    position: absolute;
    width: 0;
    height: 0;
    opacity: 0;
    pointer-events: none;
  }
  .text-btn-sm {
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: 12px;
    padding: 7px 11px;
    min-height: 32px;
  }
  .solid-btn-sm {
    border: none;
    background: var(--accent);
    color: white;
    padding: 7px 11px;
    font-size: 12px;
    min-height: 32px;
  }
  .person-context-menu {
    position: fixed;
    z-index: 220;
    min-width: 140px;
    background: var(--paper);
    border: 1px solid var(--line);
    border-radius: 12px;
    box-shadow: 0 18px 44px rgba(44, 42, 38, 0.18);
    padding: 6px;
  }
  .person-context-item {
    width: 100%;
    border: none;
    background: transparent;
    border-radius: 8px;
    padding: 8px 10px;
    text-align: left;
    font-size: 13px;
    color: var(--ink);
  }
  .person-context-item:hover {
    background: #f3ede2;
  }
  .person-context-item--danger {
    color: var(--over);
  }
</style>
