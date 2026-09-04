use crate::AppState;
use std::fs;

fn get_setting(db: &crate::db::connection::DatabaseConnection, key: &str) -> Result<Option<String>, String> {
    db.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        &[&key],
        |row| row.get(0),
    )
    .map(Some)
    .or_else(|e| {
        if e == rusqlite::Error::QueryReturnedNoRows {
            Ok(None)
        } else {
            Err(e.to_string())
        }
    })
}

fn set_setting(db: &crate::db::connection::DatabaseConnection, key: &str, value: &str) -> Result<(), String> {
    db.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
        &[&key, &value],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Save a generated HTML document to the temp folder and open it in the
/// default browser. Runs the shell command with no visible console window.
#[tauri::command]
pub fn save_and_open_html(filename: String, content: String) -> Result<(), String> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(&filename);
    fs::write(&file_path, &content).map_err(|e| format!("فشل في حفظ الملف: {}", e))?;

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .args(["/C", "start", "", file_path.to_str().unwrap_or("")])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("فشل في فتح الملف: {}", e))?;
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", file_path.to_str().unwrap_or("")])
            .spawn()
            .map_err(|e| format!("فشل في فتح الملف: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_pin_code(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(get_setting(&state.db, "pin_code")?.unwrap_or_default())
}

#[tauri::command]
pub fn set_pin_code(state: tauri::State<'_, AppState>, pin: String) -> Result<(), String> {
    set_setting(&state.db, "pin_code", &pin)
}

#[tauri::command]
pub fn verify_pin(state: tauri::State<'_, AppState>, pin: String) -> Result<(), String> {
    match get_setting(&state.db, "pin_code")? {
        None => Err("لم يتم تعيين رمز PIN. يرجى تعيينه من الإعدادات.".to_string()),
        Some(stored) if stored.is_empty() => {
            Err("لم يتم تعيين رمز PIN. يرجى تعيينه من الإعدادات.".to_string())
        }
        Some(stored) if stored == pin => Ok(()),
        Some(_) => Err("رمز PIN غير صحيح".to_string()),
    }
}