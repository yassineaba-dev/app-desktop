use crate::AppState;
use chrono::Utc;
use bcrypt;

fn user_err() -> String { "\u{0627}\u{0644}\u{0628}\u{0631}\u{064a}\u{062f} \u{0627}\u{0644}\u{0625}\u{0644}\u{0643}\u{062a}\u{0631}\u{0648}\u{0646}\u{064a} \u{063a}\u{064a}\u{0631} \u{0645}\u{0633}\u{062c}\u{0644} \u{0641}\u{064a} \u{0627}\u{0644}\u{0646}\u{0638}\u{0627}\u{0645}".to_string() }
fn ok_reset() -> String { "\u{062a}\u{0645} \u{0625}\u{0639}\u{0627}\u{062f}\u{0629} \u{062a}\u{0639}\u{064a}\u{064a}\u{0646} \u{0643}\u{0644}\u{0645}\u{0629} \u{0627}\u{0644}\u{0645}\u{0631}\u{0648}\u{0631} \u{0628}\u{0646}\u{062c}\u{0627}\u{062d}".to_string() }

fn check_user_exists(db: &crate::db::connection::DatabaseConnection, email: &str) -> bool {
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM users WHERE LOWER(email) = LOWER(?1) AND deleted_at IS NULL",
        &[&&*email],
        |row| row.get(0),
    ).unwrap_or(0);
    count > 0
}

#[tauri::command]
pub fn reset_password_by_pin(state: tauri::State<'_, AppState>, email: String, pin: String, new_password: String) -> Result<String, String> {
    if !check_user_exists(&state.db, &email) {
        return Err(user_err());
    }

    let stored_pin: Option<String> = state
        .db
        .query_row(
            "SELECT value FROM settings WHERE key = 'pin_code'",
            &[],
            |row| row.get(0),
        )
        .ok();
    match stored_pin {
        Some(p) if !p.is_empty() && p == pin => {}
        Some(_) => return Err("رمز PIN غير صحيح".to_string()),
        None => return Err("لم يتم تعيين رمز PIN. يرجى تعيينه من الإعدادات.".to_string()),
    }

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let hash = bcrypt::hash(&new_password, 10).map_err(|e| format!("Hash failed: {}", e))?;
    state.db.execute(
        "UPDATE users SET password_hash = ?1, updated_at = ?2, sync_version = sync_version + 1 WHERE LOWER(email) = LOWER(?3) AND deleted_at IS NULL",
        &[&hash, &now, &&*email],
    ).map_err(|e| e.to_string())?;

    Ok(ok_reset())
}
