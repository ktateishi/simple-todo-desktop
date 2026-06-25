<script lang="ts">
  import { dndzone } from 'svelte-dnd-action';
  import type { Group, TaskWithTags, Status } from '$lib/types';
  import TaskItem from './TaskItem.svelte';
  import ChevronDown  from 'lucide-svelte/icons/chevron-down';
  import ChevronRight from 'lucide-svelte/icons/chevron-right';

  type ReorderPayload = { movedId: number; beforeId?: number; afterId?: number };

  type Props = {
    group: Group | null;
    items: TaskWithTags[];
    onReorder:      (p: ReorderPayload) => void;
    onStatusChange: (id: number, status: Status) => void;
    onDelete:       (id: number) => void;
    onEdit:         (task: TaskWithTags) => void;
  };

  let { group, items, onReorder, onStatusChange, onDelete, onEdit }: Props = $props();

  let collapsed   = $state(false);
  let localItems  = $state<TaskWithTags[]>([]);

  $effect(() => { localItems = [...items]; });

  const flipDurationMs = 150;

  function handleConsider(e: CustomEvent<{ items: TaskWithTags[] }>) {
    localItems = e.detail.items;
  }

  function handleFinalize(e: CustomEvent<{ items: TaskWithTags[]; info: { id: number } }>) {
    localItems = e.detail.items;
    const movedId = e.detail.info.id;
    const idx = localItems.findIndex(t => t.id === movedId);
    if (idx === -1) return;
    const beforeId = idx > 0                     ? localItems[idx - 1].id : undefined;
    const afterId  = idx < localItems.length - 1 ? localItems[idx + 1].id : undefined;
    onReorder({ movedId, beforeId, afterId });
  }

  const groupColor = $derived(group?.color ?? 'var(--accent)');
  const label      = $derived(group?.name ?? 'グループなし');
</script>

<section class="group-section">
  <header
    class="group-header"
    role="button"
    tabindex="0"
    onclick={() => (collapsed = !collapsed)}
    onkeydown={(e) => e.key === 'Enter' && (collapsed = !collapsed)}
  >
    <span class="indicator" style="background:{groupColor}"></span>
    <span class="group-name" title={label}>{label}</span>
    <span class="count">{items.length}</span>
    <span class="chevron">
      {#if collapsed}
        <ChevronRight size={12} strokeWidth={2.5} />
      {:else}
        <ChevronDown size={12} strokeWidth={2.5} />
      {/if}
    </span>
  </header>

  {#if !collapsed}
    <div
      use:dndzone={{ items: localItems, flipDurationMs, type: 'tasks', dropFromOthersDisabled: true }}
      onconsider={handleConsider}
      onfinalize={handleFinalize}
      class="task-zone"
    >
      {#each localItems as task (task.id)}
        <TaskItem
          {task}
          {onStatusChange}
          {onDelete}
          {onEdit}
        />
      {/each}
      {#if localItems.length === 0}
        <p class="empty">タスクがありません</p>
      {/if}
    </div>
  {/if}
</section>

<style>
.group-section { margin-bottom: 0; }

.group-header {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 6px 12px;
  cursor: pointer;
  user-select: none;
  background: var(--bg);
  border-bottom: 1px solid var(--border-light);
  border-top: 1px solid var(--border-light);
  margin-top: 8px;
}
.group-section:first-child .group-header { margin-top: 0; }
.group-header:hover { background: var(--hover); }

.indicator {
  width: 6px; height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.group-name {
  font-weight: 500;
  font-size: 0.76rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.07em;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.count {
  font-size: 0.72rem;
  color: var(--text-muted);
  background: var(--border-light);
  border-radius: 10px;
  padding: 1px 7px;
}
.chevron {
  display: flex;
  align-items: center;
  color: var(--border);
  transition: color 0.1s;
}
.group-header:hover .chevron { color: var(--text-muted); }

.task-zone { outline: none; }

.empty {
  text-align: center;
  padding: 24px;
  color: var(--text-muted);
  font-size: 0.82rem;
  background: var(--surface);
}
</style>
