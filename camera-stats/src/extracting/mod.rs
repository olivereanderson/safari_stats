// This module contains functionality enabling extraction of the top 100 average number of pics by cameras from session log files.
use anyhow::{Context, Result};
use uuid::Uuid;

use crate::{parsing::CameraRecord, sorting::SortedCameraRecordsIter};
use common_utils::parsing_utils::Record;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::{
    path::Path,
    {cmp::Ordering, collections::HashMap, ops::AddAssign},
};

// Extracts a mapping taking each camera to the top 100 average number of pics in sessions
// found in the provided session log file.
pub(crate) fn extract_top_100_sessions_for_cameras<P: AsRef<Path>>(
    session_log_file_path: P,
) -> Result<CameraBestAvgPicsMapping> {
    const BUFFER_CAPACITY: usize = 8 * 2usize.pow(10);
    let reader = common_utils::parsing_utils::customised_csv_reader(
        session_log_file_path.as_ref(),
        BUFFER_CAPACITY,
    )
    .with_context(|| {
        format!(
            "Failed to create a CSV Reader to parse the session log file: {:?}",
            session_log_file_path.as_ref().as_os_str()
        )
    })?;
    let records_iter = reader
        .into_deserialize::<Record>()
        .filter_map(Result::ok)
        .map_into::<CameraRecord>();

    const NUM_ITEMS_IN_SORTER_MEMORY_BUFFER: usize = 50_000_000;
    println!("sorting camera records");
    let sorted_iter =
        crate::sorting::sort_camera_records(records_iter, NUM_ITEMS_IN_SORTER_MEMORY_BUFFER)
        .with_context(|| {
            format!("Could not extract the top 100 average number of pics per camera from: {:?} because sorting of the records failed.", session_log_file_path.as_ref().as_os_str())
        })?;
    println!("sorting completed");

    println!("finding top 100 average pics in sessions for each camera");
    Ok(crate::extracting::camera_best_hundred_mapping_from_sorted_iterator(sorted_iter))
}

// produces a map of present camera ids to their top 100 average number of pics in sessions found in the sorted iterator.
pub(crate) fn camera_best_hundred_mapping_from_sorted_iterator<
    F: Fn(&CameraRecord, &CameraRecord) -> Ordering + Send + Sync,
>(
    sorted_iter: SortedCameraRecordsIter<F>,
) -> CameraBestAvgPicsMapping {
    let number_of_cameras = u8::MAX; // This is likely more than the actual number of cameras.
    let mut camera_best_average_mapping: HashMap<u8, CameraBestAvgPics> =
        HashMap::with_capacity(number_of_cameras as usize);

    for (key, group) in sorted_iter
        .group_by(|x| (x.session_id, x.camera_id))
        .into_iter()
    {
        let (pics_by_camera, occurrences_of_camera) = group
            .fold((0usize, 0usize), |acc, op_record| {
                (acc.0 + op_record.nb_pics as usize, acc.1 + 1)
            });
        let (session_id, camera_id) = (&key.0, &key.1);
        let avg_num_pics = (pics_by_camera as f32) / (occurrences_of_camera as f32);
        if let Some(camera_best_avg_pics) = camera_best_average_mapping.get_mut(camera_id) {
            camera_best_avg_pics.update_on_improvement(session_id, avg_num_pics);
        } else {
            let mut camera_best_avg_pics = CameraBestAvgPics::default();
            camera_best_avg_pics.update_on_improvement(session_id, avg_num_pics);
            camera_best_average_mapping.insert(*camera_id, camera_best_avg_pics);
        }
    }
    CameraBestAvgPicsMapping::new(camera_best_average_mapping)
}

// This struct holds a map that takes an camera id to the data describing the top 100 average number of pics in sessions.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct CameraBestAvgPicsMapping {
    pub(crate) mapper: HashMap<u8, CameraBestAvgPics>,
}
impl CameraBestAvgPicsMapping {
    fn new(mapper: HashMap<u8, CameraBestAvgPics>) -> Self {
        Self { mapper }
    }
}

