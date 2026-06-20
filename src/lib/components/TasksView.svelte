<script>
  import { tasks, people, priorityColor, daysSince, dayLabel, initials } from "../stores.js";
  import { saveTasks } from "../api.js";

  const columns = [
    { key: "todo",  label: "To do",      dot: "#b0a898" },
    { key: "doing", label: "In progress", dot: "var(--due)" },
    { key: "done",  label: "Done",        dot: "var(--accent)" },
  ];

  // ── Modal state ────────────────────────────────────────────────────────
  let modalOpen    = $state(false);
  let creating     = $state(false);
  let editingIndex = $state(-1);

  let title    = $state("");
  let taskPeople = $state([]);   // array of person names
  let due      = $state("");
  let priority = $state("med");
  let peopleInput = $state("");  // autocomplete input for adding people

  // Inline quick-add in the "todo" column
  let quickTitle  = $state("");
  let quickAdding = $state(false);

  // Drag state
  let dragIndex = $state(-1);
  let showArchived = $state(false);

  // ── Derived ────────────────────────────────────────────────────────────
  const todoTasks  = $derived($tasks.map((t, i) => ({ task: t, index: i })).filter(({ task }) => task.column === "todo"));
  const doingTasks = $derived($tasks.map((t, i) => ({ task: t, index: i })).filter(({ task }) => task.column === "doing"));

  const doneGroups = $derived(() => {
    const visible = $tasks
      .map((t, i) => ({ task: t, index: i }))
      .filter(({ task }) => task.column === "done" && !task.archived && task.completed_at && daysSince(task.completed_at) <= 30);
    const map = new Map();
    for (const item of visible) {
      const key = item.task.completed_at;
      if (!map.has(key)) map.set(key, []);
      map.get(key).push(item);
    }
    return [...map.entries()]
      .sort((a, b) => (a[0] < b[0] ? 1 : -1))
      .map(([key, items]) => ({ key, label: dayLabel(key), items }));
  });

  const doneCount = $derived(
    $tasks.filter((t) => t.column === "done" && !t.archived && t.completed_at && daysSince(t.completed_at) <= 30).length
  );
  const archivedCount = $derived(
    $tasks.filter((t) => t.column === "done" && (t.archived || (t.completed_at && daysSince(t.completed_at) > 30))).length
  );
  const archivedGroups = $derived(() => {
    const archived = $tasks
      .map((t, i) => ({ task: t, index: i }))
      .filter(({ task }) => task.column === "done" && (task.archived || (task.completed_at && daysSince(task.completed_at) > 30)));
    const map = new Map();
    for (const item of archived) {
      const key = item.task.completed_at || "unknown";
      if (!map.has(key)) map.set(key, []);
      map.get(key).push(item);
    }
    return [...map.entries()]
      .sort((a, b) => (a[0] < b[0] ? 1 : -1))
      .map(([key, items]) => ({
        key,
        label: key === "unknown" ? "Unknown date" : dayLabel(key),
        items,
      }));
  });

  // People names not yet in taskPeople for the picker
  const peopleOptions = $derived(
    $people.map((p) => p.name).filter((n) => !taskPeople.includes(n))
  );

  function personFor(name) {
    return name ? $people.find((p) => p.name === name) : null;
  }

  function taskPeopleList(task) {
    if (Array.isArray(task.people)) return task.people;
    if (task.person && task.person !== "—") return [task.person];
    return [];
  }

  // ── Persistence ────────────────────────────────────────────────────────
  async function persist(nextTasks) {
    tasks.set(nextTasks);
    await saveTasks(nextTasks);
  }

  // ── Quick add ──────────────────────────────────────────────────────────
  async function submitQuick() {
    const t = quickTitle.trim();
    if (!t) { quickAdding = false; quickTitle = ""; return; }
    await persist([{
      title: t, people: [], due: "", priority: "med",
      column: "todo", done: false, completed_at: "", archived: false,
    }, ...$tasks]);
    quickTitle = "";
    quickAdding = false;
  }

  // ── Modal open/close ───────────────────────────────────────────────────
  function openNew() {
    creating = true; editingIndex = -1; modalOpen = true;
    title = ""; taskPeople = []; due = ""; priority = "med"; peopleInput = "";
  }

  function openEdit(index) {
    const task = $tasks[index];
    if (!task) return;
    creating = false; editingIndex = index; modalOpen = true;
    title = task.title;
    taskPeople = [...taskPeopleList(task)];
    due = task.due || "";
    priority = task.priority || "med";
    peopleInput = "";
  }

  function closeModal() {
    modalOpen = false; creating = false; editingIndex = -1;
  }

  function addPersonToTask(name) {
    if (name && !taskPeople.includes(name)) {
      taskPeople = [...taskPeople, name];
    }
    peopleInput = "";
  }

  function removePersonFromTask(name) {
    taskPeople = taskPeople.filter((n) => n !== name);
  }

  // ── Save/delete ────────────────────────────────────────────────────────
  async function saveEditor() {
    const next = [...$tasks];
    if (creating) {
      next.unshift({
        title: title.trim() || "Untitled task",
        people: [...taskPeople],
        due: due.trim(),
        priority,
        column: "todo",
        done: false,
        completed_at: "",
        archived: false,
      });
    } else if (editingIndex >= 0) {
      const cur = next[editingIndex];
      next[editingIndex] = {
        ...cur,
        title: title.trim() || cur.title,
        people: [...taskPeople],
        due: due.trim(),
        priority,
      };
    }
    await persist(next);
    closeModal();
  }

  async function deleteTask(index) {
    if (!confirm("Delete this task? This cannot be undone.")) return;
    await persist($tasks.filter((_, i) => i !== index));
    closeModal();
  }

  function applyColumn(task, column) {
    if (column === "done") {
      return { ...task, column, done: true, completed_at: task.completed_at || new Date().toISOString().slice(0, 10), archived: false };
    }
    return { ...task, column, done: false, completed_at: "", archived: false };
  }

  // ── Drag/drop ──────────────────────────────────────────────────────────
  function onDragStart(event, index) {
    dragIndex = index;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData("text/plain", String(index));
    }
  }

  function onDragOver(event) {
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
  }

  async function onDrop(event, column) {
    event.preventDefault();
    event.stopPropagation();
    const raw = event.dataTransfer?.getData("text/plain");
    const index = raw !== undefined && raw !== "" ? Number(raw) : dragIndex;
    dragIndex = -1;
    if (!Number.isInteger(index) || index < 0 || !$tasks[index]) return;
    const next = $tasks.map((t, i) => i === index ? applyColumn(t, column) : t);
    await persist(next);
  }

  async function moveTaskByIndex(index, column) {
    if (!Number.isInteger(index) || index < 0 || !$tasks[index]) return;
    await persist($tasks.map((t, i) => i === index ? applyColumn(t, column) : t));
  }

  async function archiveGroup(date) {
    await persist($tasks.map((t) =>
      t.column === "done" && t.completed_at === date ? { ...t, archived: true } : t
    ));
  }

