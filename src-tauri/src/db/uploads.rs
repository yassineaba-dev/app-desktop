use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

/// Root folder that holds the `incoming` and `outgoing` attachment folders.
pub fn uploads_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("فشل في تحديد مجلد التطبيق: {}", e))?;
    let dir = data_dir.join("uploads");
    fs::create_dir_all(&dir).map_err(|e| format!("فشل في إنشاء مجلد الملفات: {}", e))?;
    Ok(dir)
}

/// A relative path stored in the DB (relative to the uploads root), e.g.
/// `incoming/<uuid>.pdf`. Keeps the DB portable between machines.
pub type UploadKind = &'static str;

pub const INCOMING_KIND: UploadKind = "incoming";
pub const OUTGOING_KIND: UploadKind = "outgoing";

/// Build the relative storage path for a new attachment.
pub fn relative_path(kind: UploadKind, stored_name: &str) -> String {
    format!("{}/{}", kind, stored_name)
}

/// Turn a possibly-relative DB path into an absolute filesystem path.
/// Backwards-compatible: absolute paths already stored are returned as-is.
pub fn resolve_absolute(app: &tauri::AppHandle, stored: &str) -> PathBuf {
    let p = PathBuf::from(stored);
    if p.is_absolute() {
        return p;
    }
    uploads_dir(app)
        .map(|root| root.join(&p))
        .unwrap_or_else(|_| p)
}

/// Delete an attachment file from disk given the DB-stored path (relative or
/// absolute). Returns true if a file was actually removed. Safe no-op if the
/// file does not exist; never reaches outside the uploads root for relative
/// paths (relative paths are only resolved within the uploads folder).
pub fn delete_attachment(app: &tauri::AppHandle, stored: Option<&str>) -> bool {
    let stored = match stored {
        Some(s) if !s.trim().is_empty() => s,
        _ => return false,
    };
    let p = PathBuf::from(stored);
    let target = if p.is_absolute() {
        p
    } else {
        let Some(root) = uploads_dir(app).ok() else {
            return false;
        };
        root.join(&p)
    };
    if target.is_file() {
        let _ = fs::remove_file(&target);
        return true;
    }
    false
}

/// Copy all attachment files referenced by a table's rows (via the given path
/// columns) into `target_uploads/<kind>/`, optionally grouping by a column
/// (e.g. `registration_number`) so each row's files live in their own
/// subfolder. Rewrites DB column values to the matching relative paths.
///
/// `kind_cols` maps schema column name -> target folder kind.
/// `group_col` when `Some("registration_number")` creates per-row subfolders.
pub fn relocate_files_in_table(
    conn: &mut Connection,
    table: &str,
    path_cols: &[(&str, UploadKind)],
    source_uploads: &Path,
    target_uploads: &Path,
    group_col: Option<&str>,
) -> Result<usize, String> {
    if path_cols.is_empty() {
        return Ok(0);
    }

    let mut copied = 0usize;

    let has_id = table_has_column(conn, table, "id")?;
    let has_group = group_col
        .map(|c| table_has_column(conn, table, c).unwrap_or(false))
        .unwrap_or(false);

    for &(col, kind) in path_cols {
        if !table_has_column(conn, table, col)? {
            continue;
        }

        let select_sql = match (has_id, has_group) {
            (true, true) => format!(
                "SELECT id, {}, {} FROM {} WHERE {} IS NOT NULL",
                group_col.unwrap(), col, table, col
            ),
            (true, false) => format!(
                "SELECT id, {} FROM {} WHERE {} IS NOT NULL",
                col, table, col
            ),
            (false, true) => format!(
                "SELECT {}, {} FROM {} WHERE {} IS NOT NULL",
                group_col.unwrap(), col, table, col
            ),
            (false, false) => format!(
                "SELECT {} FROM {} WHERE {} IS NOT NULL",
                col, table, col
            ),
        };

        let rows: Vec<(String, String, String)> = {
            let mut stmt = conn
                .prepare(&select_sql)
                .map_err(|e| format!("فشل قراءة جدول {}: {}", table, e))?;
            let mut found = Vec::new();
            let iter = stmt
                .query_map([], |row| {
                    let mut idx = 0;
                    let id: String = if has_id {
                        let v = row.get(idx)?;
                        idx += 1;
                        v
                    } else {
                        String::new()
                    };
                    let group: String = if has_group {
                        let v = row.get(idx)?;
                        idx += 1;
                        v
                    } else {
                        String::new()
                    };
                    let path: String = row.get(idx)?;
                    Ok((id, group, path))
                })
                .map_err(|e| format!("فشل قراءة جدول {}: {}", table, e))?;
            for r in iter {
                found.push(r.map_err(|e| format!("فشل قراءة جدول {}: {}", table, e))?);
            }
            found
        };

        for (id, group_val, db_path) in rows {
            let absolute = {
                let p = PathBuf::from(&db_path);
                if p.is_absolute() {
                    p
                } else {
                    source_uploads.join(&db_path)
                }
            };
            if !absolute.exists() {
                continue;
            }

            // Build target directory: kind/ or kind/<group>/
            let target_kind_dir = if has_group && !group_val.trim().is_empty() {
                let dir = target_uploads.join(kind).join(group_val.trim());
                fs::create_dir_all(&dir)
                    .map_err(|e| format!("فشل في إنشاء مجلد {}: {}", kind, e))?;
                dir
            } else {
                let dir = target_uploads.join(kind);
                fs::create_dir_all(&dir)
                    .map_err(|e| format!("فشل في إنشاء مجلد {}: {}", kind, e))?;
                dir
            };

            let file_name = absolute
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            // Avoid overwriting same-named files from different rows.
            let mut out_name = file_name.to_string();
            if target_kind_dir.join(&out_name).exists() {
                let stem = Path::new(&out_name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("file");
                let ext = Path::new(&out_name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                out_name = if ext.is_empty() {
                    format!("{}_{}", stem, id)
                } else {
                    format!("{}_{}.{}", stem, id, ext)
                };
            }
            let dest_file = target_kind_dir.join(&out_name);
            if let Ok(_) = fs::copy(&absolute, &dest_file) {
                copied += 1;
                // Relative path: kind/<group>/file  or  kind/file
                let rel = if has_group && !group_val.trim().is_empty() {
                    format!("{}/{}/{}", kind, group_val.trim(), out_name)
                } else {
                    relative_path(kind, &out_name)
                };
                if has_id {
                    conn.execute(
                        &format!("UPDATE {} SET {} = ?1 WHERE id = ?2", table, col),
                        rusqlite::params![rel, id],
                    )
                    .map_err(|e| format!("فشل تحديث مسار الملف: {}", e))?;
                }
            }
        }
    }

    Ok(copied)
}

/// Create the `uploads` directory (empty) inside a target folder.
pub fn prepare_uploads_dir(target_folder: &Path) -> Result<PathBuf, String> {
    let up = target_folder.join("uploads");
    fs::create_dir_all(&up).map_err(|e| format!("فشل في إنشاء مجلد uploads: {}", e))?;
    Ok(up)
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", table))
        .map_err(|e| format!("فشل قراءة بنية جدول {}: {}", table, e))?;
    let cols = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("فشل قراءة بنية جدول {}: {}", table, e))?;
    for c in cols {
        if let Ok(name) = c {
            if name == column {
                return Ok(true);
            }
        }
    }
    Ok(false)
}