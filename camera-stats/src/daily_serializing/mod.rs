// This module handles saving (serializing) and loading (deserializing) of the
// extracted best 100 average number of pics by cameras from the daily session log files.

use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};

use crate::extracting::CameraBestAvgPicsMapping;

// Serializes CameraBestAvgPicsMapping to disk.
// If the given path does not exist we will attempt to create it.
pub(crate) fn serialize_camera_best_avg_pics_mapping_to_disk(
    path: PathBuf,
    camera_best_avg_pics: &CameraBestAvgPicsMapping,
) -> Result<()> {
    let file = std::fs::File::create(path.as_path())
        .with_context(|| format!("Failed to create the file: {:?}", &path.as_os_str()))?;
    const SERIALIZATION_WRITER_CAPACITY: usize = 150_000; // This should be more than enough for the mapping to fit in the buffer.
    let mut writer = BufWriter::with_capacity(SERIALIZATION_WRITER_CAPACITY, file);
    bincode::serialize_into(&mut writer, &camera_best_avg_pics)
        .with_context(|| "failed to serialize the CameraBestAvgPicsMapping".to_string())?;
    writer.flush().with_context(|| {
        "failed to write all the serialized CameraBestAvgPicsMapping bytes to disk"
    })?;
    Ok(())
}

// transforms paths to serialized CameraBestAvgPicsMapings to their respective deserialized structs.
pub(crate) fn deserialize_camera_best_avg_pics_mappings_from_files(
    // vector of paths to files
    paths: Vec<PathBuf>,
) -> Result<Vec<CameraBestAvgPicsMapping>> {
    let mut camera_best_avg_pic_mappings_previous_six_days: Vec<CameraBestAvgPicsMapping> =
        Vec::new();
    for path in paths {
        let file = File::open(path.as_path())
            .with_context(|| format!("Failed to open file: {:?}", path.as_path().as_os_str()))?;
        let reader = BufReader::with_capacity(150_000, file);
        let camera_best_avg_pics = bincode::deserialize_from(reader).with_context(|| {
            format!(
                "Failed to deserialize: {:?} into a CameraBestAvgPicsMapping",
                path.as_path().as_os_str()
            )
        })?;
        camera_best_avg_pic_mappings_previous_six_days.push(camera_best_avg_pics);
    }
    Ok(camera_best_avg_pic_mappings_previous_six_days)
}
