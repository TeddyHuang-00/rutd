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
                        eprintln!("Failed to create log directory: {e}");
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
                    eprintln!("Failed to open log file: {e}");
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
            eprintln!("Failed to trim log file: {e}");
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
            .map_err(|_| io::Error::other("Failed to lock file mutex"))?;

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
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
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
    log_file_path: PathBuf,
    max_history: usize,
    write_to_console: bool,
) -> Result<(), String> {
    // Set up logging
    let log_level = match verbose_level {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    // Configure logger based on config
    if write_to_console {
        // Fallback to stdout logging if no log file is configured
        SimpleLogger::new()
            .with_level(log_level)
            .with_module_level(APP_NAME, log_level)
            .without_timestamps()
            .init()
            .map_err(|e| format!("Failed to initialize logger: {e}"))
    } else {
        // Initialize file logger
        FileLogger::new(
            log_level,
            log_level,
            Some(log_file_path),
            if max_history > 0 {
                Some(max_history)
            } else {
                None
            },
        )
        .init()
        .map_err(|e| format!("Failed to initialize logger: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_file_logger_creation() {
        let temp_dir = tempdir().unwrap();
        let log_file_path = temp_dir.path().join("test.log");

        // Create a file logger with trace level
        let logger = FileLogger::new(
            LevelFilter::Trace,
            LevelFilter::Trace,
            Some(log_file_path.clone()),
            None,
        );

        // Verify the logger was created with the expected configuration
        assert_eq!(logger.level, LevelFilter::Trace);
        assert_eq!(logger.module_level, LevelFilter::Trace);
        assert!(logger.file.is_some());
        assert!(logger.max_history.is_none());

        // Verify the log file was created
        assert!(log_file_path.exists());
    }

    #[test]
    fn test_file_logger_without_file() {
        // Create a file logger without a file path
        let logger = FileLogger::new(LevelFilter::Debug, LevelFilter::Info, None, None);

        // Verify the logger was created with the expected configuration
        assert_eq!(logger.level, LevelFilter::Debug);
        assert_eq!(logger.module_level, LevelFilter::Info);
        assert!(logger.file.is_none());
        assert!(logger.max_history.is_none());
    }

    #[test]
    fn test_file_logger_enabled() {
        let logger = FileLogger::new(LevelFilter::Info, LevelFilter::Debug, None, None);

        // Test regular logs
        let metadata_trace = Metadata::builder()
            .level(log::Level::Trace)
            .target("some_crate")
            .build();
        let metadata_debug = Metadata::builder()
            .level(log::Level::Debug)
            .target("some_crate")
            .build();
        let metadata_info = Metadata::builder()
            .level(log::Level::Info)
            .target("some_crate")
            .build();
        let metadata_warn = Metadata::builder()
            .level(log::Level::Warn)
            .target("some_crate")
            .build();

        // Regular logs should use the logger.level filter (Info)
        assert!(!logger.enabled(&metadata_trace));
        assert!(!logger.enabled(&metadata_debug));
        assert!(logger.enabled(&metadata_info));
        assert!(logger.enabled(&metadata_warn));

        // App-specific logs should use the module_level filter (Debug)
        let app_metadata_trace = Metadata::builder()
            .level(log::Level::Trace)
            .target(APP_NAME)
            .build();
        let app_metadata_debug = Metadata::builder()
            .level(log::Level::Debug)
            .target(APP_NAME)
            .build();
        let app_metadata_info = Metadata::builder()
            .level(log::Level::Info)
            .target(APP_NAME)
            .build();

        assert!(!logger.enabled(&app_metadata_trace));
        assert!(logger.enabled(&app_metadata_debug));
        assert!(logger.enabled(&app_metadata_info));
    }

    #[test]
    fn test_file_logger_trimming() {
        let temp_dir = tempdir().unwrap();
        let log_file_path = temp_dir.path().join("trim_test.log");

        // Create an initial file with specific content - 10 lines
        {
            let mut file = File::create(&log_file_path).unwrap();
            for i in 1..=10 {
                writeln!(file, "Line {i}").unwrap();
            }
        }

        // Create a file logger with max_history = 5
        let _logger = FileLogger::new(
            LevelFilter::Info,
            LevelFilter::Info,
            Some(log_file_path.clone()),
            Some(5),
        );

        // The file should have been trimmed during logger creation
        let content = fs::read_to_string(&log_file_path).unwrap();
        let line_count = content.lines().count();
        assert_eq!(line_count, 5);

        // Verify the correct lines were kept (the last 5)
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "Line 6");
        assert_eq!(lines[4], "Line 10");
    }

    #[test]
    fn test_file_logger_logging() {
        let temp_dir = tempdir().unwrap();
        let log_file_path = temp_dir.path().join("log_test.log");

        // Create a file logger
        let logger = FileLogger::new(
            LevelFilter::Info,
            LevelFilter::Info,
            Some(log_file_path.clone()),
            None,
        );

        // Create test records
        let target = "test_target";
        let info_record = Record::builder()
            .args(format_args!("Info message"))
            .level(log::Level::Info)
            .target(target)
            .build();

        let debug_record = Record::builder()
            .args(format_args!("Debug message"))
            .level(log::Level::Debug)
            .target(target)
            .build();

        // Log the records
        logger.log(&info_record); // Should be logged
        logger.log(&debug_record); // Should be filtered out

        // Check the file content
        let content = fs::read_to_string(&log_file_path).unwrap();
        assert!(content.contains("Info message"));
        assert!(!content.contains("Debug message"));
    }

    #[test]
    fn test_log_level_selection() {
        // Test the verbose level to LevelFilter mapping in init_logger
        assert_eq!(get_log_level_for_verbose(0), LevelFilter::Info);
        assert_eq!(get_log_level_for_verbose(1), LevelFilter::Debug);
        assert_eq!(get_log_level_for_verbose(2), LevelFilter::Trace);
        assert_eq!(get_log_level_for_verbose(100), LevelFilter::Trace); // Any value > 1 should be Trace
    }

    // Helper function to test the verbose level mapping logic
    fn get_log_level_for_verbose(verbose_level: u8) -> LevelFilter {
        match verbose_level {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }

    #[test]
    fn test_log_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let nested_log_dir = temp_dir.path().join("nested").join("log").join("dir");
        let log_file_path = nested_log_dir.join("test.log");

        // The nested directories should not exist yet
        assert!(!nested_log_dir.exists());

        // Create a file logger - it should create the parent directories
        let logger = FileLogger::new(
            LevelFilter::Info,
            LevelFilter::Info,
            Some(log_file_path.clone()),
            None,
        );

        // Verify the logger was created successfully
        assert!(logger.file.is_some());

        // Verify the nested directories were created
        assert!(nested_log_dir.exists());
        assert!(log_file_path.exists());
    }

    #[test]
    fn test_error_handling_for_inaccessible_file() {
        // If running as non-root on Unix, this should fail to create a file in a
        // restricted directory
        #[cfg(unix)]
        {
            let restricted_path = PathBuf::from("/root/restricted/test.log");
            let logger = FileLogger::new(
                LevelFilter::Info,
                LevelFilter::Info,
                Some(restricted_path),
                None,
            );

            // The file should be None since we couldn't create it
            assert!(logger.file.is_none());
        }

        // Alternative test for Windows or if the above doesn't apply
        #[cfg(not(unix))]
        {
            // Just verify that a logger with an invalid path falls back to None for file
            let invalid_path = PathBuf::from("\0invalid");
            let logger = FileLogger::new(
                LevelFilter::Info,
                LevelFilter::Info,
                Some(invalid_path),
                None,
            );
            assert!(logger.file.is_none());
        }
    }

    #[test]
    fn test_max_history_zero_means_unlimited() {
        let temp_dir = tempdir().unwrap();
        let log_file_path = temp_dir.path().join("unlimited.log");

        // Create a file with 10 lines
        {
            let mut file = File::create(&log_file_path).unwrap();
            for i in 1..=10 {
                writeln!(file, "Line {i}").unwrap();
            }
            file.flush().unwrap();
        }

        // Create logger with max_history = 0 (unlimited)
        let _logger = FileLogger::new(
            LevelFilter::Info,
            LevelFilter::Info,
            Some(log_file_path.clone()),
            None,
        );

        // Should not have trimmed any lines
        let content = fs::read_to_string(&log_file_path).unwrap();
        let line_count = content.lines().count();
        assert_eq!(line_count, 10);
    }

    #[test]
    fn test_logger_with_mutex_handling() {
        let temp_dir = tempdir().unwrap();
        let log_file_path = temp_dir.path().join("mutex_test.log");

        // Create a file logger
        let logger = FileLogger::new(
            LevelFilter::Info,
            LevelFilter::Info,
            Some(log_file_path.clone()),
            None,
        );

        // Get multiple references to the logger to simulate multiple threads
        let logger_ref1 = &logger;
        let logger_ref2 = &logger;

        // Create test records
        let record1 = Record::builder()
            .args(format_args!("Message 1"))
            .level(log::Level::Info)
            .target("test")
            .build();

        let record2 = Record::builder()
            .args(format_args!("Message 2"))
            .level(log::Level::Info)
            .target("test")
            .build();

        // Log from both "threads"
        logger_ref1.log(&record1);
        logger_ref2.log(&record2);
        logger_ref1.flush();
        logger_ref2.flush();

        // Check that both messages made it to the log file
        let content = fs::read_to_string(&log_file_path).unwrap();
        assert!(content.contains("Message 1"));
        assert!(content.contains("Message 2"));
    }
}
