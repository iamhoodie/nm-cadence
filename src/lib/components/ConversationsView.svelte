<script>
  import {
    people,
    screen,
    selectedSlug,
    conversationsViewPage,
    conversationsViewQuery,
    targetConversation,
    formatDate,
    relative,
    initials,
    folders,
    colorForPerson,
  } from "../stores.js";

  const PAGE_SIZE_OPTIONS = [25, 50, 100, 500];
  let pageSize = $state(25);

  function openPerson(item) {
    selectedSlug.set(item.slug);
    targetConversation.set({
      slug: item.slug,
      date: item.date,
      title: item.title,
    });
    screen.set("person");
  }

  function stripHtml(html) {
    return (html || "").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
  }

  function matchesQuery(item, query) {
    if (!query) return true;
    return [
      item.personName,
      item.personRole,
      item.title,
      item.body,
      item.actions,
    ]
      .join(" ")
      .toLowerCase()
      .includes(query);
  }

  const trimmedQuery = $derived($conversationsViewQuery.trim().toLowerCase());
  const allConversations = $derived.by(() => {
    const list = [];

    for (const person of $people) {
      for (const conv of person.conversations || []) {
        list.push({
          slug: person.slug,
          personName: person.name || "",
          personRole: person.role || "",
          personColor: colorForPerson(person, $folders),
          date: conv.date || "",
          title: conv.title || "Untitled conversation",
          body: stripHtml(conv.body),
          actions: (conv.actions || []).map((action) => action?.text || "").join(" "),
        });
      }
    }

    list.sort((a, b) => {
      if (a.date === b.date) return a.title.localeCompare(b.title);
      return a.date < b.date ? 1 : -1;
    });

    return list;
  });

  const filteredConversations = $derived.by(() =>
    allConversations.filter((item) => matchesQuery(item, trimmedQuery))
  );

  const totalPages = $derived.by(() =>
    Math.max(1, Math.ceil(filteredConversations.length / pageSize))
  );
  const safePage = $derived.by(() =>
    Math.min(Math.max(1, $conversationsViewPage), totalPages)
  );
  const pagedConversations = $derived.by(() =>
    filteredConversations.slice((safePage - 1) * pageSize, safePage * pageSize)
  );

  $effect(() => {
    if ($conversationsViewPage !== safePage) {
      conversationsViewPage.set(safePage);
    }
  });

  $effect(() => {
    trimmedQuery;
    pageSize;
    conversationsViewPage.set(1);
  });
</script>

<header>
  <div>
    <h1>Conversations</h1>
    <p>Every 1:1 across your people, sorted newest first.</p>
  </div>
</header>

