use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::Mutex,
};

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use simple_logger::SimpleLogger;

// The name of the application, used for module-level logging
const APP_NAME: &str = env!("CARGO_PKG_NAME");

// Custom logger that writes to a file
pub struct FileLogger {
    level: LevelFilter,
    module_level: LevelFilter,
    file: Option<Mutex<File>>,
    max_history: Option<usize>,
}

impl FileLogger {
    pub fn new(
        level: LevelFilter,
        module_level: LevelFilter,
        log_file_path: Option<PathBuf>,
        max_history: Option<usize>,
    ) -> Self {
        let file = log_file_path.as_ref().and_then(|path| {
            // Create parent directory if it doesn't exist
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("Failed to create log directory: {}", e);
                        return None;
                    }
                }
            }

            // Open log file
            match OpenOptions::new()
                .create(true)
                .truncate(false)
                .append(true)
                .read(true)
                .open(path)
            {
                Ok(file) => Some(Mutex::new(file)),
                Err(e) => {
                    eprintln!("Failed to open log file: {}", e);
                    None
                }
            }
        });

        let mut logger = Self {
            level,
            module_level,
            file,
            max_history,
        };

        // Trim log file if necessary
        if let Err(e) = logger.trim_log_file() {
            eprintln!("Failed to trim log file: {}", e);
        }

        logger
    }

    /// Trim the log file to keep only the maximum number of lines
    fn trim_log_file(&mut self) -> io::Result<()> {
        // If max_history is not set or file is not available, do nothing
        let (Some(max_history), Some(file_mutex)) = (self.max_history, self.file.as_ref()) else {
            return Ok(());
        };

        // Lock the file to perform operations
        let mut file = file_mutex
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock file mutex"))?;

        // Reset file cursor to the beginning
        file.seek(SeekFrom::Start(0))?;

        // Count the number of lines in the file
        let reader = BufReader::new(&*file);
        let line_count = reader.lines().count();

        // If the file has fewer lines than the limit, no need to trim
        if line_count <= max_history {
            // Reset cursor to the end for future appends
            file.seek(SeekFrom::End(0))?;
            return Ok(());
        }

        // We need to trim the file, so read all lines first
        file.seek(SeekFrom::Start(0))?;
        let mut reader = BufReader::new(&*file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        // Split the content into lines and keep only the most recent ones
        let lines: Vec<&str> = content.lines().collect();
        let lines_to_keep = &lines[line_count - max_history..];

        // Create the new content
        let new_content = lines_to_keep.join("\n") + "\n";

        // Truncate the file and write the new content
        file.seek(SeekFrom::Start(0))?;
        file.set_len(0)?;
        file.write_all(new_content.as_bytes())?;

        // Reset cursor to the end for future appends
        file.seek(SeekFrom::End(0))?;

        Ok(())
    }

    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.level.max(self.module_level));
        log::set_boxed_logger(Box::new(self))
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let level = if metadata.target().starts_with(APP_NAME) {
            self.module_level
        } else {
            self.level
        };

        metadata.level() <= level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(file) = &self.file {
            if let Ok(mut file) = file.lock() {
                let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let _ = writeln!(
                    file,
                    "[{}] [{}] [{}] {}",
                    timestamp,
                    record.level(),
                    record.target(),
                    record.args()
                );
            }
        } else {
            // Fallback to standard logger behavior if file is not available
            println!(
                "[{}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file {
            if let Ok(mut file) = file.lock() {
                let _ = file.flush();
            }
        }
    }
}

/// Initialize the logger based on configuration
///
/// If a log file path is provided, logs will be written to that file.
/// Otherwise, logs will be written to stdout.
pub fn init_logger(
    verbose_level: u8,
    log_file_path: Option<PathBuf>,
    max_history: usize,
) -> Result<(), String> {
    // Set up logging
    let log_level = match verbose_level {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    // Configure logger based on config
    if let Some(log_file_path) = log_file_path {
        // Initialize file logger
        FileLogger::new(
            LevelFilter::Info,
            log_level,
            Some(log_file_path),
            if max_history > 0 {
                Some(max_history)
            } else {
                None
            },
        )
        .init()
        .map_err(|e| format!("Failed to initialize logger: {}", e))
    } else {
        // Fallback to stdout logging if no log file is configured
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .with_module_level(APP_NAME, log_level)
            .without_timestamps()
            .init()
            .map_err(|e| format!("Failed to initialize logger: {}", e))
    }
}
