// DB — SQLite 연결 및 마이그레이션
// REQ-NF-04: 스키마 버전 추적, 자동 마이그레이션, 실패 시 백업 보존

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub mod models;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// 새 SQLite 연결 생성
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database at {:?}", path))?;

        // WAL 모드 활성화 (성능)
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    /// 스키마 마이그레이션 실행
    pub fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB lock poisoned: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        let current_version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0))
            .unwrap_or(0);

        tracing::info!("Current DB schema version: {}", current_version);

        let migrations: Vec<(i32, &str)> = vec![(1, include_str!("../../migrations/001_init.sql"))];

        for (version, sql) in migrations {
            if version > current_version {
                tracing::info!("Applying migration v{}", version);

                let tx = conn.unchecked_transaction()?;
                tx.execute_batch(sql)
                    .with_context(|| format!("Migration v{} failed", version))?;
                tx.execute(
                    "INSERT INTO schema_version (version) VALUES (?1)",
                    rusqlite::params![version],
                )?;
                tx.commit()?;

                tracing::info!("Migration v{} applied successfully", version);
            }
        }

        Ok(())
    }

    /// DB 백업 (마이그레이션 실패 시 복구용)
    pub fn backup(&self, backup_path: &Path) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB lock poisoned: {}", e))?;
        let mut backup_conn = Connection::open(backup_path)?;
        {
            let _backup = rusqlite::backup::Backup::new(&conn, &mut backup_conn)?;
            _backup.step(-1)?;
        }
        tracing::info!("DB backed up to {:?}", backup_path);
        Ok(())
    }
}