import { derived, writable } from 'svelte/store';
import type { Group, StatusDef, Tag, TaskWithTags, Status } from './types';
import { api } from './api';

export const tasks    = writable<TaskWithTags[]>([]);
export const groups   = writable<Group[]>([]);
export const tags     = writable<Tag[]>([]);
export const statuses = writable<StatusDef[]>([]);

export const prevStatusMap  = writable<Map<number, Status>>(new Map());
export const clipboardTask  = writable<TaskWithTags | null>(null);

export const filterStatuses = writable<Set<Status>>(new Set());
export const filterGroupId  = writable<number | null>(null);
export const filterTagId    = writable<number | null>(null);
export const searchQuery    = writable<string>('');
export const sortMode       = writable<'manual' | 'due_asc' | 'due_desc' | 'priority_desc'>('manual');
export const viewMode       = writable<'grouped' | 'flat'>('grouped');

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
