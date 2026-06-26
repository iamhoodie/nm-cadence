<script>
  import { tick } from "svelte";
  import { Trash2, ChevronLeft } from "lucide-svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import {
    appAction,
    clearAppAction,
    people,
    tasks,
    folders,
    screen,
    selectedSlug,
    targetConversation,
    initials,
    formatDate,
    relative,
    colorForPerson,
    sidebarCollapsed,
    toggleSidebar,
  } from "../stores.js";
  import {
    addConversation,
    updateConversation,
    updatePerson,
    deletePerson,
    deleteConversation,
  } from "../api.js";

  const person = $derived($people.find((item) => item.slug === $selectedSlug));
  const linkedTasks = $derived(
    person
      ? $tasks.filter((task) => {
          const tp = Array.isArray(task.people)
            ? task.people
            : task.person && task.person !== "—"
            ? [task.person]
            : [];
          return tp.includes(person.name) && !task.archived;
        })
      : []
  );
  const folderOptions = $derived(() =>
    [...new Set([...$folders.map((folder) => folder.name), ...$people.map((item) => item.group).filter(Boolean)])].sort((a, b) =>
      a.localeCompare(b)
    )
  );

  let editingPerson = $state(false);
  let conversationModalOpen = $state(false);
  let conversationMode = $state("create");
  let conversationOriginal = $state(null);

  let noteTitle = $state("");
  let noteBody = $state("");
  let noteActions = $state([]);
  let noteActionDraft = $state("");

  let formName = $state("");
  let formRole = $state("");
  let formBio = $state("");
  let formGroup = $state("");

  let editorElement = $state();
  let confirmState = $state(null);
  let handledActionToken = $state(0);
  let highlightedConversationKey = $state("");
  let highlightTimeout = $state(null);
  let conversationAnchorVersion = $state(0);
  let notesElement = $state();
  const conversationNodes = new Map();

  function currentDateTimeValue() {
    const now = new Date();
    const local = new Date(now.getTime() - now.getTimezoneOffset() * 60000);
    return local.toISOString().slice(0, 16);
  }

  function resetConversationEditor() {
    noteTitle = "";
    noteBody = "";
    noteActions = [];
    noteActionDraft = "";
    conversationOriginal = null;
  }

  async function syncEditorFromState() {
    await tick();
    if (editorElement) editorElement.innerHTML = noteBody || "<p></p>";
  }

  function syncStateFromEditor() {
    noteBody = editorElement?.innerHTML || "";
  }

  async function openConversationModal(mode, conv = null) {
    conversationMode = mode;
    conversationModalOpen = true;
    if (conv) {
      conversationOriginal = conv;
      noteTitle = conv.title;
      noteBody = conv.body || "";
      noteActions = (conv.actions || []).map((action) => action.text);
      noteActionDraft = "";
    } else {
      resetConversationEditor();
    }
    await syncEditorFromState();
  }

  function closeConversationModal() {
    conversationModalOpen = false;
    resetConversationEditor();
  }

  function runEditor(command, value = null) {
    editorElement?.focus();
    document.execCommand(command, false, value);
    syncStateFromEditor();
  }

  async function saveConversation() {
    if (!person || !noteTitle.trim()) return;
    syncStateFromEditor();
    const conv = {
      date: conversationOriginal?.date || currentDateTimeValue().replace("T", " "),
      title: noteTitle.trim(),
      body: noteBody,
      actions: noteActions
        .map((text) => text.trim())
        .filter(Boolean)
        .map((text) => {
          const previous = (conversationOriginal?.actions || []).find((action) => action.text === text);
          return { text, done: previous?.done || false };
        }),
    };

    const updated =
      conversationMode === "edit" && conversationOriginal
        ? await updateConversation(person.slug, conversationOriginal.date, conversationOriginal.title, conv)
        : await addConversation(person.slug, conv);

    people.update((list) => list.map((item) => (item.slug === updated.slug ? updated : item)));
    closeConversationModal();
  }

  async function toggleAction(conv, action) {
    if (!person) return;
    const updatedConv = {
      ...conv,
      actions: conv.actions.map((item) =>
        item.text === action.text ? { ...item, done: !item.done } : item
      ),
    };
    const updated = await updateConversation(person.slug, conv.date, conv.title, updatedConv);
    people.update((list) => list.map((item) => (item.slug === updated.slug ? updated : item)));
  }

  async function removeConversation(conv) {
    if (!person) return;
    await deleteConversation(person.slug, conv.date, conv.title);
    const remaining = person.conversations.filter(
      (entry) => !(entry.date === conv.date && entry.title === conv.title)
    );
    people.update((list) =>
      list.map((item) =>
        item.slug === person.slug ? { ...item, conversations: remaining, last_met: remaining[0]?.date || null } : item
      )
    );
    if (conversationModalOpen) closeConversationModal();
  }

  function addActionItem() {
    const text = noteActionDraft.trim();
    if (!text || noteActions.includes(text)) {
      noteActionDraft = "";
      return;
    }
    noteActions = [...noteActions, text];
    noteActionDraft = "";
  }

  function removeActionItem(text) {
    noteActions = noteActions.filter((item) => item !== text);
  }

  function beginEditPerson() {
    if (!person) return;
    editingPerson = true;
    formName = person.name;
    formRole = person.role || "";
    formBio = person.bio || "";
    formGroup = person.group || "";
  }

  async function savePersonEdits() {
    if (!person || !formName.trim()) return;
    const updated = await updatePerson(person.slug, {
      name: formName.trim(),
      role: formRole.trim(),
      bio: formBio.trim(),
      color: person.color,
      group: formGroup.trim(),
    });
    people.update((list) => list.map((item) => (item.slug === updated.slug ? updated : item)));
    editingPerson = false;
  }

  async function removePerson() {
    if (!person) return;
    await deletePerson(person.slug);
    people.update((list) => list.filter((item) => item.slug !== person.slug));
    selectedSlug.set(null);
    screen.set("people");
  }

  function requestRemoveConversation(conv) {
    confirmState = {
      title: "Delete 1:1 note?",
      message: `Delete "${conv.title}" from ${formatDate(conv.date)}? This removes its notes and action items.`,
      confirmLabel: "Delete note",
      action: async () => {
        await removeConversation(conv);
      },
    };
  }

  function requestRemovePerson() {
    if (!person) return;
    confirmState = {
      title: `Delete ${person.name}?`,
      message: "This removes the person, their notes, and linked history permanently.",
      confirmLabel: "Delete person",
      action: async () => {
        await removePerson();
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

  function conversationKey(conv) {
    return `${conv.date}::${conv.title}`;
  }

  function setConversationNode(key, node) {
    if (node) conversationNodes.set(key, node);
    else conversationNodes.delete(key);
    conversationAnchorVersion += 1;
  }

  function conversationAnchor(node, key) {
    setConversationNode(key, node);
    return {
      update(nextKey) {
        if (nextKey !== key) {
          setConversationNode(key, null);
          key = nextKey;
          setConversationNode(key, node);
        }
      },
      destroy() {
        setConversationNode(key, null);
      },
    };
  }

  function queueConversationHighlight(key) {
    highlightedConversationKey = key;
    if (highlightTimeout) window.clearTimeout(highlightTimeout);
    highlightTimeout = window.setTimeout(() => {
      highlightedConversationKey = "";
      highlightTimeout = null;
    }, 2200);
  }

  function offsetWithinContainer(node, container) {
    const nodeRect = node.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();
    return nodeRect.top - containerRect.top + container.scrollTop;
  }

  function scrollToTargetConversation(target, attempt = 0) {
    const key = `${target.date}::${target.title}`;
    scrollToConversationKey(key, attempt, true);
  }

  function scrollToConversationKey(key, attempt = 0, clearTargetWhenDone = false) {
    const node = conversationNodes.get(key);
    if (!node || !notesElement) {
      if (attempt < 8) {
        window.setTimeout(() => scrollToConversationKey(key, attempt + 1, clearTargetWhenDone), 40);
      }
      return;
    }

    const offsetTop = offsetWithinContainer(node, notesElement);
    const targetTop = Math.max(0, offsetTop - 28);

    notesElement.scrollTo({ top: targetTop, behavior: "smooth" });
    queueConversationHighlight(key);
    if (clearTargetWhenDone) targetConversation.set(null);
  }

  $effect(() => {
    const action = $appAction;
    if (!action?.token || action.token === handledActionToken) return;
    handledActionToken = action.token;
    if (action.type === "new-1on1") {
      openConversationModal("create");
    } else if (action.type === "edit-person") {
      beginEditPerson();
    } else if (action.type === "delete-person") {
      requestRemovePerson();
    } else {
      return;
    }
    clearAppAction();
  });

  $effect(() => {
    const target = $targetConversation;
    conversationAnchorVersion;
    if (!target || !person || target.slug !== person.slug) return;

    tick().then(() => scrollToTargetConversation(target));
  });
</script>

{#if person}
  <div class="topbar">
    <div class="topbar-left">
      <button class="sidebar-toggle-btn" onclick={toggleSidebar} title={$sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"} aria-label={$sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}>
        <span class="sidebar-toggle-mark" class:sidebar-toggle-mark--collapsed={$sidebarCollapsed}>
          <ChevronLeft size={16} strokeWidth={1.8} />
        </span>
      </button>
      <button class="crumb" onclick={() => screen.set("people")}>People / {person.name}</button>
    </div>
    <button class="ghost-btn" onclick={() => openConversationModal("create")}>+ New 1:1</button>
  </div>

  <div class="hero">
    <div class="row">
      <span class="avatar" style="background:{colorForPerson(person, $folders)}">{initials(person.name)}</span>
      <div>
        <div class="name">{person.name}</div>
        <div class="role">{person.role}</div>
      </div>
    </div>
    <div class="tags">
      <span class="pill soft">Last 1:1 {relative(person.last_met)}</span>
      <span class="pill soft">{person.group || "Unfiled"}</span>
    </div>
  </div>

  <div class="split">
    <section class="scroll notes" bind:this={notesElement}>
      <div class="section-row">
        <div class="mono-label section-label">CONVERSATIONS</div>
      </div>

      {#if person.conversations.length}
        {#each person.conversations as conv, index (conv.date + conv.title)}
          <div
            class="entry"
            class:entry--highlighted={highlightedConversationKey === conversationKey(conv)}
            use:conversationAnchor={conversationKey(conv)}
          >
            <div class="rail">
              <span class="rdot"></span>
              {#if index < person.conversations.length - 1}<span class="rline"></span>{/if}
            </div>

            <div class="content">
              <div class="date-row">
                <div class="date">
                  <span class="d">{formatDate(conv.date)}</span>
                  <span class="ago"> · {relative(conv.date)}</span>
                </div>
                <div class="conv-actions">
                  <button class="conv-btn" onclick={() => openConversationModal("edit", conv)}>Edit</button>
                  <button class="conv-btn del" onclick={() => requestRemoveConversation(conv)}>Delete</button>
                </div>
              </div>
              <div class="title">{conv.title}</div>
              {#if conv.body}
                <div class="prose html-body">{@html conv.body}</div>
              {/if}
              {#if conv.actions.length}
                <div class="action-list-head">Action items</div>
                {#each conv.actions as action}
                  <button class="action" onclick={() => toggleAction(conv, action)}>
                    <span class="check" class:done={action.done}>{action.done ? "✓" : ""}</span>
                    <span class:struck={action.done}>{action.text}</span>
                  </button>
                {/each}
              {/if}
            </div>
          </div>
        {/each}
      {:else}
        <div class="empty">
          <div class="empty-title">No conversations logged yet</div>
          <div class="empty-sub">Capture notes, images, and action items from your 1:1s in one place.</div>
          <button class="solid-btn empty-cta" onclick={() => openConversationModal("create")}>+ Log first 1:1</button>
        </div>
      {/if}
    </section>

    <aside class="meta">
      <div class="meta-head">
        <div class="mono-label">PROFILE</div>
        <button class="ghost-btn-sm" onclick={beginEditPerson}>Edit</button>
      </div>
      <div class="meta-block">
        <div class="mono-label">BIO</div>
        <div class="bio">{person.bio || "No bio yet."}</div>
      </div>
      <div class="meta-block">
        <div class="mono-label">LINKED TASKS</div>
        {#each linkedTasks as task}
          <div class="ltask">
            <span class="ldot"></span>
            <div>
              <div class="lt-title">{task.title}</div>
              <div class="lt-due">{task.due || "No due date"} · {task.priority}</div>
            </div>
          </div>
        {:else}
          <div class="bio">No linked tasks.</div>
        {/each}
      </div>
    </aside>
  </div>
{/if}

{#if editingPerson && person}
  <div class="overlay">
    <div class="modal profile-modal">
      <div class="modal-top">
        <div class="modal-head">Edit person</div>
      </div>

      <div class="profile-modal-body">
        <label class="field"><span>NAME</span><input bind:value={formName} /></label>
        <label class="field"><span>ROLE</span><input bind:value={formRole} /></label>
        <label class="field"><span>BIO</span><textarea bind:value={formBio} rows="6"></textarea></label>
        <label class="field">
          <span>FOLDER</span>
          <select bind:value={formGroup}>
            <option value="">Unfiled</option>
            {#each folderOptions() as option}
              <option value={option}>{option}</option>
            {/each}
          </select>
        </label>
      </div>

      <div class="modal-foot">
        <button class="icon-btn icon-btn--danger" onclick={requestRemovePerson} title="Delete person" aria-label="Delete person">
          <Trash2 size={16} strokeWidth={1.8} aria-hidden="true" />
        </button>
        <div class="foot-right">
          <button class="text-btn" onclick={() => (editingPerson = false)}>Cancel</button>
          <button class="solid-btn" onclick={savePersonEdits} disabled={!formName.trim()}>Save</button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if conversationModalOpen}
  <div class="overlay">
    <div class="modal note-modal">
      <div class="modal-top">
        <div class="modal-head">{conversationMode === "edit" ? "Edit 1:1 note" : "New 1:1 note"}</div>
        <label class="field">
          <span>TITLE</span>
          <input bind:value={noteTitle} placeholder="Conversation title" />
        </label>
      </div>

      <div class="modal-body">
        <div class="modal-layout">
          <div class="field note-pane">
            <span>NOTES</span>
            <div class="toolbar">
              <button class="tool-btn" title="Bold" aria-label="Bold" onclick={() => runEditor("bold")}><strong>B</strong></button>
              <button class="tool-btn" title="Italic" aria-label="Italic" onclick={() => runEditor("italic")}><em>I</em></button>
              <button class="tool-btn" title="Underline" aria-label="Underline" onclick={() => runEditor("underline")}><u>U</u></button>
              <div class="tool-sep"></div>
              <button class="tool-btn" title="Normal text" aria-label="Normal text" onclick={() => runEditor("formatBlock", "p")}>¶</button>
              <button class="tool-btn" title="Heading 1" aria-label="Heading 1" onclick={() => runEditor("formatBlock", "h1")}>H1</button>
              <button class="tool-btn" title="Heading 2" aria-label="Heading 2" onclick={() => runEditor("formatBlock", "h2")}>H2</button>
              <button class="tool-btn" title="Heading 3" aria-label="Heading 3" onclick={() => runEditor("formatBlock", "h3")}>H3</button>
              <div class="tool-sep"></div>
              <button class="tool-btn" title="Bulleted list" aria-label="Bulleted list" onclick={() => runEditor("insertUnorderedList")}>• List</button>
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
                  oninput={(e) => { editorElement?.focus(); document.execCommand("foreColor", false, e.currentTarget.value); syncStateFromEditor(); }}
                />
              </div>
            </div>
            <div
              class="rich-editor"
              contenteditable="true"
              bind:this={editorElement}
              oninput={syncStateFromEditor}
            ></div>
          </div>

          <aside class="actions-pane">
            <div class="actions-pane-head">
              <span class="mono-label">ACTION ITEMS</span>
              <button class="ghost-btn-sm action-pane-add-btn" onclick={addActionItem}>Add</button>
            </div>
            <div class="action-entry">
              <input
                bind:value={noteActionDraft}
                placeholder="Add a follow-up item"
                onkeydown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    addActionItem();
                  }
                }}
              />
            </div>
            <div class="action-draft-list">
              {#each noteActions as action}
                <div class="draft-action">
                  <span>{action}</span>
                  <button class="chip-remove" onclick={() => removeActionItem(action)} title="Remove action item">×</button>
                </div>
              {:else}
                <div class="actions-empty">No action items yet.</div>
              {/each}
            </div>
          </aside>
        </div>
      </div>

      <div class="modal-foot">
        {#if conversationMode === "edit" && conversationOriginal}
          <button class="del-ghost" onclick={() => requestRemoveConversation(conversationOriginal)}>Delete</button>
        {/if}
        <div class="foot-right">
          <button class="text-btn" onclick={closeConversationModal}>Cancel</button>
          <button class="solid-btn" onclick={saveConversation} disabled={!noteTitle.trim()}>Save note</button>
        </div>
      </div>
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
  .topbar {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 22px 32px 18px;
    border-bottom: 1px solid var(--line);
  }
  .topbar-left {
    display: flex;
    align-items: center;
    gap: 12px;
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
  .crumb {
    border: none;
    background: transparent;
    color: var(--faint);
    font-family: var(--mono);
    font-size: 12px;
    cursor: pointer;
  }
  .crumb:hover { color: var(--ink); }
  .hero {
    padding: 24px 32px 18px;
    border-bottom: 1px solid var(--line);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .avatar {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    color: white;
    display: grid;
    place-items: center;
    font-size: 19px;
    font-weight: 600;
  }
  .name {
    font-family: var(--serif);
    font-size: 28px;
  }
  .role {
    color: var(--muted);
    margin-top: 4px;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 14px;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .pill.soft {
    background: #efe9dd;
    color: #6b6557;
  }
  .split {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
  .notes {
    flex: 1;
    padding: 24px 32px 40px;
  }
  .section-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 18px;
  }
  .entry {
    display: flex;
    gap: 16px;
    margin-bottom: 24px;
    scroll-margin-top: 84px;
    transition: background 0.22s ease, box-shadow 0.22s ease;
    border-radius: 14px;
  }
  .entry--highlighted {
    background: rgba(187, 160, 121, 0.12);
    box-shadow: 0 0 0 1px rgba(180, 141, 78, 0.22);
    padding: 12px;
    margin-inline: -12px;
  }
  .rail {
    width: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .rdot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--accent);
    margin-top: 6px;
  }
  .rline {
    width: 1px;
    flex: 1;
    background: #ddd6c8;
    margin-top: 6px;
  }
  .content {
    flex: 1;
    max-width: 700px;
  }
  .date-row {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 8px;
  }
  .d {
    font-family: var(--serif);
    font-size: 15px;
  }
  .ago {
    font-size: 12px;
    color: #a39c8c;
  }
  .conv-actions {
    display: flex;
    gap: 6px;
  }
  .conv-btn {
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: 12px;
  }
  .conv-btn.del {
    color: var(--over);
  }
  .title {
    font-family: var(--sans);
    font-size: 20px;
    font-weight: 700;
    margin-bottom: 8px;
  }
  .prose {
    font-size: 14px;
    font-weight: 300;
    line-height: 1.7;
    color: #56514a;
    margin-bottom: 12px;
  }
  .html-body {
    white-space: pre-wrap;
  }
  .html-body :global(img) {
    max-width: 100%;
    border-radius: 12px;
    display: block;
    margin: 12px 0;
  }
  .html-body :global(h2),
  .html-body :global(h3) {
    font-family: var(--serif);
    color: var(--ink);
    margin: 14px 0 8px;
  }
  .html-body :global(blockquote) {
    margin: 12px 0;
    padding-left: 14px;
    border-left: 3px solid var(--line-2);
    color: var(--muted);
  }
  .action {
    display: flex;
    gap: 9px;
    align-items: flex-start;
    margin-bottom: 7px;
    border: none;
    background: transparent;
    padding: 0;
    color: #4a463e;
    font-size: 13px;
  }
  .action-list-head {
    margin: 14px 0 8px;
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--faint);
  }
  .check {
    width: 15px;
    height: 15px;
    border-radius: 4px;
    border: 1.5px solid #b4ab98;
    font-size: 10px;
    line-height: 13px;
    text-align: center;
    color: transparent;
    flex: none;
  }
  .check.done {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }
  .struck {
    text-decoration: line-through;
    color: #8c877d;
  }
  .empty {
    border: 1px dashed #d8d0c0;
    border-radius: 14px;
    padding: 40px;
    text-align: center;
    width: 100%;
    max-width: none;
  }
  .empty-cta {
    width: auto;
    min-width: 220px;
    justify-content: center;
  }
  .empty-title {
    font-family: var(--serif);
    font-size: 20px;
    margin-bottom: 8px;
  }
  .empty-sub {
    color: var(--muted);
    margin-bottom: 18px;
  }
  .meta {
    width: 310px;
    flex: none;
    border-left: 1px solid var(--line);
    padding: 24px 22px;
    background: var(--panel-2);
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
  }
  .meta-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .meta-block {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .bio {
    font-size: 13px;
    line-height: 1.6;
    color: var(--ink-2);
    white-space: pre-wrap;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-family: var(--mono);
    font-size: 11px;
    letter-spacing: 0.12em;
    color: var(--faint);
  }
  .field input,
  .field select,
  .field textarea {
    width: 100%;
    border: 1px solid var(--line-2);
    border-radius: 8px;
    padding: 9px 11px;
    font-size: 14px;
    background: var(--card);
    color: var(--ink);
    font-family: inherit;
    resize: vertical;
  }
  .field input:focus,
  .field select:focus,
  .field textarea:focus {
    outline: none;
    border-color: var(--accent);
  }
  .field select {
    appearance: none;
    cursor: pointer;
    background-image:
      linear-gradient(45deg, transparent 50%, var(--muted) 50%),
      linear-gradient(135deg, var(--muted) 50%, transparent 50%);
    background-position:
      calc(100% - 18px) calc(50% - 2px),
      calc(100% - 12px) calc(50% - 2px);
    background-size: 6px 6px, 6px 6px;
    background-repeat: no-repeat;
    padding-right: 34px;
  }
  .toolbar,
  .modal-foot,
  .foot-right {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
  }
  .ltask {
    display: flex;
    gap: 8px;
    align-items: flex-start;
  }
  .ldot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--due);
    margin-top: 5px;
    flex: none;
  }
  .lt-title {
    font-size: 13px;
  }
  .lt-due {
    font-size: 11px;
    color: #a39c8c;
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(44, 42, 38, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 120;
  }
  .modal {
    background: var(--paper);
    border-radius: 16px;
    width: 480px;
    max-width: calc(100vw - 48px);
    max-height: calc(100vh - 48px);
    display: flex;
    flex-direction: column;
    box-shadow: 0 12px 40px rgba(44, 42, 38, 0.18);
    overflow: hidden;
  }
  .note-modal {
    width: min(860px, calc(100vw - 48px));
    height: min(760px, calc(100vh - 48px));
  }
  .profile-modal {
    width: min(560px, calc(100vw - 48px));
    max-height: calc(100vh - 48px);
  }
  .profile-modal-body {
    padding: 18px 28px 8px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
  }
  .modal-top {
    padding: 28px 28px 16px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 15px;
    border-bottom: 1px solid var(--line);
  }
  .modal-body {
    flex: 1;
    overflow: hidden;
    min-height: 0;
    padding: 16px 28px;
  }
  .modal-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 260px;
    gap: 20px;
    align-items: start;
    height: 100%;
    min-height: 0;
  }
  .note-pane {
    min-height: 0;
    height: 100%;
  }
  .modal-head {
    font-family: var(--serif);
    font-size: 22px;
    color: var(--ink);
  }
  .toolbar {
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
  .tool-btn:hover { background: var(--panel-2); }
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
  .tool-color-wrap:hover { background: var(--panel-2); }
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
  .solid-btn:disabled {
    cursor: not-allowed;
    opacity: 0.45;
    filter: none;
  }
  .rich-editor {
    min-height: 0;
    flex: 1;
    height: 100%;
    border: 1px solid var(--line-2);
    border-radius: 10px;
    padding: 14px;
    background: #fffdf9;
    font-family: var(--sans);
    font-weight: 300;
    font-size: 15px;
    line-height: 1.7;
    color: var(--ink);
    overflow-y: auto;
  }
  .actions-pane {
    min-height: 0;
    height: 100%;
    border: 1px solid var(--line);
    border-radius: 14px;
    background: #fbf7f0;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .actions-pane-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .actions-empty {
    font-size: 12px;
    color: var(--muted);
    line-height: 1.5;
  }
  .action-entry {
    display: flex;
  }
  .action-entry input {
    flex: 1;
    border: 1px solid var(--line-2);
    border-radius: 9px;
    padding: 10px 12px;
    font-size: 13px;
    background: white;
    color: var(--ink);
  }
  .action-entry input:focus {
    outline: none;
    border-color: var(--accent);
  }
  .action-pane-add-btn {
    flex: none;
    white-space: nowrap;
    padding-inline: 12px;
  }
  .action-draft-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    flex: 1;
    overflow-y: auto;
    padding-right: 4px;
  }
  .draft-action {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
    border-radius: 10px;
    padding: 10px 12px;
    background: white;
    border: 1px solid #e6dfd4;
    color: var(--ink-2);
    font-size: 13px;
    line-height: 1.4;
  }
  .draft-action > span:first-child {
    min-width: 0;
    flex: 1;
    white-space: normal;
    overflow-wrap: anywhere;
    word-break: break-word;
  }
  .note-pane,
  .actions-pane {
    display: flex;
    flex-direction: column;
  }
  .chip-remove {
    border: none;
    background: transparent;
    color: var(--muted-2);
    font-size: 16px;
    line-height: 1;
    padding: 0;
    cursor: pointer;
    flex: none;
  }
  .chip-remove:hover {
    color: var(--over);
  }
  .rich-editor:focus {
    outline: none;
    border-color: var(--accent);
  }
  .rich-editor :global(p),
  .rich-editor :global(div) { margin: 0; }
  .rich-editor :global(h1) { font-size: 22px; font-weight: 700; margin: 10px 0 4px; line-height: 1.3; }
  .rich-editor :global(h2) { font-size: 18px; font-weight: 600; margin: 8px 0 3px; line-height: 1.3; }
  .rich-editor :global(h3) { font-size: 15px; font-weight: 600; margin: 6px 0 2px; line-height: 1.3; }
  .rich-editor :global(img) {
    max-width: 100%;
    border-radius: 12px;
    display: block;
    margin: 12px 0;
  }
  .modal-foot {
    justify-content: space-between;
    flex-shrink: 0;
    padding: 12px 28px 20px;
    border-top: 1px solid var(--line);
  }
  .foot-right {
    margin-left: auto;
  }
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
    .split {
      flex-direction: column;
    }
    .meta {
      width: auto;
      border-left: none;
      border-top: 1px solid var(--line);
    }
    .modal-layout {
      grid-template-columns: 1fr;
    }
    .actions-pane {
      position: static;
    }
  }
  @media (max-width: 760px) {
    .topbar,
    .hero,
    .notes,
    .meta {
      padding-left: 20px;
      padding-right: 20px;
    }
    .topbar {
      flex-wrap: wrap;
      gap: 12px;
      align-items: flex-start;
    }
    .row {
      align-items: flex-start;
    }
    .name {
      font-size: 24px;
    }
    .content {
      max-width: none;
    }
    .date-row {
      flex-direction: column;
      align-items: flex-start;
    }
    .profile-modal-body {
      padding-left: 20px;
      padding-right: 20px;
    }
    .modal-foot {
      padding-left: 20px;
      padding-right: 20px;
    }
  }
</style>
