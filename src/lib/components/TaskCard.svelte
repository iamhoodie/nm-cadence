<script>
  import { AlertCircle, CalendarDays, Clock } from "lucide-svelte";
  import { colorForPerson, folders as foldersStore, initials, people as peopleStore, priorityColor } from "../stores.js";

  let {
    task,
    dueState = "",
    dueLabel = "",
    dropTarget = false,
    onActivate = () => {},
    onPointerDown = null,
    extraClass = "",
    folders = null,
    people = null,
  } = $props();

  const folderList = $derived(folders ?? $foldersStore);
  const peopleList = $derived(people ?? $peopleStore);

  function personFor(name) {
    return name ? peopleList.find((p) => p.name === name) : null;
  }

  function taskPeopleList(task) {
    if (Array.isArray(task?.people)) return task.people;
    if (task?.person && task.person !== "—") return [task.person];
    return [];
  }

  function handleKeydown(event) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onActivate();
    }
  }
</script>

<button
  type="button"
  class={`card ${extraClass}`.trim()}
  class:card--overdue={dueState === "overdue"}
  class:card--due-today={dueState === "today"}
  class:card--drop-target={dropTarget}
  onpointerdown={onPointerDown}
  onclick={onActivate}
  onkeydown={handleKeydown}
>
  {#if dueState}
    <span
      class="card-state-badge"
      class:card-state-badge--overdue={dueState === "overdue"}
      class:card-state-badge--today={dueState === "today"}
      aria-label={dueState === "overdue" ? "Overdue" : dueState === "today" ? "Due today" : "Due this week"}
      title={dueState === "overdue" ? "Overdue" : dueState === "today" ? "Due today" : "Due this week"}
    >
      {#if dueState === "overdue"}
        <AlertCircle size={14} strokeWidth={2} aria-hidden="true" />
      {:else if dueState === "today"}
        <Clock size={14} strokeWidth={2} aria-hidden="true" />
      {:else}
        <CalendarDays size={14} strokeWidth={2} aria-hidden="true" />
      {/if}
    </span>
  {/if}
  <span class="card-bar" style="background:{priorityColor(task.priority)}"></span>
  <span class="card-title">{task.title}</span>
  <div class="card-foot">
    <div class="card-people">
      {#each taskPeopleList(task).slice(0, 3) as name}
        {@const p = personFor(name)}
        {#if p}
          <span class="person-av" style="background:{colorForPerson(p, folderList)}" title={p.name}>{initials(p.name)}</span>
        {:else}
          <span class="person-text">{name}</span>
        {/if}
      {/each}
    </div>
    {#if task.due}
      <div class="card-due-row">
        <span class="due-tag" class:due-tag--overdue={dueState === "overdue"} class:due-tag--today={dueState === "today"}>{dueLabel}</span>
      </div>
    {/if}
  </div>
</button>

<style>
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
    border-inline-width: 1px;
    color: var(--ink);
  }
  .card:hover {
    box-shadow: 0 3px 12px rgba(60, 50, 30, 0.1);
    transform: translateY(-1px);
  }
  .card--doing {
    background: #fffefb;
  }
  .card--overdue {
    background: linear-gradient(180deg, #fff7f5, #fffdfb);
    border-color: #e3b7b0;
    box-shadow: inset 0 0 0 1px rgba(178, 85, 72, 0.12), 0 2px 8px rgba(178, 85, 72, 0.08);
  }
  .card--due-today {
    background: linear-gradient(180deg, #fffaf1, #fffdfb);
    border-color: #e4cfab;
  }
  .card--drop-target {
    box-shadow: inset 0 2px 0 0 var(--accent), 0 4px 14px rgba(60, 50, 30, 0.1);
  }
  .card--highlighted {
    background: linear-gradient(180deg, #fffaf1, var(--card));
    box-shadow: 0 0 0 2px rgba(180, 141, 78, 0.4), 0 3px 12px rgba(60, 50, 30, 0.1);
    border-color: #e4cfab;
  }
  .card-state-badge {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 24px;
    height: 24px;
    border-radius: 999px;
    display: grid;
    place-items: center;
    flex: none;
  }
  .card-state-badge :global(svg) {
    width: 14px;
    height: 14px;
    display: block;
  }
  .card-state-badge--overdue {
    background: #f3d8d2;
    color: var(--over);
    box-shadow: inset 0 0 0 1px rgba(178, 85, 72, 0.18);
  }
  .card-state-badge--today {
    background: #f2e3bf;
    color: #8c6430;
    box-shadow: inset 0 0 0 1px rgba(184, 137, 63, 0.18);
  }
  .card-state-badge:not(.card-state-badge--overdue):not(.card-state-badge--today) {
    background: #e7e0cf;
    color: #7b705b;
    box-shadow: inset 0 0 0 1px rgba(147, 132, 106, 0.18);
  }
  .card-bar {
    position: absolute;
    left: -1px;
    top: 11px;
    bottom: 11px;
    width: 4px;
    border-radius: 999px;
  }
  .card-title {
    font-size: 14px;
    line-height: 1.28;
    font-weight: 550;
    padding-right: 30px;
    color: var(--ink);
    word-break: break-word;
  }
  .card-foot {
    display: flex;
    flex-direction: row;
    align-items: flex-end;
    justify-content: space-between;
    gap: 8px;
    margin-top: 8px;
    width: 100%;
  }
  .card-people {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    flex-wrap: wrap;
    flex: 1;
  }
  .person-text {
    font-size: 12px;
    color: var(--muted-2);
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
  }
  .card-due-row {
    flex-shrink: 0;
  }
  .due-tag {
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.06em;
    color: #8c6430;
    background: #f5e8cc;
    border: 1px solid #e4cfab;
    border-radius: 10px;
    padding: 4px 8px;
    white-space: nowrap;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .due-tag--overdue {
    color: var(--over);
    background: #f8e1db;
    border-color: #e3b7b0;
  }
  .due-tag--today {
    color: #8c6430;
    background: #f5e8cc;
    border-color: #e4cfab;
  }
</style>
