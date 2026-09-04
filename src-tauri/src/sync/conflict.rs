use crate::db::connection::DatabaseConnection;
use std::sync::Arc;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConflictRecord {
    pub table_name: String,
    pub entity_id: String,
    pub local_updated_at: String,
    pub remote_updated_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ConflictResolution {
    KeepLocal,
    KeepRemote,
    Merged,
}

pub struct ConflictResolver {
    _db: Arc<DatabaseConnection>,
}

impl ConflictResolver {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { _db: db }
    }

    pub fn resolve(&self, local_updated_at: &str, remote_updated_at: &str) -> ConflictResolution {
        match local_updated_at.cmp(remote_updated_at) {
            std::cmp::Ordering::Greater => ConflictResolution::KeepLocal,
            std::cmp::Ordering::Less => ConflictResolution::KeepRemote,
            std::cmp::Ordering::Equal => ConflictResolution::KeepLocal,
        }
    }

    pub fn log_conflict(
        &self,
        table_name: &str,
        entity_id: &str,
        resolution: &ConflictResolution,
    ) {
        let res_str = match resolution {
            ConflictResolution::KeepLocal => "keep_local",
            ConflictResolution::KeepRemote => "keep_remote",
            ConflictResolution::Merged => "merged",
        };
        log::warn!(
            "Sync conflict resolved: table={}, entity={}, resolution={}",
            table_name,
            entity_id,
            res_str
        );
    }
}
