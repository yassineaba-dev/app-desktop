use crate::AppState;
use chrono::Local;
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::PathBuf;

/// Copy the current application database plus its attached files to the
/// user's Desktop as a backup folder containing `data.db` and `uploads/`.
#[tauri::command]
pub fn export_database_to_desktop(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    // Flush WAL data into the main DB file so the copy is complete and consistent.
    let _ = state
        .db
        .execute("PRAGMA wal_checkpoint(TRUNCATE)", &[]);

    let db_path = state.db.path().to_path_buf();
    if !db_path.exists() {
        return Err("قاعدة البيانات غير موجودة".to_string());
    }

    let desktop = dirs::desktop_dir().ok_or_else(|| "تعذر تحديد مجلد سطح المكتب".to_string())?;
    fs::create_dir_all(&desktop).map_err(|e| format!("تعذر إنشاء مجلد سطح المكتب: {}", e))?;

    let stamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let mut folder = desktop.join(format!("قاعدة_البيانات_{}", stamp));
    let mut counter = 1;
    while folder.exists() {
        folder = desktop.join(format!("قاعدة_البيانات_{}_{}", stamp, counter));
        counter += 1;
    }
    fs::create_dir_all(&folder).map_err(|e| format!("فشل في إنشاء المجلد: {}", e))?;

    let dest_db = folder.join("data.db");
    fs::copy(&db_path, &dest_db).map_err(|e| format!("فشل نسخ قاعدة البيانات: {}", e))?;

    // Copy the attached files so the backup is fully portable.
    if let Ok(src_uploads) = crate::db::uploads::uploads_dir(&state.app) {
        let target_uploads = crate::db::uploads::prepare_uploads_dir(&folder)?;
        if let Ok(mut out) = rusqlite::Connection::open(&dest_db) {
            let _ = crate::db::uploads::relocate_files_in_table(
                &mut out,
                "incoming",
                &[("file_path", crate::db::uploads::INCOMING_KIND)],
                &src_uploads,
                &target_uploads,
                Some("registration_number"),
            );
            let _ = crate::db::uploads::relocate_files_in_table(
                &mut out,
                "outgoing",
                &[
                    ("file_path", crate::db::uploads::OUTGOING_KIND),
                    ("file_path_in", crate::db::uploads::OUTGOING_KIND),
                ],
                &src_uploads,
                &target_uploads,
                Some("registration_number"),
            );
        }
    }

    let out = folder.to_string_lossy().to_string();

    // Open Explorer with the folder selected so the user sees it, without
    // flashing a terminal window.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("explorer.exe")
            .args(["/select,", &out])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn();
    }

    Ok(out)
}

/// Replace the current database contents with data from a database the user
/// selects from their PC, without closing or restarting the application.
/// Accepts either a `.db` file or an export folder containing `data.db` and an
/// `uploads/` folder with attached files (which are copied into the app).
#[tauri::command]
pub fn import_database_from_pc(
    state: tauri::State<'_, AppState>,
    source_path: String,
) -> Result<usize, String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err("الملف المحدد غير موجود".to_string());
    }

    // If the user selected a folder (export format), locate data.db inside and
    // remember any sibling uploads/ folder for later file import.
    let (db_source, uploads_source) = if source.is_dir() {
        let db = source.join("data.db");
        if !db.exists() {
            return Err("لا يوجد ملف data.db داخل المجلد المحدد".to_string());
        }
        let up = source.join("uploads");
        let up = if up.exists() { Some(up) } else { None };
        (db, up)
    } else {
        // A bare .db file selected: no bundled files.
        (source, None)
    };

    // Basic validation: must be a readable, non-empty file.
    let meta = fs::metadata(&db_source).map_err(|e| format!("تعذر قراءة الملف: {}", e))?;
    if meta.len() == 0 {
        return Err("الملف المحدد فارغ".to_string());
    }

    // Copy bundled attachment files into the app's uploads directory (relative
    // paths are already aligned). Do this before importing so the file names
    // exist; the DB rows reference the same relative locations.
    if let Some(up) = uploads_source {
        let local_uploads = crate::db::uploads::uploads_dir(&state.app)?;
        merge_uploads_dir(&up, &local_uploads);
    }

    let imported = state
        .db
        .import_from(db_source.to_str().ok_or("مسار غير صالح")?)?;
    if imported == 0 {
        return Err("لم يتم العثور على بيانات في الملف المحدد".to_string());
    }

    log::info!("Imported {} records from {}", imported, source_path);
    Ok(imported)
}

