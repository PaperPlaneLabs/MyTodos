<script lang="ts">
  import Modal from "$lib/components/common/Modal.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { whatsNewStore } from "$lib/stores/whats-new.svelte";

  function close(): void {
    void whatsNewStore.close();
  }
</script>

<Modal
  open={whatsNewStore.open && !uiStore.isCollapsed}
  title={whatsNewStore.mode === "unseen" ? "✨ What's New" : "What's New"}
  onClose={close}
>
  {#snippet children()}
    <div class="release-list">
      {#if whatsNewStore.visibleNotes.length === 0}
        <p class="empty">No release notes are available yet.</p>
      {:else}
        {#each whatsNewStore.visibleNotes as note}
          <article class="release-note">
            <div class="release-heading">
              <div>
                <span class="version">v{note.version}</span>
                <h4>{note.title}</h4>
              </div>
              <time datetime={note.date}>{note.date}</time>
            </div>

            <p class="summary">{note.summary}</p>

            {#if note.highlights.length > 0}
              <h5>Highlights</h5>
              <ul>
                {#each note.highlights as highlight}
                  <li>{highlight}</li>
                {/each}
              </ul>
            {/if}

            {#if note.fixes.length > 0}
              <h5>Fixes</h5>
              <ul>
                {#each note.fixes as fix}
                  <li>{fix}</li>
                {/each}
              </ul>
            {/if}
          </article>
        {/each}
      {/if}

      <div class="actions">
        <button type="button" class="btn btn-primary" onclick={close}>
          {whatsNewStore.mode === "unseen" ? "Got it" : "Close"}
        </button>
      </div>
    </div>
  {/snippet}
</Modal>

<style>
  .release-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .release-note + .release-note {
    border-top: 1px solid var(--border);
    padding-top: var(--spacing-lg);
  }

  .release-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-sm);
  }

  .release-heading h4 {
    margin: 3px 0 0;
    color: var(--text-primary);
    font-size: 15px;
    line-height: 1.3;
  }

  .version {
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
  }

  time {
    color: var(--text-tertiary);
    font-size: 10px;
    white-space: nowrap;
  }

  .summary,
  .empty {
    margin: var(--spacing-sm) 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.5;
  }

  h5 {
    margin: var(--spacing-md) 0 var(--spacing-xs);
    color: var(--text-primary);
    font-size: 12px;
  }

  ul {
    margin: 0;
    padding-left: 18px;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.5;
  }

  li + li {
    margin-top: 4px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    position: sticky;
    bottom: 0;
    padding-top: var(--spacing-sm);
    background: var(--bg-primary);
  }
</style>
