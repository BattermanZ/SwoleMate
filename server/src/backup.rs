use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, SqliteConnection};
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use tar::{Archive, Builder};
use tokio::fs as tokio_fs;

fn get_backup_dir() -> Result<PathBuf, std::io::Error> {
    let dir = std::env::current_dir()?.join("backups");

    // Ensure the directory exists
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
        info!("Created backups directory at: {}", dir.display());
    }

    Ok(dir)
}

fn get_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:database/swolemate.db".to_string())
}

fn get_database_path() -> Result<PathBuf, std::io::Error> {
    let database_url = get_database_url();
    let path = database_url
        .strip_prefix("sqlite:")
        .unwrap_or("database/swolemate.db")
        .trim_start_matches("//");

    Ok(std::env::current_dir()?.join(path))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupInfo {
    pub filename: String,
    pub created_at: DateTime<Utc>,
    pub backup_type: BackupType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum BackupType {
    Auto,
    Manual,
}

impl fmt::Display for BackupType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackupType::Auto => write!(f, "auto"),
            BackupType::Manual => write!(f, "manual"),
        }
    }
}

pub async fn create_backup(backup_type: BackupType) -> Result<BackupInfo, std::io::Error> {
    let backup_dir = get_backup_dir()?;
    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir)?;
    }

    let now = Utc::now();
    let filename = format!(
        "swolemate_backup_{}_{}_{}.tar.gz",
        now.format("%Y%m%d_%H%M%S"),
        backup_type,
        now.timestamp()
    );
    let backup_path = backup_dir.join(&filename);

    let backup_info = BackupInfo {
        filename: filename.clone(),
        created_at: now,
        backup_type,
    };

    // Create tar.gz file
    let file = fs::File::create(&backup_path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut archive = Builder::new(encoder);

    // Add database file (and WAL/SHM if needed) to archive
    let db_path = get_database_path()?;
    let snapshot_path = backup_dir.join(format!("swolemate_snapshot_{}.db", now.timestamp()));
    let wal_path = db_path.with_extension("db-wal");
    let shm_path = db_path.with_extension("db-shm");

    let (db_content, wal_content, shm_content) = match SqliteConnection::connect(&get_database_url()).await {
        Ok(mut conn) => {
            // Ensure any WAL content is merged before snapshotting so the backup reflects the latest writes.
            let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
                .execute(&mut conn)
                .await;

            let snapshot_path_sql = snapshot_path.to_string_lossy().replace('\'', "''");
            let vacuum_sql = format!("VACUUM INTO '{}'", snapshot_path_sql);

            match sqlx::query(&vacuum_sql).execute(&mut conn).await {
                Ok(_) => {
                    let content = fs::read(&snapshot_path)?;
                    let _ = fs::remove_file(&snapshot_path);
                    (content, None, None)
                }
                Err(e) => {
                    error!(
                        "Failed to create consistent DB snapshot, falling back to direct copy: {}",
                        e
                    );
                    let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
                        .execute(&mut conn)
                        .await;

                    let main = fs::read(&db_path)?;
                    let wal = if wal_path.exists() {
                        Some(fs::read(&wal_path)?)
                    } else {
                        None
                    };
                    let shm = if shm_path.exists() {
                        Some(fs::read(&shm_path)?)
                    } else {
                        None
                    };
                    (main, wal, shm)
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to open DB connection for snapshot, falling back to direct copy: {}",
                e
            );
            let main = fs::read(&db_path)?;
            let wal = if wal_path.exists() {
                Some(fs::read(&wal_path)?)
            } else {
                None
            };
            let shm = if shm_path.exists() {
                Some(fs::read(&shm_path)?)
            } else {
                None
            };
            (main, wal, shm)
        }
    };

    let mut header = tar::Header::new_gnu();
    header.set_size(db_content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive.append_data(&mut header, "database.db", &db_content[..])?;

    if let Some(wal_content) = wal_content {
        let mut header = tar::Header::new_gnu();
        header.set_size(wal_content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append_data(&mut header, "database.db-wal", &wal_content[..])?;
    }

    if let Some(shm_content) = shm_content {
        let mut header = tar::Header::new_gnu();
        header.set_size(shm_content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append_data(&mut header, "database.db-shm", &shm_content[..])?;
    }

    // Add metadata to archive
    let metadata = serde_json::to_string_pretty(&backup_info)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(metadata.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive.append_data(&mut header, "metadata.json", metadata.as_bytes())?;

    // Finish the archive
    archive.finish()?;

    // Clean up old backups
    cleanup_old_backups().await?;

    info!("Created {} backup: {}", backup_type, backup_info.filename);
    Ok(backup_info)
}

pub async fn restore_backup(filename: &str) -> Result<(), std::io::Error> {
    let backup_path = get_backup_dir()?.join(filename);
    let db_path = get_database_path()?;
    let wal_path = db_path.with_extension("db-wal");
    let shm_path = db_path.with_extension("db-shm");

    // Verify backup exists
    if !backup_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Backup file not found",
        ));
    }

    // Create a temporary backup of the current database and its WAL files
    let timestamp = Utc::now().timestamp();
    let temp_dir = std::env::current_dir()?.join("database").join("temp");
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir)?;
    }

    let temp_backup = temp_dir.join(format!("swolemate_temp_{}.db", timestamp));
    let temp_wal = temp_dir.join(format!("swolemate_temp_{}.db-wal", timestamp));
    let temp_shm = temp_dir.join(format!("swolemate_temp_{}.db-shm", timestamp));
    let temp_new_db = temp_dir.join(format!("swolemate_new_{}.db", timestamp));
    let temp_new_wal = temp_dir.join(format!("swolemate_new_{}.db-wal", timestamp));
    let temp_new_shm = temp_dir.join(format!("swolemate_new_{}.db-shm", timestamp));

    // Backup current files if they exist
    if db_path.exists() {
        tokio_fs::copy(&db_path, &temp_backup).await?;
    }
    if wal_path.exists() {
        tokio_fs::copy(&wal_path, &temp_wal).await?;
    }
    if shm_path.exists() {
        tokio_fs::copy(&shm_path, &temp_shm).await?;
    }

    // Extract and restore the backup
    let file = fs::File::open(&backup_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    // Extract database files to temporary location first
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        match path.to_string_lossy().as_ref() {
            "database.db" => {
                entry.unpack(&temp_new_db)?;
            }
            "database.db-wal" => {
                entry.unpack(&temp_new_wal)?;
            }
            "database.db-shm" => {
                entry.unpack(&temp_new_shm)?;
            }
            _ => {}
        }
    }

    // Remove existing WAL and SHM files to ensure clean state
    if wal_path.exists() {
        tokio_fs::remove_file(&wal_path).await?;
    }
    if shm_path.exists() {
        tokio_fs::remove_file(&shm_path).await?;
    }

    // Now safely move the new database files into place
    match fs::rename(&temp_new_db, &db_path) {
        Ok(_) => {
            if temp_new_wal.exists() {
                let _ = fs::rename(&temp_new_wal, &wal_path);
            }
            if temp_new_shm.exists() {
                let _ = fs::rename(&temp_new_shm, &shm_path);
            }

            // If successful, clean up all temporary files
            let _ = tokio_fs::remove_file(&temp_backup).await;
            let _ = tokio_fs::remove_file(&temp_wal).await;
            let _ = tokio_fs::remove_file(&temp_shm).await;
            let _ = tokio_fs::remove_dir(&temp_dir).await;
            info!("Successfully restored backup: {}", filename);
            Ok(())
        }
        Err(e) => {
            error!("Failed to move new database file: {}", e);
            // If failed, try to restore the temporary backup and WAL files
            if temp_backup.exists() {
                let _ = tokio_fs::copy(&temp_backup, &db_path).await;
            }
            if temp_wal.exists() {
                let _ = tokio_fs::copy(&temp_wal, &wal_path).await;
            }
            if temp_shm.exists() {
                let _ = tokio_fs::copy(&temp_shm, &shm_path).await;
            }

            // Clean up temporary files
            let _ = tokio_fs::remove_file(&temp_backup).await;
            let _ = tokio_fs::remove_file(&temp_wal).await;
            let _ = tokio_fs::remove_file(&temp_shm).await;
            let _ = tokio_fs::remove_file(&temp_new_db).await;
            let _ = tokio_fs::remove_file(&temp_new_wal).await;
            let _ = tokio_fs::remove_file(&temp_new_shm).await;
            let _ = tokio_fs::remove_dir(&temp_dir).await;

            Err(e)
        }
    }
}

