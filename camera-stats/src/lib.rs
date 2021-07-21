use anyhow::{Context, Result};

use itertools::Itertools;

use std::{fs::File, io::BufWriter, path::PathBuf};

pub mod configuration;
mod daily_serializing;
mod extracting;
pub(crate) mod parsing;
mod sorting;
mod writing;

use configuration::SerializationFilesConfig;
use writing::CameraBestAvgPicsRecord;

pub fn run(from_path: PathBuf, to_path: PathBuf) -> Result<()> {
    for unprocessed_log_file in
        common_utils::file_utils::unprocessed_session_log_files(from_path, |datestamp| {
            SerializationFilesConfig::serialization_file_from_datestamp(datestamp).exists()
        })
    {
        let unprocessed_log_file = unprocessed_log_file?;
        println!("processing {:?}", &unprocessed_log_file.path.as_os_str());

        println!(
            "extracting the top 100 average number of pics by camera from: {:?}",
            &unprocessed_log_file.path.as_os_str()
        );
        let camera_top_100_mapping = crate::extracting::extract_top_100_sessions_for_cameras(
            unprocessed_log_file.path.clone(),
        )
        .unwrap();
        println!(
            "extraction completed. Now compactly saving this information for subsequent reuse"
        );
        let serialization_path = SerializationFilesConfig::serialization_file_from_datestamp(
            unprocessed_log_file.date.clone(),
        );
        crate::daily_serializing::serialize_camera_best_avg_pics_mapping_to_disk(
            serialization_path,
            &camera_top_100_mapping,
        )?;
    }
    // Now we can load all of the serialized daily camera stats
    let cameras_best_per_day =
        crate::daily_serializing::deserialize_camera_best_avg_pics_mappings_from_files(
            SerializationFilesConfig::serialized_file_paths_last_seven_days(),
        )?;
    // We now have a vector of the top 100 average pics in sessions by camera per day, but we are interested in seeing this over the last seven days so we merge the top 100 from all of these results.
    let best_avg_pics_over_seven_days_by_camera_mapper =
        crate::extracting::merge_camera_best_avg_pics(cameras_best_per_day)?;

    // We now have a mapping taking camera ids to their best average number of pics in sessions over the last seven days.
    // We now transform this mapping into an iterator over key value pairs, where keys are ordered from smallest to largest.
    let best_avg_pics_over_seven_days_by_camera_iter =
        best_avg_pics_over_seven_days_by_camera_mapper
            .mapper
            .into_iter()
            .sorted_by_key(|(camera_id, _camera_best_avg_pics)| *camera_id)
            .map(|(id, camera_best_avg_pics)| {
                CameraBestAvgPicsRecord::new(id, camera_best_avg_pics)
            });
    // finally we write these results to file in the given output directory
    let todays_camera_stats_path = crate::configuration::todays_camera_stats_file_path(to_path);
    let outfile = File::create(todays_camera_stats_path.as_path()).with_context(|| {
        format!(
            "Failed to create file: {:?}",
            todays_camera_stats_path.as_path().as_os_str()
        )
    })?;
    let mut buf_writer = BufWriter::with_capacity(400_000, outfile);
    crate::writing::write_records(
        &mut buf_writer,
        best_avg_pics_over_seven_days_by_camera_iter,
        4000,
    )?;
    println!(
        "The results have been saved as {:?}",
        todays_camera_stats_path.as_os_str()
    );
    Ok(())
}
