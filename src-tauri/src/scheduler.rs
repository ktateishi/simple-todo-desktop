use std::time::Duration;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;
use tokio::sync::watch;

use crate::db::{repo, AppState};

pub fn start(app: tauri::AppHandle, mut rx: watch::Receiver<()>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let next = {
                let state = app.state::<AppState>();
                let db = state.db.lock().unwrap();
                repo::next_notify(&db)
            };

            match next {
                Err(_) | Ok(None) => {
                    // No upcoming notifications; sleep until tasks change
                    let _ = rx.changed().await;
                }
                Ok(Some((task_id, title, notify_at_ms))) => {
                    let now_ms = repo::now_ms();
                    let wait_ms = (notify_at_ms - now_ms).max(0) as u64;

                    if wait_ms == 0 {
                        send_notification(&app, &title);
                        mark_done(&app, task_id);
                        continue;
                    }

                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_millis(wait_ms)) => {
                            send_notification(&app, &title);
                            mark_done(&app, task_id);
                        }
                        // Tasks changed — recalculate next deadline
                        _ = rx.changed() => {}
                    }
                }
            }
        }
    });
}

fn send_notification(app: &tauri::AppHandle, title: &str) {
    if let Err(e) = app
        .notification()
        .builder()
        .title("Simple TODO")
        .body(title)
        .show()
    {
        eprintln!("[scheduler] notification failed: {e}");
    }
}

fn mark_done(app: &tauri::AppHandle, task_id: i64) {
    let state = app.state::<AppState>();
    let db = state.db.lock().unwrap();
    let _ = repo::mark_notified(&db, task_id);
}
