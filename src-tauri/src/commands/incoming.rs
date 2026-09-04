use crate::db::connection::{row_to_incoming, INCOMING_COLUMNS};
use crate::db::models::*;
use crate::{db::uploads, AppState};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[tauri::command]
pub fn get_incoming(
    state: tauri::State<'_, AppState>,
    page: Option<i64>,
    per_page: Option<i64>,
    search: Option<String>,
    exact_date: Option<String>,
) -> Result<PaginatedResult<Incoming>, String> {
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
            "(subject LIKE ?{a} OR sender LIKE ?{b} OR registration_number LIKE ?{c} OR correspondence_number LIKE ?{d})",
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

    let count_sql = format!("SELECT COUNT(*) FROM incoming {}", where_clause);
    let total: i64 = state.db.query_row(&count_sql, &params.iter().map(|p| p.as_ref()).collect::<Vec<_>>(), |r| r.get(0)).map_err(|e| e.to_string())?;

    let query_sql = format!(
        "SELECT {} FROM incoming {} ORDER BY CAST(registration_number AS INTEGER) DESC, id ASC LIMIT ?{} OFFSET ?{}",
        INCOMING_COLUMNS,
        where_clause,
        n + 1,
        n + 2,
    );
    let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = params;
    all_params.push(Box::new(per_page));
    all_params.push(Box::new(offset));

    let items = state
        .db
        .query_all(&query_sql, &all_params.iter().map(|p| p.as_ref()).collect::<Vec<_>>(), row_to_incoming)
        .map_err(|e| e.to_string())?;

    Ok(PaginatedResult::new(items, total, page, per_page))
}

#[tauri::command]
pub fn get_incoming_by_id(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Incoming, String> {
    state
        .db
        .query_row(
            &format!("SELECT {} FROM incoming WHERE id = ?1 AND deleted_at IS NULL", INCOMING_COLUMNS),
            &[&&*id],
            row_to_incoming,
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_incoming(
    state: tauri::State<'_, AppState>,
    data: CreateIncomingData,
) -> Result<Incoming, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    state.db.execute(
        "INSERT INTO incoming (id, registration_number, correspondence_number, date, arrival_date, subject, sender, destination_service, source, notes, created_at, updated_at, file_name, file_path, is_duplicate, sync_version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, 1)",
        &[
            &&*id,
            &&*data.registration_number,
            &data.correspondence_number.as_deref(),
            &&*data.date,
            &data.arrival_date.as_deref(),
            &&*data.subject,
            &&*data.sender,
            &&*data.destination_service,
            &data.source.as_deref(),
            &data.notes.as_deref(),
            &&*now,
            &&*now,
            &data.file_name.as_deref(),
            &data.file_path.as_deref(),
            &data.is_duplicate.unwrap_or(false),
        ],
    ).map_err(|e| e.to_string())?;

    state.db.query_row(
        &format!("SELECT {} FROM incoming WHERE id = ?1", INCOMING_COLUMNS),
        &[&&*id],
        row_to_incoming,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_incoming(
    state: tauri::State<'_, AppState>,
    id: String,
    data: UpdateIncomingData,
) -> Result<Incoming, String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    state.db.execute(
        "UPDATE incoming SET registration_number=COALESCE(?2, registration_number), correspondence_number=COALESCE(?3, correspondence_number), date=COALESCE(?4, date), arrival_date=COALESCE(?5, arrival_date), subject=COALESCE(?6, subject), sender=COALESCE(?7, sender), destination_service=COALESCE(?8, destination_service), source=COALESCE(?9, source), notes=COALESCE(?10, notes), file_name=COALESCE(?11, file_name), file_path=COALESCE(?12, file_path), is_duplicate=COALESCE(?14, is_duplicate), updated_at=?13, sync_version=sync_version+1 WHERE id=?1",
        &[
            &&*id,
            &data.registration_number.as_deref(),
            &data.correspondence_number.as_deref(),
            &data.date.as_deref(),
            &data.arrival_date.as_deref(),
            &data.subject.as_deref(),
            &data.sender.as_deref(),
            &data.destination_service.as_deref(),
            &data.source.as_deref(),
            &data.notes.as_deref(),
            &data.file_name.as_deref(),
            &data.file_path.as_deref(),
            &&*now,
            &data.is_duplicate,
        ],
    ).map_err(|e| e.to_string())?;

    state.db.query_row(
        &format!("SELECT {} FROM incoming WHERE id = ?1", INCOMING_COLUMNS),
        &[&&*id],
        row_to_incoming,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_incoming(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // Remove the attached file (if any) before soft-deleting the row.
    if let Ok(path) = state.db.query_row(
        "SELECT file_path FROM incoming WHERE id = ?1",
        &[&&*id],
        |r| r.get::<_, Option<String>>(0),
    ) {
        let _ = uploads::delete_attachment(&state.app, path.as_deref());
    }

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    state.db.execute(
        "UPDATE incoming SET deleted_at = ?1, updated_at = ?1, sync_version = sync_version + 1 WHERE id = ?2",
        &[&&*now, &&*id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn save_incoming_file(
    app: tauri::AppHandle,
    source_path: String,
) -> Result<IncomingFileInfo, String> {
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
    let dir = uploads::uploads_dir(&app)?.join(uploads::INCOMING_KIND);
    fs::create_dir_all(&dir).map_err(|e| format!("فشل في إنشاء مجلد الملفات: {}", e))?;
    let stored_file = dir.join(&stored_name);

    fs::copy(&source, &stored_file).map_err(|e| format!("فشل في حفظ الملف: {}", e))?;

    Ok(IncomingFileInfo {
        file_name: original_name,
        file_path: uploads::relative_path(uploads::INCOMING_KIND, &stored_name),
    })
}

#[tauri::command]
pub fn download_incoming_file(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let incoming: Incoming = state
        .db
        .query_row(
            &format!("SELECT {} FROM incoming WHERE id = ?1 AND deleted_at IS NULL", INCOMING_COLUMNS),
            &[&&*id],
            row_to_incoming,
        )
        .map_err(|e| format!("لم يتم العثور على السجل: {}", e))?;

    let file_path = incoming
        .file_path
        .ok_or_else(|| "لا يوجد ملف مرفق لهذا السجل".to_string())?;
    let stored = uploads::resolve_absolute(&state.app, &file_path);
    if !stored.exists() {
        return Err("الملف المرفق غير موجود على القرص".to_string());
    }

    let file_name = if incoming.file_name.is_some() {
        incoming.file_name.clone().unwrap()
    } else {
        stored
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string()
    };

    let downloads = dirs_or_downloads();
    fs::create_dir_all(&downloads).map_err(|e| format!("فشل في إنشاء مجلد التنزيلات: {}", e))?;

    // Overwrite an existing file so each download produces a single copy.
    let final_dest = downloads.join(&file_name);
    fs::copy(&stored, &final_dest).map_err(|e| format!("فشل في تنزيل الملف: {}", e))?;

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
