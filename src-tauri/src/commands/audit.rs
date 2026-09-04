use crate::db::connection::{row_to_audit_log, AUDIT_COLUMNS};
use crate::db::models::*;
use crate::AppState;
use chrono::Utc;
use uuid::Uuid;

#[tauri::command]
pub fn get_audit_logs(
    state: tauri::State<'_, AppState>,
    page: Option<i64>,
    per_page: Option<i64>,
    entity: Option<String>,
) -> Result<PaginatedResult<AuditLog>, String> {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (where_clause, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match &entity {
        Some(e) if !e.is_empty() => (
            "WHERE entity = ?1".to_string(),
            vec![Box::new(e.clone())],
        ),
        _ => (String::new(), vec![]),
    };

    let count_sql = format!("SELECT COUNT(*) FROM audit_logs {}", where_clause);
    let total: i64 = state.db.query_row(&count_sql, &params.iter().map(|p| p.as_ref()).collect::<Vec<_>>(), |r| r.get(0)).map_err(|e| e.to_string())?;

    let query_sql = format!(
        "SELECT {} FROM audit_logs {} ORDER BY timestamp DESC LIMIT ?{} OFFSET ?{}",
        AUDIT_COLUMNS,
        where_clause,
        params.len() + 1,
        params.len() + 2,
    );
    let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = params;
    all_params.push(Box::new(per_page));
    all_params.push(Box::new(offset));

    let items = state
        .db
        .query_all(&query_sql, &all_params.iter().map(|p| p.as_ref()).collect::<Vec<_>>(), row_to_audit_log)
        .map_err(|e| e.to_string())?;

    Ok(PaginatedResult::new(items, total, page, per_page))
}

#[allow(dead_code)]
pub fn log_action(
    db: &crate::db::connection::DatabaseConnection,
    user_id: Option<&str>,
    action: &str,
    entity: &str,
    entity_id: &str,
    metadata: Option<&str>,
) -> Result<(), String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    db.execute(
        "INSERT INTO audit_logs (id, user_id, action, entity, entity_id, timestamp, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        &[
            &&*id,
            &user_id,
            &action,
            &entity,
            &entity_id,
            &&*now,
            &metadata,
        ],
    ).map_err(|e| e.to_string())?;
    Ok(())
}
