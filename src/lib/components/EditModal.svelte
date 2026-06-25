<script lang="ts">
  import { untrack } from 'svelte';
  import type { TaskWithTags, TaskUpdate, RecurType, RecurMonthRule } from '$lib/types';
  import { groups, tags as allTags } from '$lib/stores';

  import Pencil     from 'lucide-svelte/icons/pencil';
  import FileText   from 'lucide-svelte/icons/file-text';
  import Calendar   from 'lucide-svelte/icons/calendar';
  import Bell       from 'lucide-svelte/icons/bell';
  import RefreshCw  from 'lucide-svelte/icons/refresh-cw';
  import FolderOpen from 'lucide-svelte/icons/folder-open';
  import Tag        from 'lucide-svelte/icons/tag';
  import X          from 'lucide-svelte/icons/x';
  import Check      from 'lucide-svelte/icons/check';
  import Flag       from 'lucide-svelte/icons/flag';

  type SavePayload = { id: number; update: TaskUpdate; tagIds: number[] };

  type Props = {
    task: TaskWithTags;
    onSave:   (p: SavePayload) => void;
    onCancel: () => void;
  };

  let { task, onSave, onCancel }: Props = $props();

  let title      = $state(untrack(() => task.title));
  let notes      = $state(untrack(() => task.notes ?? ''));
  let dueStr     = $state(untrack(() => task.due_at    ? toDateStr(task.due_at)    : ''));
  let remindStr  = $state(untrack(() => task.remind_at ? toDateTimeStr(task.remind_at) : ''));
  let groupId    = $state<number | null>(untrack(() => task.group_id));
  let selectedTagIds = $state(untrack(() => new Set(task.tags.map(t => t.id))));

  let priority       = $state<number>(untrack(() => task.priority ?? 0));
  let recurType      = $state<RecurType | ''>(untrack(() => task.recur_type ?? ''));
  let recurInterval  = $state<number>(untrack(() => task.recur_interval ?? 1));
  let recurWeekdays  = $state<Set<number>>(untrack(() => new Set(task.recur_weekdays ?? [])));
  let recurMonthRule = $state<RecurMonthRule>(untrack(() => task.recur_month_rule ?? 'day'));
  let recurMonthDay  = $state<number>(untrack(() => task.recur_month_day ?? 1));

  const WD_LABELS: { n: number; label: string }[] = [
    { n: 1, label: '月' }, { n: 2, label: '火' }, { n: 3, label: '水' },
    { n: 4, label: '木' }, { n: 5, label: '金' }, { n: 6, label: '土' },
    { n: 0, label: '日' },
  ];

  function toggleWeekday(n: number) {
    const s = new Set(recurWeekdays);
    s.has(n) ? s.delete(n) : s.add(n);
    recurWeekdays = s;
  }

  function setWeekdaysOnly() {
    recurWeekdays = new Set([1, 2, 3, 4, 5]);
  }

  function toggleTag(id: number) {
    const next = new Set(selectedTagIds);
    next.has(id) ? next.delete(id) : next.add(id);
    selectedTagIds = next;
  }

  function toDateStr(ts: number): string {
    const d = new Date(ts);
    const p = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
  }

  function toDateTimeStr(ts: number): string {
    const d = new Date(ts);
    const p = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
  }

  function save() {
    if (!title.trim()) return;
    const rt = recurType || undefined;
    onSave({
      id: task.id,
      update: {
        title:     title.trim(),
        notes:     notes || undefined,
        due_at:    dueStr    ? new Date(dueStr + 'T00:00').getTime() : undefined,
        remind_at: remindStr ? new Date(remindStr).getTime() : undefined,
        group_id:  groupId ?? undefined,
        priority,
        recur_type:       rt,
        recur_interval:   rt === 'weekly' ? recurInterval : undefined,
        recur_weekdays:   rt === 'weekly' ? [...recurWeekdays] : undefined,
        recur_month_rule: rt === 'monthly' ? recurMonthRule : undefined,
        recur_month_day:  (rt === 'monthly' && recurMonthRule === 'day') ? recurMonthDay : undefined,
      },
      tagIds: [...selectedTagIds],
    });
  }

  function onOverlayKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onCancel();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="overlay"
  onclick={onCancel}
  onkeydown={onOverlayKeydown}
  role="dialog"
  aria-modal="true"
  aria-label="タスクを編集"
  tabindex="-1"
