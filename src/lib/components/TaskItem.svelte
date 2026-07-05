<script lang="ts">
  import { untrack } from 'svelte';
  import type { TaskWithTags, Status, StatusDef } from '$lib/types';
  import { prevStatusMap, clipboardTask, statuses } from '$lib/stores';
  import Circle      from 'lucide-svelte/icons/circle';
  import CircleCheck from 'lucide-svelte/icons/circle-check';
  import Trash2      from 'lucide-svelte/icons/trash-2';
  import Copy        from 'lucide-svelte/icons/copy';
  import RefreshCw   from 'lucide-svelte/icons/refresh-cw';

  type Props = {
    task: TaskWithTags;
    onStatusChange: (id: number, status: Status) => void;
    onDelete:       (id: number) => void;
    onEdit:         (task: TaskWithTags) => void;
    groupLabel?: string;
    groupColor?: string | null;
    extraBadge?: string;
  };

  let { task, onStatusChange, onDelete, onEdit, groupLabel, groupColor, extraBadge }: Props = $props();

  function toggleComplete() {
    if (task.status === 'done') {
      const prev = $prevStatusMap.get(task.id) ?? 'todo';
      prevStatusMap.update(m => { const n = new Map(m); n.delete(task.id); return n; });
      onStatusChange(task.id, prev);
    } else {
      prevStatusMap.update(m => new Map(m).set(task.id, task.status as Status));
      onStatusChange(task.id, 'done');
    }
  }

  // Fallback labels until the statuses store loads
  const FALLBACK_LABELS: Record<string, string> = {
    todo:    '未着手',
    doing:   '進行中',
    pending: '保留',
    done:    '完了',
  };
  const CUSTOM_FALLBACK_COLOR = '#64748B';

  const statusDef  = $derived($statuses.find(s => s.key === task.status));
  const statusName = $derived(statusDef?.name ?? FALLBACK_LABELS[task.status] ?? task.status);

  function customBadgeStyle(def: StatusDef): string {
    const c = def.color ?? CUSTOM_FALLBACK_COLOR;
    return `background:${c}22;color:${c};border-color:${c}4D`;
  }

  let pickerOpen = $state(false);

  function openPicker(e: MouseEvent) {
    e.stopPropagation();
    pickerOpen = !pickerOpen;
  }

  function pickStatus(e: MouseEvent, s: Status) {
    e.stopPropagation();
    onStatusChange(task.id, s);
    pickerOpen = false;
  }

  // Close picker when clicking outside
  $effect(() => {
    if (!pickerOpen) return;
    const close = () => { pickerOpen = false; };
    const timer = setTimeout(() => document.addEventListener('click', close, { once: true }), 0);
    return () => { clearTimeout(timer); document.removeEventListener('click', close); };
  });

  const isOverdue = $derived((): boolean => {
    if (task.status === 'done' || task.due_at === null) return false;
    const n = new Date();
    return task.due_at < new Date(n.getFullYear(), n.getMonth(), n.getDate()).getTime();
  });

  function shortDate(ts: number): string {
    return new Date(ts).toLocaleDateString('ja-JP', { month: 'numeric', day: 'numeric' });
  }

  const dueDateLabel = $derived((): string | null => {
    if (!task.due_at) return null;
    // Period task: show "開始〜期限"
    if (task.start_at) return `${shortDate(task.start_at)}〜${shortDate(task.due_at)}`;
    const now    = new Date();
    const due    = new Date(task.due_at);
    const today  = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const dueDay = new Date(due.getFullYear(), due.getMonth(), due.getDate());
    const diff   = Math.round((dueDay.getTime() - today.getTime()) / 86400000);
    if (diff ===  0) return '今日';
    if (diff ===  1) return '明日';
    if (diff === -1) return '昨日';
    if (diff > 1  && diff <=  6) return `${diff}日後`;
    if (diff < -1 && diff >= -6) return `${-diff}日前`;
    return due.toLocaleDateString('ja-JP', { month: 'short', day: 'numeric' });
  });

  const WD = ['日','月','火','水','木','金','土'];
  const recurLabel = $derived((): string => {
    if (!task.recur_type) return '';
    switch (task.recur_type) {
      case 'daily':   return '毎日';
      case 'weekly': {
        const days = (task.recur_weekdays ?? []).map(d => WD[d]).join('');
        return (task.recur_interval === 2 ? '隔週' : '毎週') + days;
      }
      case 'monthly':
        if (task.recur_month_rule === 'last_day')     return '月末';
        if (task.recur_month_rule === 'last_weekday') return '月末平日';
        return `毎月${task.recur_month_day ?? ''}日`;
      case 'yearly': return '毎年';
      default:       return '';
    }
  });
