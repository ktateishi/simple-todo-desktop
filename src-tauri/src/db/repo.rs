use rusqlite::{params, Connection, OptionalExtension, Result};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Weekday};
use crate::domain::*;

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// ───── Recurrence ─────────────────────────────────────────────────────────────

fn ms_to_date(ms: i64) -> Option<NaiveDate> {
    DateTime::from_timestamp_millis(ms).map(|dt| dt.naive_utc().date())
}

fn date_to_ms(date: NaiveDate) -> Option<i64> {
    Some(date.and_hms_opt(0, 0, 0)?.and_utc().timestamp_millis())
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 { (year + 1, 1u32) } else { (year, month + 1) };
    (NaiveDate::from_ymd_opt(ny, nm, 1).unwrap()
        - NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as u32
}

pub fn compute_next_occurrence(task: &Task) -> Option<i64> {
    let recur_type = task.recur_type.as_deref()?;
    let interval = task.recur_interval.unwrap_or(1);
    let base = ms_to_date(task.due_at.unwrap_or_else(now_ms))?;

    let next: Option<NaiveDate> = match recur_type {
        "daily" => Some(base + Duration::days(interval)),

        "weekly" => {
            let days = task.recur_weekdays.as_deref().unwrap_or(&[]);
            if days.is_empty() { return None; }

            if interval <= 1 {
                (1i64..=7)
                    .map(|i| base + Duration::days(i))
                    .find(|d| days.contains(&(d.weekday().num_days_from_sunday() as u8)))
            } else {
                // Bi-weekly: advance from start of current week (Monday-based)
                let days_from_mon = base.weekday().num_days_from_monday() as i64;
                let week_start = base - Duration::days(days_from_mon);
                let next_week_start = week_start + Duration::weeks(interval);
                (0i64..7)
                    .map(|i| next_week_start + Duration::days(i))
                    .find(|d| days.contains(&(d.weekday().num_days_from_sunday() as u8)))
            }
        }

        "monthly" => {
            let (ny, nm) = if base.month() == 12 {
                (base.year() + 1, 1u32)
            } else {
                (base.year(), base.month() + 1)
            };
            match task.recur_month_rule.as_deref() {
                Some("day") => {
                    let day = task.recur_month_day.unwrap_or(base.day() as i64) as u32;
                    let max_day = days_in_month(ny, nm);
                    NaiveDate::from_ymd_opt(ny, nm, day.min(max_day))
                }
                Some("last_day") => {
                    NaiveDate::from_ymd_opt(ny, nm, days_in_month(ny, nm))
                }
                Some("last_weekday") => {
                    let last = NaiveDate::from_ymd_opt(ny, nm, days_in_month(ny, nm))?;
                    let offset = match last.weekday() {
                        Weekday::Sat => 1,
                        Weekday::Sun => 2,
                        _ => 0,
                    };
                    Some(last - Duration::days(offset))
                }
                _ => None,
            }
        }

        "yearly" => base.with_year(base.year() + 1),

        _ => None,
    };

    next.and_then(date_to_ms)
}

fn weekdays_to_json(v: &[u8]) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string())
}

// ───── Tasks ─────────────────────────────────────────────────────────────────

const TASK_SELECT: &str =
    "SELECT id,title,notes,status,due_at,remind_at,notified,group_id,
            sort_order,created_at,updated_at,completed_at,
            recur_type,recur_interval,recur_weekdays,recur_month_rule,recur_month_day,
            COALESCE(priority,0),start_at
     FROM tasks";

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    let recur_weekdays_str: Option<String> = row.get(14)?;
    let recur_weekdays = recur_weekdays_str
        .as_deref()
        .and_then(|s| serde_json::from_str::<Vec<u8>>(s).ok());

    Ok(Task {
        id:              row.get(0)?,
        title:           row.get(1)?,
        notes:           row.get(2)?,
        status:          row.get(3)?,
        due_at:          row.get(4)?,
        remind_at:       row.get(5)?,
        notified:        row.get::<_, i64>(6)? != 0,
        group_id:        row.get(7)?,
        sort_order:      row.get(8)?,
        created_at:      row.get(9)?,
        updated_at:      row.get(10)?,
        completed_at:    row.get(11)?,
        recur_type:      row.get(12)?,
        recur_interval:  row.get(13)?,
        recur_weekdays,
        recur_month_rule: row.get(15)?,
        recur_month_day:  row.get(16)?,
        priority:         row.get(17)?,
        start_at:         row.get(18)?,
    })
}

pub fn get_task(conn: &Connection, id: i64) -> Result<Task> {
    conn.query_row(
        &format!("{} WHERE id=?1", TASK_SELECT),
        params![id],
        row_to_task,
    )
}