/// Recursively copy `src` into `dst` (dst/uploads/incoming, dst/uploads/outgoing
/// merge into the local uploads folders, including per-registration_number
/// subfolders). Files that already exist keep the existing copy.
fn merge_uploads_dir(src: &std::path::Path, dst: &std::path::Path) {
    for kind in ["incoming", "outgoing"] {
        let s = src.join(kind);
        let d = dst.join(kind);
        if !s.exists() {
            continue;
        }
        fs::create_dir_all(&d).ok();
        if let Ok(rd) = fs::read_dir(&s) {
            for entry in rd.flatten() {
                let fpath = entry.path();
                if fpath.is_dir() {
                    // Subfolder (e.g. registration_number) — copy recursively.
                    let sub = d.join(entry.file_name());
                    fs::create_dir_all(&sub).ok();
                    if let Ok(sub_rd) = fs::read_dir(&fpath) {
                        for sub_entry in sub_rd.flatten() {
                            let sub_file = sub_entry.path();
                            if sub_file.is_file() {
                                let target = sub.join(sub_entry.file_name());
                                if !target.exists() {
                                    let _ = fs::copy(&sub_file, &target);
                                }
                            }
                        }
                    }
                } else if fpath.is_file() {
                    let name = entry.file_name();
                    let target = d.join(&name);
                    if !target.exists() {
                        let _ = fs::copy(&fpath, &target);
                    }
                }
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FilteredExportRequest {
    #[serde(default)]
    pub incoming: bool,
    #[serde(default)]
    pub outgoing: bool,
    #[serde(default)]
    pub incoming_search: Option<String>,
    #[serde(default)]
    pub incoming_date: Option<String>,
    #[serde(default)]
    pub outgoing_search: Option<String>,
    #[serde(default)]
    pub outgoing_date: Option<String>,
}

fn build_where(
    search: &Option<String>,
    date: &Option<String>,
    params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
    is_outgoing: bool,
) -> String {
    let mut parts: Vec<String> = vec!["deleted_at IS NULL".to_string()];
    let mut n = 0;
    if let Some(s) = search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let q = format!("%{}%", s);
        if is_outgoing {
            let a = n + 1;
            parts.push(format!(
                "(subject LIKE ?{a} OR recipient LIKE ?{b} OR registration_number LIKE ?{c} OR correspondence_number LIKE ?{d})",
                a = a, b = a + 1, c = a + 2, d = a + 3
            ));
            for _ in 0..4 {
                params.push(Box::new(q.clone()));
            }
            n += 4;
        } else {
            let a = n + 1;
            parts.push(format!(
                "(subject LIKE ?{a} OR sender LIKE ?{b} OR registration_number LIKE ?{c} OR correspondence_number LIKE ?{d})",
                a = a, b = a + 1, c = a + 2, d = a + 3
            ));
            for _ in 0..4 {
                params.push(Box::new(q.clone()));
            }
            n += 4;
        }
    }
    if let Some(d) = date.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        n += 1;
        parts.push(format!("substr(date,1,10) = ?{}", n));
        params.push(Box::new(d.to_string()));
    }

    format!(" WHERE {}", parts.join(" AND "))
}

/// Build a standalone folder containing a filtered database plus the attached
/// files of the filtered records, and save it to the user's Downloads folder,
/// then reveal it in Explorer.
#[tauri::command]
pub fn download_filtered_db(
    state: tauri::State<'_, AppState>,
    request: FilteredExportRequest,
) -> Result<String, String> {
    if !request.incoming && !request.outgoing {
        return Err("حدد قاعدة واحدة على الأقل (الواردات أو الصادرات)".to_string());
    }

    let stamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    let downloads = dirs_or_downloads()
        .ok_or_else(|| "تعذر تحديد مجلد التنزيلات".to_string())?;
    fs::create_dir_all(&downloads).map_err(|e| format!("فشل في إنشاء مجلد التنزيلات: {}", e))?;

    let mut folder = downloads.join(format!("بيانات_مفلترة_{}", stamp));
    let mut counter = 1;
    while folder.exists() {
        folder = downloads.join(format!("بيانات_مفلترة_{}_{}", stamp, counter));
        counter += 1;
    }
    fs::create_dir_all(&folder).map_err(|e| format!("فشل في إنشاء المجلد: {}", e))?;

    let db_file = folder.join("data.db");
    if db_file.exists() {
        fs::remove_file(&db_file).ok();
    }

    let mut out = Connection::open_with_flags(
        &db_file,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
    )
    .map_err(|e| format!("فشل في إنشاء قاعدة البيانات: {}", e))?;

    out.execute_batch(
        "CREATE TABLE incoming (id TEXT PRIMARY KEY NOT NULL, registration_number TEXT NOT NULL, correspondence_number TEXT, date TEXT NOT NULL, arrival_date TEXT, subject TEXT NOT NULL, sender TEXT NOT NULL, destination_service TEXT NOT NULL, source TEXT, notes TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT, created_by TEXT, sync_version INTEGER NOT NULL DEFAULT 1, file_name TEXT, file_path TEXT, is_duplicate INTEGER NOT NULL DEFAULT 0);
         CREATE TABLE outgoing (id TEXT PRIMARY KEY NOT NULL, registration_number TEXT NOT NULL, correspondence_number TEXT, date TEXT NOT NULL, subject TEXT NOT NULL, recipient TEXT NOT NULL, destination_service TEXT NOT NULL, source TEXT, notes TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT, created_by TEXT, sync_version INTEGER NOT NULL DEFAULT 1, file_name TEXT, file_path TEXT, file_name_in TEXT, file_path_in TEXT);
         CREATE TABLE settings (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL DEFAULT '', updated_at TEXT NOT NULL DEFAULT (datetime('now')));
         CREATE TABLE sync_metadata (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL DEFAULT '', updated_at TEXT NOT NULL DEFAULT (datetime('now')));",
    )
    .map_err(|e| format!("فشل في إنشاء جداول قاعدة البيانات: {}", e))?;

    // Attach the source database so we can SELECT from it while INSERTing
    // into the new destination database.
    let src_path = state.db.path().to_string_lossy();
    out.execute_batch(&format!(
        "ATTACH DATABASE '{}' AS src",
        src_path.replace('\'', "''")
    ))
    .map_err(|e| format!("فشل ربط قاعدة البيانات المصدر: {}", e))?;

    if request.incoming {
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let where_clause = build_where(&request.incoming_search, &request.incoming_date, &mut params, false);
        let sql = format!(
            "INSERT INTO incoming ({}) SELECT {} FROM src.incoming{}",
            crate::db::connection::INCOMING_COLUMNS,
            crate::db::connection::INCOMING_COLUMNS,
            where_clause
        );
        let p: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        out.execute(&sql, rusqlite::params_from_iter(p.iter()))
            .map_err(|e| format!("فشل في نسخ الواردات: {}", e))?;
    }

    if request.outgoing {
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let where_clause = build_where(&request.outgoing_search, &request.outgoing_date, &mut params, true);
        let sql = format!(
            "INSERT INTO outgoing ({}) SELECT {} FROM src.outgoing{}",
            crate::db::connection::OUTGOING_COLUMNS,
            crate::db::connection::OUTGOING_COLUMNS,
            where_clause
        );
        let p: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        out.execute(&sql, rusqlite::params_from_iter(p.iter()))
            .map_err(|e| format!("فشل في نسخ الصادرات: {}", e))?;
    }

    out.execute_batch("DETACH DATABASE src").ok();

    let _ = out.execute(
        "INSERT INTO settings (key, value) VALUES ('pin_code', ?1)",
        &[&get_setting_value(&state, "pin_code")],
    );

    // Copy the attached files referenced by the filtered rows into the
    // folder's `uploads` directory and rewrite the DB paths so they stay valid
    // when the folder is moved or imported on another machine.
    let local_uploads = crate::db::uploads::uploads_dir(&state.app)?;
    let target_uploads = crate::db::uploads::prepare_uploads_dir(&folder)?;

    if request.incoming {
        crate::db::uploads::relocate_files_in_table(
            &mut out,
            "incoming",
            &[("file_path", crate::db::uploads::INCOMING_KIND)],
            &local_uploads,
            &target_uploads,
            Some("registration_number"),
        )?;
    }
    if request.outgoing {
        crate::db::uploads::relocate_files_in_table(
            &mut out,
            "outgoing",
            &[
                ("file_path", crate::db::uploads::OUTGOING_KIND),
                ("file_path_in", crate::db::uploads::OUTGOING_KIND),
            ],
            &local_uploads,
            &target_uploads,
            Some("registration_number"),
        )?;
    }

    drop(out);

    // Reveal the folder in Explorer without flashing a terminal window.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("explorer.exe")
            .args(["/select,", &folder.to_string_lossy()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn();
    }

    let out_path = folder.to_string_lossy().to_string();
    log::info!("Filtered database saved to {}", out_path);
    Ok(out_path)
}

fn get_setting_value(state: &tauri::State<'_, AppState>, key: &str) -> String {
    state
        .db
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            &[&key],
            |row| row.get(0),
        )
        .unwrap_or_default()
}

fn dirs_or_downloads() -> Option<PathBuf> {
    if let Some(downloads) = dirs::download_dir() {
        return Some(downloads);
    }
    dirs::home_dir().map(|h| h.join("Downloads"))
}