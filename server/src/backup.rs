use chrono::{DateTime, Utc};
use log::{error, info};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;
use serde::{Serialize, Deserialize};
use std::fmt;
use std::io::{Read, Write};
use zip::{ZipWriter, write::FileOptions};

fn get_backup_dir() -> PathBuf {
    std::env::current_dir()
        .expect("Failed to get current directory")
        .join("backups")
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
        "swolemate_backup_{}_{}_{}.zip",
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

    // Create zip file
    let file = fs::File::create(&backup_path)?;
    let mut zip = ZipWriter::new(file);

    // Add database file to zip
    let db_content = fs::read(get_database_path())?;
    zip.start_file("database.db", FileOptions::default().compression_method(zip::CompressionMethod::Deflated))?;
    zip.write_all(&db_content)?;

    // Add metadata to zip
    let metadata = serde_json::to_string_pretty(&backup_info)?;
    zip.start_file("metadata.json", FileOptions::default().compression_method(zip::CompressionMethod::Deflated))?;
    zip.write_all(metadata.as_bytes())?;

    // Finish zip file
    zip.finish()?;

    // Clean up old backups
    cleanup_old_backups().await?;

    info!("Created {} backup: {}", backup_type, backup_info.filename);
    Ok(backup_info)
}

pub async fn restore_backup(filename: &str) -> Result<(), std::io::Error> {
    let backup_path = get_backup_dir().join(filename);
    let db_path = get_database_path();

    // Verify backup exists
    if !backup_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Backup file not found",
        ));
    }

    // Create a temporary backup of the current database
    let temp_backup = format!("database/swolemate_temp_{}.db", Utc::now().timestamp());
    tokio_fs::copy(&db_path, &temp_backup).await?;

    // Extract and restore the backup
    let file = fs::File::open(&backup_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Extract database file
    let mut db_file = archive.by_name("database.db")?;
    let mut db_content = Vec::new();
    db_file.read_to_end(&mut db_content)?;

    // Write the database file
    match fs::write(&db_path, db_content) {
        Ok(_) => {
            // If successful, remove the temporary backup
            tokio_fs::remove_file(&temp_backup).await?;
            info!("Successfully restored backup: {}", filename);
            Ok(())
        }
        Err(e) => {
            // If failed, try to restore the temporary backup
            tokio_fs::copy(&temp_backup, &db_path).await?;
            tokio_fs::remove_file(&temp_backup).await?;
            error!("Failed to restore backup: {}", e);
            Err(e)
        }
    }
}

pub async fn list_backups() -> Result<Vec<BackupInfo>, std::io::Error> {
    let backup_dir = get_backup_dir();
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups: Vec<BackupInfo> = Vec::new();
    let mut entries = tokio_fs::read_dir(&backup_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.ends_with(".zip") {
                if let Ok(file) = fs::File::open(entry.path()) {
                    if let Ok(mut archive) = zip::ZipArchive::new(file) {
                        if let Ok(mut metadata_file) = archive.by_name("metadata.json") {
                            let mut metadata_content = String::new();
                            if metadata_file.read_to_string(&mut metadata_content).is_ok() {
                                if let Ok(backup_info) = serde_json::from_str(&metadata_content) {
                                    backups.push(backup_info);
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