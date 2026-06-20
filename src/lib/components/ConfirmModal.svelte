<script>
  let {
    open = false,
    title = "Confirm action",
    message = "",
    confirmLabel = "Delete",
    cancelLabel = "Cancel",
    onConfirm = () => {},
    onCancel = () => {},
  } = $props();
</script>

{#if open}
  <div
    class="overlay"
    role="presentation"
    onkeydown={(event) => event.key === "Escape" && onCancel()}
  >
    <button class="scrim" type="button" aria-label="Close confirmation" onclick={onCancel}></button>
    <div class="modal" role="dialog" aria-modal="true" aria-label={title} tabindex="-1">
      <div class="eyebrow">Please confirm</div>
      <div class="title">{title}</div>
      {#if message}
        <p class="message">{message}</p>
      {/if}
      <div class="actions">
        <button class="text-btn" onclick={onCancel}>{cancelLabel}</button>
        <button class="danger-btn" onclick={onConfirm}>{confirmLabel}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    z-index: 120;
  }
  .scrim {
    position: absolute;
    inset: 0;
    border: none;
    background: rgba(34, 28, 20, 0.42);
    backdrop-filter: blur(5px);
    cursor: default;
  }
  .modal {
    position: relative;
    width: min(420px, 100%);
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 18px;
    box-shadow: 0 24px 60px rgba(38, 29, 12, 0.18);
    padding: 22px 22px 18px;
  }
  .eyebrow {
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--faint);
    margin-bottom: 8px;
  }
  .title {
    font-family: var(--serif);
    font-size: 25px;
    line-height: 1.1;
    color: var(--ink);
  }
  .message {
    margin: 10px 0 0;
    font-size: 14px;
    line-height: 1.6;
    color: var(--muted);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }
  .text-btn,
  .danger-btn {
    border: none;
    border-radius: 10px;
    padding: 10px 14px;
    font-size: 13px;
    cursor: pointer;
  }
  .text-btn {
    background: transparent;
    color: var(--muted);
  }
  .text-btn:hover {
    color: var(--ink);
    background: #ece4d7;
  }
  .danger-btn {
    background: #a94235;
    color: white;
  }
  .danger-btn:hover {
    background: #94372b;
  }
</style>
