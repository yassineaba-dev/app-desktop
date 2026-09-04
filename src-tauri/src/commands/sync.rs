use crate::db::models::SyncStats;
use crate::AppState;

#[tauri::command]
pub async fn sync_push(state: tauri::State<'_, AppState>) -> Result<SyncStats, String> {
    let result = state.sync.push().await?;
    Ok(SyncStats {
        pushed: result.pushed,
        pulled: result.pulled,
        last_sync_at: result.last_sync_at,
        pending_push: result.pending_push,
        status: result.status,
    })
}

#[tauri::command]
pub async fn sync_pull(state: tauri::State<'_, AppState>) -> Result<SyncStats, String> {
    let result = state.sync.pull().await?;
    Ok(SyncStats {
        pushed: result.pushed,
        pulled: result.pulled,
        last_sync_at: result.last_sync_at,
        pending_push: result.pending_push,
        status: result.status,
    })
}

#[tauri::command]
pub fn get_sync_status(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.sync.current_status().as_str().to_string())
}

#[tauri::command]
pub fn get_sync_arabic_status(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.sync.current_status().arabic_label().to_string())
}

#[tauri::command]
pub async fn sync_full(state: tauri::State<'_, AppState>) -> Result<SyncStats, String> {
    let pull_result = state.sync.pull().await?;
    let push_result = state.sync.push().await?;
    Ok(SyncStats {
        pushed: push_result.pushed + pull_result.pushed,
        pulled: push_result.pulled + pull_result.pulled,
        last_sync_at: push_result.last_sync_at,
        pending_push: push_result.pending_push,
        status: push_result.status,
    })
}