>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal" onclick={(e) => e.stopPropagation()}>

    <!-- Header -->
    <div class="modal-header">
      <h2 class="modal-title">タスクを編集</h2>
      <button class="close-btn" onclick={onCancel} aria-label="閉じる">
        <X size={16} strokeWidth={2} />
      </button>
    </div>

    <!-- Body -->
    <div class="modal-body">

    <!-- Priority -->
    <div class="priority-field">
      <span class="field-label"><Flag size={12} strokeWidth={2} />優先度</span>
      <div class="priority-btns">
        {#each ([
          { v: 0, label: 'なし',  cls: 'p-none'   },
          { v: 1, label: '低',    cls: 'p-low'    },
          { v: 2, label: '中',    cls: 'p-medium' },
          { v: 3, label: '高',    cls: 'p-high'   },
        ] as const) as p}
          <button
            type="button"
            class="prio-btn {p.cls}"
            class:active={priority === p.v}
            onclick={() => priority = p.v}
          >{p.label}</button>
        {/each}
      </div>
    </div>

    <label class="field">
      <span class="field-label"><Pencil size={12} strokeWidth={2} />タイトル</span>
      <input bind:value={title} class="input" placeholder="タスク名" />
    </label>

    <!-- Notes -->
    <label class="field">
      <span class="field-label"><FileText size={12} strokeWidth={2} />メモ</span>
      <textarea bind:value={notes} class="input textarea" placeholder="メモ（任意）" rows="3"></textarea>
    </label>

    <!-- Due / Remind -->
    <div class="row-2">
      <label class="field">
        <span class="field-label"><Calendar size={12} strokeWidth={2} />期限</span>
        <input type="date" bind:value={dueStr} class="input" />
      </label>
      <label class="field">
        <span class="field-label"><Bell size={12} strokeWidth={2} />リマインド</span>
        <input type="datetime-local" bind:value={remindStr} class="input" />
      </label>
    </div>

    <!-- Recurrence -->
    <div class="recur-section">
      <div class="recur-header">
        <span class="field-label"><RefreshCw size={12} strokeWidth={2} />繰り返し</span>
        <select bind:value={recurType} class="input recur-select">
          <option value="">なし</option>
          <option value="daily">毎日</option>
          <option value="weekly">毎週</option>
          <option value="monthly">毎月</option>
          <option value="yearly">毎年</option>
        </select>
      </div>

      {#if recurType === 'weekly'}
        <div class="weekday-row">
          {#each WD_LABELS as wd (wd.n)}
            <button
              type="button"
              class="wd-btn"
              class:selected={recurWeekdays.has(wd.n)}
              onclick={() => toggleWeekday(wd.n)}
            >{wd.label}</button>
          {/each}
          <button type="button" class="quick-btn" onclick={setWeekdaysOnly}>平日のみ</button>
        </div>
        <div class="interval-row">
          <label class="radio-opt">
            <input type="radio" bind:group={recurInterval} value={1} />
            毎週
          </label>
          <label class="radio-opt">
            <input type="radio" bind:group={recurInterval} value={2} />
            隔週
          </label>
        </div>
      {/if}

      {#if recurType === 'monthly'}
        <div class="monthly-opts">
          <label class="radio-opt">
            <input type="radio" bind:group={recurMonthRule} value="day" />
            毎月
            <input
              type="number" min="1" max="31"
              bind:value={recurMonthDay}
              disabled={recurMonthRule !== 'day'}
              class="day-input"
            />
            日
          </label>
          <label class="radio-opt">
            <input type="radio" bind:group={recurMonthRule} value="last_day" />
            月末
          </label>
          <label class="radio-opt">
            <input type="radio" bind:group={recurMonthRule} value="last_weekday" />
            月末の平日
          </label>
        </div>
      {/if}
    </div>

    <!-- Group -->
    <label class="field">
      <span class="field-label"><FolderOpen size={12} strokeWidth={2} />グループ</span>
      <select bind:value={groupId} class="input">
        <option value={null}>グループなし</option>
        {#each $groups as g (g.id)}
          <option value={g.id}>{g.name}</option>
        {/each}
      </select>
    </label>

    <!-- Tags -->
    <div class="tag-section">
      <span class="field-label"><Tag size={12} strokeWidth={2} />タグ</span>
      <div class="tag-chips">
        {#each $allTags as tag (tag.id)}
          <button
            type="button"
            class="tag-chip"
            class:selected={selectedTagIds.has(tag.id)}
            onclick={() => toggleTag(tag.id)}
            style={tag.color ? `--c:${tag.color}` : ''}
          >#{tag.name}</button>
        {/each}
      </div>
    </div>

    </div><!-- /modal-body -->

    <!-- Actions -->
    <div class="actions">
      <button class="btn-cancel" onclick={onCancel}>キャンセル</button>
      <button class="btn-save" onclick={save} disabled={!title.trim()}>
        <Check size={14} strokeWidth={2.5} />
        保存
      </button>
    </div>
  </div>
</div>

<style>
.overlay {
  position: fixed; inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex; align-items: center; justify-content: center;
  z-index: 100;
  backdrop-filter: blur(2px);
}
.modal {
  background: var(--surface);
  border-radius: 14px;
  width: min(480px, 90vw);
  max-height: 90vh;
  overflow: hidden;
  display: flex; flex-direction: column;
  box-shadow: 0 16px 48px rgba(0,0,0,0.25), 0 2px 8px rgba(0,0,0,0.1);
  border: 1px solid var(--border-light);
}

/* Header */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 22px 14px;
  border-bottom: 1px solid var(--border-light);
  background: var(--surface);
  border-radius: 14px 14px 0 0;
  flex-shrink: 0;
}
.modal-title {
  font-size: 1rem;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--text);
}
.close-btn {
  display: flex; align-items: center; justify-content: center;
  width: 28px; height: 28px;
  border-radius: 7px;
  color: var(--text-muted);
  background: transparent;
  transition: background 0.1s, color 0.1s;
}
.close-btn:hover { background: var(--hover); color: var(--text); }

/* Body */
.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px 22px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

/* Priority */
.priority-field {
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.priority-btns {
  display: flex;
  gap: 6px;
}
.prio-btn {
  flex: 1;
  padding: 6px 0;
  border-radius: 8px;
  border: 1.5px solid var(--border);
  background: transparent;
  font-size: 0.82rem;
  font-weight: 500;
  color: var(--text-muted);
  cursor: pointer;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
}
.prio-btn:hover { background: var(--hover); color: var(--text); }
.p-none.active  { background: var(--border-light); border-color: var(--border); color: var(--text-muted); }
.p-low.active   { background: rgba(59,130,246,0.12); border-color: #3B82F6; color: #3B82F6; }
.p-medium.active{ background: rgba(234,179,8,0.12);  border-color: var(--warning); color: var(--warning); }
.p-high.active  { background: rgba(239,68,68,0.12);  border-color: var(--danger); color: var(--danger); }

/* Fields */
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 0.78rem;
  font-weight: 500;
  color: var(--text-muted);
  letter-spacing: 0.01em;
}
:global(.field-label svg) { flex-shrink: 0; }

.input {
  padding: 7px 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg);
  color: var(--text);
  font-size: 0.92rem;
  font-family: inherit;
  transition: border-color 0.12s;
}
.input:focus { outline: none; border-color: var(--accent); }
.textarea { resize: vertical; min-height: 64px; }
.row-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

/* Recurrence */
.recur-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.recur-header { display: flex; align-items: center; gap: 10px; }
.recur-select { flex: 1; }

.weekday-row { display: flex; gap: 4px; align-items: center; flex-wrap: wrap; }
.wd-btn {
  width: 30px; height: 30px;
  border-radius: 50%;
  border: 1.5px solid var(--border);
  background: transparent;
  color: var(--text-muted);
  font-size: 0.78rem;
  cursor: pointer;
  transition: background 0.1s, border-color 0.1s;
}
.wd-btn.selected { background: var(--accent); color: #fff; border-color: var(--accent); }
.wd-btn:hover:not(.selected) { background: var(--hover); border-color: var(--text-muted); }
.quick-btn {
  padding: 4px 10px;
  border: 1.5px solid var(--border);
  border-radius: 12px;
  background: transparent;
  color: var(--text-muted);
  font-size: 0.75rem;
  cursor: pointer;
  margin-left: 4px;
  transition: border-color 0.1s, color 0.1s;
}
.quick-btn:hover { border-color: var(--accent); color: var(--accent); }

.interval-row { display: flex; gap: 20px; padding-left: 2px; }
.radio-opt {
  display: flex; align-items: center; gap: 6px;
  font-size: 0.88rem; color: var(--text); cursor: pointer;
}
.radio-opt input[type="radio"] { cursor: pointer; accent-color: var(--accent); }

.monthly-opts { display: flex; flex-direction: column; gap: 8px; }
.day-input {
  width: 44px;
  padding: 3px 6px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  color: var(--text);
  font-size: 0.88rem;
  text-align: center;
}
.day-input:disabled { opacity: 0.35; }
.day-input:focus { outline: none; border-color: var(--accent); }

/* Tags */
.tag-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.tag-chips { display: flex; flex-wrap: wrap; gap: 6px; }
.tag-chip {
  padding: 4px 11px;
  border-radius: 14px;
  border: 1.5px solid var(--c, var(--border));
  background: transparent;
  color: var(--c, var(--text-muted));
  font-size: 0.8rem;
  cursor: pointer;
  transition: background 0.1s, color 0.1s;
}
.tag-chip.selected { background: var(--c, var(--accent)); color: #fff; }
.tag-chip:not(.selected):hover { background: var(--hover); }

/* Actions */
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 14px 22px 18px;
  border-top: 1px solid var(--border-light);
  background: var(--surface);
  border-radius: 0 0 14px 14px;
  flex-shrink: 0;
}
.btn-cancel {
  padding: 7px 16px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 0.88rem;
  transition: background 0.1s, color 0.1s;
}
.btn-cancel:hover { background: var(--hover); color: var(--text); }
.btn-save {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 18px;
  border-radius: 8px;
  background: var(--accent);
  color: #fff;
  cursor: pointer;
  font-size: 0.88rem;
  font-weight: 500;
  transition: opacity 0.12s;
}
.btn-save:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-save:not(:disabled):hover { opacity: 0.85; }
</style>
