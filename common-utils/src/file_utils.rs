//! # File utils
//!
//! This module contains structures and functionality related to filenames and paths
//! associated with processing Safari log files.
//!

use std::path::PathBuf;

use crate::date_utils::DateStamp;

/// Struct representing the metadata (date and path) of a session log file that has yet to be processed.
pub struct UnprocessedSessionLogFile {
    /// The date of the log file. Must be of the form YYYYMMDD
    pub date: DateStamp,
    /// The path to the session log file
    pub path: PathBuf,
}
impl UnprocessedSessionLogFile {
    pub fn new(date: DateStamp, path: PathBuf) -> Self {
        Self { date, path }
    }
}

/// Struct providing settings for filenames of session log files to be processed
pub struct SessionLogFilesConfig;

impl SessionLogFilesConfig {
    /// The prefix for the log files containing the session data that is to be processed on a daily basis.
    pub const DAILY_SESSIONS_PREFIX: &'static str = "safari-sessions-";
    /// The file extension for the daily log files.
    pub const DAILY_SESSIONS_EXTENSION: &'static str = ".log";
}

/// Provides the metadata (date, and path) of todays session log file.
pub fn todays_file_for_processing(session_directory: PathBuf) -> UnprocessedSessionLogFile {
    let today_ymd = crate::date_utils::today_ymd().into_string();
    let mut todays_sessions_path = session_directory;
    let filename: String = [
        SessionLogFilesConfig::DAILY_SESSIONS_PREFIX,
        today_ymd.as_str(),
        SessionLogFilesConfig::DAILY_SESSIONS_EXTENSION,
    ]
    .iter()
    .flat_map(|s| s.chars())
    .collect();
    todays_sessions_path.push(filename);

    UnprocessedSessionLogFile::new(DateStamp::from_ymd(today_ymd), todays_sessions_path)
}

/// Provides a vector of unprocessed session log files produced within the last seven days.
/// If any of these log files do not exist in the specified directory an Error is placed
/// at the corresponding position(s) in the vector.
pub fn unprocessed_session_log_files<F: (Fn(DateStamp) -> bool)>(
    // The directory where daily session log files are kept
    session_log_files_directory: PathBuf,
    // a closure determining whether the log file of the corresponding date has been processed.
    processed_on_date: F,
) -> Vec<Result<UnprocessedSessionLogFile, std::io::Error>> {
    let mut unprocessed_log_files: Vec<Result<UnprocessedSessionLogFile, std::io::Error>> =
        Vec::new();
    for day in crate::date_utils::last_seven_days_ymd() {
        if !processed_on_date(day.clone()) {
            let mut log_filename = SessionLogFilesConfig::DAILY_SESSIONS_PREFIX.to_string();
            log_filename.push_str(day.clone().into_string().as_str());
            log_filename.push_str(SessionLogFilesConfig::DAILY_SESSIONS_EXTENSION);
            let mut log_file_path = session_log_files_directory.clone();
            log_file_path.push(log_filename);
            if !log_file_path.exists() {
                let error = std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "The unprocessed session log file: {:?} could not be found.",
                        log_file_path.as_os_str()
                    ),
                );

                unprocessed_log_files.push(Err(error));
            }
            unprocessed_log_files.push(Ok(UnprocessedSessionLogFile::new(day, log_file_path)));
        }
    }
    unprocessed_log_files
}
