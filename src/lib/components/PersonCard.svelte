<script>
  import { folders, initials, relative, colorForPerson } from "../stores.js";

  let { person, onOpen } = $props();
</script>

<button class="card" onclick={() => onOpen(person.slug)}>
  <div class="head">
    <span class="avatar" style="background:{colorForPerson(person, $folders)}">{initials(person.name)}</span>
    <span class="who">
      <span class="name">{person.name}</span>
      <span class="role">{person.role}</span>
    </span>
  </div>
  <div class="foot">
    <span class="last">Last 1:1 — {relative(person.last_met)}</span>
  </div>
</button>

<style>
  .card {
    background: var(--card);
    border: 1px solid #ece6da;
    border-radius: var(--radius);
    padding: 16px;
    text-align: left;
    width: 100%;
    transition: box-shadow 0.12s ease, transform 0.12s ease;
  }
  .card:hover {
    box-shadow: 0 4px 14px rgba(60, 50, 30, 0.07);
    transform: translateY(-1px);
  }
  .head {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .avatar {
    width: 40px;
    height: 40px;
    flex: none;
    border-radius: 50%;
    color: #fff;
    font-size: 14px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .who {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .name {
    font-size: 15px;
    font-weight: 600;
    color: var(--ink);
  }
  .role {
    font-size: 12px;
    color: var(--muted-2);
  }
  .foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 14px;
  }
  .last {
    font-size: 12px;
    color: var(--muted-2);
  }
</style>
