use crate::db::connection::{row_to_user, row_to_user_internal, USER_COLUMNS, USER_INTERNAL_COLUMNS};
use crate::db::models::*;
use crate::AppState;
use chrono::Utc;
use uuid::Uuid;

pub fn seed_default_user(db: &crate::db::connection::DatabaseConnection) {
    let count: i64 = db
        .query_row("SELECT COUNT(*) FROM users", &[], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        return;
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let hash = bcrypt::hash("Admin@123", 10).expect("Failed to hash default password");

    let result = db.execute(
        "INSERT INTO users (id, full_name, email, role, password_hash, created_at, updated_at, sync_version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
        &[
            &&*id,
            &"المدير العام",
            &"dpj.archive@gmail.com",
            &"admin",
            &&*hash,
            &&*now,
            &&*now,
        ],
    );

    match result {
        Ok(_) => log::info!("Default user seeded: dpj.archive@gmail.com"),
        Err(e) => log::error!("Failed to seed default user: {}", e),
    }
}

#[tauri::command]
pub fn login(
    state: tauri::State<'_, AppState>,
    data: LoginData,
) -> Result<LoginResponse, String> {
    let internal = state
        .db
        .query_row(
            &format!(
                "SELECT {} FROM users WHERE email = ?1 AND deleted_at IS NULL",
                USER_INTERNAL_COLUMNS
            ),
            &[&&*data.email],
            |row| row_to_user_internal(row),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => "البريد الإلكتروني أو كلمة المرور غير صحيحة".to_string(),
            _ => e.to_string(),
        })?;

    let password_hash = internal.password_hash.clone();
    let is_blocked = internal.blocked;

    if is_blocked {
        return Err("هذا الحساب محظور. تواصل مع المدير".to_string());
    }

    let is_valid = bcrypt::verify(&data.password, &password_hash)
        .map_err(|e| format!("Verification failed: {}", e))?;

    if !is_valid {
        return Err("البريد الإلكتروني أو كلمة المرور غير صحيحة".to_string());
    }

    let token = Uuid::new_v4().to_string();

    Ok(LoginResponse {
        user: internal.into(),
        token,
    })
}

#[tauri::command]
pub fn get_users(state: tauri::State<'_, AppState>) -> Result<Vec<User>, String> {
    state
        .db
        .query_all(
            &format!(
                "SELECT {} FROM users WHERE deleted_at IS NULL ORDER BY created_at DESC",
                USER_COLUMNS
            ),
            &[],
            row_to_user,
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_user_by_id(state: tauri::State<'_, AppState>, id: String) -> Result<User, String> {
    state
        .db
        .query_row(
            &format!(
                "SELECT {} FROM users WHERE id = ?1 AND deleted_at IS NULL",
                USER_COLUMNS
            ),
            &[&&*id],
            row_to_user,
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_user(
    state: tauri::State<'_, AppState>,
    data: CreateUserData,
) -> Result<User, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let role = data.role.unwrap_or_else(|| "user".to_string());
    let hash =
        bcrypt::hash(&data.password, 10).map_err(|e| format!("Failed to hash password: {}", e))?;

    state
        .db
        .execute(
            "INSERT INTO users (id, full_name, email, role, password_hash, created_at, updated_at, sync_version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
            &[
                &&*id,
                &&*data.full_name,
                &&*data.email,
                &&*role,
                &&*hash,
                &&*now,
                &&*now,
            ],
        )
        .map_err(|e| e.to_string())?;

    state.db.query_row(
        &format!("SELECT {} FROM users WHERE id = ?1", USER_COLUMNS),
        &[&&*id],
        row_to_user,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_user(
    state: tauri::State<'_, AppState>,
    id: String,
    data: UpdateUserData,
) -> Result<User, String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    state
        .db
        .execute(
            "UPDATE users SET full_name=COALESCE(?2, full_name), email=COALESCE(?3, email), role=COALESCE(?4, role), updated_at=?5, sync_version=sync_version+1 WHERE id=?1",
            &[
                &&*id,
                &data.full_name.as_deref(),
                &data.email.as_deref(),
                &data.role.as_deref(),
                &&*now,
            ],
        )
        .map_err(|e| e.to_string())?;

    state.db.query_row(
        &format!("SELECT {} FROM users WHERE id = ?1", USER_COLUMNS),
        &[&&*id],
        row_to_user,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_user(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    state
        .db
        .execute(
            "UPDATE users SET deleted_at = ?1, updated_at = ?1, sync_version = sync_version + 1 WHERE id = ?2",
            &[&&*now, &&*id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn block_user(state: tauri::State<'_, AppState>, id: String, blocked: bool) -> Result<User, String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let blocked_val: i64 = if blocked { 1 } else { 0 };
    state
        .db
        .execute(
            "UPDATE users SET blocked = ?1, updated_at = ?2, sync_version = sync_version + 1 WHERE id = ?3",
            &[&blocked_val, &&*now, &&*id],
        )
        .map_err(|e| e.to_string())?;

    state.db.query_row(
        &format!("SELECT {} FROM users WHERE id = ?1", USER_COLUMNS),
        &[&&*id],
        row_to_user,
    ).map_err(|e| e.to_string())
}