pub fn get_task_tags(conn: &Connection, task_id: i64) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color FROM tags t
         JOIN task_tags tt ON tt.tag_id=t.id
         WHERE tt.task_id=?1"
    )?;
    let rows = stmt.query_map(params![task_id], |r| {
        Ok(Tag { id: r.get(0)?, name: r.get(1)?, color: r.get(2)? })
    })?;
    rows.collect()
}

fn with_tags(conn: &Connection, task: Task) -> Result<TaskWithTags> {
    let tags = get_task_tags(conn, task.id)?;
    Ok(TaskWithTags { task, tags })
}

pub fn list_all_tasks(conn: &Connection) -> Result<Vec<TaskWithTags>> {
    let mut stmt = conn.prepare(
        &format!("{} ORDER BY sort_order ASC", TASK_SELECT)
    )?;
    let rows = stmt.query_map([], row_to_task)?;
    let tasks: Vec<Task> = rows.collect::<Result<_>>()?;
    tasks.into_iter().map(|t| with_tags(conn, t)).collect()
}

pub fn insert_task(conn: &Connection, input: &NewTask) -> Result<TaskWithTags> {
    let now = now_ms();
    let max_order: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order),0.0) FROM tasks", [], |r| r.get(0)
    )?;

    let recur_weekdays_json = input.recur_weekdays.as_ref()
        .map(|w| weekdays_to_json(w));

    conn.execute(
        "INSERT INTO tasks
             (title,notes,status,due_at,remind_at,group_id,sort_order,created_at,updated_at,
              recur_type,recur_interval,recur_weekdays,recur_month_rule,recur_month_day,start_at)
         VALUES (?1,?2,'todo',?3,?4,?5,?6,?7,?7,?8,?9,?10,?11,?12,?13)",
        params![
            input.title, input.notes, input.due_at, input.remind_at,
            input.group_id, max_order + 1.0, now,
            input.recur_type, input.recur_interval, recur_weekdays_json,
            input.recur_month_rule, input.recur_month_day, input.start_at,
        ],
    )?;
    let id = conn.last_insert_rowid();

    if let Some(ids) = &input.tag_ids {
        for tag_id in ids {
            conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id,tag_id) VALUES (?1,?2)",
                params![id, tag_id],
            )?;
        }
    }
    if let Some(names) = &input.new_tag_names {
        for name in names {
            conn.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)", params![name]
            )?;
            let tag_id: i64 = conn.query_row(
                "SELECT id FROM tags WHERE name=?1", params![name], |r| r.get(0)
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id,tag_id) VALUES (?1,?2)",
                params![id, tag_id],
            )?;
        }
    }

    with_tags(conn, get_task(conn, id)?)
}

pub fn update_task(conn: &Connection, id: i64, upd: &TaskUpdate) -> Result<TaskWithTags> {
    let now = now_ms();
    let recur_weekdays_json = upd.recur_weekdays.as_ref()
        .map(|w| weekdays_to_json(w));

    conn.execute(
        "UPDATE tasks SET title=?1,notes=?2,due_at=?3,remind_at=?4,
                          group_id=?5,notified=0,updated_at=?6,
                          recur_type=?7,recur_interval=?8,recur_weekdays=?9,
                          recur_month_rule=?10,recur_month_day=?11,
                          priority=?12,start_at=?13
         WHERE id=?14",
        params![
            upd.title, upd.notes, upd.due_at, upd.remind_at, upd.group_id, now,
            upd.recur_type, upd.recur_interval, recur_weekdays_json,
            upd.recur_month_rule, upd.recur_month_day, upd.priority, upd.start_at, id,
        ],
    )?;
    with_tags(conn, get_task(conn, id)?)
}

pub fn delete_task(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM tasks WHERE id=?1", params![id])?;
    Ok(())
}

pub fn set_status(conn: &Connection, id: i64, status: &str) -> Result<TaskWithTags> {
    let now = now_ms();

    // Recurring tasks: advance to next occurrence instead of marking done
    if status == "done" {
        let current = get_task(conn, id)?;
        if current.recur_type.is_some() {
            if let Some(next_due) = compute_next_occurrence(&current) {
                conn.execute(
                    "UPDATE tasks SET status='todo',due_at=?1,notified=0,
                                      completed_at=NULL,updated_at=?2 WHERE id=?3",
                    params![next_due, now, id],
                )?;
                return with_tags(conn, get_task(conn, id)?);
            }
        }
    }

    let completed_at: Option<i64> = if status == "done" { Some(now) } else { None };
    conn.execute(
        "UPDATE tasks SET status=?1,completed_at=?2,updated_at=?3 WHERE id=?4",
        params![status, completed_at, now, id],
    )?;
    with_tags(conn, get_task(conn, id)?)
}

