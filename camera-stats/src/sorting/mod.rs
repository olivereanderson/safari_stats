use extsort::{ExternalSorter, Sortable};
use std::io::{Read, Write};

// Creates a sorter that may hold upto segment_size elements in memory
// before writing its progress to disk. Our sorter is configured to use rayon to sort
// the data in the in-memory buffer in parallel.
pub(crate) fn customized_external_sorter(segment_size: usize) -> ExternalSorter {
    ExternalSorter::new()
        .with_segment_size(segment_size)
        .with_parallel_sort()
}
use crate::parsing::CameraRecord;
use anyhow::{Context, Result};
use extsort::SortedIterator;
use std::cmp::Ordering;

// describes how the sorter should serialise and deserialise camera records
// when the in-memory buffer gets full and we need to write and/or read progress
// from disk.
impl Sortable for CameraRecord {
    fn encode<W: Write>(&self, writer: &mut W) {
        bincode::serialize_into(writer, self).unwrap();
    }

    fn decode<R: Read>(reader: &mut R) -> Option<Self> {
        bincode::deserialize_from(reader).ok()
    }
}

// takes an iterator of camera records and returns a new iterator over the records sorted lexicographically by session_id and camera_id.
// The segment_size parameter determines the maximum number of camera records the sorter
// may hold in-memory at any time.
pub(crate) fn sort_camera_records<I: Iterator<Item = CameraRecord>>(
    record_iter: I,
    segment_size: usize,
) -> Result<
    SortedCameraRecordsIter<
        impl (Fn(&CameraRecord, &CameraRecord) -> Ordering) + Send + Sync,
    >,
> {
    let sorter = customized_external_sorter(segment_size);
    let sorted_iter = sorter.sort_by(record_iter, |x, y| match x.session_id.cmp(&y.session_id) {
        std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        std::cmp::Ordering::Less => std::cmp::Ordering::Less,
        std::cmp::Ordering::Equal => x.camera_id.cmp(&y.camera_id),
    }).with_context(|| "Failed to sort the CameraRecords lexicographically with respect to session id followed by camera id")?;
    Ok(SortedCameraRecordsIter::new(sorted_iter))
}

pub(crate) struct SortedCameraRecordsIter<
    F: Fn(&CameraRecord, &CameraRecord) -> Ordering + Send + Sync,
> {
    sorted_iter: SortedIterator<CameraRecord, F>,
}
impl<F: Fn(&CameraRecord, &CameraRecord) -> Ordering + Send + Sync> Iterator
    for SortedCameraRecordsIter<F>
{
    type Item = CameraRecord;
    fn next(&mut self) -> Option<Self::Item> {
        self.sorted_iter.next()
    }
}

impl<F: Fn(&CameraRecord, &CameraRecord) -> Ordering + Send + Sync>
    SortedCameraRecordsIter<F>
{
    fn new(sorted_iter: SortedIterator<CameraRecord, F>) -> Self {
        Self { sorted_iter }
    }
}
