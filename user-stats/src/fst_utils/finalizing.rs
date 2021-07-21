use crate::writing::UserBestStats;
use anyhow::{Context, Result};
use fst::{Set, Streamer};
use memmap::Mmap;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use uuid::Uuid;

// Load the stored fst sets (produced by super::storing::from_batched_fst_maps_to_fst_set) and finds the top 10 session for each user present in the union of these sets.
// write these user stats to the given file with the following format:
// user_id|session_id1:nb_pics1,session_id2:nb_pics2, ...,session_id10:nb_pics10
//
// WARNING: This function uses memory maps which can lead to undefined behaviour if some other process/program modifies the corresponding file(s)
// while our program is running.
pub(crate) fn from_fst_sets_to_stats_file<P: AsRef<Path>>(
    stored_fst_set_paths: Vec<P>,
    output_file_path: P,
) -> Result<()> {
    // Open files defined in the given paths.
    let files = stored_fst_set_paths
        .iter()
        .map(File::open)
        .collect::<Result<Vec<_>, std::io::Error>>()
        .with_context(|| {
            "Could not open all the specified files. Perhaps there is a permission issue?"
        })?;

    // Memory map each of the open files.
    let memory_maps = files
        .iter()
        .map(|file| unsafe { Mmap::map(file) })
        .collect::<Result<Vec<_>, std::io::Error>>()
        .with_context(|| "Could not memory map all the specified files")?;

    // Produce FST sets for each of the memory maps
    let mut fst_sets: Vec<Set<Mmap>> = Vec::new();
    for memory_map in memory_maps {
        let set = Set::new(memory_map)
            .with_context(|| "Unable to obtain an FST set from the given memory map")?;
        fst_sets.push(set);
    }
    // take the union of all the FST sets.
    let mut op_builder = fst::set::OpBuilder::new();
    for fst_set in fst_sets.iter() {
        op_builder.push(fst_set);
    }
    let mut union = op_builder.union();
    // The keys in this union correspond to (user_id, u8::MAX - nb_pics, session_id) and are ordered lexicographically.
    // that is the first 16 bytes give us the user id, the 17'th byte is u8::MAX - nb_pics, and the 18'th until the 33rd byte gives us the session id.
    // The last observed user id encoded as bytes
    let mut current_pid_bytes = [0u8; 16];
    // The number of records we have recorded for the
    let mut pushed_records_for_current_user = 0;

    let mut current_user_best_stats = UserBestStats::default();
    let mut buf_writer = BufWriter::new(File::create(&output_file_path).with_context(|| {
        format!(
            "Could not create file: {:?}",
            &output_file_path.as_ref().as_os_str()
        )
    })?);

    while let Some(key) = union.next() {
        // throughout recall once more that the key corresponds to (user_id, u8::MAX - nb_pics, session_id).
        // that is the first 16 bytes give us the user_id, the 17'th byte is u8::MAX - nb_pics, and the 18'th until the 33rd byte gives us the session id.

        // as soon as we see another user id, we write the current user's best stats to file.
        if key[..16] != current_pid_bytes[..] {
            // the exception is on the very first iteration. Here we are assumming that Uuid::default() is not an actual user id!
            if current_pid_bytes != [0u8; 16] {
                writeln!(buf_writer, "{}", current_user_best_stats).with_context(|| {
                    format!("Failed writing {} to file", current_user_best_stats)
                })?;
            }
            // reset the current user best stats data:
            current_user_best_stats.clear(); // consider assinging to default value instead here.
                                               // update the current user id
            current_pid_bytes.clone_from_slice(&key[..16]);
            current_user_best_stats.update_user_id(current_pid_bytes);
            // reset the number of records pushed into current_user_best_stats
            pushed_records_for_current_user = 0;
        }
        // since the first ten entries per user id correspond to their best sessions (here we are using the u8::MAX - nb_pics trick!)
        // we only need to consider these ten first entries for each user.
        if pushed_records_for_current_user < 10 {
            let mut session_id_bytes = [0u8; 16];
            session_id_bytes.clone_from_slice(&key[17..]);
            let session_id = Uuid::from_bytes(session_id_bytes);
            let nb_pics = (u8::MAX as u64) - (key[16] as u64);
            current_user_best_stats.push_session_pics_pair(session_id, nb_pics as u8);
            pushed_records_for_current_user += 1;
        }
    }
    buf_writer.flush()?;
    Ok(())
}

impl UserBestStats {
    fn clear(&mut self) {
        self.user_id = UserBestStats::default().user_id;
        self.session_id_num_pics_pairs = UserBestStats::default().session_id_num_pics_pairs;
    }

    fn update_user_id(&mut self, user_id_bytes: [u8; 16]) {
        self.user_id = Uuid::from_bytes(user_id_bytes);
    }

    fn push_session_pics_pair(&mut self, session_id: Uuid, nb_pics: u8) {
        self.session_id_num_pics_pairs.push((session_id, nb_pics));
    }
}
