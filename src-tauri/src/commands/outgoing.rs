use crate::db::connection::{row_to_outgoing, OUTGOING_COLUMNS};
use crate::db::models::*;
use crate::{db::uploads, AppState};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[tauri::command]
pub fn get_outgoing(
    state: tauri::State<'_, AppState>,
    page: Option<i64>,
    per_page: Option<i64>,
    search: Option<String>,
    exact_date: Option<String>,
) -> Result<PaginatedResult<Outgoing>, String> {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let mut where_parts: Vec<String> = vec!["deleted_at IS NULL".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    let mut n = 0;
    if let Some(s) = search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let q = format!("%{}%", s);
        let a = n + 1;
        where_parts.push(format!(
            "(subject LIKE ?{a} OR recipient LIKE ?{b} OR registration_number LIKE ?{c} OR correspondence_number LIKE ?{d})",
            a = a, b = a + 1, c = a + 2, d = a + 3
        ));
        n += 4;
        for _ in 0..4 {
            params.push(Box::new(q.clone()));
        }
    }
    if let Some(d) = exact_date.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        n += 1;
        where_parts.push(format!("substr(date,1,10) = ?{}", n));
        params.push(Box::new(d.to_string()));
    }

    let where_clause = format!("WHERE {}", where_parts.join(" AND "));

    let count_sql = format!("SELECT COUNT(*) FROM outgoing {}", where_clause);
    let total: i64 = state.db.query_row(&count_sql, &params.iter().map(|p| p.as_ref()).collect::<Vec<_>>(), |r| r.get(0)).map_err(|e| e.to_string())?;

    let query_sql = format!(
        "SELECT {} FROM outgoing {} ORDER BY CAST(registration_number AS INTEGER) DESC, id ASC LIMIT ?{} OFFSET ?{}",
        OUTGOING_COLUMNS,
        where_clause,
        n + 1,
        n + 2,
    );
    let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = params;
    all_params.push(Box::new(per_page));
    all_params.push(Box::new(offset));

    let items = state
        .db
        .query_all(&query_sql, &all_params.iter().map(|p| p.as_ref()).collect::<Vec<_>>(), row_to_outgoing)
        .map_err(|e| e.to_string())?;

    Ok(PaginatedResult::new(items, total, page, per_page))
}

