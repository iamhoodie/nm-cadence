<script>
  import { onMount } from "svelte";
  import { AlertCircle, CalendarDays, ChevronLeft, Clock3 } from "lucide-svelte";
  import TaskCard from "./TaskCard.svelte";
  import {
    colorForPerson,
    folders,
    formatDate,
    initials,
    people,
    screen,
    selectedSlug,
    selectedTaskTitle,
    sidebarCollapsed,
    tasks,
    toggleSidebar,
  } from "../stores.js";
  import { getAppSettings } from "../api.js";

  let stale1on1Days = $state(14);

  function parseIsoDate(iso) {
    if (!iso) return null;
    const [year, month, day] = String(iso).split("-").map(Number);
    if (!year || !month || !day) return null;
    return new Date(year, month - 1, day);
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

  function startOfDay(date) {
    return new Date(date.getFullYear(), date.getMonth(), date.getDate());
  }

  function friendlyDue(task) {
    if (!task?.due) return "";
    const date = parseIsoDate(task.due);
    if (!date) return task.due;

    const label = date.toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
      year: "numeric",
    });

    if (!task.due_time) return label;

    const [hoursText, minutesText] = task.due_time.split(":");
    const hours = Number(hoursText);
    const minutes = Number(minutesText);
    if (!Number.isInteger(hours) || !Number.isInteger(minutes)) {
      return `${label} ${task.due_time}`;
    }

    const meridiem = hours >= 12 ? "PM" : "AM";
    const hour12 = hours % 12 || 12;
    return `${label} ${hour12}:${String(minutes).padStart(2, "0")} ${meridiem}`;
  }

  function daysSinceLastMet(person) {
    if (!person?.last_met) return Number.POSITIVE_INFINITY;
    const parsed = new Date(String(person.last_met).replace(" ", "T"));
    if (Number.isNaN(parsed.getTime())) return Number.POSITIVE_INFINITY;
    return Math.round((startOfDay(new Date()) - startOfDay(parsed)) / 86400000);
  }

  const actionableTasks = $derived(
    $tasks.filter((task) => !task.archived && task.column !== "done" && task.due)
  );

  const dashboardTaskBuckets = $derived.by(() => {
    const now = new Date();
    const today = startOfDay(now);
    const weekEnd = new Date(today);
    weekEnd.setDate(weekEnd.getDate() + 7);

    const overdue = [];
    const todayItems = [];
    const thisWeek = [];

    for (const task of actionableTasks) {
      const due = taskDueDate(task);
      if (!due) continue;
      const dueStart = startOfDay(due);

      if (dueStart < today || due.getTime() < now.getTime()) {
        overdue.push(task);
      } else if (dueStart.getTime() === today.getTime()) {
        todayItems.push(task);
      } else if (dueStart <= weekEnd) {
        thisWeek.push(task);
      }
    }

    const sorter = (a, b) => {
      const aDue = taskDueDate(a)?.getTime() || 0;
      const bDue = taskDueDate(b)?.getTime() || 0;
      return aDue - bDue;
    };

    overdue.sort(sorter);
    todayItems.sort(sorter);
    thisWeek.sort(sorter);

    return { overdue, today: todayItems, thisWeek };
  });

  const stalePeople = $derived(
    $people
      .map((person) => ({
        ...person,
        staleDays: daysSinceLastMet(person),
      }))
      .filter((person) => {
        if ($folders.find((f) => f.name === person.group)?.exclude_checkin) return false;
        return !person.last_met || person.staleDays > stale1on1Days;
      })
      .sort((a, b) => {
        if (!a.last_met && !b.last_met) return a.name.localeCompare(b.name);
        if (!a.last_met) return -1;
        if (!b.last_met) return 1;
        return b.staleDays - a.staleDays;
      })
  );

  function openPerson(slug) {
    selectedSlug.set(slug);
    screen.set("person");
  }

  function openTasks(task = null) {
    if (task?.title) selectedTaskTitle.set(task.title);
    screen.set("tasks");
  }

  onMount(async () => {
    try {
      const settings = await getAppSettings();
      stale1on1Days = settings.stale_1on1_days || 14;
    } catch {
      stale1on1Days = 14;
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
      <h1>Dashboard</h1>
      <p>See what needs attention this week</p>
    </div>
  </div>
  <div class="header-date">{new Date().toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric", year: "numeric" })}</div>
</header>

<div class="body">
  <section class="panel">
    <div class="panel-head">
      <div>
        <!-- <div class="mono-label">TASKS</div> -->
        <div class="panel-title">What needs attention</div>
      </div>
      <button class="text-btn" onclick={openTasks}>Open tasks</button>
    </div>

    <div class="task-grid">

      <div class="task-section task-section--today">
        <div class="task-section-head">
          <span class="task-icon"><Clock3 size={16} strokeWidth={2} /></span>
          <span>Due today</span>
          <span class="task-count">{dashboardTaskBuckets.today.length}</span>
        </div>
        {#if dashboardTaskBuckets.today.length}
          <div class="task-list">
            {#each dashboardTaskBuckets.today as task}
              <TaskCard {task} dueState="today" dueLabel={friendlyDue(task)} onActivate={() => openTasks(task)} />
            {/each}
          </div>
        {:else}
          <div class="empty-copy">Nothing due today.</div>
        {/if}
      </div>

      <div class="task-section">
        <div class="task-section-head">
          <span class="task-icon"><CalendarDays size={16} strokeWidth={2} /></span>
          <span>Due this week</span>
          <span class="task-count">{dashboardTaskBuckets.thisWeek.length}</span>
        </div>
        {#if dashboardTaskBuckets.thisWeek.length}
          <div class="task-list">
            {#each dashboardTaskBuckets.thisWeek as task}
              <TaskCard {task} dueState="week" dueLabel={friendlyDue(task)} onActivate={() => openTasks(task)} />
            {/each}
          </div>
        {:else}
          <div class="empty-copy">Nothing else due this week.</div>
        {/if}
      </div>
      <div class="task-section task-section--overdue">
        <div class="task-section-head">
          <span class="task-icon"><AlertCircle size={16} strokeWidth={2} /></span>
          <span>Overdue</span>
          <span class="task-count">{dashboardTaskBuckets.overdue.length}</span>
        </div>
        {#if dashboardTaskBuckets.overdue.length}
          <div class="task-list">
            {#each dashboardTaskBuckets.overdue as task}
              <TaskCard {task} dueState="overdue" dueLabel={friendlyDue(task)} onActivate={() => openTasks(task)} />
            {/each}
          </div>
        {:else}
          <div class="empty-copy">Nothing overdue right now.</div>
        {/if}
      </div>

    </div>
  </section>

  <section class="panel panel--grow">
    <div class="panel-head">
      <div>
        <!-- <div class="mono-label">1:1 CADENCE</div> -->
        <div class="panel-title">People needing a check-in</div>
      </div>
      <button class="text-btn" onclick={() => screen.set("settings")}>Threshold: {stale1on1Days} days</button>
    </div>

    {#if stalePeople.length}
      <div class="people-list">
        {#each stalePeople as person}
          <button class="person-row" onclick={() => openPerson(person.slug)}>
            <span class="person-avatar" style="background:{colorForPerson(person, $folders)}">{initials(person.name)}</span>
            <span class="person-copy">
              <span class="person-name">{person.name}</span>
              <span class="person-meta">
                {#if person.last_met}
                  Last 1:1 {formatDate(person.last_met)} · {person.staleDays} days ago
                {:else}
                  No 1:1 logged yet
                {/if}
              </span>
            </span>
          </button>
        {/each}
      </div>
    {:else}
      <div class="empty-large">
        <!-- <div class="empty-title">Everyone is within cadence</div> -->
        <div class="empty-sub">Nobody is beyond your {stale1on1Days}-day 1:1 threshold right now.</div>
      </div>
    {/if}
  </section>
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
  .header-date {
    font-family: var(--serif);
    font-size: 14px;
    color: var(--faint);
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
  }
  .body {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    padding: 24px 32px 24px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  .panel {
    border: 1px solid var(--line);
    border-radius: 18px;
    background: var(--panel);
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .panel--grow {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .panel-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
  }
  .panel-title {
    font-family: var(--serif);
    font-size: 24px;
    color: var(--ink);
  }
  .task-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 14px;
    flex: 1;
    min-height: 0;
  }
  .task-section {
    min-width: 0;
    min-height: 0;
    border: 1px solid var(--line);
    background: rgba(251, 247, 240, 0.72);
    border-radius: 16px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: hidden;
  }
  .task-section--overdue {
    background: rgba(251, 247, 240, 0.72);
    border-color: rgba(160, 72, 72, 0.2);
  }
  .task-section--today {
    background: rgba(251, 247, 240, 0.72);
    border-color: rgba(184, 141, 78, 0.24);
  }
  .task-section-head {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--ink);
  }
  .task-icon {
    width: 28px;
    height: 28px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: #efe7d9;
    color: var(--ink);
    flex: none;
  }
  .task-section--overdue .task-icon {
    color: var(--over);
    background: rgba(160, 72, 72, 0.08);
  }
  .task-section--today .task-icon {
    color: var(--due);
    background: rgba(184, 141, 78, 0.12);
  }
  .task-count {
    margin-left: auto;
    font-size: 11px;
    color: var(--faint);
    background: #ebe3d5;
    border-radius: 999px;
    padding: 2px 8px;
  }
  .task-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-right: 4px;
  }
  .people-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-right: 4px;
  }
  .person-row {
    width: 100%;
    border: 1px solid var(--line);
    background: white;
    border-radius: 14px;
    padding: 12px 14px;
    text-align: left;
    color: var(--ink);
    display: flex;
    gap: 12px;
    align-items: center;
  }
  .person-row:hover,
  .person-row:focus-visible {
    background: #f4ede1;
    outline: none;
  }
  .person-name {
    font-size: 15px;
    font-weight: 600;
  }
  .person-meta,
  .empty-copy,
  .empty-sub {
    font-size: 12px;
    color: var(--muted-2);
    line-height: 1.5;
  }
  .person-avatar {
    width: 40px;
    height: 40px;
    border-radius: 999px;
    display: grid;
    place-items: center;
    color: white;
    font-weight: 700;
    font-size: 14px;
    flex: none;
  }
  .person-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .empty-large {
    border: 1px dashed var(--line-2);
    border-radius: 16px;
    padding: 26px 18px;
    text-align: center;
    background: rgba(251, 247, 240, 0.66);
  }
  .empty-title {
    font-family: var(--serif);
    font-size: 24px;
    color: var(--ink);
  }
  .empty-sub {
    margin-top: 6px;
  }
  @media (max-width: 1080px) {
    .task-grid {
      grid-template-columns: 1fr;
    }
  }
  @media (max-width: 760px) {
    header {
      padding: 18px 20px 16px;
      align-items: flex-start;
      gap: 12px;
      flex-direction: column;
    }
    .body {
      padding: 20px;
    }
    .panel-head {
      flex-direction: column;
    }
  }
</style>
