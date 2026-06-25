export type Status = 'todo' | 'doing' | 'pending' | 'done';
export type RecurType = 'daily' | 'weekly' | 'monthly' | 'yearly';
export type RecurMonthRule = 'day' | 'last_day' | 'last_weekday';

export interface Task {
  id: number;
  title: string;
  notes: string | null;
  status: Status;
  due_at: number | null;
  remind_at: number | null;
  notified: boolean;
  group_id: number | null;
  sort_order: number;
  created_at: number;
  updated_at: number;
  completed_at: number | null;
  // Recurrence
  recur_type: RecurType | null;
  recur_interval: number | null;    // 1=毎週 2=隔週
  recur_weekdays: number[] | null;  // 0=Sun, 1=Mon, ..., 6=Sat
  recur_month_rule: RecurMonthRule | null;
  recur_month_day: number | null;   // 1-31
  priority: number;                 // 0=none 1=low 2=medium 3=high
}

export interface TaskWithTags extends Task {
  tags: Tag[];
}

export interface NewTask {
  title: string;
  notes?: string;
  due_at?: number;
  remind_at?: number;
  group_id?: number;
  tag_ids?: number[];
  new_tag_names?: string[];
  // Recurrence
  recur_type?: RecurType;
  recur_interval?: number;
  recur_weekdays?: number[];
  recur_month_rule?: RecurMonthRule;
  recur_month_day?: number;
}

export interface TaskUpdate {
  title: string;
  notes?: string;
  due_at?: number;
  remind_at?: number;
  group_id?: number;
  // Recurrence
  recur_type?: RecurType;
  recur_interval?: number;
  recur_weekdays?: number[];
  recur_month_rule?: RecurMonthRule;
  recur_month_day?: number;
  priority: number;
}

export interface Group {
  id: number;
  name: string;
  color: string | null;
  sort_order: number;
  created_at: number;
}

export interface Tag {
  id: number;
  name: string;
  color: string | null;
}
