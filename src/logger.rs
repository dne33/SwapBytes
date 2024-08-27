use simplelog::*;
use std::fs::{File, rename};
use std::path::{Path, PathBuf};
use std::io::Error;

pub fn initialize() {
    let log_file_path = "Logs/app.log";
    let backup_log_file_prefix = "Logs/app_backup";
    let mut backup_log_file_path = PathBuf::from(log_file_path);

    // Check if the log file already exists
    if Path::new(log_file_path).exists() {
        // Find an available backup file name
        let mut counter = 1;
        loop {
            backup_log_file_path = PathBuf::from(format!("{}_{:02}.log", backup_log_file_prefix, counter));
            if !backup_log_file_path.exists() {
                break;
            }
            counter += 1;
        }

        // Rename the existing log file to avoid overlap
        if let Err(e) = rename(log_file_path, &backup_log_file_path) {
            panic!("Failed to rename existing log file: {}", e);
        }
    }

    // Initialize the logger
    CombinedLogger::init(vec![
        // Write logs to a file
        WriteLogger::new(
            LevelFilter::Info, // Set the logging level
            Config::default(),
            File::create(log_file_path).unwrap(), // The log file
        ),
    ])
    .unwrap();
}

pub use log::{debug, error, info};
