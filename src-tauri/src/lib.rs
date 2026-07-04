use std::sync::Mutex;
use tauri::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

mod commands;
mod db;
mod domain;
mod scheduler;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // ── Init DB ──
            let db_path = app.path().app_data_dir()?.join("todo.db");
            std::fs::create_dir_all(db_path.parent().unwrap())?;
            let mut conn = rusqlite::Connection::open(&db_path)?;
            db::init_db(&mut conn)?;

            // ── Scheduler channel ──
            let (tx, rx) = tokio::sync::watch::channel(());

            app.manage(db::AppState {
                db: Mutex::new(conn),
                scheduler_tx: tx,
            });

            // ── Start scheduler ──
            scheduler::start(app.handle().clone(), rx);

            // ── Menu bar ──
            let menu = build_menu(app)?;
            app.set_menu(menu)?;

            // ── System tray ──
            setup_tray(app)?;

            // ── Close → hide (instead of quit) ──
            if let Some(window) = app.get_webview_window("main") {
                let win = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_tasks,
            commands::add_task,
            commands::update_task,
            commands::delete_task,
            commands::set_task_status,
            commands::reorder_task,
            commands::set_task_tags,
            commands::list_groups,
            commands::add_group,
            commands::update_group,
            commands::delete_group,
            commands::list_statuses,
            commands::add_status,
            commands::update_status,
            commands::delete_status,
            commands::list_tags,
            commands::add_tag,
            commands::delete_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_menu(app: &tauri::App) -> tauri::Result<Menu<tauri::Wry>> {
    // ── アプリメニュー ──
    let app_m = Submenu::with_items(
        app,
        "Simple TODO",
        true,
        &[
            &PredefinedMenuItem::about(app, Some("Simple TODO について"), None::<AboutMetadata>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, Some("Simple TODO を隠す"))?,
            &PredefinedMenuItem::hide_others(app, Some("ほかを隠す"))?,
            &PredefinedMenuItem::show_all(app, Some("すべてを表示"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some("Simple TODO を終了"))?,
        ],
    )?;

    // ── 編集 ──
    let edit_m = Submenu::with_items(
        app,
        "編集",
        true,
        &[
            &PredefinedMenuItem::undo(app, Some("取り消す"))?,
            &PredefinedMenuItem::redo(app, Some("やり直す"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, Some("カット"))?,
            &PredefinedMenuItem::copy(app, Some("コピー"))?,
            &PredefinedMenuItem::paste(app, Some("ペースト"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::select_all(app, Some("すべてを選択"))?,
        ],
    )?;

    // ── 表示 ──
    let view_m = Submenu::with_items(
        app,
        "表示",
        true,
        &[&PredefinedMenuItem::fullscreen(app, Some("フルスクリーン"))?],
    )?;

    // ── ウィンドウ ──
    let window_m = Submenu::with_items(
        app,
        "ウィンドウ",
        true,
        &[
            &PredefinedMenuItem::minimize(app, Some("最小化"))?,
            &PredefinedMenuItem::maximize(app, Some("ズーム"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, Some("ウィンドウを閉じる"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::bring_all_to_front(app, Some("すべてを前面へ"))?,
        ],
    )?;

    Menu::with_items(app, &[&app_m, &edit_m, &view_m, &window_m])
}

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "開く", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .build(app)?;
    Ok(())
}
