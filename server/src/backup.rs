use chrono::{DateTime, Utc};
use log::{error, info};
use std::fs;
use std::path::PathBuf;
use tokio::fs as tokio_fs;
use serde::{Serialize, Deserialize};
use std::fmt;
use std::io::Read;
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use tar::{Archive, Builder};

fn get_backup_dir() -> PathBuf {
    let dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("backups");
    
    // Ensure the directory exists
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .expect("Failed to create backups directory");
        info!("Created backups directory at: {}", dir.display());
    }
    
    dir
}

fn get_database_path() -> PathBuf {
    std::env::current_dir()
        .expect("Failed to get current directory")
        .join("database")
        .join("swolemate.db")
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
    let backup_dir = get_backup_dir();
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

    // Add database file to archive
    let db_path = get_database_path();
    let db_content = fs::read(&db_path)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(db_content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive.append_data(&mut header, "database.db", &db_content[..])?;

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
    let backup_path = get_backup_dir().join(filename);
    let db_path = get_database_path();
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

    // Extract database file to temporary location first
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.to_string_lossy() == "database.db" {
            entry.unpack(&temp_new_db)?;
            break;
        }
    }

    // Remove existing WAL and SHM files to ensure clean state
    if wal_path.exists() {
        tokio_fs::remove_file(&wal_path).await?;
    }
    if shm_path.exists() {
        tokio_fs::remove_file(&shm_path).await?;
    }

    // Now safely move the new database file into place
    match fs::rename(&temp_new_db, &db_path) {
        Ok(_) => {
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
            let _ = tokio_fs::remove_dir(&temp_dir).await;

            Err(e)
        }
    }
}

pub async fn list_backups() -> Result<Vec<BackupInfo>, std::io::Error> {
    let backup_dir = get_backup_dir();
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
                                        if let Ok(backup_info) = serde_json::from_str(&metadata_content) {
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
            let path = get_backup_dir().join(&backup.filename);
            if path.exists() {
                tokio_fs::remove_file(path).await?;
                info!("Removed old backup: {}", backup.filename);
            }
        }
    }

    Ok(())
}

pub async fn delete_backup(filename: &str) -> Result<(), std::io::Error> {
    let backup_path = get_backup_dir().join(filename);

    // Delete the backup file
    if backup_path.exists() {
        tokio_fs::remove_file(&backup_path).await?;
        info!("Deleted backup file: {}", filename);
    }

    Ok(())
} 