</script>

<div
  class="task-item"
  class:done={task.status === 'done'}
  data-priority={task.priority}
>
  <span class="drag-handle" aria-hidden="true">⋮⋮</span>

  <!-- Complete toggle -->
  <button
    class="complete-btn"
    class:is-done={task.status === 'done'}
    onclick={toggleComplete}
    title={task.status === 'done' ? '完了を取り消す' : '完了にする'}
    aria-label={task.status === 'done' ? '完了を取り消す' : '完了にする'}
  >
    {#if task.status === 'done'}
      <CircleCheck size={18} strokeWidth={2} />
    {:else}
      <Circle size={18} strokeWidth={1.5} />
    {/if}
  </button>

  <!-- Status badge + picker -->
  <div class="status-wrap">
    <button
      class="status-badge {statusDef?.is_custom ? '' : `s-${task.status}`}"
      style={statusDef?.is_custom ? customBadgeStyle(statusDef) : ''}
      onclick={openPicker}
      title="ステータスを変更"
      aria-label="ステータス: {statusName}"
    >
      {statusName}
    </button>

    {#if pickerOpen}
      <div class="status-picker" role="menu">
        {#each $statuses as s (s.id)}
          <button
            class="picker-opt {s.is_custom ? '' : `s-${s.key}`}"
            class:current={task.status === s.key}
            style={s.is_custom && task.status === s.key ? `color:${s.color ?? CUSTOM_FALLBACK_COLOR}` : ''}
            onclick={(e) => pickStatus(e, s.key)}
            role="menuitem"
          >
            {#if s.is_custom}
              <span class="picker-dot" style="background:{s.color ?? CUSTOM_FALLBACK_COLOR}"></span>
            {/if}
            {s.name}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Task body (title + tags) -->
  <div
    class="task-body"
    role="button"
    tabindex="0"
    onclick={() => onEdit(task)}
    onkeydown={(e) => e.key === 'Enter' && onEdit(task)}
  >
    <span class="title">{task.title}</span>
    {#if task.tags.length > 0}
      <span class="tags">
        {#each task.tags as tag (tag.id)}
          <span class="tag-chip" style={tag.color ? `background:${tag.color}22;color:${tag.color}` : ''} title="#{tag.name}">
            #{tag.name}
          </span>
        {/each}
      </span>
    {/if}
  </div>

  {#if groupLabel}
    <span
      class="group-badge"
      style="background:{groupColor ?? 'var(--accent)'}22;color:{groupColor ?? 'var(--accent)'}"
      title={groupLabel}
    >{groupLabel}</span>
  {/if}

  {#if task.recur_type}
    <span class="recur-badge" title={recurLabel()}>
      <RefreshCw size={11} strokeWidth={2.5} />
    </span>
  {/if}

  {#if task.due_at}
    <span class="due" class:overdue={isOverdue()} title={new Date(task.due_at).toLocaleDateString('ja-JP')}>
      {dueDateLabel()}
    </span>
  {/if}

  {#if extraBadge}
    <span class="extra-badge">{extraBadge}</span>
  {/if}

  <button
    class="copy-btn"
    onclick={() => clipboardTask.set(task)}
    aria-label="コピー"
    title="タスクをコピー"
  ><Copy size={13} strokeWidth={2} /></button>

  <button
    class="del-btn"
    onclick={() => onDelete(task.id)}
    aria-label="削除"
    title="削除"
  ><Trash2 size={13} strokeWidth={2} /></button>
</div>

<style>
.task-item {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 8px 12px 8px 10px;
  background: var(--surface);
  border-bottom: 1px solid var(--border-light);
  border-left: 3px solid transparent;
  transition: background 0.1s;
}
.task-item:hover { background: var(--hover); }
.task-item.done .title { text-decoration: line-through; opacity: 0.4; }

.task-item[data-priority="1"] { border-left-color: #3B82F6; }
.task-item[data-priority="2"] { border-left-color: var(--warning); }
.task-item[data-priority="3"] { border-left-color: var(--danger); }

/* ── Drag handle ── */
.drag-handle {
  color: var(--border);
  cursor: grab;
  font-size: 0.9rem;
  letter-spacing: -2px;
  user-select: none;
  flex-shrink: 0;
  transition: color 0.1s;
}
.task-item:hover .drag-handle { color: var(--text-muted); }
.drag-handle:active { cursor: grabbing; }

/* ── Complete button ── */
.complete-btn {
  width: 22px; height: 22px;
  border-radius: 50%;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
  padding: 0;
  color: var(--border);
  transition: color 0.12s, transform 0.12s;
}
.complete-btn:not(.is-done):hover { color: var(--success); transform: scale(1.1); }
.complete-btn.is-done             { color: var(--success); }
.complete-btn.is-done:hover       { opacity: 0.7; transform: scale(1.05); }

/* ── Status badge ── */
.status-wrap { position: relative; flex-shrink: 0; }

.status-badge {
  font-size: 0.68rem;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  cursor: pointer;
  white-space: nowrap;
  border: 1px solid transparent;
  transition: opacity 0.1s, transform 0.1s;
  letter-spacing: 0.01em;
}
.status-badge:hover { opacity: 0.8; transform: scale(1.04); }

.s-todo    { background: var(--border-light);          color: var(--text-muted);  border-color: var(--border); }
.s-doing   { background: rgba(245,158,11,0.12);        color: var(--warning);     border-color: rgba(245,158,11,0.3); }
.s-pending { background: rgba(139, 92,246,0.12);       color: #8B5CF6;            border-color: rgba(139,92,246,0.3); }
.s-done    { background: rgba( 34,197, 94,0.12);       color: var(--success);     border-color: rgba(34,197,94,0.3); }

/* ── Status picker dropdown ── */
.status-picker {
  position: absolute;
  top: calc(100% + 5px);
  left: 0;
  z-index: 200;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  box-shadow: 0 6px 20px rgba(0,0,0,0.14);
  overflow: hidden;
  min-width: 88px;
}
.picker-dot {
  display: inline-block;
  width: 7px; height: 7px;
  border-radius: 50%;
  margin-right: 5px;
  vertical-align: middle;
}
.picker-opt {
  display: block;
  width: 100%;
  padding: 7px 14px;
  text-align: left;
  font-size: 0.78rem;
  font-weight: 500;
  cursor: pointer;
  background: transparent;
  color: var(--text);
  transition: background 0.1s;
  border-bottom: 1px solid var(--border-light);
}
.picker-opt:last-child { border-bottom: none; }
.picker-opt:hover { background: var(--hover); }
.picker-opt.current { font-weight: 700; }
.picker-opt.s-todo.current    { color: var(--text-muted); }
.picker-opt.s-doing.current   { color: var(--warning); }
.picker-opt.s-pending.current { color: #8B5CF6; }
.picker-opt.s-done.current    { color: var(--success); }

/* ── Task body ── */
.task-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  outline: none;
}
.title {
  font-size: 0.88rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  letter-spacing: -0.01em;
}
.tags { display: flex; flex-wrap: wrap; gap: 4px; }
.tag-chip {
  font-size: 0.7rem;
  padding: 1px 6px;
  border-radius: 10px;
  background: var(--accent-soft);
  color: var(--accent);
  max-width: 80px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ── Group badge ── */
.group-badge {
  font-size: 0.68rem;
  font-weight: 500;
  padding: 1px 7px;
  border-radius: 10px;
  white-space: nowrap;
  flex-shrink: 1;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ── Recur badge ── */
.recur-badge {
  display: flex;
  align-items: center;
  color: var(--accent);
  flex-shrink: 0;
  opacity: 0.75;
}

/* ── Due date ── */
.due {
  font-size: 0.75rem;
  color: var(--text-muted);
  white-space: nowrap;
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}
.due.overdue { color: var(--danger); font-weight: 500; }

/* ── Extra badge (e.g. 期限切れ経過時間) ── */
.extra-badge {
  font-size: 0.68rem;
  font-weight: 600;
  color: var(--danger);
  background: var(--danger-soft);
  padding: 1px 7px;
  border-radius: 10px;
  white-space: nowrap;
  flex-shrink: 0;
}

/* ── Copy / Delete buttons ── */
.copy-btn {
  opacity: 0;
  width: 22px; height: 22px;
  border: none; background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 6px;
  flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.1s, color 0.1s, opacity 0.1s;
}
.task-item:hover .copy-btn { opacity: 1; }
.copy-btn:hover { background: var(--accent-soft); color: var(--accent); }

.del-btn {
  opacity: 0;
  width: 22px; height: 22px;
  border: none; background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 6px;
  flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.1s, color 0.1s, opacity 0.1s;
}
.task-item:hover .del-btn { opacity: 1; }
.del-btn:hover { background: var(--danger-soft); color: var(--danger); }
</style>
