<script>
  import ConfirmModal from "./ConfirmModal.svelte";
  import { appAction, clearAppAction, tasks, people, priorityColor, daysSince, dayLabel, initials } from "../stores.js";
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
  let peoplePickerOpen = $state(false);

  // Inline quick-add in the "todo" column
  let quickTitle  = $state("");
  let quickAdding = $state(false);
  let datePickerOpen = $state(false);
  let visibleMonth = $state(startOfMonthIso(todayIso()));

  // Drag state
  let dragIndex = $state(-1);
  let dragMoved = $state(false);
  let activeDropColumn = $state("");
  let activeDropIndex = $state(-1);
  let showArchived = $state(false);
  let confirmState = $state(null);
  let handledActionToken = $state(0);
  let pointerDrag = $state(null);

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
  const filteredPeopleOptions = $derived(
    peopleOptions.filter((name) =>
      !peopleInput.trim() || name.toLowerCase().includes(peopleInput.trim().toLowerCase())
    )
  );
  const calendarDays = $derived(buildCalendarDays(visibleMonth));

  function personFor(name) {
    return name ? $people.find((p) => p.name === name) : null;
  }

  function todayIso() {
    return new Date().toISOString().slice(0, 10);
  }

  function parseIsoDate(iso) {
    if (!iso) return null;
    const [year, month, day] = iso.split("-").map(Number);
    if (!year || !month || !day) return null;
    return new Date(year, month - 1, day);
  }

  function formatIsoDate(date) {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
  }

  function startOfMonthIso(iso) {
    const date = parseIsoDate(iso) || new Date();
    return formatIsoDate(new Date(date.getFullYear(), date.getMonth(), 1));
  }

  function monthLabel(iso) {
    const date = parseIsoDate(iso) || new Date();
    return date.toLocaleDateString("en-US", { month: "long", year: "numeric" });
  }

  function dueDisplay(iso) {
    if (!iso) return "No due date";
    const date = parseIsoDate(iso);
    if (!date) return iso;
    return date.toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  function shiftMonth(iso, delta) {
    const date = parseIsoDate(iso) || new Date();
    return formatIsoDate(new Date(date.getFullYear(), date.getMonth() + delta, 1));
  }

  function buildCalendarDays(monthIso) {
    const monthStart = parseIsoDate(monthIso) || new Date();
    const gridStart = new Date(monthStart);
    gridStart.setDate(1 - monthStart.getDay());
    return Array.from({ length: 42 }, (_, index) => {
      const date = new Date(gridStart);
      date.setDate(gridStart.getDate() + index);
      return {
        iso: formatIsoDate(date),
        label: String(date.getDate()),
        inMonth: date.getMonth() === monthStart.getMonth(),
      };
    });
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
    peoplePickerOpen = false;
    datePickerOpen = false;
    visibleMonth = startOfMonthIso(todayIso());
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
    peoplePickerOpen = false;
    datePickerOpen = false;
    visibleMonth = startOfMonthIso(task.due || todayIso());
  }

  function closeModal() {
    modalOpen = false; creating = false; editingIndex = -1;
    peoplePickerOpen = false;
    datePickerOpen = false;
  }

  function addPersonToTask(name) {
    if (name && !taskPeople.includes(name)) {
      taskPeople = [...taskPeople, name];
    }
    peopleInput = "";
    peoplePickerOpen = false;
  }

  function removePersonFromTask(name) {
    taskPeople = taskPeople.filter((n) => n !== name);
  }

  function toggleDatePicker() {
    visibleMonth = startOfMonthIso(due || todayIso());
    datePickerOpen = !datePickerOpen;
  }

  function selectDueDate(iso) {
    due = iso;
    visibleMonth = startOfMonthIso(iso);
    datePickerOpen = false;
  }

  function clearDueDate() {
    due = "";
    datePickerOpen = false;
  }

  function commitPeopleInput() {
    const value = peopleInput.trim();
    if (!value) return;
    const exact = peopleOptions.find((name) => name.toLowerCase() === value.toLowerCase());
    const selected = exact || filteredPeopleOptions[0];
    if (selected) addPersonToTask(selected);
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
    await persist($tasks.filter((_, i) => i !== index));
    closeModal();
  }

  function requestDeleteTask(index) {
    const task = $tasks[index];
    if (!task) return;
    confirmState = {
      title: "Delete task?",
      message: `Delete "${task.title}" permanently? This cannot be undone.`,
      confirmLabel: "Delete task",
      action: async () => {
        await deleteTask(index);
      },
    };
  }

  function applyColumn(task, column) {
    if (column === "done") {
      return { ...task, column, done: true, completed_at: task.completed_at || new Date().toISOString().slice(0, 10), archived: false };
    }
    return { ...task, column, done: false, completed_at: "", archived: false };
  }

  function setActiveDropColumn(column) {
    activeDropColumn = column;
  }

  function clearActiveDropColumn() {
    activeDropColumn = "";
    activeDropIndex = -1;
  }

  function taskColumnOrder(column) {
    return columns.findIndex((item) => item.key === column);
  }

  function reorderTasks(list, fromIndex, column, targetIndex = -1) {
    const task = list[fromIndex];
    if (!task) return list;

    const moved = applyColumn(task, column);
    if (targetIndex === fromIndex && task.column === column) {
      return list;
    }

    const next = list.filter((_, i) => i !== fromIndex);

    if (targetIndex >= 0) {
      const insertAt = targetIndex > fromIndex ? targetIndex - 1 : targetIndex;
      next.splice(insertAt, 0, moved);
      return next;
    }

    const lastInColumn = next.reduce((found, item, idx) => (item.column === column ? idx : found), -1);
    let insertAt = next.length;

    if (lastInColumn >= 0) {
      insertAt = lastInColumn + 1;
    } else {
      const currentOrder = taskColumnOrder(column);
      const firstLater = next.findIndex((item) => taskColumnOrder(item.column) > currentOrder);
      insertAt = firstLater >= 0 ? firstLater : next.length;
    }

    next.splice(insertAt, 0, moved);
    return next;
  }

  async function moveTaskByIndex(index, column, targetIndex = -1) {
    if (!Number.isInteger(index) || index < 0 || !$tasks[index]) return;
    await persist(reorderTasks($tasks, index, column, targetIndex));
  }

  async function archiveGroup(date) {
    await persist($tasks.map((t) =>
      t.column === "done" && t.completed_at === date ? { ...t, archived: true } : t
    ));
  }

  function requestArchiveGroup(date) {
    const group = doneGroups().find((entry) => entry.key === date);
    if (!group) return;
    confirmState = {
      title: "Archive completed tasks?",
      message: `Move ${group.items.length} completed ${group.items.length === 1 ? "task" : "tasks"} from ${group.label} into the archive?`,
      confirmLabel: "Archive",
      action: async () => {
        await archiveGroup(date);
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

  function maybeOpenEdit(index) {
    if (dragMoved) {
      dragMoved = false;
      return;
    }
    openEdit(index);
  }

  function updateDropColumnAtPoint(clientX, clientY) {
    const element = document.elementFromPoint(clientX, clientY);
    const zone = element?.closest("[data-drop-column]");
    const card = element?.closest("[data-task-index]");
    activeDropColumn = zone?.dataset.dropColumn || "";
    activeDropIndex = card ? Number(card.dataset.taskIndex) : -1;
    return activeDropColumn;
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
      updateDropColumnAtPoint(event.clientX, event.clientY);
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
    dragIndex = -1;

    if (!drag?.active) {
      clearActiveDropColumn();
      return;
    }

    const column = updateDropColumnAtPoint(event.clientX, event.clientY);
    const targetIndex = activeDropIndex;
    clearActiveDropColumn();
    if (column) {
      await moveTaskByIndex(drag.index, column, targetIndex);
    }
  }

  function beginPointerDrag(event, index) {
    if (event.button !== 0) return;
    event.preventDefault();
    const task = $tasks[index];
    if (!task) return;

    dragIndex = index;
    dragMoved = false;
    pointerDrag = {
      index,
      title: task.title,
      startX: event.clientX,
      startY: event.clientY,
      x: event.clientX,
      y: event.clientY,
      active: false,
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }

  $effect(() => {
    const action = $appAction;
    if (!action?.token || action.token === handledActionToken) return;
    if (action.type !== "new-task") return;
    handledActionToken = action.token;
    openNew();
    clearAppAction();
  });

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
        class:dropzone--active={activeDropColumn === "todo"}
        data-drop-column="todo"
        role="group"
      >
        {#each todoTasks as { task, index } (`todo-${index}`)}
          <div
            class="card"
            class:card--drop-target={activeDropIndex === index && dragIndex !== index}
            role="button"
            tabindex="0"
            data-task-index={index}
            onpointerdown={(event) => beginPointerDrag(event, index)}
            onclick={() => maybeOpenEdit(index)}
            onkeydown={(e) => e.key === "Enter" && maybeOpenEdit(index)}
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
        class:dropzone--active={activeDropColumn === "doing"}
        data-drop-column="doing"
        role="group"
      >
        {#each doingTasks as { task, index } (`doing-${index}`)}
          <div
            class="card card--doing"
            class:card--drop-target={activeDropIndex === index && dragIndex !== index}
            role="button"
            tabindex="0"
            data-task-index={index}
            onpointerdown={(event) => beginPointerDrag(event, index)}
            onclick={() => maybeOpenEdit(index)}
            onkeydown={(e) => e.key === "Enter" && maybeOpenEdit(index)}
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
        class:dropzone--active={activeDropColumn === "done"}
        data-drop-column="done"
        role="group"
      >
        {#each doneGroups() as group}
          <div class="done-group">
            <div class="done-head">
              <span class="done-date">{group.label}</span>
              <button class="archive-btn" onclick={() => requestArchiveGroup(group.key)}>Archive</button>
            </div>
            {#each group.items as { task, index } (`done-${index}`)}
              <div
                class="card card--done"
                class:card--drop-target={activeDropIndex === index && dragIndex !== index}
                role="button"
                tabindex="0"
                data-task-index={index}
                onpointerdown={(event) => beginPointerDrag(event, index)}
                onclick={() => maybeOpenEdit(index)}
                onkeydown={(e) => e.key === "Enter" && maybeOpenEdit(index)}
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
                        class:card--drop-target={activeDropIndex === index && dragIndex !== index}
                        role="button"
                        tabindex="0"
                        data-task-index={index}
                        onpointerdown={(event) => beginPointerDrag(event, index)}
                        onclick={() => maybeOpenEdit(index)}
                        onkeydown={(e) => e.key === "Enter" && maybeOpenEdit(index)}
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
          <div class="people-picker">
            <input
              class="people-input"
              bind:value={peopleInput}
              placeholder="Add person…"
              onfocus={() => (peoplePickerOpen = true)}
              oninput={() => (peoplePickerOpen = true)}
              onblur={() => window.setTimeout(() => (peoplePickerOpen = false), 120)}
              onkeydown={(e) => {
                if (e.key === "Enter") {
                  commitPeopleInput();
                  e.preventDefault();
                }
                if (e.key === "Escape") {
                  peoplePickerOpen = false;
                }
              }}
            />
            {#if peoplePickerOpen && filteredPeopleOptions.length}
              <div class="people-suggestions">
                {#each filteredPeopleOptions as name}
                  <button class="people-suggestion" onclick={() => addPersonToTask(name)}>
                    {#if personFor(name)}
                      <span class="people-suggestion-av" style="background:{personFor(name).color}">{initials(personFor(name).name)}</span>
                    {/if}
                    <span class="people-suggestion-name">{name}</span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>

      <div class="field">
        <span>DUE DATE</span>
        <div class="date-picker">
          <button class="date-input-wrap" onclick={toggleDatePicker} aria-expanded={datePickerOpen}>
            <span class="date-input-icon" aria-hidden="true">◷</span>
            <span class:date-input-empty={!due} class="date-input-label">{dueDisplay(due)}</span>
          </button>
          {#if datePickerOpen}
            <div class="date-popover">
              <div class="date-popover-head">
                <button class="date-nav-btn" onclick={() => (visibleMonth = shiftMonth(visibleMonth, -1))} aria-label="Previous month">‹</button>
                <div class="date-month-label">{monthLabel(visibleMonth)}</div>
                <button class="date-nav-btn" onclick={() => (visibleMonth = shiftMonth(visibleMonth, 1))} aria-label="Next month">›</button>
              </div>
              <div class="date-weekdays">
                {#each ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as weekday}
                  <span>{weekday}</span>
                {/each}
              </div>
              <div class="date-grid">
                {#each calendarDays as day}
                  <button
                    class="date-day"
                    class:date-day--muted={!day.inMonth}
                    class:date-day--selected={day.iso === due}
                    class:date-day--today={day.iso === todayIso()}
                    onclick={() => selectDueDate(day.iso)}
                  >
                    {day.label}
                  </button>
                {/each}
              </div>
              <div class="date-popover-actions">
                <button class="text-btn-sm" onclick={clearDueDate}>Clear</button>
                <button class="solid-btn-sm" onclick={() => selectDueDate(todayIso())}>Today</button>
              </div>
            </div>
          {/if}
        </div>
      </div>

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
          <button class="icon-btn icon-btn--danger" onclick={() => requestDeleteTask(editingIndex)} title="Delete task" aria-label="Delete task">
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M9 3h6l1 2h4v2H4V5h4l1-2Zm1 7h2v7h-2v-7Zm4 0h2v7h-2v-7ZM7 10h2v7H7v-7Zm-1 10h12l1-12H5l1 12Z" fill="currentColor"></path>
            </svg>
          </button>
        {/if}
        <div class="foot-right">
          <button class="text-btn" onclick={closeModal}>Cancel</button>
          <button class="solid-btn" onclick={saveEditor} disabled={!title.trim()}>Save</button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if pointerDrag?.active}
  <div
    class="drag-ghost"
    class:drag-ghost--done={$tasks[pointerDrag.index]?.column === "done"}
    style={`left:${pointerDrag.x + 14}px;top:${pointerDrag.y + 14}px;`}
  >
    {#if $tasks[pointerDrag.index]?.column === "done"}
      <span class="drag-ghost-check">✓</span>
    {:else}
      <span class="drag-ghost-bar" style={`background:${priorityColor($tasks[pointerDrag.index]?.priority)}`}></span>
    {/if}
    <div class="drag-ghost-body">
      <div class="drag-ghost-title">{pointerDrag.title}</div>
      {#if $tasks[pointerDrag.index]?.due}
        <div class="drag-ghost-meta">{$tasks[pointerDrag.index].due}</div>
      {/if}
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
    transition: background 0.14s ease, box-shadow 0.14s ease;
  }
  .dropzone--active {
    background: linear-gradient(180deg, rgba(187, 160, 121, 0.12), rgba(187, 160, 121, 0.03));
    box-shadow: inset 0 0 0 1px rgba(180, 141, 78, 0.24);
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
  .card--drop-target {
    box-shadow: inset 0 2px 0 0 var(--accent), 0 4px 14px rgba(60, 50, 30, 0.1);
  }
  .card:active { cursor: grabbing; }
  .drag-ghost {
    position: fixed;
    z-index: 160;
    max-width: 240px;
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
    align-items: flex-start;
    gap: 10px;
  }
  .drag-ghost--done {
    background: rgba(246, 241, 232, 0.97);
  }
  .drag-ghost-bar {
    width: 4px;
    min-height: 34px;
    border-radius: 999px;
    flex: none;
  }
  .drag-ghost-check {
    width: 20px;
    height: 20px;
    border-radius: 6px;
    background: var(--accent);
    color: white;
    display: grid;
    place-items: center;
    font-size: 10px;
    flex: none;
  }
  .drag-ghost-body {
    min-width: 0;
  }
  .drag-ghost-title {
    font-weight: 500;
  }
  .drag-ghost-meta {
    margin-top: 4px;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.06em;
    color: var(--muted-2);
  }
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
    overflow-y: auto;
    padding: 24px;
  }
  .modal {
    background: var(--paper);
    border-radius: 18px;
    padding: 24px 26px 20px;
    width: 460px;
    max-width: calc(100vw - 48px);
    max-height: calc(100vh - 48px);
    overflow: visible;
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
  .date-picker {
    position: relative;
  }
  .date-input-wrap {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    border: 1px solid var(--line-2);
    border-radius: 12px;
    padding: 0 14px;
    min-height: 50px;
    background: var(--card);
    color: var(--ink);
    cursor: pointer;
    text-align: left;
  }
  .date-input-wrap:hover,
  .date-input-wrap[aria-expanded="true"] {
    background: #f7f2ea;
  }
  .date-input-wrap:focus-visible {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(180, 141, 78, 0.12);
  }
  .date-input-icon {
    color: var(--muted-2);
    font-size: 14px;
    line-height: 1;
    flex: none;
  }
  .date-input-label {
    flex: 1;
    font-size: 15px;
    font-family: var(--serif);
    color: var(--ink);
  }
  .date-input-empty {
    color: var(--faint);
  }
  .date-popover {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    z-index: 150;
    width: min(264px, calc(100vw - 96px));
    border: 1px solid var(--line);
    border-radius: 14px;
    background: var(--paper);
    box-shadow: 0 18px 44px rgba(44, 42, 38, 0.18);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .date-popover-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .date-nav-btn {
    width: 30px;
    height: 30px;
    border: 1px solid var(--line-2);
    border-radius: 9px;
    background: var(--card);
    color: var(--ink);
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
  }
  .date-nav-btn:hover {
    background: #f2ebdf;
  }
  .date-month-label {
    font-family: var(--serif);
    font-size: 15px;
    color: var(--ink);
  }
  .date-weekdays,
  .date-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 4px;
  }
  .date-weekdays span {
    text-align: center;
    font-family: var(--mono);
    font-size: 8px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--faint);
  }
  .date-day {
    aspect-ratio: 1;
    border: 1px solid transparent;
    border-radius: 9px;
    background: transparent;
    color: var(--ink);
    font-size: 12px;
    cursor: pointer;
  }
  .date-day:hover {
    background: #f2ebdf;
  }
  .date-day--muted {
    color: #b0a89b;
  }
  .date-day--today {
    border-color: #d9cfbe;
  }
  .date-day--selected {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
  .date-popover-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .solid-btn:disabled {
    cursor: not-allowed;
    opacity: 0.45;
    filter: none;
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
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .people-picker {
    position: relative;
    flex: 1;
  }
  .people-input {
    width: 100%;
    border: 1px solid var(--line-2);
    border-radius: 8px;
    padding: 8px 11px;
    font-size: 13px;
    background: var(--card);
    color: var(--ink);
  }
  .people-input:focus { outline: none; border-color: var(--accent); }
  .people-suggestions {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    right: 0;
    z-index: 150;
    max-height: 220px;
    overflow-y: auto;
    background: var(--paper);
    border: 1px solid var(--line);
    border-radius: 12px;
    box-shadow: 0 18px 44px rgba(44, 42, 38, 0.16);
    padding: 6px;
  }
  .people-suggestion {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    border: none;
    background: transparent;
    border-radius: 10px;
    padding: 8px 10px;
    color: var(--ink);
    text-align: left;
    cursor: pointer;
  }
  .people-suggestion:hover {
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
  .people-suggestion-name {
    font-size: 13px;
  }

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
