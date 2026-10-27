import { derived, writable } from 'svelte/store';
import type { Group, StatusDef, Tag, TaskWithTags, Status } from './types';
import { api } from './api';
import { startOfDay } from './dateUtils';

export const tasks    = writable<TaskWithTags[]>([]);
export const groups   = writable<Group[]>([]);
export const tags     = writable<Tag[]>([]);
export const statuses = writable<StatusDef[]>([]);

export const prevStatusMap  = writable<Map<number, Status>>(new Map());
export const clipboardTask  = writable<TaskWithTags | null>(null);

const FILTER_STATUSES_KEY = 'filter_statuses';

function loadFilterStatuses(): Set<Status> {
  try {
    const saved = JSON.parse(localStorage.getItem(FILTER_STATUSES_KEY) ?? '[]');
    if (Array.isArray(saved)) return new Set(saved.filter(k => typeof k === 'string'));
  } catch { /* localStorage 利用不可・破損時は絞り込みなし */ }
  return new Set();
}

export const filterStatuses = writable<Set<Status>>(loadFilterStatuses());
filterStatuses.subscribe(v => {
  try { localStorage.setItem(FILTER_STATUSES_KEY, JSON.stringify([...v])); } catch { /* 無視 */ }
});
/** localStorage に保存される number | null の絞り込み用 store */
function persistedId(key: string) {
  let initial: number | null = null;
  try {
    const saved = localStorage.getItem(key);
    if (saved !== null && Number.isInteger(Number(saved))) initial = Number(saved);
  } catch { /* localStorage 利用不可時は絞り込みなし */ }

  const store = writable<number | null>(initial);
  store.subscribe(v => {
    try {
      if (v === null) localStorage.removeItem(key);
      else localStorage.setItem(key, String(v));
    } catch { /* 無視 */ }
  });
  return store;
}

export const filterGroupId = persistedId('filter_group_id');
export const filterTagId   = persistedId('filter_tag_id');
export const searchQuery    = writable<string>('');
export type SortMode = 'manual' | 'due_asc' | 'due_desc' | 'priority_desc';
export type ViewMode = 'grouped' | 'flat' | 'schedule';

const SORT_MODE_KEY = 'sort_mode';
const SORT_MODES: SortMode[] = ['manual', 'due_asc', 'due_desc', 'priority_desc'];

function loadSortMode(): SortMode {
  try {
    const saved = localStorage.getItem(SORT_MODE_KEY);
    if (SORT_MODES.includes(saved as SortMode)) return saved as SortMode;
  } catch { /* localStorage 利用不可時はデフォルト */ }
  return 'manual';
}

export const sortMode = writable<SortMode>(loadSortMode());
sortMode.subscribe(v => {
  try { localStorage.setItem(SORT_MODE_KEY, v); } catch { /* 無視 */ }
});

const VIEW_MODE_KEY = 'view_mode';
const VIEW_MODES: ViewMode[] = ['grouped', 'flat', 'schedule'];

function loadViewMode(): ViewMode {
  try {
    const saved = localStorage.getItem(VIEW_MODE_KEY);
    if (VIEW_MODES.includes(saved as ViewMode)) return saved as ViewMode;
  } catch { /* localStorage 利用不可時はデフォルト */ }
  return 'grouped';
}

export const viewMode = writable<ViewMode>(loadViewMode());
viewMode.subscribe(v => {
  try { localStorage.setItem(VIEW_MODE_KEY, v); } catch { /* 無視 */ }
});

export const filteredTasks = derived(
  [tasks, filterStatuses, filterGroupId, filterTagId, searchQuery],
  ([$t, $fs, $gid, $tid, $q]) =>
    $t.filter(t => {
      if ($fs.size > 0 && !$fs.has(t.status as Status)) return false;
      if ($gid !== null && t.group_id !== $gid) return false;
      if ($tid !== null && !t.tags.some(tag => tag.id === $tid)) return false;
      if ($q  && !t.title.toLowerCase().includes($q.toLowerCase())) return false;
      return true;
    })
);

// Tasks due today or spanning today, used for the top "今日のタスク" section.
// Statuses with show_in_today=false (default: pending, done) are excluded.
export const todayTasks = derived([filteredTasks, statuses], ([$tasks, $statuses]) => {
  const now      = new Date();
  const dayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const dayEnd   = dayStart + 86_400_000;
  const hidden   = $statuses.length > 0
    ? new Set($statuses.filter(s => !s.show_in_today).map(s => s.key))
    : new Set<Status>(['done', 'pending']); // fallback until statuses load
  return $tasks.filter(t => {
    if (hidden.has(t.status) || t.due_at === null) return false;
    if (t.start_at !== null) {
      // Period task: show every day from start_at to due_at
      return t.start_at < dayEnd && t.due_at >= dayStart;
    }
    // Single due-date task: show only on the due date
    return t.due_at >= dayStart && t.due_at < dayEnd;
  });
});

// Tasks whose due date has passed (completed tasks excluded), used for the
// "期限切れ" section shown above the group/flat task lists.
export const overdueTasks = derived([filteredTasks, sortMode], ([$tasks, $sort]) => {
  const dayStart = startOfDay(Date.now());
  const overdue = $tasks.filter(t => t.status !== 'done' && t.due_at !== null && t.due_at < dayStart);
  return applySort(overdue, $sort);
});

function applySort(tasks: TaskWithTags[], sort: string): TaskWithTags[] {
  if (sort === 'manual') return tasks;
  return [...tasks].sort((a, b) => {
    if (sort === 'priority_desc') return b.priority - a.priority;
    if (a.due_at === null && b.due_at === null) return 0;
    if (a.due_at === null) return 1;
    if (b.due_at === null) return -1;
    return sort === 'due_asc' ? a.due_at - b.due_at : b.due_at - a.due_at;
  });
}

export const sortedTasks = derived(
  [filteredTasks, sortMode],
  ([$tasks, $sort]) => applySort($tasks, $sort)
);

export const tasksByGroup = derived(
  [filteredTasks, groups, sortMode],
  ([$tasks, $groups, $sort]) => {
    const sorted = applySort($tasks, $sort);

    const map = new Map<number | null, TaskWithTags[]>();
    map.set(null, []);
    $groups.forEach(g => map.set(g.id, []));
    sorted.forEach(t => {
      const key = t.group_id ?? null;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(t);
    });
    return { map, groups: $groups };
  }
);

export async function loadAll() {
  const [t, g, ta, st] = await Promise.all([
    api.listTasks(), api.listGroups(), api.listTags(), api.listStatuses(),
  ]);
  tasks.set(t);
  groups.set(g);
  tags.set(ta);
  statuses.set(st);

  // 保存済みの絞り込みから、存在しなくなったグループ・タグ・ステータスを取り除く
  filterGroupId.update(prev => (prev !== null && !g.some(x => x.id === prev) ? null : prev));
  filterTagId.update(prev => (prev !== null && !ta.some(x => x.id === prev) ? null : prev));
  const keys = new Set(st.map(d => d.key));
  filterStatuses.update(prev => {
    const next = new Set([...prev].filter(k => keys.has(k)));
    return next.size === prev.size ? prev : next;
  });
}

export async function refreshStatuses() {
  statuses.set(await api.listStatuses());
}

export async function refreshTasks() {
  tasks.set(await api.listTasks());
}

export async function refreshTags() {
  tags.set(await api.listTags());
}
