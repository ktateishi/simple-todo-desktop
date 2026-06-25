<script lang="ts">
  import { tags, filterTagId } from '$lib/stores';
</script>

<div class="tag-filter">
  <span class="label">タグ</span>
  <button
    class="chip"
    class:active={$filterTagId === null}
    onclick={() => filterTagId.set(null)}
  >すべて</button>
  {#each $tags as tag (tag.id)}
    <button
      class="chip"
      class:active={$filterTagId === tag.id}
      onclick={() => filterTagId.set($filterTagId === tag.id ? null : tag.id)}
      style={tag.color ? `--chip-color:${tag.color}` : ''}
    >#{tag.name}</button>
  {/each}
</div>

<style>
.tag-filter {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
}
.label {
  font-size: 0.78rem;
  color: var(--text-muted);
  margin-right: 4px;
}
.chip {
  padding: 3px 10px;
  border-radius: 12px;
  border: 1px solid var(--chip-color, var(--border));
  background: transparent;
  color: var(--chip-color, var(--text-muted));
  font-size: 0.8rem;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.chip:hover { background: var(--chip-color, var(--accent)); color: #fff; opacity: 0.85; }
.chip.active { background: var(--chip-color, var(--accent)); color: #fff; border-color: var(--chip-color, var(--accent)); }
</style>