pub fn reorder_task(
    conn: &Connection,
    id: i64,
    before_id: Option<i64>,
    after_id: Option<i64>,
) -> Result<()> {
    let prev: f64 = match before_id {
        Some(bid) => conn.query_row(
            "SELECT sort_order FROM tasks WHERE id=?1", params![bid], |r| r.get(0)
        )?,
        None => 0.0,
    };
    let next: f64 = match after_id {
        Some(aid) => conn.query_row(
            "SELECT sort_order FROM tasks WHERE id=?1", params![aid], |r| r.get(0)
        )?,
        None => {
            let max: f64 = conn.query_row(
                "SELECT COALESCE(MAX(sort_order),0.0) FROM tasks", [], |r| r.get(0)
            )?;
            max + 2.0
        }
    };
    conn.execute(
        "UPDATE tasks SET sort_order=?1 WHERE id=?2",
        params![(prev + next) / 2.0, id],
    )?;
    Ok(())
}

pub fn set_task_tags(conn: &Connection, task_id: i64, tag_ids: &[i64]) -> Result<()> {
    conn.execute("DELETE FROM task_tags WHERE task_id=?1", params![task_id])?;
    for tag_id in tag_ids {
        conn.execute(
            "INSERT OR IGNORE INTO task_tags (task_id,tag_id) VALUES (?1,?2)",
            params![task_id, tag_id],
        )?;
    }
    Ok(())
}

// ───── Groups ────────────────────────────────────────────────────────────────

fn row_to_group(row: &rusqlite::Row) -> rusqlite::Result<Group> {
    Ok(Group {
        id:         row.get(0)?,
        name:       row.get(1)?,
        color:      row.get(2)?,
        sort_order: row.get(3)?,
        created_at: row.get(4)?,
    })
}

pub fn list_groups(conn: &Connection) -> Result<Vec<Group>> {
    let mut stmt = conn.prepare(
        "SELECT id,name,color,sort_order,created_at FROM groups ORDER BY sort_order ASC"
    )?;
    let rows = stmt.query_map([], row_to_group)?;
    rows.collect()
}

pub fn insert_group(conn: &Connection, name: &str, color: Option<&str>) -> Result<Group> {
    let now = now_ms();
    let max_order: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order),0.0) FROM groups", [], |r| r.get(0)
    )?;
    conn.execute(
        "INSERT INTO groups (name,color,sort_order,created_at) VALUES (?1,?2,?3,?4)",
        params![name, color, max_order + 1.0, now],
    )?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id,name,color,sort_order,created_at FROM groups WHERE id=?1",
        params![id], row_to_group,
    )
}

pub fn update_group(conn: &Connection, id: i64, name: &str, color: Option<&str>) -> Result<Group> {
    conn.execute(
        "UPDATE groups SET name=?1,color=?2 WHERE id=?3",
        params![name, color, id],
    )?;
    conn.query_row(
        "SELECT id,name,color,sort_order,created_at FROM groups WHERE id=?1",
        params![id], row_to_group,
    )
}

pub fn delete_group(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM groups WHERE id=?1", params![id])?;
    Ok(())
}

// ───── Tags ──────────────────────────────────────────────────────────────────

pub fn list_tags(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id,name,color FROM tags ORDER BY name ASC"
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Tag { id: r.get(0)?, name: r.get(1)?, color: r.get(2)? })
    })?;
    rows.collect()
}

pub fn insert_tag(conn: &Connection, name: &str, color: Option<&str>) -> Result<Tag> {
    conn.execute(
        "INSERT OR IGNORE INTO tags (name,color) VALUES (?1,?2)", params![name, color]
    )?;
    conn.query_row(
        "SELECT id,name,color FROM tags WHERE name=?1", params![name],
        |r| Ok(Tag { id: r.get(0)?, name: r.get(1)?, color: r.get(2)? }),
    )
}

pub fn delete_tag(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM tags WHERE id=?1", params![id])?;
    Ok(())
}

// ───── Scheduler helper ───────────────────────────────────────────────────────

pub fn next_notify(conn: &Connection) -> Result<Option<(i64, String, i64)>> {
    conn.query_row(
        "SELECT id, title, COALESCE(remind_at, due_at) AS notify_at
         FROM tasks
         WHERE status != 'done'
           AND notified = 0
           AND COALESCE(remind_at, due_at) IS NOT NULL
         ORDER BY notify_at ASC
         LIMIT 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).optional()
}

pub fn mark_notified(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("UPDATE tasks SET notified=1 WHERE id=?1", params![id])?;
    Ok(())
}
