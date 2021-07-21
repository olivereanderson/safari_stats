// This module mostly contains functionality related to paths and filenames
// that need to be read and/or written to by this application.
use std::{path::PathBuf, str::FromStr};

use common_utils::date_utils::DateStamp;

pub(crate) struct DailyCameraBestAvgPicsFilesConfig {}

impl DailyCameraBestAvgPicsFilesConfig {
    pub(crate) const FILE_PREFIX: &'static str = "camera_top100_";
    pub(crate) const FILE_EXTENSION: &'static str = ".txt";
}

/// Configuration for serialization of the top 100 average number of pics per camera on a given date.
pub struct SerializationFilesConfig;

impl SerializationFilesConfig {
    /// The path to the directory where we serialize daily camera and user stats.
    /// The files in this directory are not supposed to be viewed by anyone or anything apart from this program.
    /// The exception is files dating more than seven days back. One could/should set up a cronjob that deletes those.
    pub const SERIALIZATION_DIRECTORY_PATH: &'static str = "./serialized_camera_stats";

    /// The prefix for the serialized camera stats. Their suffix will be a date of the form YYYYMMDD.
    pub const SERIALIZATION_OPERATOR_PREFIX: &'static str = "camera-top-100-pics-average-";

    /// The path to the directory where serialized camera stats are stored
    pub fn serialization_directory() -> PathBuf {
        PathBuf::from_str(Self::SERIALIZATION_DIRECTORY_PATH).unwrap()
    }

    // Returns the path to the serialized camera stats file corresponding to the given date: (YYYYMMDD)
    pub(crate) fn serialization_file_from_datestamp(datestamp: DateStamp) -> PathBuf {
        let date_ymd = datestamp.into_string();
        let mut path = SerializationFilesConfig::serialization_directory();
        let mut serialization_filename =
            SerializationFilesConfig::SERIALIZATION_OPERATOR_PREFIX.to_string();
        serialization_filename.push_str(date_ymd.as_str());
        path.push(serialization_filename);
        path
    }
    // Returns a vector of file paths for the serialized camera stats files for the last seven days.
    pub(crate) fn serialized_file_paths_last_seven_days() -> Vec<PathBuf> {
        common_utils::date_utils::last_seven_days_ymd()
            .into_iter()
            .map(Self::serialization_file_from_datestamp)
            .collect()
    }
}

// The path for todays camera stats file.
// The contents of this file should be the top 100 average pics by each camera over a seven day period.
pub(crate) fn todays_camera_stats_file_path(out_directory: PathBuf) -> PathBuf {
    let today_ymd = common_utils::date_utils::today_ymd().into_string();
    let mut todays_camera_stats_path = out_directory;
    let filename: String = [
        DailyCameraBestAvgPicsFilesConfig::FILE_PREFIX,
        today_ymd.as_str(),
        DailyCameraBestAvgPicsFilesConfig::FILE_EXTENSION,
    ]
    .iter()
    .flat_map(|s| s.chars())
    .collect();
    todays_camera_stats_path.push(filename);
    todays_camera_stats_path
}