<div class="body">
  <section class="search-panel">
    <div class="search-head">
      <div class="mono-label">SEARCH</div>
      <div class="search-copy">Search by person, title, note content, or action items.</div>
    </div>
    <div class="conversation-search-wrap">
      <input
        class="conversation-search"
        placeholder="Search all conversations"
        bind:value={$conversationsViewQuery}
      />
      {#if trimmedQuery}
        <button
          class="conversation-search-clear"
          onclick={() => conversationsViewQuery.set("")}
          aria-label="Clear conversation search"
        >
          ×
        </button>
      {/if}
    </div>
  </section>

  <section class="results-panel">
    <div class="results-head">
      <div>
        <div class="mono-label">RESULTS</div>
        <div class="results-copy">
          {filteredConversations.length} conversation{filteredConversations.length === 1 ? "" : "s"}
        </div>
      </div>
      <div class="pagination-copy">
        Showing {(safePage - 1) * pageSize + (pagedConversations.length ? 1 : 0)}-{(safePage - 1) * pageSize + pagedConversations.length} of {filteredConversations.length}
      </div>
    </div>

    {#if pagedConversations.length}
      <div class="results-list">
        {#each pagedConversations as item (`${item.slug}-${item.date}-${item.title}`)}
          <button class="result-card" onclick={() => openPerson(item)}>
            <span class="result-avatar" style="background:{item.personColor}">
              {initials(item.personName)}
            </span>
            <span class="result-body">
              <span class="result-title">{item.title}</span>
              <span class="result-meta">
                {item.personName} · {formatDate(item.date)} · {relative(item.date)}
              </span>
              {#if item.body}
                <span class="result-preview">{item.body}</span>
              {/if}
            </span>
          </button>
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-title">No conversations found</div>
        <div class="empty-sub">
          {#if trimmedQuery}
            Try a different search term.
          {:else}
            Start logging 1:1s and they’ll appear here.
          {/if}
        </div>
      </div>
    {/if}
  </section>
</div>

<div class="pagination-bar">
  <div class="pagination-bar-inner">
    <div class="pagination-status">
      <span>Page {safePage} of {totalPages}</span>
    </div>
    <div class="pagination-controls">
      <button
        class="ghost-btn-sm"
        onclick={() => conversationsViewPage.set(1)}
        disabled={safePage <= 1}
      >
        First
      </button>
      <button
        class="ghost-btn-sm"
        onclick={() => conversationsViewPage.set(Math.max(1, safePage - 1))}
        disabled={safePage <= 1}
      >
        Prev
      </button>
      <div class="pagination-pages">
        {#each Array.from({ length: totalPages }, (_, i) => i + 1).slice(Math.max(0, safePage - 3), Math.min(totalPages, safePage + 2)) as page}
          <button
            class="page-btn"
            class:page-btn--active={page === safePage}
            onclick={() => conversationsViewPage.set(page)}
          >
            {page}
          </button>
        {/each}
      </div>
      <button
        class="ghost-btn-sm"
        onclick={() => conversationsViewPage.set(Math.min(totalPages, safePage + 1))}
        disabled={safePage >= totalPages}
      >
        Next
      </button>
      <button
        class="ghost-btn-sm"
        onclick={() => conversationsViewPage.set(totalPages)}
        disabled={safePage >= totalPages}
      >
        Last
      </button>
    </div>
    <div class="page-size">
      <span>Items</span>
      <div class="page-size-select-wrap">
        <select class="page-size-select" bind:value={pageSize}>
          {#each PAGE_SIZE_OPTIONS as option}
            <option value={option}>{option}</option>
          {/each}
        </select>
      </div>
    </div>
  </div>
</div>

<style>
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 22px 32px 18px;
    border-bottom: 1px solid var(--line);
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
    padding: 24px 32px 116px;
    display: flex;
    flex-direction: column;
    gap: 18px;
    min-height: 0;
    overflow: hidden;
  }
  .search-panel,
  .results-panel {
    border: 1px solid var(--line);
    border-radius: 16px;
    background: var(--panel);
    padding: 18px;
  }
  .search-panel {
    flex: none;
  }
  .search-head {
    margin-bottom: 10px;
  }
  .search-copy,
  .results-copy,
  .pagination-copy {
    margin-top: 6px;
    color: var(--muted);
    font-size: 13px;
  }
  .conversation-search-wrap {
    position: relative;
  }
  .conversation-search {
    width: 100%;
    border: 1px solid var(--line-2);
    border-radius: 10px;
    padding: 11px 42px 11px 13px;
    font-size: 14px;
    background: var(--card);
    color: var(--ink);
  }
  .conversation-search:focus {
    outline: none;
    border-color: var(--accent);
  }
  .conversation-search-clear {
    position: absolute;
    top: 50%;
    right: 10px;
    transform: translateY(-50%);
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 999px;
    background: transparent;
    color: var(--faint);
    display: grid;
    place-items: center;
    font-size: 18px;
    line-height: 1;
    padding: 0;
  }
  .conversation-search-clear:hover {
    background: #ebe3d5;
    color: var(--ink);
  }
  .results-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 14px;
    flex: none;
  }
  .results-panel {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .results-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding-right: 4px;
  }
  .result-card {
    width: 100%;
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 13px;
    border: 1px solid #e6dfd4;
    border-radius: 12px;
    background: var(--card);
    text-align: left;
    cursor: pointer;
  }
  .result-card:hover {
    background: #faf6ef;
    box-shadow: 0 4px 14px rgba(60, 50, 30, 0.06);
  }
  .result-avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: white;
    font-size: 11px;
    font-weight: 700;
    flex: none;
  }
  .result-body {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .result-title {
    font-size: 15px;
    color: var(--ink);
    font-weight: 500;
  }
  .result-meta {
    font-size: 11px;
    color: var(--muted-2);
  }
  .result-preview {
    font-size: 12px;
    line-height: 1.55;
    color: var(--muted);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .empty-state {
    text-align: center;
    padding: 52px 24px;
    flex: 1;
    display: grid;
    align-content: center;
  }
  .empty-title {
    font-family: var(--serif);
    font-size: 22px;
    margin-bottom: 8px;
  }
  .empty-sub {
    font-size: 14px;
    color: var(--muted);
  }
  .pagination-bar {
    position: fixed;
    left: 272px;
    right: 0;
    bottom: 0;
    z-index: 40;
    padding: 14px 24px 18px;
    background: linear-gradient(180deg, rgba(244, 238, 226, 0), rgba(244, 238, 226, 0.94) 24%, rgba(244, 238, 226, 0.98) 100%);
    backdrop-filter: blur(8px);
  }
  .pagination-bar-inner {
    border: 1px solid var(--line);
    border-radius: 16px;
    background: rgba(251, 247, 240, 0.94);
    box-shadow: 0 10px 30px rgba(44, 42, 38, 0.08);
    padding: 12px 14px;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 14px;
  }
  .pagination-status {
    font-size: 12px;
    color: var(--muted);
    justify-self: start;
  }
  .pagination-controls {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    justify-self: center;
    justify-content: center;
  }
  .pagination-pages {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
    justify-content: center;
  }
  .page-size {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--faint);
    justify-self: end;
  }
  .page-size-select-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
  }
  .page-size-select-wrap::after {
    content: "▾";
    position: absolute;
    right: 11px;
    font-size: 11px;
    color: var(--muted);
    pointer-events: none;
  }
  .page-btn,
  .ghost-btn-sm {
    border: 1px solid var(--line-2);
    background: var(--card);
    color: var(--ink);
    border-radius: 8px;
    padding: 7px 11px;
    font-size: 12px;
    cursor: pointer;
  }
  .page-size-select {
    appearance: none;
    -webkit-appearance: none;
    border: 1px solid var(--line-2);
    background: var(--card);
    color: var(--ink);
    border-radius: 8px;
    padding: 7px 28px 7px 11px;
    font-size: 12px;
    cursor: pointer;
    min-width: 72px;
  }
  .page-size-select:focus {
    outline: none;
    border-color: var(--accent);
  }
  .page-btn--active {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }
  .page-btn:disabled,
  .ghost-btn-sm:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  @media (max-width: 760px) {
    header,
    .body {
      padding-left: 20px;
      padding-right: 20px;
    }
    .body {
      padding-bottom: 164px;
      overflow: hidden;
    }
    .results-head,
    .pagination-controls {
      flex-direction: column;
      align-items: stretch;
    }
    .pagination-bar-inner {
      grid-template-columns: 1fr;
      justify-items: stretch;
    }
    .pagination-bar {
      left: 0;
      padding: 12px 16px 16px;
    }
    .pagination-status,
    .pagination-controls,
    .page-size {
      justify-self: stretch;
      justify-content: center;
    }
  }
</style>
