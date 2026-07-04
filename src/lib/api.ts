import { invoke } from '@tauri-apps/api/core';
import type { Group, NewTask, StatusDef, Tag, TaskUpdate, TaskWithTags } from './types';

export const api = {
  // Tasks
  listTasks:     ()                                          => invoke<TaskWithTags[]>('list_tasks'),
  addTask:       (input: NewTask)                            => invoke<TaskWithTags>('add_task', { input }),
  updateTask:    (id: number, update: TaskUpdate)            => invoke<TaskWithTags>('update_task', { id, update }),
  deleteTask:    (id: number)                                => invoke<void>('delete_task', { id }),
  setTaskStatus: (id: number, status: string)                => invoke<TaskWithTags>('set_task_status', { id, status }),
  reorderTask:   (id: number, beforeId?: number, afterId?: number) =>
    invoke<void>('reorder_task', { id, beforeId: beforeId ?? null, afterId: afterId ?? null }),
  setTaskTags:   (taskId: number, tagIds: number[])          => invoke<void>('set_task_tags', { taskId, tagIds }),

  // Groups
  listGroups:   ()                                         => invoke<Group[]>('list_groups'),
  addGroup:     (name: string, color?: string)             => invoke<Group>('add_group', { name, color: color ?? null }),
  updateGroup:  (id: number, name: string, color?: string) => invoke<Group>('update_group', { id, name, color: color ?? null }),
  deleteGroup:  (id: number)                               => invoke<void>('delete_group', { id }),

  // Statuses
  listStatuses: ()                                                             => invoke<StatusDef[]>('list_statuses'),
  addStatus:    (name: string, color: string | null, showInToday: boolean)     => invoke<StatusDef>('add_status', { name, color, showInToday }),
  updateStatus: (id: number, name: string, color: string | null, showInToday: boolean) =>
    invoke<StatusDef>('update_status', { id, name, color, showInToday }),
  deleteStatus: (id: number)                                                   => invoke<void>('delete_status', { id }),

  // Tags
  listTags:  ()                              => invoke<Tag[]>('list_tags'),
  addTag:    (name: string, color?: string)  => invoke<Tag>('add_tag', { name, color: color ?? null }),
  deleteTag: (id: number)                    => invoke<void>('delete_tag', { id }),
};
