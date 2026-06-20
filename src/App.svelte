<script>
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { screen, people, tasks, folders, vaultPath, selectedSlug, fireAppAction } from "./lib/stores.js";
  import { listPeople, listTasks, getVaultPath, listFolders, listenToMenuActions } from "./lib/api.js";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import PeopleView from "./lib/components/PeopleView.svelte";
  import PersonDetail from "./lib/components/PersonDetail.svelte";
  import TasksView from "./lib/components/TasksView.svelte";
  import ConversationsView from "./lib/components/ConversationsView.svelte";
  import SettingsView from "./lib/components/SettingsView.svelte";

  async function refresh() {
    people.set(await listPeople());
    tasks.set(await listTasks());
    vaultPath.set(await getVaultPath());
    folders.set(await listFolders());
  }

  onMount(() => {
    refresh();

    let disposed = false;
    let unlisten = () => {};

    listenToMenuActions(async (payload) => {
      const action = typeof payload === "string" ? payload : payload?.action;
      const detail = typeof payload === "string" ? null : payload?.detail;

      if (action === "show-people") {
        screen.set("people");
        return;
      }

      if (action === "show-tasks") {
        screen.set("tasks");
        return;
      }

      if (action === "new-person") {
        screen.set("people");
        setTimeout(() => fireAppAction("new-person"), 0);
        return;
      }

      if (action === "new-task") {
        screen.set("tasks");
        setTimeout(() => fireAppAction("new-task"), 0);
        return;
      }

      if (action === "new-1on1") {
        if (detail?.slug) {
          selectedSlug.set(detail.slug);
          screen.set("person");
          setTimeout(() => fireAppAction("new-1on1", detail), 0);
        } else if (get(selectedSlug)) {
          screen.set("person");
          setTimeout(() => fireAppAction("new-1on1"), 0);
        } else {
          screen.set("people");
        }
      }
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });

    return () => {
      disposed = true;
      unlisten();
    };
  });
</script>

<div class="shell">
  <Sidebar />
  <main>
    {#if $screen === "people"}
      <PeopleView />
    {:else if $screen === "person"}
      <PersonDetail />
    {:else if $screen === "tasks"}
      <TasksView />
    {:else if $screen === "conversations"}
      <ConversationsView />
    {:else if $screen === "settings"}
      <SettingsView />
    {/if}
  </main>
</div>

<style>
  .shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
    background: var(--paper);
  }
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }
</style>
