use fst::Map;
use fst::{map::OpBuilder, SetBuilder, Streamer};
use memmap::Mmap;
use std::{
    fs::{self, File},
    io::{self, BufWriter},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

// builds an fst::Set with keys corresponding to triples (user_id, u8::MAX - sum_pics, session_id) from the temporarily stored fst::Maps
// created by super::batching::from_log_file_to_batched_fst_maps. The fst::Set will be saved to the given output_file_path.
// The temporary fst directory is deleted at the end of this function.
//
// WARNING: This function uses memory maps which can lead to undefined behaviour if some other process/program modifies the corresponding file(s)
// while our program is running.
pub(crate) fn from_batched_fst_maps_to_fst_set<P: AsRef<Path>>(
    temporary_fst_dir_path: PathBuf,
    output_file_path: P,
) -> Result<()> {
    // Open all the files found in temporary_fst_dir_path
    let files = fs::read_dir(&temporary_fst_dir_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?
        .iter()
        .filter(|path| path.is_file())
        .map(File::open)
        .collect::<Result<Vec<_>, io::Error>>()?;

    // Memory map all the open files
    let memory_maps = files
        .iter()
        .map(|file| unsafe { Mmap::map(file) })
        .collect::<Result<Vec<_>, io::Error>>()?;

    // Create FST maps for all the memory mapped files.
    let mut fst_maps: Vec<Map<Mmap>> = Vec::new();
    for memory_map in memory_maps {
        let map = Map::new(memory_map).with_context(|| {
            format!(
                "Failed to produce FST maps for all files in {:?}",
                temporary_fst_dir_path.as_os_str()
            )
        })?;
        fst_maps.push(map);
    }

    // Compute the union of these FST maps.
    let mut op_builder = OpBuilder::new();
    for fst_map in fst_maps.iter() {
        op_builder.push(fst_map);
    }
    let mut union = op_builder.union();
    // we will be keep updating the users best pics in sessions from inside the stream.
    let mut user_best_sum_pics = UserBestSumPics::default();
    // once we are sure we have found a user's top 10 best pics in sessions we will write this information to our fst::Set.
    let wtr = BufWriter::new(File::create(&output_file_path).with_context(|| {
        format!(
            "Failed to create file: {:?}",
            output_file_path.as_ref().as_os_str()
        )
    })?);
    let mut set_builder = SetBuilder::new(wtr).with_context(|| {
        "Unable to build an FST set from the temporary FST maps. Failed to produce a Set builder"
    })?;
    // the last observed user id encoded as bytes.
    let mut current_pid = [0u8; 16];
    // the last observed session id encoded as bytes.
    let mut current_session_id = [0u8; 16];
    // we can transfomr the union of FST Maps into a stream. The items returned from this stream are of the form ([user id as bytes session_id as bytes], [IndexValue] where each IndexValue contains an index and the corresponding value.
    // The index corresponds to which FST Map the value comes from.
    while let Some((key, value)) = union.next() {
        // once the last observed user id changes we store the users top 10 sessions in the FST set.
        if key[..16] != current_pid[..] {
            if current_pid != [0u8; 16] {
                // this means we are passed the very first iteration and the stream has finished with the last user_id
                // each iteration of the following for loop corresponds to writing the ordered triple (user_id,u8::MAX - sum_pics, session_id)
                // into the fst::Set. We use u8::MAX - sum pics so that we can easily retrieve the highest sums of pics when we later load
                // this stored fst::Set.
                for set_key in fst_set_keys_iter(&user_best_sum_pics) {
                    set_builder.insert(&set_key).with_context(|| {
                        format!("Failed to insert {:?} into the the FST Set", set_key)
                    })?;
                }
            }
            user_best_sum_pics.clear(); //todo: Consider setting user_best_sum_pics = UserBestSumPics::default() here.
            current_pid.clone_from_slice(&key[..16]); // update current_pid to the new user id.
            user_best_sum_pics.update_user_id(current_pid);
        }
        // This way we easily sum up all the pics a user had in the same session
        let sum_pics = value
            .iter()
            .fold(0u8, |acc, index| acc + (index.value as u8));
        if user_best_sum_pics.is_improvement(sum_pics) {
            current_session_id.clone_from_slice(&key[16..]);
            user_best_sum_pics.update(&current_session_id, sum_pics as i16);
        }
    }
    set_builder
        .finish()
        .with_context(|| "Failed to save the built fst Set to disk")?;
    drop(union);
    drop(fst_maps);
    drop(files);
    fs::remove_dir_all(temporary_fst_dir_path.as_path()).with_context(|| {
        format!(
            "Failed removing the temporary directory: {:?}",
            temporary_fst_dir_path.as_path().as_os_str()
        )
    })?;
    Ok(())
}

struct UserBestSumPics {
    // the user id, this time in byte form.
    user_id: [u8; 16],
    // the session ids in byte form.
    best_session_ids: [[u8; 16]; 10],
    // the 10 best pics in sessions
    best_pics_in_sessions: [i16; 10], // use i16 so we can keep the default value as -1 this makes adding sessions with 0 pics easier
}

impl Default for UserBestSumPics {
    fn default() -> Self {
        let user_id = [0u8; 16];
        let best_session_ids = [[0u8; 16]; 10];
        let best_pics_in_sessions: [i16; 10] = [-1; 10];
        Self {
            user_id,
            best_session_ids,
            best_pics_in_sessions,
        }
    }
}

impl UserBestSumPics {
    fn clear(&mut self) {
        let default = Self::default();
        self.user_id = default.user_id;
        self.best_session_ids = default.best_session_ids;
        self.best_pics_in_sessions = default.best_pics_in_sessions;
    }
    fn update_user_id(&mut self, user_id: [u8; 16]) {
        self.user_id = user_id;
    }

    fn is_improvement(&self, sum_pics: u8) -> bool {
        (sum_pics as i16) > self.best_pics_in_sessions[9]
    }

    fn update(&mut self, session_id: &[u8; 16], sum_pics: i16) {
        // we assume that the best_sum_pics array is ordered from highest to lowest
        self.best_session_ids[9] = *session_id;
        self.best_pics_in_sessions[9] = sum_pics;
        let mut i = 9;
        while i > 0 && self.best_pics_in_sessions[i] > self.best_pics_in_sessions[i - 1] {
            self.best_pics_in_sessions.swap(i - 1, i);
            self.best_session_ids.swap(i - 1, i);
            i -= 1;
        }
        // we also want the session_ids to be sorted from lowest to highest when the corresponding pics are the same
        while i > 0 && self.best_pics_in_sessions[i] == self.best_pics_in_sessions[i - 1] {
            if self.best_session_ids[i - 1] > self.best_session_ids[i] {
                self.best_session_ids.swap(i, i - 1);
            }
            i -= 1;
        }
    }
}

// Returns an iterator of keys for the fst::Set we are building
fn fst_set_keys_iter(user_best_sum_pics: &UserBestSumPics) -> impl Iterator<Item = Vec<u8>> {
    let mut keys: Vec<Vec<u8>> = Vec::new();
    for (session_id, num_pics) in user_best_sum_pics
        .best_session_ids
        .iter()
        .zip(user_best_sum_pics.best_pics_in_sessions)
    {
        let mut key = Vec::<u8>::with_capacity(33);
        if num_pics >= 0 {
            key.extend_from_slice(user_best_sum_pics.user_id.as_ref());
            key.push(u8::MAX - (num_pics as u8)); // trick to get the highest values to appear first in the fst::Set we are producing
            key.extend_from_slice(session_id);
            keys.push(key);
        }
    }
    keys.into_iter()
}
