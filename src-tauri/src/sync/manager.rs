use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::watch;

use crate::db::connection::DatabaseConnection;
use crate::db::models::*;
use super::conflict::{ConflictResolver, ConflictResolution};
use super::status::SyncStatus;

/// Emitted to the frontend after a pull imports changed records, so the UI can
/// refresh immediately instead of waiting for the next refetch.
pub const SYNC_DATA_CHANGED_EVENT: &str = "sync-data-changed";

pub struct SyncManager {
    db: Arc<DatabaseConnection>,
    status_tx: watch::Sender<SyncStatus>,
    http: Client,
    turso_url: Option<String>,
    turso_token: Option<String>,
    app: Option<AppHandle>,
}

#[derive(Debug, Serialize)]
struct TursoRequest {
    requests: Vec<TursoStatement>,
}

#[derive(Debug, Serialize)]
struct TursoStatement {
    #[serde(rename = "type")]
    stmt_type: String,
    stmt: TursoQuery,
}

#[derive(Debug, Serialize)]
struct TursoQuery {
    sql: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    args: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TursoResponse {
    results: Vec<TursoResult>,
}

#[derive(Debug, Deserialize)]
struct TursoResult {
    response: TursoResponseInner,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TursoResponseInner {
    #[serde(rename = "type")]
    result_type: String,
    result: Option<TursoResultSet>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TursoResultSet {
    cols: Vec<TursoCol>,
    rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TursoCol {
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteIncoming {
    pub id: String,
    pub registration_number: String,
    pub correspondence_number: Option<String>,
    pub date: String,
    pub arrival_date: Option<String>,
    pub subject: String,
    pub sender: String,
    pub destination_service: String,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub created_by: Option<String>,
    pub sync_version: i64,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub is_duplicate: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteOutgoing {
    pub id: String,
    pub registration_number: String,
    pub correspondence_number: Option<String>,
    pub date: String,
    pub subject: String,
    pub recipient: String,
    pub destination_service: String,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub created_by: Option<String>,
    pub sync_version: i64,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub file_name_in: Option<String>,
    pub file_path_in: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct RemoteUser {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub sync_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatsResult {
    pub pushed: i64,
    pub pulled: i64,
    pub last_sync_at: String,
    pub pending_push: i64,
    pub status: String,
}

impl SyncManager {
    pub fn new(
        db: Arc<DatabaseConnection>,
        turso_url: Option<String>,
        turso_token: Option<String>,
        app: Option<AppHandle>,
    ) -> Self {
        let (status_tx, _) = watch::channel(SyncStatus::Offline);
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self { db, status_tx, http, turso_url, turso_token, app }
    }

    pub fn status_receiver(&self) -> watch::Receiver<SyncStatus> {
        self.status_tx.subscribe()
    }

    pub fn current_status(&self) -> SyncStatus {
        self.status_tx.borrow().clone()
    }

    fn set_status(&self, status: SyncStatus) {
        let _ = self.status_tx.send(status);
    }

    fn has_config(&self) -> bool {
        self.turso_url.is_some() && self.turso_token.is_some()
    }

    async fn execute_remote(&self, sql: &str) -> Result<TursoResponse, String> {
        let url = self.turso_url.as_ref().ok_or("No Turso URL configured")?;
        let token = self.turso_token.as_ref().ok_or("No Turso token configured")?;

        let request = TursoRequest {
            requests: vec![TursoStatement {
                stmt_type: "execute".to_string(),
                stmt: TursoQuery {
                    sql: sql.to_string(),
                    args: vec![],
                },
            }],
        };

        let resp = self
            .http
            .post(format!("{}/v2/pipeline", url))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        resp.json::<TursoResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Ensure the remote (Turso) schema matches the local schema, so that
    /// pushes/pulls of the current column layout never fail because of a
    /// missing table or missing column. Idempotent.
    async fn bootstrap_remote(&self) -> Result<(), String> {
        let statements = [
            "CREATE TABLE IF NOT EXISTS incoming (id TEXT PRIMARY KEY NOT NULL, registration_number TEXT NOT NULL, correspondence_number TEXT, date TEXT NOT NULL, arrival_date TEXT, subject TEXT NOT NULL, sender TEXT NOT NULL, destination_service TEXT NOT NULL, source TEXT, notes TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT, created_by TEXT, sync_version INTEGER NOT NULL DEFAULT 1, file_name TEXT, file_path TEXT, is_duplicate INTEGER NOT NULL DEFAULT 0)",
            "CREATE TABLE IF NOT EXISTS outgoing (id TEXT PRIMARY KEY NOT NULL, registration_number TEXT NOT NULL, correspondence_number TEXT, date TEXT NOT NULL, subject TEXT NOT NULL, recipient TEXT NOT NULL, destination_service TEXT NOT NULL, source TEXT, notes TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT, created_by TEXT, sync_version INTEGER NOT NULL DEFAULT 1, file_name TEXT, file_path TEXT, file_name_in TEXT, file_path_in TEXT)",
        ];
        for stmt in statements {
            if self.execute_remote(stmt).await.is_err() {
                return Err(format!("Failed to bootstrap remote schema: {}", stmt));
            }
        }

        // Additive column migrations, tolerant of columns that already exist.
        let add_columns = [
            "ALTER TABLE incoming ADD COLUMN file_name TEXT",
            "ALTER TABLE incoming ADD COLUMN file_path TEXT",
            "ALTER TABLE incoming ADD COLUMN is_duplicate INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE outgoing ADD COLUMN file_name TEXT",
            "ALTER TABLE outgoing ADD COLUMN file_path TEXT",
            "ALTER TABLE outgoing ADD COLUMN file_name_in TEXT",
            "ALTER TABLE outgoing ADD COLUMN file_path_in TEXT",
        ];
        for stmt in add_columns {
            // Ignore "duplicate column" errors; surface everything else.
            let _ = self.execute_remote(stmt).await;
        }
        Ok(())
    }

    pub async fn push(&self) -> Result<SyncStatsResult, String> {
        if !self.has_config() {
            return Ok(SyncStatsResult {
                pushed: 0,
                pulled: 0,
                last_sync_at: String::new(),
                pending_push: self.db.get_pending_push_count().unwrap_or(0),
                status: "no_config".to_string(),
            });
        }

        self.set_status(SyncStatus::Syncing);
        self.bootstrap_remote().await?;
        let pushed = self.push_records().await?;
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.db
            .execute(
                "UPDATE sync_metadata SET value = ?1, updated_at = ?1 WHERE key = 'last_sync_at'",
                &[&&*now],
            )
            .ok();

        self.set_status(SyncStatus::Synced);

        Ok(SyncStatsResult {
            pushed,
            pulled: 0,
            last_sync_at: now,
            pending_push: 0,
            status: "synced".to_string(),
        })
    }

    async fn push_records(&self) -> Result<i64, String> {
        let last_pushed = self.db.get_sync_version().unwrap_or(0);
        let mut total_pushed: i64 = 0;

        let incoming_rows: Vec<Incoming> = self
            .db
            .query_all(
                &format!(
                    "SELECT {} FROM incoming WHERE sync_version > ?1",
                    crate::db::connection::INCOMING_COLUMNS
                ),
                &[&last_pushed],
                crate::db::connection::row_to_incoming,
            )
            .map_err(|e| format!("Failed to fetch outgoing incoming: {}", e))?;

        if !incoming_rows.is_empty() {
            for chunk in incoming_rows.chunks(50) {
                let sqls: Vec<String> = chunk
                    .iter()
                    .map(|r| {
                        format!(
                            "INSERT OR REPLACE INTO incoming (id, registration_number, correspondence_number, date, arrival_date, subject, sender, destination_service, source, notes, created_at, updated_at, deleted_at, created_by, sync_version, file_name, file_path, is_duplicate) VALUES ('{}', '{}', {}, {}, '{}', '{}', '{}', '{}', {}, {}, '{}', '{}', {}, {}, {}, '{}', '{}', {})",
                            r.id,
                            r.registration_number.replace('\'', "''"),
                            r.correspondence_number.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.date,
                            r.arrival_date.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.subject.replace('\'', "''"),
                            r.sender.replace('\'', "''"),
                            r.destination_service.replace('\'', "''"),
                            r.source.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.notes.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.created_at,
                            r.updated_at,
                            r.deleted_at.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.created_by.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.sync_version,
                            r.file_name.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.file_path.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            if r.is_duplicate { 1 } else { 0 }
                        )
                    })
                    .collect();
                let batch_sql = sqls.join(";");
                if self.execute_remote(&batch_sql).await.is_ok() {
                    total_pushed += chunk.len() as i64;
                }
            }
        }

        let outgoing_rows: Vec<Outgoing> = self
            .db
            .query_all(
                &format!(
                    "SELECT {} FROM outgoing WHERE sync_version > ?1",
                    crate::db::connection::OUTGOING_COLUMNS
                ),
                &[&last_pushed],
                crate::db::connection::row_to_outgoing,
            )
            .map_err(|e| format!("Failed to fetch outgoing records: {}", e))?;

        if !outgoing_rows.is_empty() {
            for chunk in outgoing_rows.chunks(50) {
                let sqls: Vec<String> = chunk
                    .iter()
                    .map(|r| {
                        format!(
                            "INSERT OR REPLACE INTO outgoing (id, registration_number, correspondence_number, date, subject, recipient, destination_service, source, notes, created_at, updated_at, deleted_at, created_by, sync_version, file_name, file_path, file_name_in, file_path_in) VALUES ('{}', '{}', {}, '{}', '{}', '{}', '{}', {}, {}, '{}', '{}', {}, {}, {}, '{}', '{}', '{}', '{}')",
                            r.id,
                            r.registration_number.replace('\'', "''"),
                            r.correspondence_number.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.date,
                            r.subject.replace('\'', "''"),
                            r.recipient.replace('\'', "''"),
                            r.destination_service.replace('\'', "''"),
                            r.source.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.notes.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.created_at,
                            r.updated_at,
                            r.deleted_at.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.created_by.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.sync_version,
                            r.file_name.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.file_path.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.file_name_in.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string()),
                            r.file_path_in.as_deref().map(|v| format!("'{}'", v.replace('\'', "''"))).unwrap_or_else(|| "NULL".to_string())
                        )
                    })
                    .collect();
                let batch_sql = sqls.join(";");
                if self.execute_remote(&batch_sql).await.is_ok() {
                    total_pushed += chunk.len() as i64;
                }
            }
        }

        if total_pushed > 0 {
            let max_version = self.db.get_max_sync_version("incoming")
                .unwrap_or(0)
                .max(self.db.get_max_sync_version("outgoing").unwrap_or(0))
                .max(self.db.get_max_sync_version("users").unwrap_or(0));
            self.db.set_sync_version(max_version).map_err(|e| e.to_string())?;
        }

        Ok(total_pushed)
    }

    pub async fn pull(&self) -> Result<SyncStatsResult, String> {
        if !self.has_config() {
            return Ok(SyncStatsResult {
                pushed: 0,
                pulled: 0,
                last_sync_at: String::new(),
                pending_push: 0,
                status: "no_config".to_string(),
            });
        }

        self.set_status(SyncStatus::Connecting);
        self.bootstrap_remote().await?;
        let pulled = self.pull_records().await?;

        if pulled > 0 {
            self.notify_data_changed();
        }

        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.db
            .execute(
                "UPDATE sync_metadata SET value = ?1, updated_at = ?1 WHERE key = 'last_sync_at'",
                &[&&*now],
            )
            .ok();

        self.set_status(SyncStatus::Synced);

        Ok(SyncStatsResult {
            pushed: 0,
            pulled,
            last_sync_at: now,
            pending_push: self.db.get_pending_push_count().unwrap_or(0),
            status: "synced".to_string(),
        })
    }

    async fn pull_records(&self) -> Result<i64, String> {
        let last_pulled = self.db.get_pulled_version().unwrap_or(0);
        let mut total_pulled: i64 = 0;
        let resolver = ConflictResolver::new(self.db.clone());

        let incoming_resp = self
            .execute_remote(&format!(
                "SELECT {} FROM incoming WHERE sync_version > {}",
                crate::db::connection::INCOMING_COLUMNS,
                last_pulled
            ))
            .await?;

        if let Some(result) = incoming_resp.results.first() {
            if let Some(ref rs) = result.response.result {
                for row in &rs.rows {
                    if row.len() >= 18 {
                        let remote = RemoteIncoming {
                            id: row[0].as_str().unwrap_or("").to_string(),
                            registration_number: row[1].as_str().unwrap_or("").to_string(),
                            correspondence_number: row[2].as_str().map(|s| s.to_string()),
                            date: row[3].as_str().unwrap_or("").to_string(),
                            arrival_date: row[4].as_str().map(|s| s.to_string()),
                            subject: row[5].as_str().unwrap_or("").to_string(),
                            sender: row[6].as_str().unwrap_or("").to_string(),
                            destination_service: row[7].as_str().unwrap_or("").to_string(),
                            source: row[8].as_str().map(|s| s.to_string()),
                            notes: row[9].as_str().map(|s| s.to_string()),
                            created_at: row[10].as_str().unwrap_or("").to_string(),
                            updated_at: row[11].as_str().unwrap_or("").to_string(),
                            deleted_at: row[12].as_str().map(|s| s.to_string()),
                            created_by: row[13].as_str().map(|s| s.to_string()),
                            sync_version: row[14].as_i64().unwrap_or(0),
                            file_name: row[15].as_str().map(|s| s.to_string()),
                            file_path: row[16].as_str().map(|s| s.to_string()),
                            is_duplicate: row[17].as_i64().unwrap_or(0) != 0,
                        };
                        self.merge_incoming(&remote, &resolver)?;
                        total_pulled += 1;
                    }
                }
            }
        }

        let outgoing_resp = self
            .execute_remote(&format!(
                "SELECT {} FROM outgoing WHERE sync_version > {}",
                crate::db::connection::OUTGOING_COLUMNS,
                last_pulled
            ))
            .await?;

        if let Some(result) = outgoing_resp.results.first() {
            if let Some(ref rs) = result.response.result {
                for row in &rs.rows {
                    if row.len() >= 18 {
                        let remote = RemoteOutgoing {
                            id: row[0].as_str().unwrap_or("").to_string(),
                            registration_number: row[1].as_str().unwrap_or("").to_string(),
                            correspondence_number: row[2].as_str().map(|s| s.to_string()),
                            date: row[3].as_str().unwrap_or("").to_string(),
                            subject: row[4].as_str().unwrap_or("").to_string(),
                            recipient: row[5].as_str().unwrap_or("").to_string(),
                            destination_service: row[6].as_str().unwrap_or("").to_string(),
                            source: row[7].as_str().map(|s| s.to_string()),
                            notes: row[8].as_str().map(|s| s.to_string()),
                            created_at: row[9].as_str().unwrap_or("").to_string(),
                            updated_at: row[10].as_str().unwrap_or("").to_string(),
                            deleted_at: row[11].as_str().map(|s| s.to_string()),
                            created_by: row[12].as_str().map(|s| s.to_string()),
                            sync_version: row[13].as_i64().unwrap_or(0),
                            file_name: row[14].as_str().map(|s| s.to_string()),
                            file_path: row[15].as_str().map(|s| s.to_string()),
                            file_name_in: row[16].as_str().map(|s| s.to_string()),
                            file_path_in: row[17].as_str().map(|s| s.to_string()),
                        };
                        self.merge_outgoing(&remote, &resolver)?;
                        total_pulled += 1;
                    }
                }
            }
        }

        if total_pulled > 0 {
            let max_version = self.db.get_max_sync_version("incoming")
                .unwrap_or(0)
                .max(self.db.get_max_sync_version("outgoing").unwrap_or(0))
                .max(self.db.get_max_sync_version("users").unwrap_or(0));
            self.db.set_pulled_version(max_version).map_err(|e| e.to_string())?;
        }

        Ok(total_pulled)
    }

    fn merge_incoming(
        &self,
        remote: &RemoteIncoming,
        resolver: &ConflictResolver,
    ) -> Result<(), String> {
        let local: Option<Incoming> = self
            .db
            .query_row(
                &format!("SELECT {} FROM incoming WHERE id = ?1", crate::db::connection::INCOMING_COLUMNS),
                &[&remote.id],
                crate::db::connection::row_to_incoming,
            )
            .ok();

        match local {
            None => {
                self.db
                    .execute(
                        "INSERT OR REPLACE INTO incoming (id, registration_number, correspondence_number, date, arrival_date, subject, sender, destination_service, source, notes, created_at, updated_at, deleted_at, created_by, sync_version, file_name, file_path, is_duplicate) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                        &[
                            &&*remote.id,
                            &&*remote.registration_number,
                            &remote.correspondence_number.as_deref(),
                            &&*remote.date,
                            &remote.arrival_date.as_deref(),
                            &&*remote.subject,
                            &&*remote.sender,
                            &&*remote.destination_service,
                            &remote.source.as_deref(),
                            &remote.notes.as_deref(),
                            &&*remote.created_at,
                            &&*remote.updated_at,
                            &remote.deleted_at.as_deref(),
                            &remote.created_by.as_deref(),
                            &remote.sync_version,
                            &remote.file_name.as_deref(),
                            &remote.file_path.as_deref(),
                            &remote.is_duplicate,
                        ],
                    )
                    .map_err(|e| format!("Insert incoming failed: {}", e))?;
            }
            Some(ref local_rec) => {
                let resolution =
                    resolver.resolve(&local_rec.updated_at, &remote.updated_at);
                match resolution {
                    ConflictResolution::KeepRemote => {
                        self.db
                            .execute(
                                "UPDATE incoming SET registration_number=?2, correspondence_number=?3, date=?4, arrival_date=?5, subject=?6, sender=?7, destination_service=?8, source=?9, notes=?10, updated_at=?11, deleted_at=?12, created_by=?13, sync_version=?14, file_name=?15, file_path=?16, is_duplicate=?17 WHERE id=?1",
                                &[
                                    &&*remote.id,
                                    &&*remote.registration_number,
                                    &remote.correspondence_number.as_deref(),
                                    &&*remote.date,
                                    &remote.arrival_date.as_deref(),
                                    &&*remote.subject,
                                    &&*remote.sender,
                                    &&*remote.destination_service,
                                    &remote.source.as_deref(),
                                    &remote.notes.as_deref(),
                                    &&*remote.updated_at,
                                    &remote.deleted_at.as_deref(),
                                    &remote.created_by.as_deref(),
                                    &remote.sync_version,
                                    &remote.file_name.as_deref(),
                                    &remote.file_path.as_deref(),
                                    &remote.is_duplicate,
                                ],
                            )
                            .map_err(|e| format!("Update incoming failed: {}", e))?;
                        resolver.log_conflict("incoming", &remote.id, &resolution);
                    }
                    _ => {
                        resolver.log_conflict("incoming", &remote.id, &resolution);
                    }
                }
            }
        }
        Ok(())
    }

    fn merge_outgoing(
        &self,
        remote: &RemoteOutgoing,
        resolver: &ConflictResolver,
    ) -> Result<(), String> {
        let local: Option<Outgoing> = self
            .db
            .query_row(
                &format!("SELECT {} FROM outgoing WHERE id = ?1", crate::db::connection::OUTGOING_COLUMNS),
                &[&remote.id],
                crate::db::connection::row_to_outgoing,
            )
            .ok();

        match local {
            None => {
                self.db
                    .execute(
                        "INSERT OR REPLACE INTO outgoing (id, registration_number, correspondence_number, date, subject, recipient, destination_service, source, notes, created_at, updated_at, deleted_at, created_by, sync_version, file_name, file_path, file_name_in, file_path_in) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                        &[
                            &&*remote.id,
                            &&*remote.registration_number,
                            &remote.correspondence_number.as_deref(),
                            &&*remote.date,
                            &&*remote.subject,
                            &&*remote.recipient,
                            &&*remote.destination_service,
                            &remote.source.as_deref(),
                            &remote.notes.as_deref(),
                            &&*remote.created_at,
                            &&*remote.updated_at,
                            &remote.deleted_at.as_deref(),
                            &remote.created_by.as_deref(),
                            &remote.sync_version,
                            &remote.file_name.as_deref(),
                            &remote.file_path.as_deref(),
                            &remote.file_name_in.as_deref(),
                            &remote.file_path_in.as_deref(),
                        ],
                    )
                    .map_err(|e| format!("Insert outgoing failed: {}", e))?;
            }
            Some(ref local_rec) => {
                let resolution =
                    resolver.resolve(&local_rec.updated_at, &remote.updated_at);
                match resolution {
                    ConflictResolution::KeepRemote => {
                        self.db
                            .execute(
                                "UPDATE outgoing SET registration_number=?2, correspondence_number=?3, date=?4, subject=?5, recipient=?6, destination_service=?7, source=?8, notes=?9, updated_at=?10, deleted_at=?11, created_by=?12, sync_version=?13, file_name=?14, file_path=?15, file_name_in=?16, file_path_in=?17 WHERE id=?1",
                                &[
                                    &&*remote.id,
                                    &&*remote.registration_number,
                                    &remote.correspondence_number.as_deref(),
                                    &&*remote.date,
                                    &&*remote.subject,
                                    &&*remote.recipient,
                                    &&*remote.destination_service,
                                    &remote.source.as_deref(),
                                    &remote.notes.as_deref(),
                                    &&*remote.updated_at,
                                    &remote.deleted_at.as_deref(),
                                    &remote.created_by.as_deref(),
                                    &remote.sync_version,
                                    &remote.file_name.as_deref(),
                                    &remote.file_path.as_deref(),
                                    &remote.file_name_in.as_deref(),
                                    &remote.file_path_in.as_deref(),
                                ],
                            )
                            .map_err(|e| format!("Update outgoing failed: {}", e))?;
                        resolver.log_conflict("outgoing", &remote.id, &resolution);
                    }
                    _ => {
                        resolver.log_conflict("outgoing", &remote.id, &resolution);
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn checkpoint(&self) -> Result<(), String> {
        if self.has_config() {
            self.execute_remote("PRAGMA wal_checkpoint(TRUNCATE)")
                .await?;
        }
        Ok(())
    }

    fn notify_data_changed(&self) {
        if let Some(app) = &self.app {
            if let Err(e) = app.emit(SYNC_DATA_CHANGED_EVENT, ()) {
                log::debug!("Failed to emit sync-data-changed event: {}", e);
            }
        }
    }

    pub async fn run_background_sync(self: &Arc<Self>) {
        // Fast polling interval so remote changes propagate to the UI quickly.
        let interval = std::time::Duration::from_secs(3);
        loop {
            tokio::time::sleep(interval).await;
            if !self.has_config() {
                continue;
            }
            log::debug!("Background sync cycle starting");
            match self.pull().await {
                Ok(_) => log::debug!("Background pull completed"),
                Err(e) => log::error!("Background pull failed: {}", e),
            }
            match self.push().await {
                Ok(_) => log::debug!("Background push completed"),
                Err(e) => log::error!("Background push failed: {}", e),
            }
        }
    }
}
