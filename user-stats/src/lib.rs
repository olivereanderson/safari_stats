pub mod configuration;
mod fst_utils;
mod parsing;
mod sorting;
mod writing;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

/// parses session log files and prints the top 10 pics in session for each user in the course of the last seven days.
///
/// This process consists of several steps. The session log files from the last seven days that have yet to be processed by this program are detected
/// and processed in turn. For each of these we create an FST set where the keys are of the form [user_id, u8:MAX - num pics, session_id] and store this
/// for subsequent use. After all the FST Sets have been created we take their union and use the encoded information to find the top 10 pics in session by user
/// which we then write to file.
///
/// In order to avoid high memory consumption we write temporary FST Maps to file in batches during construction of the FST sets.
///
/// WARNING: This function applies memory maps which are only safe if the underlying files are left unmodified by other processes/programs
/// thoroughout the execution of this program. Moreover despite memory maps being very fast on an SSD hard drive, it may be very slow on cheaper hard drives.
/// Finally your operating system may decide to use a lot of memory for the page cache while reading our FST sets/maps from disk which can make it look like
/// we are consuming an awful lot of RAM.
pub fn run(from_path: PathBuf, to_path: PathBuf) -> Result<()> {
    for unprocessed_log_file in
        common_utils::file_utils::unprocessed_session_log_files(from_path, |datestamp| {
            crate::configuration::SavedFstSetFilesConfig::file_path_from_date(datestamp).exists()
        })
    {
        let unprocessed_log_file = unprocessed_log_file?;
        println!("processing {:?}", &unprocessed_log_file.path.as_os_str());

        println!("Parsing, Sorting and Collecting in batches. Encoding information in FST maps: Keys [user_id, session_id] and values are the corresponding number of pics found within the batch");
        // Create a temporary directory to temporarily store FST maps.
        let temp_fst_dir_path_string = format!("./temporary_fsts_{}", Uuid::new_v4());
        let temporary_fst_dir_path = PathBuf::from_str(temp_fst_dir_path_string.as_str())
            .with_context(|| {
                format!(
                    "Failed to create path: {} for temporary batch storage.",
                    temp_fst_dir_path_string
                )
            })?;
        // Maximum number of (user_id, session_id, nb_pics) triples we can keep in a batch before we have to write it to memory.
        const CAPACITY_LIMIT: usize = 3 * 10usize.pow(7);
        // We are parsing records and summing up the number of pics for records with the same user and session ids
        // this frees up space in our batch vector, so we do not necessarily have to write the batch to disk after the first CAPACITY_LIMIT has been reached
        // however we also do not want to sort and collect too often.
        // The following constant determines that whenever (number of elements in batch vector after sorting)/CAPACITY_LIMIT > MAX_CAPACITY_RATIO_AFTER_SORT_COLLECT
        // we have to write the batch to disk.
        const MAX_CAPACITY_RATIO_AFTER_SORT_COLLECT: f64 = 0.5;
        crate::fst_utils::batching::from_log_file_to_batched_fst_maps(
            unprocessed_log_file.path.clone(),
            temporary_fst_dir_path.clone(),
            CAPACITY_LIMIT,
            MAX_CAPACITY_RATIO_AFTER_SORT_COLLECT,
        )?;
        println!("Constructing an FST set describing the top 10 number of pics in session per user that were found in {:?}.", &unprocessed_log_file.path.as_os_str());
        let fst_set_storage_path =
            crate::configuration::SavedFstSetFilesConfig::file_path_from_date(
                unprocessed_log_file.date,
            );
        crate::fst_utils::storing::from_batched_fst_maps_to_fst_set(
            temporary_fst_dir_path,
            fst_set_storage_path,
        )?;
        println!(
            "Stored FST set corresponding to {:?} for reuse. The keys are of the form [user_id, (u8::MAX - nb_pics),session_id]",
            unprocessed_log_file.path.as_os_str()
        );
    }
    println!("Extracting top 10 pics in session by user over a seven day period.");

    let output_file_path = crate::configuration::todays_users_stats_file_path(to_path);

    crate::fst_utils::finalizing::from_fst_sets_to_stats_file(
        crate::configuration::SavedFstSetFilesConfig::file_paths_last_seven_days(),
        output_file_path.clone(),
    )?;
    println!(
        "The results have been saved as {:?}",
        output_file_path.as_os_str()
    );
    Ok(())
}
