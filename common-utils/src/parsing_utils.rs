//! # Parsing utils
//!
//! This module contains structures and functions related to parsing safari session logs.
//!

use csv::{Reader, ReaderBuilder};
use serde::Deserialize;
use std::{fs::File, path::Path};
use uuid::Uuid;

/// This represents a valid row/record from a daily safari session log file (safari-sessions-YYYYMMDD.log)
#[derive(Debug, Deserialize)]
pub struct Record {
    /// User unique identifier
    pub user_id: Uuid,
    /// Session unique identifier
    pub session_id: Uuid,
    /// Camera unique identifier. There are dozens of cameras available.
    /// If this number exceeds u8::MAX in the future we might have to make backward incompatible changes.
    pub camera_id: u8,
    /// The number of pics by the user.
    pub nb_pics: u8,
}

/// Produces a csv reader with a predefined buffer capacity that presumes no headers in the file.
/// This reader can be used to parse safari session log files.
pub fn customised_csv_reader<P: AsRef<Path>>(
    path: P,
    buffer_capacity: usize,
) -> csv::Result<Reader<File>> {
    ReaderBuilder::new()
        .has_headers(false)
        .buffer_capacity(buffer_capacity)
        .from_path(path)
}
