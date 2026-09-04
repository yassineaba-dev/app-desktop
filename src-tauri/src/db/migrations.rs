use super::connection::DatabaseConnection;

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "001_initial_schema", include_str!("../../migrations/001_initial_schema.sql")),
    (2, "002_add_indexes", include_str!("../../migrations/002_add_indexes.sql")),
    (3, "003_add_password_hash", include_str!("../../migrations/003_add_password_hash.sql")),
    (4, "004_password_resets", include_str!("../../migrations/004_password_resets.sql")),
    (5, "005_add_arrival_date", include_str!("../../migrations/005_add_arrival_date.sql")),
    (6, "006_add_blocked", include_str!("../../migrations/006_add_blocked.sql")),
    (7, "007_add_incoming_file", include_str!("../../migrations/007_add_incoming_file.sql")),
    (8, "008_add_outgoing_file", include_str!("../../migrations/008_add_outgoing_file.sql")),
    (9, "009_add_outgoing_in_file", include_str!("../../migrations/009_add_outgoing_in_file.sql")),
    (10, "010_add_settings_table", include_str!("../../migrations/010_add_settings_table.sql")),
    (11, "011_add_incoming_duplicate", include_str!("../../migrations/011_add_incoming_duplicate.sql")),
];

pub fn run_migrations(db: &DatabaseConnection) -> Result<(), String> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("Failed to create migration table: {}", e))?;

    for &(version, name, sql) in MIGRATIONS {
        let applied: bool = db
            .query_row(
                "SELECT COUNT(*) > 0 FROM schema_migrations WHERE version = ?1",
                &[&version],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !applied {
            log::info!("Applying migration {}: {}", version, name);
            db.execute_batch(sql)
                .map_err(|e| format!("Migration {} failed: {}", name, e))?;
            db.execute(
                "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
                &[&version, &name],
            )
            .map_err(|e| format!("Failed to record migration {}: {}", name, e))?;
            log::info!("Migration {} applied successfully", name);
        }
    }

    log::info!("All migrations completed");
    Ok(())
}