</script>

<header>
  <div>
    <h1>Tasks</h1>
    <p>Track follow-ups, active work, and completed items</p>
  </div>
  <button class="ghost-btn" onclick={openNew}>+ New task</button>
</header>

<div class="body">
  <div class="board">

    <!-- ── To do ── -->
    <div class="column">
      <div class="col-head">
        <span class="col-dot" style="background:#b0a898"></span>
        <span class="col-name">To do</span>
        <span class="col-count">{todoTasks.length}</span>
        <button class="col-add" onclick={() => { quickAdding = true; }} title="Quick add">+</button>
      </div>

      {#if quickAdding}
        <div class="quick-add">
          <input
            class="quick-input"
            placeholder="Task name…"
            bind:value={quickTitle}
            onkeydown={(e) => {
              if (e.key === "Enter") submitQuick();
              if (e.key === "Escape") { quickAdding = false; quickTitle = ""; }
            }}
          />
          <div class="quick-btns">
            <button class="text-btn-sm" onclick={() => { quickAdding = false; quickTitle = ""; }}>Cancel</button>
            <button class="solid-btn-sm" onclick={submitQuick}>Add</button>
          </div>
        </div>
      {/if}

      <div
        class="dropzone"
        role="group"
        ondragover={onDragOver}
        ondrop={(e) => onDrop(e, "todo")}
      >
        {#each todoTasks as { task, index } (`todo-${index}`)}
          <div
            class="card"
            role="button"
            tabindex="0"
            draggable="true"
            ondragstart={(e) => onDragStart(e, index)}
            ondragend={() => (dragIndex = -1)}
            ondragover={onDragOver}
            ondrop={(e) => onDrop(e, "todo")}
            onclick={() => openEdit(index)}
            onkeydown={(e) => e.key === "Enter" && openEdit(index)}
          >
            <span class="card-bar" style="background:{priorityColor(task.priority)}"></span>
            <span class="card-title">{task.title}</span>
            <div class="card-foot">
              <div class="card-people">
                {#each taskPeopleList(task).slice(0, 3) as name}
                  {@const p = personFor(name)}
                  {#if p}
                    <span class="person-av" style="background:{p.color}" title={p.name}>{initials(p.name)}</span>
                  {:else}
                    <span class="person-text">{name}</span>
                  {/if}
                {/each}
                {#if taskPeopleList(task).length === 0}
                  <span class="person-none">Unassigned</span>
                {/if}
              </div>
              {#if task.due}
                <span class="due-tag">{task.due}</span>
              {/if}
            </div>
          </div>
        {/each}

        {#if todoTasks.length === 0 && !quickAdding}
          <div class="col-empty">No tasks yet</div>
        {/if}
      </div>
    </div>

    <!-- ── In progress ── -->
    <div class="column">
      <div class="col-head">
        <span class="col-dot" style="background:var(--due)"></span>
        <span class="col-name">In progress</span>
        <span class="col-count">{doingTasks.length}</span>
      </div>

      <div
        class="dropzone"
        role="group"
        ondragover={onDragOver}
        ondrop={(e) => onDrop(e, "doing")}
      >
        {#each doingTasks as { task, index } (`doing-${index}`)}
          <div
            class="card card--doing"
            role="button"
            tabindex="0"
            draggable="true"
            ondragstart={(e) => onDragStart(e, index)}
            ondragend={() => (dragIndex = -1)}
            ondragover={onDragOver}
            ondrop={(e) => onDrop(e, "doing")}
            onclick={() => openEdit(index)}
            onkeydown={(e) => e.key === "Enter" && openEdit(index)}
          >
            <span class="card-bar" style="background:{priorityColor(task.priority)}"></span>
            <span class="card-title">{task.title}</span>
            <div class="card-foot">
              <div class="card-people">
                {#each taskPeopleList(task).slice(0, 3) as name}
                  {@const p = personFor(name)}
                  {#if p}
                    <span class="person-av" style="background:{p.color}" title={p.name}>{initials(p.name)}</span>
                  {:else}
                    <span class="person-text">{name}</span>
                  {/if}
                {/each}
                {#if taskPeopleList(task).length === 0}
                  <span class="person-none">Unassigned</span>
                {/if}
              </div>
              {#if task.due}
                <span class="due-tag">{task.due}</span>
              {/if}
            </div>
          </div>
        {/each}

        {#if doingTasks.length === 0}
          <div class="col-empty">Drag tasks here when started</div>
        {/if}
      </div>
    </div>

    <!-- ── Done ── -->
    <div class="column">
      <div class="col-head">
        <span class="col-dot" style="background:var(--accent)"></span>
        <span class="col-name">Done</span>
        <span class="col-count">{doneCount}</span>
      </div>

      <div
        class="dropzone"
        role="group"
        ondragover={onDragOver}
        ondrop={(e) => onDrop(e, "done")}
      >
        {#each doneGroups() as group}
          <div class="done-group">
            <div class="done-head">
              <span class="done-date">{group.label}</span>
              <button class="archive-btn" onclick={() => archiveGroup(group.key)}>Archive</button>
            </div>
            {#each group.items as { task, index } (`done-${index}`)}
              <div
                class="card card--done"
                role="button"
                tabindex="0"
                draggable="true"
                ondragstart={(e) => onDragStart(e, index)}
                ondragend={() => (dragIndex = -1)}
                ondragover={onDragOver}
                ondrop={(e) => onDrop(e, "done")}
                onclick={() => openEdit(index)}
                onkeydown={(e) => e.key === "Enter" && openEdit(index)}
              >
                <span class="check-icon">✓</span>
                <div class="card-done-body">
                  <span class="card-title card-title--done">{task.title}</span>
                  {#if taskPeopleList(task).length > 0}
                    <span class="person-text">{taskPeopleList(task).join(", ")}</span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/each}

        {#if doneCount === 0}
          <div class="col-empty">Completed tasks appear here</div>
        {/if}

        {#if archivedCount}
          <div class="archive-panel">
            <button
              class="archived-toggle"
              aria-expanded={showArchived}
              onclick={() => (showArchived = !showArchived)}
            >
              <span>{showArchived ? "Hide archive" : "View archive"}</span>
              <span class="archived-toggle-count">{archivedCount}</span>
            </button>

            {#if showArchived}
              <div class="archived-list">
                {#each archivedGroups() as group}
                  <div class="done-group done-group--archived">
                    <div class="done-head">
                      <span class="done-date">{group.label}</span>
                      <span class="archive-status">Archived</span>
                    </div>
                    {#each group.items as { task, index } (`archived-${index}`)}
                      <div
                        class="card card--done card--archived"
                        role="button"
                        tabindex="0"
                        draggable="true"
                        ondragstart={(e) => onDragStart(e, index)}
                        ondragend={() => (dragIndex = -1)}
                        ondragover={onDragOver}
                        ondrop={(e) => onDrop(e, "done")}
                        onclick={() => openEdit(index)}
                        onkeydown={(e) => e.key === "Enter" && openEdit(index)}
                      >
                        <span class="check-icon">✓</span>
                        <div class="card-done-body">
                          <span class="card-title card-title--done">{task.title}</span>
                          {#if taskPeopleList(task).length > 0}
                            <span class="person-text">{taskPeopleList(task).join(", ")}</span>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>

  </div>
</div>

<!-- ── Task modal ── -->
{#if modalOpen}
  <div class="overlay" role="dialog">
    <div class="modal">
      <div class="modal-head">{creating ? "New task" : "Edit task"}</div>

      <label class="field">
        <span>TITLE</span>
        <input bind:value={title} placeholder="What needs doing?" />
      </label>

      <div class="field">
        <span>PEOPLE</span>
        <div class="people-chips">
          {#each taskPeople as name}
            {@const p = personFor(name)}
            <span class="people-chip">
              {#if p}
                <span class="chip-av" style="background:{p.color}">{initials(p.name)}</span>
              {/if}
              <span>{name}</span>
              <button class="chip-remove" onclick={() => removePersonFromTask(name)} title="Remove">×</button>
            </span>
          {/each}
        </div>
        <div class="people-add-row">
          <input
            class="people-input"
            list="task-people-list"
            bind:value={peopleInput}
            placeholder="Add person…"
            onchange={(e) => { addPersonToTask(e.currentTarget.value.trim()); e.currentTarget.value = ""; }}
            onkeydown={(e) => {
              if (e.key === "Enter") { addPersonToTask(peopleInput.trim()); e.preventDefault(); }
            }}
          />
          <datalist id="task-people-list">
            {#each peopleOptions as name}
              <option value={name}></option>
            {/each}
          </datalist>
        </div>
      </div>

      <label class="field">
        <span>DUE DATE</span>
        <input type="date" bind:value={due} />
      </label>

      <div class="field">
        <span>PRIORITY</span>
        <div class="priority-row">
          {#each ["high", "med", "low"] as level}
            <button
              class="pri-btn"
              class:sel={priority === level}
              style="--c:{priorityColor(level)}"
              onclick={() => (priority = level)}
            >{level}</button>
          {/each}
        </div>
      </div>

      {#if !creating && editingIndex >= 0}
        <div class="field">
          <span>MOVE TO</span>
          <div class="move-row">
            {#each columns as col}
              <button
                class="move-btn"
                class:cur={$tasks[editingIndex]?.column === col.key}
                onclick={() => moveTaskByIndex(editingIndex, col.key)}
                disabled={$tasks[editingIndex]?.column === col.key}
              >{col.label}</button>
            {/each}
          </div>
        </div>
      {/if}

      <div class="modal-foot">
        {#if !creating && editingIndex >= 0}
          <button class="icon-btn icon-btn--danger" onclick={() => deleteTask(editingIndex)} title="Delete task" aria-label="Delete task">
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M9 3h6l1 2h4v2H4V5h4l1-2Zm1 7h2v7h-2v-7Zm4 0h2v7h-2v-7ZM7 10h2v7H7v-7Zm-1 10h12l1-12H5l1 12Z" fill="currentColor"></path>
            </svg>
          </button>
        {/if}
        <div class="foot-right">
          <button class="text-btn" onclick={closeModal}>Cancel</button>
          <button class="solid-btn" onclick={saveEditor}>Save</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Layout ── */
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 22px 32px 18px;
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
  }
  h1 {
    font-family: var(--serif);
    font-size: 28px;
    margin: 0;
    line-height: 1;
    font-weight: 500;
  }
  header p {
    font-size: 13px;
    color: var(--muted-2);
    margin: 4px 0 0;
  }
  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 20px 24px;
    min-height: 0;
  }
  .board {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
    min-height: 0;
    overflow: hidden;
  }

  /* ── Column ── */
  .column {
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 14px;
  }
  .col-head {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 14px 14px 10px;
    flex-shrink: 0;
    border-bottom: 1px solid var(--line);
  }
  .col-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex: none;
  }
  .col-name {
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--ink-2);
    text-transform: uppercase;
    flex: 1;
  }
  .col-count {
    font-size: 10px;
    color: #a39c8c;
    background: #e8e2d6;
    padding: 1px 7px;
    border-radius: 20px;
  }
  .col-add {
    border: none;
    background: transparent;
    color: var(--muted-2);
    font-size: 18px;
    line-height: 1;
    padding: 0 2px;
    cursor: pointer;
  }
  .col-add:hover { color: var(--ink); }

  /* ── Quick add ── */
  .quick-add {
    padding: 10px 12px;
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: var(--card);
  }
  .quick-input {
    border: none;
    background: transparent;
    font-size: 13px;
    color: var(--ink);
    outline: none;
    width: 100%;
  }
  .quick-btns { display: flex; justify-content: flex-end; gap: 6px; }
  .text-btn-sm {
    border: none; background: transparent; color: var(--muted);
    font-size: 12px; padding: 4px 8px; border-radius: 6px; cursor: pointer;
  }
  .text-btn-sm:hover { color: var(--ink); }
  .solid-btn-sm {
    border: none; background: var(--accent); color: white;
    padding: 4px 12px; font-size: 12px; border-radius: 6px; cursor: pointer;
  }

  /* ── Dropzone ── */
  .dropzone {
    flex: 1;
    overflow-y: auto;
    padding: 10px 10px 20px;
  }

  .col-empty {
    font-size: 12px;
    color: var(--faint);
    padding: 28px 8px;
    text-align: center;
    font-style: italic;
  }

  /* ── Task cards ── */
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    background: var(--card);
    border: 1px solid #e6dfd4;
    border-radius: 10px;
    padding: 11px 12px 10px 15px;
    margin-bottom: 7px;
    cursor: grab;
    text-align: left;
    box-shadow: 0 1px 3px rgba(60, 50, 30, 0.05);
    transition: box-shadow 0.1s ease, transform 0.1s ease;
    user-select: none;
  }
  .card:hover {
    box-shadow: 0 3px 12px rgba(60, 50, 30, 0.1);
    transform: translateY(-1px);
  }
  .card:active { cursor: grabbing; }
  .card--doing { background: #fffefb; }
  .card--done {
    flex-direction: row;
    align-items: flex-start;
    gap: 9px;
    background: #f6f1e8;
    border-color: #e6dfd4;
    padding: 9px 12px;
  }
  .card--archived {
    opacity: 0.72;
  }
  .card-bar {
    position: absolute;
    left: 0;
    top: 5px;
    bottom: 5px;
    width: 3px;
    border-radius: 3px;
  }
  .card-title {
    font-size: 13px;
    line-height: 1.4;
    color: var(--ink);
    font-weight: 500;
  }
  .card-title--done {
    text-decoration: line-through;
    color: var(--muted-2);
    font-weight: 400;
  }
  .card-done-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .card-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }
  .card-people {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-wrap: wrap;
    min-width: 0;
  }
  .person-av {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: white;
    font-size: 8px;
    font-weight: 700;
    flex: none;
    cursor: pointer;
  }
  .person-text { font-size: 11px; color: var(--muted-2); }
  .person-none { font-size: 11px; color: var(--faint); font-style: italic; }
  .due-tag {
    font-size: 10px;
    font-family: var(--mono);
    color: var(--muted-2);
    background: #ece6da;
    border-radius: 5px;
    padding: 2px 6px;
    flex: none;
  }
  .check-icon {
    width: 20px; height: 20px; border-radius: 6px;
    background: var(--accent); color: white;
    display: grid; place-items: center;
    font-size: 10px; flex: none; margin-top: 1px;
  }

  /* ── Done groups ── */
  .done-group { margin-bottom: 16px; }
  .done-head {
    display: flex; align-items: center;
    justify-content: space-between; margin-bottom: 7px; padding: 0 2px;
  }
  .done-date {
    font-family: var(--mono); font-size: 10px;
    letter-spacing: 0.08em; color: var(--muted-2); text-transform: uppercase;
  }
  .archive-btn {
    border: none; background: transparent; color: var(--faint);
    font-size: 11px; cursor: pointer; padding: 2px 4px; border-radius: 5px;
  }
  .archive-btn:hover { color: var(--muted); background: #e8e2d6; }
  .archive-panel {
    padding-top: 8px;
  }
  .archived-toggle {
    width: 100%;
    border: 1px dashed #d8d0c0;
    background: #f7f2ea;
    color: var(--ink-2);
    border-radius: 10px;
    padding: 10px 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    cursor: pointer;
  }
  .archived-toggle:hover {
    background: #f1ebdf;
  }
  .archived-toggle-count {
    min-width: 22px;
    height: 22px;
    border-radius: 999px;
    display: grid;
    place-items: center;
    background: #e8e2d6;
    font-family: var(--mono);
    font-size: 10px;
    color: var(--muted);
  }
  .archived-list {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--line);
  }
  .done-group--archived .done-date {
    color: var(--faint);
  }
  .archive-status {
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--faint);
  }

  /* ── Modal ── */
  .overlay {
    position: fixed; inset: 0;
    background: rgba(44, 42, 38, 0.45);
    display: flex; align-items: center; justify-content: center;
    z-index: 120;
  }
  .modal {
    background: var(--paper);
    border-radius: 18px;
    padding: 24px 26px 20px;
    width: 460px;
    max-width: calc(100vw - 48px);
    max-height: calc(100vh - 48px);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: 0 16px 48px rgba(44, 42, 38, 0.2);
  }
  .modal-head {
    font-family: var(--serif);
    font-size: 22px;
    color: var(--ink);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.12em;
    color: var(--faint);
  }
  .field > input {
    border: 1px solid var(--line-2);
    border-radius: 8px;
    padding: 9px 11px;
    font-size: 14px;
    background: var(--card);
    color: var(--ink);
  }
  .field > input:focus {
    outline: none; border-color: var(--accent);
  }
  /* date input styling */
  .field > input[type="date"] {
    color-scheme: light;
  }

  /* People chips */
  .people-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    min-height: 0;
  }
  .people-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: #ece6da;
    border-radius: 20px;
    padding: 3px 8px 3px 4px;
    font-size: 12px;
    color: var(--ink-2);
    font-family: var(--sans);
    letter-spacing: 0;
  }
  .chip-av {
    width: 18px; height: 18px; border-radius: 50%;
    display: grid; place-items: center;
    color: white; font-size: 8px; font-weight: 700;
  }
  .chip-remove {
    border: none; background: transparent; color: var(--muted-2);
    font-size: 14px; line-height: 1; padding: 0 1px; cursor: pointer;
    font-family: var(--sans);
  }
  .chip-remove:hover { color: var(--over); }
  .people-add-row {
    display: flex; align-items: center; gap: 8px;
  }
  .people-input {
    flex: 1;
    border: 1px solid var(--line-2);
    border-radius: 8px;
    padding: 8px 11px;
    font-size: 13px;
    background: var(--card);
    color: var(--ink);
  }
  .people-input:focus { outline: none; border-color: var(--accent); }

  .priority-row, .move-row { display: flex; gap: 7px; flex-wrap: wrap; }
  .pri-btn {
    border: 1.5px solid var(--c); color: var(--c); background: transparent;
    border-radius: 7px; padding: 5px 13px; font-size: 11px;
    font-family: var(--mono); text-transform: capitalize; cursor: pointer;
  }
  .pri-btn.sel { background: var(--c); color: white; }
  .move-btn {
    border: 1px solid var(--line-2); background: transparent;
    border-radius: 7px; padding: 6px 11px; font-size: 11px;
    color: var(--muted); cursor: pointer;
  }
  .move-btn.cur, .move-btn:disabled {
    background: #e8e2d6; color: var(--ink); opacity: 1; cursor: default;
  }
  .modal-foot {
    display: flex; justify-content: space-between;
    align-items: center; margin-top: 4px;
  }
  .foot-right { display: flex; gap: 8px; align-items: center; margin-left: auto; }
  .icon-btn {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    border: 1px solid var(--line-2);
    background: var(--card);
    color: var(--muted);
    display: grid;
    place-items: center;
    cursor: pointer;
    flex: none;
  }
  .icon-btn svg {
    width: 16px;
    height: 16px;
  }
  .icon-btn:hover {
    color: var(--ink);
    background: var(--panel-2);
  }
  .icon-btn--danger:hover {
    color: var(--over);
    border-color: #e0b9b4;
    background: #fbf0ee;
  }

  @media (max-width: 1100px) {
    .board { grid-template-columns: 1fr; }
  }
</style>
