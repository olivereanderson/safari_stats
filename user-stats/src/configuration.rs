use std::{path::PathBuf, str::FromStr};

use common_utils::date_utils::DateStamp;

/// Configuration for storage of FST sets describing the top 10 number of pics in sessions a user had on a given date.
pub struct SavedFstSetFilesConfig;

impl SavedFstSetFilesConfig {
    /// The path to the directory where we serialize daily camera and user stats.
    /// The files in this directory are not supposed to be viewed by anyone or anything apart from this program.
    /// The exception is files dating more than seven days back. One could/should set up a cronjob that deletes those.
    pub const DIRECTORY_PATH: &'static str = "./saved_fst_files";

    /// The prefix for the serialized camera stats. Their suffix will be a date of the form YYYYMMDD.
    pub const FILE_PREFIX: &'static str = "user-top-10-pics-";

    pub const FILE_EXTENSION: &'static str = ".fst";

    pub fn storage_directory() -> PathBuf {
        PathBuf::from_str(Self::DIRECTORY_PATH).unwrap()
    }

    // Returns the path to the saved file corresponding to the given date: (YYYYMMDD)
    pub(crate) fn file_path_from_date(datestamp: DateStamp) -> PathBuf {
        let date = datestamp.into_string();
        let mut path = SavedFstSetFilesConfig::storage_directory();
        let mut serialization_filename = SavedFstSetFilesConfig::FILE_PREFIX.to_string();
        serialization_filename.push_str(date.as_str());
        serialization_filename.push_str(SavedFstSetFilesConfig::FILE_EXTENSION);
        path.push(serialization_filename);
        path
    }

    pub(crate) fn file_paths_last_seven_days() -> Vec<PathBuf> {
        common_utils::date_utils::last_seven_days_ymd()
            .into_iter()
            .map(Self::file_path_from_date)
            .collect()
    }
}
/// Configuration describing filenames of hunamly readable files containing the top 10 number of pics in sessions by each user
/// over the last seven days.
pub struct DailyUsersStatsConfig;

impl DailyUsersStatsConfig {
    pub const FILE_PREFIX: &'static str = "user_top_10_";
    pub const FILE_EXTENSION: &'static str = ".txt";
}

pub(crate) fn todays_users_stats_file_path(out_directory: PathBuf) -> PathBuf {
    let today_ymd = common_utils::date_utils::today_ymd().into_string();
    let mut todays_camera_stats_path = out_directory;
    let filename: String = [
        DailyUsersStatsConfig::FILE_PREFIX,
        today_ymd.as_str(),
        DailyUsersStatsConfig::FILE_EXTENSION,
    ]
    .iter()
    .flat_map(|s| s.chars())
    .collect();
    todays_camera_stats_path.push(filename);
    todays_camera_stats_path
}
