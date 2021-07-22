use fst::MapBuilder;
use itertools::Itertools;
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::parsing::{UserSessionRecord, UserRecord};
use anyhow::{Context, Result};
use common_utils::parsing_utils::Record;

// parses a session log file. Extracts user records from the parsed file (user_id, session_id, sum num_pics) and places this into a vector.
// When the vector reaches the provided capacity limit. The vector is sorted and records with the same user and session ids are merged to a single record (where num_pics is the sum of the pics).
// If (number of elements in batch vector after sorting and collecting)/capcity_limit > max_capacity_ratio_after_sort_collect then we write the contents of the batch vector to disk and clear the vector.
// Note that a records with the same (user_id and session_id) pairs can end up in different files.
pub(crate) fn from_log_file_to_batched_fst_maps<P: AsRef<Path>>(
    log_file_path: P,
    temporary_fst_dir_path: PathBuf,
    capacity_limit: usize,
    max_capacity_ratio_after_sort_collect: f64,
) -> Result<()> {
    // recreate the temporary fst dir path if it exists.
    if temporary_fst_dir_path.exists() {
        std::fs::remove_dir_all(&temporary_fst_dir_path).with_context(|| {
            format!(
                "the directory {:?} exists, but we were not able to recursively delete it.",
                &temporary_fst_dir_path.as_os_str()
            )
        })?;
    }
    std::fs::create_dir_all(&temporary_fst_dir_path).with_context(|| {
        format!(
            "failed to create directory {:?}",
            &temporary_fst_dir_path.as_os_str()
        )
    })?;
    const BUFFER_CAPACITY: usize = 8 * 2usize.pow(10);
    let reader = common_utils::parsing_utils::customised_csv_reader(log_file_path, BUFFER_CAPACITY)
        .with_context(|| "Failer to create a csv reader for session log file parsing")?;
    // create an iterator with items (UserMatchRecord, num_pics)
    let records_iter = reader
        .into_deserialize::<Record>()
        .filter_map(Result::ok)
        .map_into::<UserRecord>()
        .map(|record| record.split());
    let mut batch_vector: Vec<(UserSessionRecord, u8)> = Vec::with_capacity(capacity_limit);
    let mut batch_counter = 0;
    for pair in records_iter {
        if batch_vector.len() >= capacity_limit {
            crate::sorting::sort_collect_splitted_user_records(&mut batch_vector);
            // If there were multiple entries with equal UserSessionRecord then the vector's length
            // will have decreased. If the decrease was not sufficient we save our progress to a temporary fst map and clear the vector.
            if batch_vector.len()
                >= (max_capacity_ratio_after_sort_collect * (capacity_limit as f64)) as usize
            {
                batch_counter += 1;
                let path = temporary_fst_dir_path
                    .clone()
                    .join(format!("{}.fst", batch_counter));
                write_batch_fst_map(path, &mut batch_vector).with_context(|| {
                    format!("Failed to write batch number {} to disk", batch_counter)
                })?;
            }
        }
        batch_vector.push(pair);
    }
    if !batch_vector.is_empty() {
        // if there is anything left in the batch vector, then we write this final batch to disk as well.
        crate::sorting::sort_collect_splitted_user_records(&mut batch_vector);
        batch_counter += 1;
        let path = temporary_fst_dir_path.join(format!("{}.fst", batch_counter));
        write_batch_fst_map(path, &mut batch_vector)
            .with_context(|| format!("Could not write batch number {} to disk", batch_counter))?;
    }
    println!(
        "We have sucessfully encoded {} batches as FST maps",
        batch_counter
    );
    Ok(())
}

// drains the batch_vector and saves a temporary fst where the keys are obtained from the function "user_session_record_temp_fs_key"
// and values correpond to the sum of pics.
fn write_batch_fst_map<P: AsRef<Path>>(
    path: P,
    batch_vector: &mut Vec<(UserSessionRecord, u8)>,
) -> Result<()> {
    let wtr = BufWriter::new(
        File::create(&path)
            .with_context(|| format!("Could not create file {:?}", path.as_ref().as_os_str()))?,
    );
    let mut map_builder = MapBuilder::new(wtr).with_context(|| {
        "Writing batch to disk was not possible because we failed to create a Map builder."
    })?;
    for (key, value) in batch_vector
        .drain(..)
        .map(|(user_session_record, sum_pics)| {
            (
                user_session_record_batch_fst_key(user_session_record),
                sum_pics as u64,
            )
        })
    {
        map_builder.insert(key, value).with_context(|| format!("Writing batch to disk was not possible because we failed to insert the Key: {:?}, Value: {} pair into the corresponding map builder.", key, value))?;
    }
    map_builder
        .finish()
        .with_context(|| "Failed writing batch to disk as an FST map")?;
    Ok(())
}

// converts a UserMatchRecord into keys for the temporary fst's.
fn user_session_record_batch_fst_key(user_session_record: UserSessionRecord) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for (idx, byte) in user_session_record
        .user_id
        .as_bytes()
        .iter()
        .chain(user_session_record.session_id.as_bytes().iter())
        .enumerate()
    {
        bytes[idx] = *byte;
    }
    bytes
}
