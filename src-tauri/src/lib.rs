mod commands;
mod db;
mod sync;

use std::sync::Arc;
use tauri::Manager;

pub struct AppState {
    pub db: Arc<db::connection::DatabaseConnection>,
    pub sync: Arc<sync::SyncManager>,
    pub app: tauri::AppHandle,
}

/// Load `.env.local` from the current working directory or the parent directory
/// so the Rust backend picks up Turso / SMTP configuration.
fn load_env_local() {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(dir) = std::env::current_dir() {
        candidates.push(dir.join(".env.local"));
        if let Some(parent) = dir.parent() {
            candidates.push(parent.join(".env.local"));
        }
    }
    for path in &candidates {
        if path.exists() {
            match dotenvy::from_path(path) {
                Ok(_) => {
                    log::info!("Loaded environment from {}", path.display());
                    return;
                }
                Err(e) => {
                    log::debug!("Could not load {}: {}", path.display(), e);
                }
            }
        }
    }
    if let Err(e) = dotenvy::from_filename(".env.local") {
        log::debug!("No .env.local found in working directory: {}", e);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    load_env_local();
    log::info!("Starting application...");

    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                window.set_min_size(Some(tauri::LogicalSize::new(960.0, 600.0)))?;
            }

            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir).ok();

            let db_path = data_dir.join("app.db");
            let db = Arc::new(
                db::connection::DatabaseConnection::new(db_path.to_str().unwrap_or("app.db"))
                    .expect("Failed to open database"),
            );

            if let Err(e) = db::migrations::run_migrations(&db) {
                log::error!("Migration failed: {}", e);
            }

            commands::users::seed_default_user(&db);

            let turso_url = std::env::var("TURSO_DATABASE_URL").ok();
            let turso_token = std::env::var("TURSO_AUTH_TOKEN").ok();
            let sync = Arc::new(sync::SyncManager::new(
                db.clone(),
                turso_url,
                turso_token,
                Some(app.handle().clone()),
            ));

            {
                let sync_bg = sync.clone();
                tauri::async_runtime::spawn(async move {
                    sync_bg.run_background_sync().await;
                });
            }

            app.manage(AppState { db, sync, app: app.handle().clone() });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            commands::incoming::get_incoming,
            commands::incoming::get_incoming_by_id,
            commands::incoming::create_incoming,
            commands::incoming::update_incoming,
            commands::incoming::delete_incoming,
            commands::incoming::save_incoming_file,
            commands::incoming::download_incoming_file,
            commands::outgoing::get_outgoing,
            commands::outgoing::get_outgoing_by_id,
            commands::outgoing::create_outgoing,
            commands::outgoing::update_outgoing,
            commands::outgoing::delete_outgoing,
            commands::outgoing::save_outgoing_file,
            commands::outgoing::download_outgoing_file,
            commands::outgoing::download_outgoing_file_in,
            commands::users::get_users,
            commands::users::get_user_by_id,
            commands::users::create_user,
            commands::users::update_user,
            commands::users::delete_user,
            commands::users::block_user,
            commands::users::login,
            commands::audit::get_audit_logs,
            commands::sync::sync_push,
            commands::sync::sync_pull,
            commands::sync::sync_full,
            commands::sync::get_sync_status,
            commands::sync::get_sync_arabic_status,
            commands::auth::reset_password_by_pin,
            commands::settings::save_and_open_html,
            commands::settings::get_pin_code,
            commands::settings::set_pin_code,
            commands::settings::verify_pin,
            commands::database::export_database_to_desktop,
            commands::database::import_database_from_pc,
            commands::database::download_filtered_db,
            commands::excel::analyze_excel,
            commands::excel::import_excel,
            commands::excel::generate_excel_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
