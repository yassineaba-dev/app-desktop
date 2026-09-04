use chrono::Utc;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::models::*;

#[derive(Clone)]
pub struct DatabaseConnection {
    conn: Arc<Mutex<Connection>>,
    _path: PathBuf,
}

impl DatabaseConnection {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let path_buf = PathBuf::from(path);
        let parent = path_buf.parent().unwrap_or_else(|| std::path::Path::new("."));
        std::fs::create_dir_all(parent).ok();

        let conn = Connection::open_with_flags(
            &path_buf,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-64000;",
        )?;

        log::info!("Database opened at: {}", path);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            _path: path_buf,
        })
    }

    pub fn path(&self) -> &std::path::Path {
        &self._path
    }

    pub fn execute_batch(&self, sql: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(sql)
    }

    pub fn execute(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::types::ToSql],
    ) -> Result<usize, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(sql, params)
    }

    pub fn query_row<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::types::ToSql],
        f: F,
    ) -> Result<T, rusqlite::Error>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>,
    {
        let conn = self.conn.lock().unwrap();
        conn.query_row(sql, params, f)
    }

    pub fn query_all<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::types::ToSql],
        f: F,
    ) -> Result<Vec<T>, rusqlite::Error>
    where
        F: FnMut(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>,
    {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, f)?;
        rows.collect()
    }

    pub fn last_insert_rowid(&self) -> i64 {
        let conn = self.conn.lock().unwrap();
        conn.last_insert_rowid()
    }

    pub fn get_sync_version(&self) -> Result<i64, rusqlite::Error> {
        self.query_row(
            "SELECT CAST(value AS INTEGER) FROM sync_metadata WHERE key = 'last_pushed_version'",
            &[],
            |row| row.get(0),
        )
    }

    pub fn set_sync_version(&self, version: i64) -> Result<(), rusqlite::Error> {
        self.execute(
            "UPDATE sync_metadata SET value = ?1, updated_at = datetime('now') WHERE key = 'last_pushed_version'",
            &[&version.to_string()],
        )?;
        Ok(())
    }

    pub fn get_pulled_version(&self) -> Result<i64, rusqlite::Error> {
        self.query_row(
            "SELECT CAST(value AS INTEGER) FROM sync_metadata WHERE key = 'last_pulled_version'",
            &[],
            |row| row.get(0),
        )
    }

    pub fn set_pulled_version(&self, version: i64) -> Result<(), rusqlite::Error> {
        self.execute(
            "UPDATE sync_metadata SET value = ?1, updated_at = datetime('now') WHERE key = 'last_pulled_version'",
            &[&version.to_string()],
        )?;
        Ok(())
    }

    pub fn get_max_sync_version(&self, table: &str) -> Result<i64, rusqlite::Error> {
        let sql = format!("SELECT COALESCE(MAX(sync_version), 0) FROM {}", table);
        self.query_row(&sql, &[], |row| row.get(0))
    }

    pub fn get_pending_push_count(&self) -> Result<i64, rusqlite::Error> {
        let pushed = self.get_sync_version()?;
        let max_incoming = self.get_max_sync_version("incoming")?;
        let max_outgoing = self.get_max_sync_version("outgoing")?;
        let max_users = self.get_max_sync_version("users")?;
        let max = max_incoming.max(max_outgoing).max(max_users);
        Ok((max - pushed).max(0))
    }

    pub fn increment_sync_version(
        &self,
        table: &str,
        id: &str,
    ) -> Result<(), rusqlite::Error> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let sql = format!(
            "UPDATE {} SET sync_version = sync_version + 1, updated_at = ?1 WHERE id = ?2",
            table
        );
        self.execute(&sql, &[&&*now, &id])?;
        Ok(())
    }

    /// Replace the current database contents with the data from another SQLite
    /// file. Runs inside a single transaction on the already-open connection,
    /// so no restart or file swapping is required.
    pub fn import_from(&self, source_path: &str) -> Result<usize, String> {
        let src = Connection::open_with_flags(
            source_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .map_err(|e| format!("تعذر فتح القاعدة المصدر: {}", e))?;

        // Confirm the selected file is actually a SQLite database we can read.
        {
            let exists: i64 = src
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view')",
                    [],
                    |row| row.get(0),
                )
                .map_err(|e| format!("الملف المحدد ليس قاعدة بيانات صالحة: {}", e))?;
            if exists == 0 {
                return Err("الملف المحدد ليس قاعدة بيانات صالحة".to_string());
            }
        }

        // Children must be removed before their parents to satisfy FK rules.
        let delete_order = [
            "incoming", "outgoing", "audit_logs", "users", "settings", "sync_metadata",
        ];
        let insert_order = [
            "users", "incoming", "outgoing", "audit_logs", "settings", "sync_metadata",
        ];

        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|e| format!("فشل بدء المعاملة: {}", e))?;

        for &table in &delete_order {
            if table_columns(&tx, table)?.is_empty() {
                continue;
            }
            tx.execute(&format!("DELETE FROM {}", table), [])
                .map_err(|e| format!("فشل مسح جدول {}: {}", table, e))?;
        }

        let mut total: usize = 0;
        for &table in &insert_order {
            let target_cols = table_columns(&tx, table)?;
            if target_cols.is_empty() {
                continue;
            }
            let src_cols = table_columns(&src, table)?;
            if src_cols.is_empty() {
                continue;
            }

            let common: Vec<&String> = target_cols
                .iter()
                .filter(|c| src_cols.iter().any(|s| s == *c))
                .collect();
            if common.is_empty() {
                continue;
            }

            let cols_sql: Vec<String> = common.iter().map(|c| (*c).clone()).collect();
            let cols_list = cols_sql.join(", ");
            let select_sql = format!("SELECT {} FROM {}", cols_list, table);

            let mut rows_data: Vec<Vec<rusqlite::types::Value>> = Vec::new();
            {
                let mut stmt = src
                    .prepare(&select_sql)
                    .map_err(|e| format!("فشل قراءة جدول {}: {}", table, e))?;
                let mut rows = stmt
                    .query([])
                    .map_err(|e| format!("فشل قراءة جدول {}: {}", table, e))?;
                while let Some(row) = rows
                    .next()
                    .map_err(|e| format!("فشل قراءة جدول {}: {}", table, e))?
                {
                    let mut vals = Vec::with_capacity(common.len());
                    for i in 0..common.len() {
                        vals.push(
                            row.get::<_, rusqlite::types::Value>(i)
                                .map_err(|e| format!("فشل قراءة جدول {}: {}", table, e))?,
                        );
                    }
                    rows_data.push(vals);
                }
            }

            let placeholders: Vec<String> = (1..=common.len()).map(|i| format!("?{}", i)).collect();
            let insert_sql = format!(
                "INSERT OR REPLACE INTO {} ({}) VALUES ({})",
                table,
                cols_list,
                placeholders.join(", ")
            );

            for vals in &rows_data {
                tx.execute(&insert_sql, rusqlite::params_from_iter(vals.iter()))
                    .map_err(|e| format!("فشل استيراد جدول {}: {}", table, e))?;
            }
            total += rows_data.len();
        }

        tx.commit()
            .map_err(|e| format!("فشل حفظ التغييرات: {}", e))?;

        Ok(total)
    }
}

