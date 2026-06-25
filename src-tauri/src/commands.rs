use tauri::State;
use crate::db::{repo, AppState};
use crate::domain::*;

macro_rules! db {
    ($state:expr) => {
        $state.db.lock().unwrap()
    };
}

fn notify_scheduler(state: &AppState) {
    let _ = state.scheduler_tx.send(());
}

// ───── Tasks ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_tasks(state: State<AppState>) -> Result<Vec<TaskWithTags>, String> {
    let db = db!(state);
    repo::list_all_tasks(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_task(state: State<AppState>, input: NewTask) -> Result<TaskWithTags, String> {
    let db = db!(state);
    let result = repo::insert_task(&db, &input).map_err(|e| e.to_string())?;
    notify_scheduler(&state);
    Ok(result)
}

#[tauri::command]
pub fn update_task(state: State<AppState>, id: i64, update: TaskUpdate) -> Result<TaskWithTags, String> {
    let db = db!(state);
    let result = repo::update_task(&db, id, &update).map_err(|e| e.to_string())?;
    notify_scheduler(&state);
    Ok(result)
}

#[tauri::command]
pub fn delete_task(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = db!(state);
    repo::delete_task(&db, id).map_err(|e| e.to_string())?;
    notify_scheduler(&state);
    Ok(())
}

#[tauri::command]
pub fn set_task_status(state: State<AppState>, id: i64, status: String) -> Result<TaskWithTags, String> {
    let db = db!(state);
    let result = repo::set_status(&db, id, &status).map_err(|e| e.to_string())?;
    notify_scheduler(&state);
    Ok(result)
}

#[tauri::command]
pub fn reorder_task(
    state: State<AppState>,
    id: i64,
    before_id: Option<i64>,
    after_id: Option<i64>,
) -> Result<(), String> {
    let db = db!(state);
    repo::reorder_task(&db, id, before_id, after_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_task_tags(state: State<AppState>, task_id: i64, tag_ids: Vec<i64>) -> Result<(), String> {
    let db = db!(state);
    repo::set_task_tags(&db, task_id, &tag_ids).map_err(|e| e.to_string())
}

// ───── Groups ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_groups(state: State<AppState>) -> Result<Vec<Group>, String> {
    let db = db!(state);
    repo::list_groups(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_group(state: State<AppState>, name: String, color: Option<String>) -> Result<Group, String> {
    let db = db!(state);
    repo::insert_group(&db, &name, color.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_group(state: State<AppState>, id: i64, name: String, color: Option<String>) -> Result<Group, String> {
    let db = db!(state);
    repo::update_group(&db, id, &name, color.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_group(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = db!(state);
    repo::delete_group(&db, id).map_err(|e| e.to_string())
}

// ───── Tags ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_tags(state: State<AppState>) -> Result<Vec<Tag>, String> {
    let db = db!(state);
    repo::list_tags(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag(state: State<AppState>, name: String, color: Option<String>) -> Result<Tag, String> {
    let db = db!(state);
    repo::insert_tag(&db, &name, color.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = db!(state);
    repo::delete_tag(&db, id).map_err(|e| e.to_string())
}
