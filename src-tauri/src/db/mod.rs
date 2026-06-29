use rusqlite::{Connection, Result};
use std::sync::Mutex;
use tokio::sync::watch;

pub mod repo;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub scheduler_tx: watch::Sender<()>,
}

pub fn init_db(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;

         CREATE TABLE IF NOT EXISTS groups (
             id         INTEGER PRIMARY KEY AUTOINCREMENT,
             name       TEXT    NOT NULL,
             color      TEXT,
             sort_order REAL    NOT NULL DEFAULT 0.0,
             created_at INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS tasks (
             id           INTEGER PRIMARY KEY AUTOINCREMENT,
             title        TEXT    NOT NULL,
             notes        TEXT,
             status       TEXT    NOT NULL DEFAULT 'todo',
             due_at       INTEGER,
             remind_at    INTEGER,
             notified     INTEGER NOT NULL DEFAULT 0,
             group_id     INTEGER REFERENCES groups(id) ON DELETE SET NULL,
             sort_order   REAL    NOT NULL DEFAULT 0.0,
             created_at   INTEGER NOT NULL,
             updated_at   INTEGER NOT NULL,
             completed_at INTEGER
         );

         CREATE TABLE IF NOT EXISTS tags (
             id    INTEGER PRIMARY KEY AUTOINCREMENT,
             name  TEXT NOT NULL UNIQUE,
             color TEXT
         );

         CREATE TABLE IF NOT EXISTS task_tags (
             task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
             tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
             PRIMARY KEY (task_id, tag_id)
         );

         CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
         CREATE INDEX IF NOT EXISTS idx_tasks_due    ON tasks(due_at);
         CREATE INDEX IF NOT EXISTS idx_tasks_group  ON tasks(group_id, sort_order);",
    )?;

    // Migration: add columns to existing databases (errors ignored if already exist)
    for (col, def) in &[
        ("recur_type",       "TEXT"),
        ("recur_interval",   "INTEGER DEFAULT 1"),
        ("recur_weekdays",   "TEXT"),
        ("recur_month_rule", "TEXT"),
        ("recur_month_day",  "INTEGER"),
        ("priority",         "INTEGER NOT NULL DEFAULT 0"),
        ("start_at",         "INTEGER"),
    ] {
        let _ = conn.execute(
            &format!("ALTER TABLE tasks ADD COLUMN {} {}", col, def),
            [],
        );
    }

    Ok(())
}
