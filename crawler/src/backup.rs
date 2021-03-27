use crate::config::BackupConfig;
use std::fs::*;
use std::time::UNIX_EPOCH;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use std::io::prelude::*;

pub fn backup(database_filename: &str, backup_config: &BackupConfig) {
    let folder = match read_dir(&backup_config.folder) {
        Ok(folder) => folder,
        Err(_) => {
            let _ = create_dir(&backup_config.folder);
            read_dir(&backup_config.folder).expect("Failed to read the backup folder")
        }
    };

    let mut backups: Vec<(PathBuf, u64)> = Vec::new();
    for entry in folder.filter_map(|e| e.ok()) {
        // Is a file
        if entry.file_type().map(|t| t.is_file()).ok() != Some(true) {
            continue;
        }

        let path = entry.path();
        let modified = match entry.metadata() {
            Ok(metadata) => match metadata.modified() {
                Ok(time) => time.duration_since(UNIX_EPOCH).unwrap().as_secs(),
                Err(e) => {
                    eprintln!("Unable to access modification date of the file {:?}: {}", path, e);
                    continue;
                }
            },
            Err(e) => {
                eprintln!("Unable to access metadata of the file {:?}: {}", path, e);
                continue;
            }
        };

        let filename: String = match entry.file_name().into_string() {
            Ok(filename) => filename,
            Err(e) => {
                eprintln!("Unable to convert {:?} to string", e);
                continue;
            }
        };

        if filename.starts_with("crawler_backup_") && filename.ends_with(".json") {
            backups.push((path, modified))
        }
    }

    if backups.len() >= backup_config.max {
        backups.sort_unstable_by_key(|(_p, t)| *t);
        while backups.len() >= backup_config.max {
            let (path, _t) = backups.remove(0);
            let _ = remove_file(path);
        }
    }

    let now: DateTime<Utc> = Utc::now();
    let path = format!("{}/crawler_backup_{}.json", backup_config.folder, now.format("%R-%d-%b-%C"));

    let mut file = File::create(&path).expect("Unable to create backup file");
    file.write_all(b"This data will disappear...").unwrap();

    copy(database_filename, path).unwrap();
}