<script>
  import { people, tasks, screen, selectedSlug, priorityColor, initials, relative } from "../stores.js";

  const todayStr = new Date().toLocaleDateString("en-US", {
    weekday: "long", month: "long", day: "numeric",
  });

  const dueToday = $derived($tasks.filter((t) => t.due === "Today" && t.column !== "done"));
  const overdue = $derived($people.filter((p) => p.status === "over"));

  function open(slug) {
    selectedSlug.set(slug);
    screen.set("person");
  }
</script>

<header>
  <h1>Today</h1>
  <p>{todayStr} · {dueToday.length} tasks due, {overdue.length} need a check-in</p>
</header>

<div class="scroll body">
  <div class="mono-label" style="margin-bottom:14px">TASKS DUE TODAY</div>
  <div class="panel">
    {#each dueToday as t}
      <div class="trow">
        <span class="check"></span>
        <span class="prio" style="background:{priorityColor(t.priority)}"></span>
        <span class="ttitle">{t.title}</span>
        <span class="meta">{t.person || "—"}</span>
      </div>
    {:else}
      <div class="muted">Nothing due today. Nice.</div>
    {/each}
  </div>

  <div class="mono-label" style="color:var(--over); margin-bottom:14px">NEEDS ATTENTION</div>
  <div class="panel">
    {#each overdue as p (p.slug)}
      <button class="prow" on:click={() => open(p.slug)}>
        <span class="avatar" style="background:{p.color}">{initials(p.name)}</span>
        <span class="who">
          <span class="pname">{p.name}</span>
          <span class="psub">Last 1:1 — {relative(p.last_met)}</span>
        </span>
        <span class="schedule">Schedule</span>
      </button>
    {:else}
      <div class="muted">Everyone is on cadence.</div>
    {/each}
  </div>
</div>

<style>
  header {
    padding: 22px 32px 18px;
    border-bottom: 1px solid var(--line);
  }
  h1 {
    font-family: var(--serif);
    font-size: 28px;
    color: var(--ink);
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
    max-width: 760px;
  }
  .panel {
    background: var(--card);
    border: 1px solid #ece6da;
    border-radius: 12px;
    padding: 8px 16px;
    margin-bottom: 32px;
  }
  .trow {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 11px 0;
    border-bottom: 1px solid #f0ebe1;
  }
  .trow:last-child {
    border-bottom: none;
  }
  .check {
    width: 16px;
    height: 16px;
    flex: none;
    border: 1.5px solid #b4ab98;
    border-radius: 4px;
  }
  .prio {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex: none;
  }
  .ttitle {
    flex: 1;
    font-size: 14px;
    color: var(--ink-2);
  }
  .meta {
    font-size: 11px;
    color: var(--muted-2);
  }
  .prow {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 11px 0;
    border-bottom: 1px solid #f0ebe1;
    border-top: none;
    border-left: none;
    border-right: none;
    background: transparent;
    width: 100%;
    text-align: left;
  }
  .prow:last-child {
    border-bottom: none;
  }
  .avatar {
    width: 32px;
    height: 32px;
    flex: none;
    border-radius: 50%;
    color: #fff;
    font-size: 12px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .who {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .pname {
    font-size: 14px;
    font-weight: 500;
    color: var(--ink);
  }
  .psub {
    font-size: 12px;
    color: var(--muted-2);
  }
  .schedule {
    font-size: 11px;
    color: var(--over);
    background: var(--over-tint);
    padding: 4px 10px;
    border-radius: 20px;
  }
  .muted {
    font-size: 13px;
    color: var(--muted-2);
    padding: 10px 0;
  }
</style>
