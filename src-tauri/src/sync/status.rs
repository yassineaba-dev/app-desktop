use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Offline,
    Connecting,
    Syncing,
    Synced,
    Error(String),
}

impl SyncStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SyncStatus::Offline => "offline",
            SyncStatus::Connecting => "connecting",
            SyncStatus::Syncing => "syncing",
            SyncStatus::Synced => "synced",
            SyncStatus::Error(_) => "error",
        }
    }

    pub fn arabic_label(&self) -> &str {
        match self {
            SyncStatus::Offline => "غير متصل",
            SyncStatus::Connecting => "جارٍ الاتصال...",
            SyncStatus::Syncing => "جارٍ المزامنة...",
            SyncStatus::Synced => "متصل",
            SyncStatus::Error(_) => "تعذر الاتصال",
        }
    }
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.arabic_label())
    }
}