#[tauri::command]
pub fn get_outgoing_by_id(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Outgoing, String> {
    state
        .db
        .query_row(
            &format!("SELECT {} FROM outgoing WHERE id = ?1 AND deleted_at IS NULL", OUTGOING_COLUMNS),
            &[&&*id],
            row_to_outgoing,
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_outgoing(
    state: tauri::State<'_, AppState>,
    data: CreateOutgoingData,
) -> Result<Outgoing, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    state.db.execute(
        "INSERT INTO outgoing (id, registration_number, correspondence_number, date, subject, recipient, destination_service, source, notes, created_at, updated_at, file_name, file_path, file_name_in, file_path_in, sync_version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, 1)",
        &[
            &&*id,
            &&*data.registration_number,
            &data.correspondence_number.as_deref(),
            &&*data.date,
            &&*data.subject,
            &&*data.recipient,
            &&*data.destination_service,
            &data.source.as_deref(),
            &data.notes.as_deref(),
            &&*now,
            &&*now,
            &data.file_name.as_deref(),
            &data.file_path.as_deref(),
            &data.file_name_in.as_deref(),
            &data.file_path_in.as_deref(),
        ],
    ).map_err(|e| e.to_string())?;

    state.db.query_row(
        &format!("SELECT {} FROM outgoing WHERE id = ?1", OUTGOING_COLUMNS),
        &[&&*id],
        row_to_outgoing,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_outgoing(
    state: tauri::State<'_, AppState>,
    id: String,
    data: UpdateOutgoingData,
) -> Result<Outgoing, String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    state.db.execute(
        "UPDATE outgoing SET registration_number=COALESCE(?2, registration_number), correspondence_number=COALESCE(?3, correspondence_number), date=COALESCE(?4, date), subject=COALESCE(?5, subject), recipient=COALESCE(?6, recipient), destination_service=COALESCE(?7, destination_service), source=COALESCE(?8, source), notes=COALESCE(?9, notes), updated_at=?10, file_name=?11, file_path=?12, file_name_in=?13, file_path_in=?14, sync_version=sync_version+1 WHERE id=?1",
        &[
            &&*id,
            &data.registration_number.as_deref(),
            &data.correspondence_number.as_deref(),
            &data.date.as_deref(),
            &data.subject.as_deref(),
            &data.recipient.as_deref(),
            &data.destination_service.as_deref(),
            &data.source.as_deref(),
            &data.notes.as_deref(),
            &&*now,
            &data.file_name.as_deref(),
            &data.file_path.as_deref(),
            &data.file_name_in.as_deref(),
            &data.file_path_in.as_deref(),
        ],
    ).map_err(|e| e.to_string())?;

    state.db.query_row(
        &format!("SELECT {} FROM outgoing WHERE id = ?1", OUTGOING_COLUMNS),
        &[&&*id],
        row_to_outgoing,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_outgoing(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // Remove the attached files (if any) before soft-deleting the row.
    if let Ok((out, in_)) = state.db.query_row(
        "SELECT file_path, file_path_in FROM outgoing WHERE id = ?1",
        &[&&*id],
        |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<String>>(1)?)),
    ) {
        let _ = uploads::delete_attachment(&state.app, out.as_deref());
        let _ = uploads::delete_attachment(&state.app, in_.as_deref());
    }

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    state.db.execute(
        "UPDATE outgoing SET deleted_at = ?1, updated_at = ?1, sync_version = sync_version + 1 WHERE id = ?2",
        &[&&*now, &&*id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn save_outgoing_file(
    app: tauri::AppHandle,
    source_path: String,
) -> Result<OutgoingFileInfo, String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err("الملف المحدد غير موجود".to_string());
    }

    let original_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();
    let stored_name = format!("{}.{}", Uuid::new_v4(), if ext.is_empty() { "bin" } else { &ext });
    let dir = uploads::uploads_dir(&app)?.join(uploads::OUTGOING_KIND);
    fs::create_dir_all(&dir).map_err(|e| format!("فشل في إنشاء مجلد الملفات: {}", e))?;
    let stored_file = dir.join(&stored_name);

    fs::copy(&source, &stored_file).map_err(|e| format!("فشل في حفظ الملف: {}", e))?;

    Ok(OutgoingFileInfo {
        file_name: original_name,
        file_path: uploads::relative_path(uploads::OUTGOING_KIND, &stored_name),
    })
}

#[tauri::command]
pub fn download_outgoing_file(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let outgoing: Outgoing = state
        .db
        .query_row(
            &format!("SELECT {} FROM outgoing WHERE id = ?1 AND deleted_at IS NULL", OUTGOING_COLUMNS),
            &[&&*id],
            row_to_outgoing,
        )
        .map_err(|e| format!("لم يتم العثور على السجل: {}", e))?;

    let file_path = outgoing
        .file_path
        .ok_or_else(|| "لا يوجد ملف مرفق لهذا السجل".to_string())?;
    let stored = uploads::resolve_absolute(&state.app, &file_path);
    copy_to_downloads(&stored, outgoing.file_name.clone())
}

#[tauri::command]
pub fn download_outgoing_file_in(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let outgoing: Outgoing = state
        .db
        .query_row(
            &format!("SELECT {} FROM outgoing WHERE id = ?1 AND deleted_at IS NULL", OUTGOING_COLUMNS),
            &[&&*id],
            row_to_outgoing,
        )
        .map_err(|e| format!("لم يتم العثور على السجل: {}", e))?;

    let file_path = outgoing
        .file_path_in
        .ok_or_else(|| "لا يوجد ملف مرفق لهذا السجل".to_string())?;
    let stored = uploads::resolve_absolute(&state.app, &file_path);
    copy_to_downloads(&stored, outgoing.file_name_in.clone())
}

fn copy_to_downloads(stored: &PathBuf, name: Option<String>) -> Result<String, String> {
    if !stored.exists() {
        return Err("الملف المرفق غير موجود على القرص".to_string());
    }

    let file_name = if let Some(n) = name {
        n
    } else {
        stored
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string()
    };

    let downloads = dirs_or_downloads();
    fs::create_dir_all(&downloads).map_err(|e| format!("فشل في إنشاء مجلد التنزيلات: {}", e))?;
    let final_dest = downloads.join(&file_name);

    // Overwrite an existing file so each download produces a single copy.
    fs::copy(stored, &final_dest).map_err(|e| format!("فشل في تنزيل الملف: {}", e))?;

    let out = final_dest.to_string_lossy().to_string();
    let out_clone = out.clone();
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", &out_clone])
        .spawn();

    Ok(out)
}

fn dirs_or_downloads() -> PathBuf {
    if let Some(downloads) = dirs::download_dir() {
        return downloads;
    }
    if let Some(home) = dirs::home_dir() {
        return home.join("Downloads");
    }
    std::env::temp_dir()
}