/// Read the column names of a table via `PRAGMA table_info`. Returns an empty
/// Vec when the table does not exist.
fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", table))
        .map_err(|e| format!("فشل قراءة بنية جدول {}: {}", table, e))?;
    let cols = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("فشل قراءة بنية جدول {}: {}", table, e))?;
    let mut out = Vec::new();
    for c in cols {
        out.push(c.map_err(|e| format!("فشل قراءة بنية جدول {}: {}", table, e))?);
    }
    Ok(out)
}

pub fn row_to_incoming(row: &rusqlite::Row<'_>) -> Result<Incoming, rusqlite::Error> {
    Ok(Incoming {
        id: row.get(0)?,
        registration_number: row.get(1)?,
        correspondence_number: row.get(2)?,
        date: row.get(3)?,
        arrival_date: row.get(4)?,
        subject: row.get(5)?,
        sender: row.get(6)?,
        destination_service: row.get(7)?,
        source: row.get(8)?,
        notes: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        deleted_at: row.get(12)?,
        created_by: row.get(13)?,
        sync_version: row.get(14)?,
        file_name: row.get(15)?,
        file_path: row.get(16)?,
        is_duplicate: row.get::<_, bool>(17)?,
    })
}

pub fn row_to_outgoing(row: &rusqlite::Row<'_>) -> Result<Outgoing, rusqlite::Error> {
    Ok(Outgoing {
        id: row.get(0)?,
        registration_number: row.get(1)?,
        correspondence_number: row.get(2)?,
        date: row.get(3)?,
        subject: row.get(4)?,
        recipient: row.get(5)?,
        destination_service: row.get(6)?,
        source: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        deleted_at: row.get(11)?,
        created_by: row.get(12)?,
        sync_version: row.get(13)?,
        file_name: row.get(14)?,
        file_path: row.get(15)?,
        file_name_in: row.get(16)?,
        file_path_in: row.get(17)?,
    })
}

pub fn row_to_user(row: &rusqlite::Row<'_>) -> Result<User, rusqlite::Error> {
    Ok(User {
        id: row.get(0)?,
        full_name: row.get(1)?,
        email: row.get(2)?,
        role: row.get(3)?,
        blocked: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        deleted_at: row.get(7)?,
        sync_version: row.get(8)?,
    })
}

pub fn row_to_user_internal(row: &rusqlite::Row<'_>) -> Result<UserInternal, rusqlite::Error> {
    Ok(UserInternal {
        id: row.get(0)?,
        full_name: row.get(1)?,
        email: row.get(2)?,
        role: row.get(3)?,
        blocked: row.get::<_, i64>(4)? != 0,
        password_hash: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        deleted_at: row.get(8)?,
        sync_version: row.get(9)?,
    })
}

pub fn row_to_audit_log(row: &rusqlite::Row<'_>) -> Result<AuditLog, rusqlite::Error> {
    Ok(AuditLog {
        id: row.get(0)?,
        user_id: row.get(1)?,
        action: row.get(2)?,
        entity: row.get(3)?,
        entity_id: row.get(4)?,
        timestamp: row.get(5)?,
        metadata: row.get(6)?,
    })
}

pub const INCOMING_COLUMNS: &str =
    "id, registration_number, correspondence_number, date, arrival_date, subject, sender, destination_service, source, notes, created_at, updated_at, deleted_at, created_by, sync_version, file_name, file_path, is_duplicate";

pub const OUTGOING_COLUMNS: &str =
    "id, registration_number, correspondence_number, date, subject, recipient, destination_service, source, notes, created_at, updated_at, deleted_at, created_by, sync_version, file_name, file_path, file_name_in, file_path_in";

pub const USER_COLUMNS: &str =
    "id, full_name, email, role, blocked, created_at, updated_at, deleted_at, sync_version";

pub const USER_INTERNAL_COLUMNS: &str =
    "id, full_name, email, role, blocked, password_hash, created_at, updated_at, deleted_at, sync_version";

pub const AUDIT_COLUMNS: &str =
    "id, user_id, action, entity, entity_id, timestamp, metadata";
