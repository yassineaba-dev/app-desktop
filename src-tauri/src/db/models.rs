use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub blocked: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub sync_version: i64,
}

#[derive(Debug, Clone)]
pub struct UserInternal {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub blocked: bool,
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub sync_version: i64,
}

impl From<UserInternal> for User {
    fn from(u: UserInternal) -> Self {
        User {
            id: u.id,
            full_name: u.full_name,
            email: u.email,
            role: u.role,
            blocked: u.blocked,
            created_at: u.created_at,
            updated_at: u.updated_at,
            deleted_at: u.deleted_at,
            sync_version: u.sync_version,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginData {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: User,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserData {
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserData {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incoming {
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

#[derive(Debug, Clone, Deserialize)]
pub struct CreateIncomingData {
    pub registration_number: String,
    pub correspondence_number: Option<String>,
    pub date: String,
    pub arrival_date: Option<String>,
    pub subject: String,
    pub sender: String,
    pub destination_service: String,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub is_duplicate: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateIncomingData {
    pub registration_number: Option<String>,
    pub correspondence_number: Option<String>,
    pub date: Option<String>,
    pub arrival_date: Option<String>,
    pub subject: Option<String>,
    pub sender: Option<String>,
    pub destination_service: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub is_duplicate: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingFileInfo {
    pub file_name: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outgoing {
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

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOutgoingData {
    pub registration_number: String,
    pub correspondence_number: Option<String>,
    pub date: String,
    pub subject: String,
    pub recipient: String,
    pub destination_service: String,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub file_name_in: Option<String>,
    pub file_path_in: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateOutgoingData {
    pub registration_number: Option<String>,
    pub correspondence_number: Option<String>,
    pub date: Option<String>,
    pub subject: Option<String>,
    pub recipient: Option<String>,
    pub destination_service: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub file_name_in: Option<String>,
    pub file_path_in: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingFileInfo {
    pub file_name: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub entity: String,
    pub entity_id: String,
    pub timestamp: String,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CreateAuditLogData {
    pub user_id: Option<String>,
    pub action: String,
    pub entity: String,
    pub entity_id: String,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub pushed: i64,
    pub pulled: i64,
    pub last_sync_at: String,
    pub pending_push: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SyncConflict {
    pub table_name: String,
    pub entity_id: String,
    pub local_version: String,
    pub remote_version: String,
    pub resolution: String,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        Self { items, total, page, per_page }
    }
}
