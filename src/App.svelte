<script>
  import { onMount } from "svelte";
  import { screen, people, tasks, folders, vaultPath } from "./lib/stores.js";
  import { listPeople, listTasks, getVaultPath, listFolders } from "./lib/api.js";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import PeopleView from "./lib/components/PeopleView.svelte";
  import PersonDetail from "./lib/components/PersonDetail.svelte";
  import TasksView from "./lib/components/TasksView.svelte";

  async function refresh() {
    people.set(await listPeople());
    tasks.set(await listTasks());
    vaultPath.set(await getVaultPath());
    folders.set(await listFolders());
  }

  onMount(refresh);
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
