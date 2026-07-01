<script>
  import { tick } from "svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import TaskCard from "./TaskCard.svelte";
  import { Trash2, ChevronLeft, ChevronRight } from "lucide-svelte";
  import { appAction, clearAppAction, tasks, people, folders, priorityColor, daysSince, dayLabel, initials, colorForPerson, sidebarCollapsed, toggleSidebar, selectedTaskTitle, screen, selectedSlug } from "../stores.js";
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
  let dueTime  = $state("");
  let priority = $state("med");
  let peopleInput = $state("");  // autocomplete input for adding people
  let peoplePickerOpen = $state(false);
  let peopleInputEl = $state();
  let peoplePickerPos = $state({ top: 0, left: 0, width: 0 });

  // Description rich editor
  let description = $state("");
  let descEditorElement = $state();


  // Date picker portal positioning
  let dateBtnEl = $state();
  let datePopoverPos = $state({ top: 0, left: 0, width: 0 });

  // View modal (read-only card detail)
  let viewModalOpen = $state(false);
  let viewingIndex = $state(-1);

  function syncDescFromEditor() {
    description = descEditorElement?.innerHTML || "";
  }

  async function syncDescEditorFromState() {
    await tick();
    if (descEditorElement) descEditorElement.innerHTML = description || "<p></p>";
  }

  function runDescEditor(command, value = null) {
    descEditorElement?.focus();
    document.execCommand(command, false, value);
    syncDescFromEditor();
  }

  function openView(index) {
    if (dragMoved) { dragMoved = false; return; }
    viewingIndex = index;
    viewModalOpen = true;
  }

  function closeView() {
    viewModalOpen = false;
    viewingIndex = -1;
  }

  function openEditFromView() {
    const idx = viewingIndex;
    closeView();
    openEdit(idx);
  }

  // View mode
  let viewMode = $state("board"); // "board" | "calendar"
  let calMonth = $state(startOfMonthIso(todayIso()));

  const calMonthGrid = $derived(buildMonthGrid(calMonth));

  const tasksByDate = $derived.by(() => {
    const map = new Map();
    $tasks.forEach((task, i) => {
      if (!task.due || task.archived) return;
      if (!map.has(task.due)) map.set(task.due, []);
      map.get(task.due).push({ task, index: i });
    });
    return map;
  });

  // Keyed by "MM-DD" for calendar birthday display
  const birthdaysByMMDD = $derived.by(() => {
    const map = new Map();
    $people.forEach((p) => {
      if (!p.birthday) return;
      if (!map.has(p.birthday)) map.set(p.birthday, []);
      map.get(p.birthday).push(p);
    });
    return map;
  });

  function buildMonthGrid(isoMonth) {
    const first = parseIsoDate(isoMonth) || new Date();
    const year = first.getFullYear();
    const month = first.getMonth();
    const firstDow = new Date(year, month, 1).getDay();
    const daysInMonth = new Date(year, month + 1, 0).getDate();
    const cells = [];
    for (let i = 0; i < firstDow; i++) {
      const d = new Date(year, month, 1 - firstDow + i);
      cells.push({ iso: formatIsoDate(d), inMonth: false, label: d.getDate() });
    }
    for (let d = 1; d <= daysInMonth; d++) {
      cells.push({ iso: formatIsoDate(new Date(year, month, d)), inMonth: true, label: d });
    }
    const rem = cells.length % 7;
    if (rem > 0) {
      for (let i = 1; i <= 7 - rem; i++) {
        const d = new Date(year, month + 1, i);
        cells.push({ iso: formatIsoDate(d), inMonth: false, label: d.getDate() });
      }
    }
    const weeks = [];
    for (let i = 0; i < cells.length; i += 7) weeks.push(cells.slice(i, i + 7));
    return weeks;
  }

  function openNewOnDate(iso) {
    creating = true; editingIndex = -1; modalOpen = true;
    title = ""; taskPeople = []; due = iso; priority = "med"; peopleInput = "";
    dueTime = ""; description = "";
    peoplePickerOpen = false;
    datePickerOpen = false;
    visibleMonth = startOfMonthIso(iso);
    syncDescEditorFromState();
  }

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
  let showBirthdays = $state(true);

  // Task highlight (from dashboard navigation)
  let highlightedTitle = $state("");
  let highlightTimeout = null;

  $effect(() => {
    const title = $selectedTaskTitle;
    if (!title) return;
    highlightedTitle = title;
    selectedTaskTitle.set("");
    if (highlightTimeout) window.clearTimeout(highlightTimeout);
    tick().then(() => {
      const el = document.querySelector(".card--highlighted");
      if (el) el.scrollIntoView({ behavior: "smooth", block: "nearest" });
    });
    highlightTimeout = window.setTimeout(() => {
      highlightedTitle = "";
      highlightTimeout = null;
    }, 2000);
  });

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
    const d = new Date();
    return formatIsoDate(d);
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

  function formatTaskDue(task) {
    if (!task?.due) return "";
    const formattedDate = dueDisplay(task.due);
    if (!task.due_time) return formattedDate;

    const [hoursText, minutesText] = task.due_time.split(":");
    const hours = Number(hoursText);
    const minutes = Number(minutesText);
    if (!Number.isInteger(hours) || !Number.isInteger(minutes)) {
      return `${formattedDate} ${task.due_time}`;
    }

    const meridiem = hours >= 12 ? "PM" : "AM";
    const hour12 = hours % 12 || 12;
    const formattedTime = `${hour12}:${String(minutes).padStart(2, "0")} ${meridiem}`;
    return `${formattedDate} ${formattedTime}`;
  }

  function taskDueDate(task) {
    if (!task?.due) return null;
    if (task.due_time) {
      const parsed = new Date(`${task.due}T${task.due_time}`);
      return Number.isNaN(parsed.getTime()) ? null : parsed;
    }
    const parsed = new Date(`${task.due}T23:59:59`);
    return Number.isNaN(parsed.getTime()) ? null : parsed;
  }

  function taskDueState(task) {
    const dueDate = taskDueDate(task);
    if (!dueDate || task?.done || task?.column === "done" || task?.archived) return "";

    const now = new Date();
    const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const dueStart = new Date(dueDate.getFullYear(), dueDate.getMonth(), dueDate.getDate());
    const weekEnd = new Date(todayStart);
    weekEnd.setDate(weekEnd.getDate() + 7);

    if (dueStart < todayStart || dueDate.getTime() < now.getTime()) return "overdue";
    if (dueStart.getTime() === todayStart.getTime()) return "today";
    if (dueStart <= weekEnd) return "week";
    return "";
  }

  function dueTagLabel(task) {
    return formatTaskDue(task);
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
      due_time: "",
      column: "todo", done: false, completed_at: "", archived: false,
    }, ...$tasks]);
    quickTitle = "";
    quickAdding = false;
  }

  // ── Modal open/close ───────────────────────────────────────────────────
  function openNew() {
    creating = true; editingIndex = -1; modalOpen = true;
    title = ""; taskPeople = []; due = ""; priority = "med"; peopleInput = "";
    dueTime = ""; description = "";
    peoplePickerOpen = false;
    datePickerOpen = false;
    visibleMonth = startOfMonthIso(todayIso());
    syncDescEditorFromState();
  }

  function openEdit(index) {
    const task = $tasks[index];
    if (!task) return;
    creating = false; editingIndex = index; modalOpen = true;
    title = task.title;
    taskPeople = [...taskPeopleList(task)];
    due = task.due || "";
    dueTime = task.due_time || "";
    priority = task.priority || "med";
    description = task.description || "";
    peopleInput = "";
    peoplePickerOpen = false;
    datePickerOpen = false;
    visibleMonth = startOfMonthIso(task.due || todayIso());
    syncDescEditorFromState();
  }

  function closeModal() {
    modalOpen = false; creating = false; editingIndex = -1;
    peoplePickerOpen = false;
    datePickerOpen = false;
    description = "";
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
    if (!datePickerOpen && dateBtnEl) {
      const rect = dateBtnEl.getBoundingClientRect();
      datePopoverPos = { bottom: window.innerHeight - rect.top + 6, left: rect.left };
    }
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
    syncDescFromEditor();
    const next = [...$tasks];
    if (creating) {
      next.unshift({
        title: title.trim() || "Untitled task",
        description,
        people: [...taskPeople],
        due: due.trim(),
        due_time: dueTime.trim(),
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
        description,
        people: [...taskPeople],
        due: due.trim(),
        due_time: dueTime.trim(),
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

  async function moveTaskInModal(column) {
    if (editingIndex < 0 || !$tasks[editingIndex]) return;
    await persist($tasks.map((t, i) => i === editingIndex ? applyColumn(t, column) : t));
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
    openView(index);
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
  <div class="header-left">
    <button class="sidebar-toggle-btn" onclick={toggleSidebar} title={$sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"} aria-label={$sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}>
      <span class="sidebar-toggle-mark" class:sidebar-toggle-mark--collapsed={$sidebarCollapsed}>
        <ChevronLeft size={16} strokeWidth={1.8} />
      </span>
    </button>
    <div>
      <h1>Tasks</h1>
      <p>Track follow-ups, active work, and completed items</p>
    </div>
  </div>
  <div class="header-right">
    <div class="view-toggle">
      <button class="view-toggle-btn" class:active={viewMode === 'board'} onclick={() => (viewMode = 'board')}>Board</button>
      <button class="view-toggle-btn" class:active={viewMode === 'calendar'} onclick={() => (viewMode = 'calendar')}>Calendar</button>
    </div>
    <button class="ghost-btn" onclick={openNew}>+ New task</button>
  </div>
</header>

<div class="body">

{#if viewMode === 'calendar'}
  <div class="cal">
    <div class="cal-nav">
      <button class="cal-nav-btn" onclick={() => (calMonth = shiftMonth(calMonth, -1))}>‹</button>
      <span class="cal-month-label">{monthLabel(calMonth)}</span>
      <button class="cal-nav-btn" onclick={() => (calMonth = shiftMonth(calMonth, 1))}>›</button>
      <button
        class="cal-birthday-toggle"
        class:cal-birthday-toggle--on={showBirthdays}
        onclick={() => (showBirthdays = !showBirthdays)}
        title={showBirthdays ? "Hide birthdays" : "Show birthdays"}
      >🎂 Birthdays</button>
    </div>
    <div class="cal-weekdays">
      {#each ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as wd}
        <div class="cal-weekday">{wd}</div>
      {/each}
    </div>
    <div class="cal-grid" style="grid-template-rows: repeat({calMonthGrid.length}, 1fr);">
      {#each calMonthGrid as week}
        {#each week as cell}
          {@const dayTasks = tasksByDate.get(cell.iso) || []}
          {@const dayBirthdays = showBirthdays ? (birthdaysByMMDD.get(cell.iso.slice(5)) || []) : []}
          <div
            class="cal-cell"
            class:cal-cell--out={!cell.inMonth}
            class:cal-cell--today={cell.iso === todayIso()}
            role="button"
            tabindex="0"
            onclick={() => openNewOnDate(cell.iso)}
            onkeydown={(e) => e.key === "Enter" && openNewOnDate(cell.iso)}
          >
            <div class="cal-day-num">{cell.label}</div>
            <div class="cal-task-list">
              {#each dayBirthdays as person}
                <button
                  class="cal-birthday-chip"
                  onclick={(e) => { e.stopPropagation(); selectedSlug.set(person.slug); screen.set("person"); }}
                  title="{person.name}'s birthday"
                >
                  🎂 {person.name}
                </button>
              {/each}
              {#each dayTasks as { task, index }}
                <button
                  class="cal-task-chip"
                  class:cal-task-chip--done={task.column === "done"}
                  style="border-left: 3px solid {priorityColor(task.priority)}"
                  onclick={(e) => { e.stopPropagation(); openView(index); }}
                >
                  {task.title}
                </button>
              {/each}
            </div>
          </div>
        {/each}
      {/each}
    </div>
  </div>
{/if}

{#if viewMode === 'board'}
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
          {@const dueState = taskDueState(task)}
          <div data-task-index={index}>
            <TaskCard
              {task}
              dueState={dueState}
              dueLabel={dueTagLabel(task)}
              dropTarget={activeDropIndex === index && dragIndex !== index}
              onPointerDown={(event) => beginPointerDrag(event, index)}
              onActivate={() => maybeOpenEdit(index)}
              extraClass={highlightedTitle === task.title ? "card--highlighted" : ""}
            />
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
          {@const dueState = taskDueState(task)}
          <div data-task-index={index}>
            <TaskCard
              {task}
              dueState={dueState}
              dueLabel={dueTagLabel(task)}
              dropTarget={activeDropIndex === index && dragIndex !== index}
              onPointerDown={(event) => beginPointerDrag(event, index)}
              onActivate={() => maybeOpenEdit(index)}
              extraClass={highlightedTitle === task.title ? "card--doing card--highlighted" : "card--doing"}
            />
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
                class:card--highlighted={highlightedTitle === task.title}
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
                  {#if taskPeopleList(task).length}<span class="person-text">{taskPeopleList(task).join(", ")}</span>{/if}
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
                          {#if taskPeopleList(task).length}<span class="person-text">{taskPeopleList(task).join(", ")}</span>{/if}
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
{/if}

</div>

<!-- ── Task modal ── -->
{#if modalOpen}
  <div class="overlay" role="dialog" tabindex="-1">
    <div class="modal">
      <div class="modal-head">{creating ? "New task" : "Edit task"}</div>

      <div class="modal-scroll">
      <label class="field">
        <span>TITLE</span>
        <textarea class="task-title-input" bind:value={title} placeholder="What needs doing?" rows="1"></textarea>
      </label>

      <div class="field">
        <span>DESCRIPTION</span>
        <div class="toolbar">
          <button type="button" class="tool-btn" title="Bold" aria-label="Bold" onclick={() => runDescEditor("bold")}><strong>B</strong></button>
          <button type="button" class="tool-btn" title="Italic" aria-label="Italic" onclick={() => runDescEditor("italic")}><em>I</em></button>
          <button type="button" class="tool-btn" title="Underline" aria-label="Underline" onclick={() => runDescEditor("underline")}><u>U</u></button>
          <div class="tool-sep"></div>
          <button type="button" class="tool-btn" title="Normal text" aria-label="Normal text" onclick={() => runDescEditor("formatBlock", "p")}>¶</button>
          <button type="button" class="tool-btn" title="Heading 1" aria-label="Heading 1" onclick={() => runDescEditor("formatBlock", "h1")}>H1</button>
          <button type="button" class="tool-btn" title="Heading 2" aria-label="Heading 2" onclick={() => runDescEditor("formatBlock", "h2")}>H2</button>
          <button type="button" class="tool-btn" title="Heading 3" aria-label="Heading 3" onclick={() => runDescEditor("formatBlock", "h3")}>H3</button>
          <div class="tool-sep"></div>
          <button type="button" class="tool-btn" title="Bulleted list" aria-label="Bulleted list" onclick={() => runDescEditor("insertUnorderedList")}>• List</button>
          <div class="tool-sep"></div>
          <div class="tool-color-wrap" title="Font color">
            <span class="tool-color-meta">
              <span class="tool-color-label">A</span>
              <span class="tool-color-copy">Font color</span>
            </span>
            <input
              class="tool-color-input"
              type="color"
              value="#000000"
              oninput={(e) => { descEditorElement?.focus(); document.execCommand("foreColor", false, e.currentTarget.value); syncDescFromEditor(); }}
            />
          </div>
        </div>
        <div
          class="desc-editor"
          contenteditable="true"
          bind:this={descEditorElement}
          oninput={syncDescFromEditor}
        ></div>
      </div>

      <div class="field">
        <span>PEOPLE</span>
        <div class="people-chips">
          {#each taskPeople as name}
            {@const p = personFor(name)}
            <span class="people-chip">
              {#if p}
                <span class="chip-av" style="background:{colorForPerson(p, $folders)}">{initials(p.name)}</span>
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
              bind:this={peopleInputEl}
              bind:value={peopleInput}
              placeholder="Add person…"
              onfocus={() => {
                if (peopleInputEl) {
                  const rect = peopleInputEl.getBoundingClientRect();
                  const dropH = Math.min(filteredPeopleOptions.length * 44 + 12, 220);
                  const spaceBelow = window.innerHeight - rect.bottom - 6;
                  const top = spaceBelow >= dropH ? rect.bottom + 6 : Math.max(8, rect.top - dropH - 6);
                  peoplePickerPos = { top, left: rect.left, width: rect.width };
                }
                peoplePickerOpen = true;
              }}
              oninput={() => {
                if (peopleInputEl) {
                  const rect = peopleInputEl.getBoundingClientRect();
                  const dropH = Math.min(filteredPeopleOptions.length * 44 + 12, 220);
                  const spaceBelow = window.innerHeight - rect.bottom - 6;
                  const top = spaceBelow >= dropH ? rect.bottom + 6 : Math.max(8, rect.top - dropH - 6);
                  peoplePickerPos = { top, left: rect.left, width: rect.width };
                }
                peoplePickerOpen = true;
              }}
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
          </div>
        </div>
      </div>

      <div class="due-time-row">
      <div class="field" style="flex:1">
        <span>DUE DATE</span>
        <div class="date-picker">
          <button class="date-input-wrap" bind:this={dateBtnEl} onclick={toggleDatePicker} aria-expanded={datePickerOpen}>
            <span class="date-input-icon" aria-hidden="true">◷</span>
            <span class:date-input-empty={!due} class="date-input-label">{dueDisplay(due)}</span>
          </button>
        </div>
      </div>

      <label class="field">
        <span>TIME</span>
        <input bind:value={dueTime} type="time" />
      </label>
      </div><!-- end due-time-row -->

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
                onclick={() => moveTaskInModal(col.key)}
                disabled={$tasks[editingIndex]?.column === col.key}
              >{col.label}</button>
            {/each}
          </div>
        </div>
      {/if}
      </div><!-- end modal-scroll -->

      <div class="modal-foot">
        {#if !creating && editingIndex >= 0}
          <button class="del-ghost" onclick={() => requestDeleteTask(editingIndex)}>Delete</button>
        {/if}
        <div class="foot-right">
          <button class="text-btn" onclick={closeModal}>Cancel</button>
          <button class="solid-btn" onclick={saveEditor} disabled={!title.trim()}>Save</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- ── Task view modal ── -->
{#if viewModalOpen && $tasks[viewingIndex]}
  {@const vt = $tasks[viewingIndex]}
  {@const vtPeople = Array.isArray(vt.people) ? vt.people : (vt.person && vt.person !== "—" ? [vt.person] : [])}
  <div class="overlay" role="dialog" tabindex="-1" onclick={(e) => e.target === e.currentTarget && closeView()} onkeydown={(e) => e.key === "Escape" && closeView()}>
    <div class="view-modal">
      <span class="view-bar" style="background:{priorityColor(vt.priority)}"></span>
      <div class="view-head">
        <div class="view-head-body">
          <h2 class="view-title">{vt.title}</h2>
          <div class="view-meta">
            {#if vtPeople.length}
              <div class="view-meta-people">
                {#each vtPeople as name}
                  {@const p = $people.find((person) => person.name === name)}
                  {#if p}
                    <span class="view-av" style="background:{colorForPerson(p, $folders)}" title={p.name}>{initials(p.name)}</span>
                  {:else}
                    <span class="view-meta-text">{name}</span>
                  {/if}
                {/each}
              </div>
            {/if}
            {#if vt.due}
              {#if vtPeople.length}<span class="view-meta-sep">·</span>{/if}
              <span class="view-meta-text">{dueTagLabel(vt)}</span>
            {/if}
            {#if vt.due || vtPeople.length}<span class="view-meta-sep">·</span>{/if}
            <span class="view-meta-text">{vt.column === "todo" ? "To do" : vt.column === "doing" ? "In progress" : "Done"}</span>
          </div>
        </div>
        <button class="view-close-btn" onclick={closeView} title="Close" aria-label="Close">×</button>
      </div>

      {#if vt.description}
        <div class="view-scroll">
          <div class="view-desc">{@html vt.description}</div>
        </div>
      {/if}

      <div class="modal-foot">
        <div class="foot-right">
          <button class="text-btn" onclick={closeView}>Close</button>
          <button class="solid-btn" onclick={openEditFromView}>Edit</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- ── Date picker portal (fixed, escapes modal overflow) ── -->
{#if peoplePickerOpen && filteredPeopleOptions.length}
  <div
    class="people-suggestions"
    style="top:{peoplePickerPos.top}px;left:{peoplePickerPos.left}px;width:{peoplePickerPos.width}px;"
  >
    {#each filteredPeopleOptions as name}
      <button class="people-suggestion" onmousedown={() => addPersonToTask(name)}>
        {#if personFor(name)}
          <span class="people-suggestion-av" style="background:{colorForPerson(personFor(name), $folders)}">{initials(personFor(name).name)}</span>
        {/if}
        <span class="people-suggestion-name">{name}</span>
      </button>
    {/each}
  </div>
{/if}

{#if datePickerOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="date-portal-backdrop" onclick={() => (datePickerOpen = false)}></div>
  <div class="date-popover date-popover--fixed" style="bottom:{datePopoverPos.bottom}px;left:{datePopoverPos.left}px;">
    <div class="date-popover-head">
      <button class="date-nav-btn" onclick={() => (visibleMonth = shiftMonth(visibleMonth, -1))} aria-label="Previous month">‹</button>
      <div class="date-month-label">{monthLabel(visibleMonth)}</div>
      <button class="date-nav-btn" onclick={() => (visibleMonth = shiftMonth(visibleMonth, 1))} aria-label="Next month">›</button>
    </div>
    <div class="date-weekdays">
      {#each ["S", "M", "T", "W", "T", "F", "S"] as weekday}
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
  /* ── View toggle ── */
  .header-right {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .view-toggle {
    display: flex;
    border: 1px solid var(--line);
    border-radius: 10px;
    overflow: hidden;
    background: var(--card);
  }
  .view-toggle-btn {
    padding: 6px 14px;
    font-size: 13px;
    color: var(--muted-2);
    background: transparent;
    border: none;
    cursor: pointer;
    font-family: inherit;
  }
  .view-toggle-btn:hover { background: #f2ebe0; color: var(--ink); }
  .view-toggle-btn.active { background: var(--accent); color: white; }

  /* ── Calendar ── */
  .cal {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .cal-nav {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .cal-nav > .cal-month-label {
    flex: 1;
    text-align: center;
  }
  .cal-birthday-toggle {
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    color: var(--muted-2);
    font-size: 12px;
    padding: 5px 10px;
    cursor: pointer;
    white-space: nowrap;
  }
  .cal-birthday-toggle:hover { background: #f2ebe0; color: var(--ink); }
  .cal-birthday-toggle--on { background: #fdf3e8; color: var(--accent); border-color: #e8c98a; }
  .cal-nav-btn {
    width: 32px;
    height: 32px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    color: var(--ink);
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .cal-nav-btn:hover { background: #f2ebe0; }
  .cal-month-label {
    font-family: var(--serif);
    font-size: 20px;
    color: var(--ink);
  }
  .cal-weekdays {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 4px;
    flex-shrink: 0;
  }
  .cal-weekday {
    text-align: center;
    font-size: 10px;
    font-family: var(--mono);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--faint);
    padding: 4px 0;
  }
  .cal-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 4px;
    flex: 1;
    min-height: 0;
  }
  .cal-cell {
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--card);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow: hidden;
    cursor: pointer;
    min-height: 0;
    transition: background 0.1s;
  }
  .cal-cell:hover { background: #f7f2ea; }
  .cal-cell--out {
    background: rgba(251, 247, 240, 0.4);
    border-color: rgba(210, 200, 185, 0.35);
  }
  .cal-cell--out:hover { background: rgba(247, 242, 234, 0.7); }
  .cal-cell--today {
    border-color: var(--accent);
    background: rgba(180, 141, 78, 0.05);
  }
  .cal-day-num {
    font-size: 12px;
    font-weight: 600;
    color: var(--ink);
    flex-shrink: 0;
    line-height: 1;
  }
  .cal-cell--out .cal-day-num { color: var(--faint); }
  .cal-cell--today .cal-day-num { color: var(--accent); }
  .cal-task-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }
  .cal-task-chip {
    width: 100%;
    text-align: left;
    font-size: 11px;
    line-height: 1.3;
    color: var(--ink);
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 5px;
    padding: 3px 6px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
    font-family: inherit;
  }
  .cal-task-chip:hover { background: #ede6d8; }
  .cal-birthday-chip {
    width: 100%;
    text-align: left;
    font-size: 11px;
    line-height: 1.3;
    color: #7a4f2a;
    background: #fdf3e8;
    border: 1px solid #e8c98a;
    border-radius: 5px;
    padding: 3px 6px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
    font-family: inherit;
  }
  .cal-birthday-chip:hover { background: #fde8c6; }
  .cal-task-chip--done {
    text-decoration: line-through;
    color: var(--muted-2);
    border-left-color: transparent !important;
    opacity: 0.7;
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
  .card--highlighted {
    background: linear-gradient(180deg, #fffaf1, var(--card));
    box-shadow: 0 0 0 2px rgba(180, 141, 78, 0.4), 0 3px 12px rgba(60, 50, 30, 0.1);
    border-color: #e4cfab;
  }
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
  .card-title {
    font-size: 13px;
    line-height: 1.4;
    color: var(--ink);
    font-weight: 500;
    padding-right: 28px;
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
  .person-text { font-size: 11px; color: var(--muted-2); }
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
    width: 580px;
    max-width: calc(100vw - 48px);
    max-height: calc(100vh - 48px);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 16px 48px rgba(44, 42, 38, 0.2);
  }
  .modal-head {
    padding: 24px 26px 0;
    font-family: var(--serif);
    font-size: 22px;
    color: var(--ink);
    flex-shrink: 0;
  }
  .modal-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 14px 26px;
    display: flex;
    flex-direction: column;
    gap: 14px;
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
  .field > input,
  .field > textarea {
    border: 1px solid var(--line-2);
    border-radius: 8px;
    padding: 9px 11px;
    font-size: 14px;
    background: var(--card);
    color: var(--ink);
  }
  .field > input:focus,
  .field > textarea:focus {
    outline: none; border-color: var(--accent);
  }
  .task-title-input {
    min-height: 38px;
    resize: none;
    line-height: 1.45;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
    margin-bottom: 8px;
  }
  .tool-btn {
    border: 1px solid var(--line-2);
    background: var(--card);
    border-radius: 8px;
    padding: 7px 10px;
    font-size: 12px;
    color: var(--ink);
    cursor: pointer;
    white-space: nowrap;
  }
  .tool-btn:hover { background: var(--panel-2, #f5efe4); }
  .tool-sep {
    width: 1px;
    height: 20px;
    background: var(--line-2);
    align-self: center;
  }
  .tool-color-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
    border: 1px solid var(--line-2);
    background: var(--card);
    border-radius: 8px;
    padding: 5px 12px;
    cursor: pointer;
    gap: 8px;
    min-width: 132px;
  }
  .tool-color-wrap:hover { background: var(--panel-2, #f5efe4); }
  .tool-color-meta {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    pointer-events: none;
  }
  .tool-color-label {
    font-family: var(--serif);
    font-size: 18px;
    color: var(--ink);
    line-height: 1;
  }
  .tool-color-copy {
    font-family: var(--mono);
    font-size: 9px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--faint);
  }
  .tool-color-input {
    position: absolute;
    inset: 0;
    opacity: 0;
    width: 100%;
    height: 100%;
    cursor: pointer;
    border: none;
    padding: 0;
  }
  .desc-editor {
    height: 180px;
    overflow-y: auto;
    border: 1px solid var(--line-2);
    border-radius: 10px;
    padding: 14px;
    background: #fffdf9;
    font-family: var(--sans);
    font-weight: 300;
    font-size: 15px;
    line-height: 1.7;
    color: var(--ink);
  }
  .desc-editor:focus {
    outline: none;
    border-color: var(--accent);
  }
  .desc-editor :global(p),
  .desc-editor :global(div) { margin: 0; }
  .desc-editor :global(h1) { font-size: 20px; font-weight: 700; margin: 8px 0 3px; line-height: 1.3; }
  .desc-editor :global(h2) { font-size: 16px; font-weight: 600; margin: 6px 0 2px; line-height: 1.3; }
  .desc-editor :global(h3) { font-size: 14px; font-weight: 600; margin: 5px 0 2px; line-height: 1.3; }
  .desc-editor :global(ul) { margin: 0 0 6px; padding-left: 20px; }
  .desc-editor :global(li) { margin-bottom: 2px; }
  .due-time-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  .due-time-row .field:last-child {
    flex: none;
    width: 140px;
  }
  .due-time-row .field > input[type="time"] {
    height: 38px;
    box-sizing: border-box;
    padding: 0 11px;
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
    border-radius: 8px;
    padding: 9px 11px;
    height: 38px;
    box-sizing: border-box;
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
  .date-portal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 149;
  }
  .date-popover {
    z-index: 150;
    width: 240px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--paper);
    box-shadow: 0 12px 32px rgba(44, 42, 38, 0.16);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .date-popover--fixed {
    position: fixed;
  }
  .date-popover-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .date-nav-btn {
    width: 22px;
    height: 22px;
    border: 1px solid var(--line-2);
    border-radius: 6px;
    background: var(--card);
    color: var(--ink);
    cursor: pointer;
    font-size: 13px;
    line-height: 1;
  }
  .date-nav-btn:hover {
    background: #f2ebdf;
  }
  .date-month-label {
    font-family: var(--serif);
    font-size: 12px;
    color: var(--ink);
  }
  .date-weekdays,
  .date-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 1px;
  }
  .date-weekdays span {
    text-align: center;
    font-family: var(--mono);
    font-size: 7px;
    letter-spacing: 0.04em;
    color: var(--faint);
    padding: 2px 0;
  }
  .date-day {
    aspect-ratio: 1;
    border: 1px solid transparent;
    border-radius: 5px;
    background: transparent;
    color: var(--ink);
    font-size: 10px;
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
    position: fixed;
    z-index: 200;
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
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 26px 20px;
    border-top: 1px solid var(--line);
    flex-shrink: 0;
  }
  .foot-right { display: flex; gap: 8px; align-items: center; margin-left: auto; }

  @media (max-width: 1100px) {
    .board { grid-template-columns: 1fr; }
  }

  /* ── View modal ── */
  .view-modal {
    position: relative;
    background: var(--paper);
    border-radius: 18px;
    width: 600px;
    max-width: calc(100vw - 48px);
    max-height: calc(100vh - 48px);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 16px 48px rgba(44, 42, 38, 0.2);
  }
  .view-bar {
    position: absolute;
    left: 0;
    top: 16px;
    bottom: 16px;
    width: 4px;
    border-radius: 999px;
  }
  .view-head {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 24px 28px 18px 32px;
    flex-shrink: 0;
  }
  .view-head-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .view-title {
    font-family: var(--serif);
    font-size: 24px;
    font-weight: 500;
    line-height: 1.2;
    color: var(--ink);
    margin: 0;
  }
  .view-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .view-meta-people {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .view-av {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: white;
    font-size: 8px;
    font-weight: 700;
    flex: none;
  }
  .view-meta-text {
    font-size: 12px;
    color: var(--muted-2);
  }
  .view-meta-sep {
    font-size: 11px;
    color: var(--line-2);
  }
  .view-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 28px 18px 32px;
  }
  .view-desc {
    font-size: 15px;
    line-height: 1.7;
    color: var(--ink);
  }
  .view-desc :global(p) { margin: 0 0 6px; }
  .view-desc :global(p:last-child) { margin-bottom: 0; }
  .view-desc :global(h2) { font-size: 16px; font-weight: 600; margin: 8px 0 4px; }
  .view-desc :global(h3) { font-size: 14px; font-weight: 600; margin: 6px 0 3px; }
  .view-desc :global(ul) { margin: 0 0 6px; padding-left: 20px; }
  .view-desc :global(li) { margin-bottom: 2px; }
  .person-av {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: white;
    font-size: 9px;
    font-weight: 700;
    flex: none;
  }
  .view-close-btn {
    border: none;
    background: none;
    font-size: 20px;
    line-height: 1;
    color: var(--muted-2);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 6px;
    flex: none;
  }
  .view-close-btn:hover { color: var(--ink); background: var(--panel); }
</style>