// We implement this trait in order to merge CameraBEstAvgPicsMappings together.
// Loosely speaking mapping_1.add_assign(mapping2) mutates mapping_1 to the following mapping:
// (mapping_1,mapping_2)(camera_id) = top 100 average number of pics (and corresponding sessions) from the union of mapping_1(camera_id) and mapping_2(camera_id).
impl AddAssign for CameraBestAvgPicsMapping {
    fn add_assign(&mut self, other: Self) {
        for (id_other, best_avg_pics_other) in other.mapper.into_iter() {
            if let std::collections::hash_map::Entry::Vacant(e) = self.mapper.entry(id_other) {
                e.insert(best_avg_pics_other);
            } else {
                let best_avg_pics_self = self.mapper.get_mut(&id_other).unwrap();
                best_avg_pics_self.add_assign(best_avg_pics_other);
            }
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct CameraBestAvgPics {
    // the ids of the top 100 sessions.
    #[serde(with = "BigArray")]
    pub(crate) sessions: [Uuid; 100],
    // sorted array of the top 100 avg pics by the camera in sessions.
    // We require that avg_pics[i] was the average number of pics by the camera in sessions[i] for all i.
    // This relationship between the two arrays would be made clearer if we used a hashmap instead, but
    // since this data structure is intended to be mutated very many times, the use of a hashmap would lead to
    // a big drop in performance.
    #[serde(with = "BigArray")]
    pub(crate) avg_pics: [f32; 100],
    // the 100'th highest number of pics
}

impl Default for CameraBestAvgPics {
    fn default() -> Self {
        let sessions = [Uuid::default(); 100];
        let avg_pics = [0.0; 100];
        Self { sessions, avg_pics }
    }
}
impl CameraBestAvgPics {
    pub(crate) fn update_on_improvement(&mut self, session_id: &Uuid, avg_num_pics: f32) {
        if self.is_improvement(avg_num_pics) {
            self.update(*session_id, avg_num_pics)
        }
    }

    fn is_improvement(&self, avg_num_pics: f32) -> bool {
        // We may always assume that self.avg_pics is sorted in such a way that lower indexes correspond to higher values.
        self.avg_pics[99] < avg_num_pics
    }

    // todo: Using a Vec instead of arrays might improve performance here. We should investigate this option.
    fn update(&mut self, session_id: Uuid, avg_num_pics: f32) {
        // first replace the last element
        self.avg_pics[99] = avg_num_pics;
        self.sessions[99] = session_id;
        // Now sort such that the lower indexes correspond to higher values
        let mut i = 99usize;
        while i > 0 && self.avg_pics[i] > self.avg_pics[i - 1] {
            self.avg_pics.swap(i - 1, i);
            self.sessions.swap(i - 1, i);
            i -= 1;
        }
    }
}

impl AddAssign for CameraBestAvgPics {
    fn add_assign(&mut self, other: Self) {
        for (session_id, avg_num_pics) in other.sessions.iter().zip(other.avg_pics.iter().cloned()) {
            if self.is_improvement(avg_num_pics) {
                self.update(*session_id, avg_num_pics);
            } else {
                break;
            }
        }
    }
}

// Takes a vector of CameraBestAvgPicsMappings and merges them together to a single CameraBestAvgPicsMapping.
// The merge is obtained by collecting the top 100 average number of pics for each camera that can be obtained from any of the provided mappings.
pub(crate) fn merge_camera_best_avg_pics(
    mut mappings: Vec<CameraBestAvgPicsMapping>,
) -> Result<CameraBestAvgPicsMapping> {
    let mut mapping = mappings
        .pop()
        .with_context(|| "An empty vector was provided")?;
    for other_mapping in mappings {
        mapping.add_assign(other_mapping);
    }
    Ok(mapping)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_on_single_improvement_updates_on_improvement() {
        let mut camera_best_avg_pics = CameraBestAvgPics::default();
        let session_id = Uuid::from_u128(42);
        let avg_num_pics = 2.0 as f32;
        camera_best_avg_pics.update_on_improvement(&session_id, avg_num_pics);
        let max_camera_avg_pics = camera_best_avg_pics
            .avg_pics
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap();
        assert_eq!(*max_camera_avg_pics, avg_num_pics);
    }

    #[test]
    fn update_on_multiple_improvements_works() {
        let mut camera_best_avg_pics = CameraBestAvgPics::default();
        let mut session_ids = [Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3)];
        let avg_pics = [1.0f32, 2.0, 3.0];
        for (session_id, avg_num_pics) in session_ids.iter().zip(avg_pics.iter()) {
            camera_best_avg_pics.update_on_improvement(session_id, *avg_num_pics);
        }
        assert_eq!([3.0, 2.0, 1.0], camera_best_avg_pics.avg_pics[..3]);
        session_ids.reverse();
        assert_eq!(session_ids, camera_best_avg_pics.sessions[..3]);
    }
}