pub async fn list_backups() -> Result<Vec<BackupInfo>, std::io::Error> {
    let backup_dir = get_backup_dir()?;
    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir)?;
        info!("Created backups directory at: {}", backup_dir.display());
        return Ok(Vec::new());
    }

    let mut backups: Vec<BackupInfo> = Vec::new();
    let mut entries = tokio_fs::read_dir(&backup_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.ends_with(".tar.gz") {
                if let Ok(file) = fs::File::open(entry.path()) {
                    let decoder = GzDecoder::new(file);
                    let mut archive = Archive::new(decoder);
                    if let Ok(entries) = archive.entries() {
                        for mut entry in entries.flatten() {
                            if let Ok(path) = entry.path() {
                                if path.to_string_lossy() == "metadata.json" {
                                    let mut metadata_content = String::new();
                                    if entry.read_to_string(&mut metadata_content).is_ok() {
                                        if let Ok(backup_info) =
                                            serde_json::from_str(&metadata_content)
                                        {
                                            backups.push(backup_info);
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(backups)
}

async fn cleanup_old_backups() -> Result<(), std::io::Error> {
    let mut backups = list_backups().await?;
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Separate auto and manual backups
    let auto_backups: Vec<_> = backups
        .iter()
        .filter(|b| b.backup_type == BackupType::Auto)
        .collect();

    // Keep only the last 4 auto backups
    if auto_backups.len() > 4 {
        for backup in auto_backups.iter().skip(4) {
            let path = get_backup_dir()?.join(&backup.filename);
            if path.exists() {
                tokio_fs::remove_file(path).await?;
                info!("Removed old backup: {}", backup.filename);
            }
        }
    }

    Ok(())
}

pub async fn delete_backup(filename: &str) -> Result<(), std::io::Error> {
    let backup_path = get_backup_dir()?.join(filename);

    // Delete the backup file
    if backup_path.exists() {
        tokio_fs::remove_file(&backup_path).await?;
        info!("Deleted backup file: {}", filename);
    }

    Ok(())
}
