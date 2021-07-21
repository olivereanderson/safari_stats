// This module contains functionality related to formatting and writing of the top 100 average number of pics by cameras in sessions.
use std::{
    fmt::{Display, Formatter},
    io::{BufWriter, Write},
};

use uuid::Uuid;

use crate::extracting::CameraBestAvgPics;

use itertools::Itertools;

use anyhow::{Context, Result};

// Write items from an iterator to file, one line per item.
// the BufWriter will be told to flush when the difference between the buffers internal capacity and the buffered data
// is below the given flush_threshold.
pub(crate) fn write_records<W: Write, Record: Display, I: IntoIterator<Item = Record>>(
    buf_writer: &mut BufWriter<W>,
    record_iterator: I,
    flush_threshold: usize,
) -> Result<()> {
    for record in record_iterator {
        if buf_writer.capacity() - buf_writer.buffer().len() < flush_threshold {
            buf_writer
                .flush()
                .with_context(|| "Failed flushing all bytes".to_string())?;
        }
        writeln!(buf_writer, "{}", record)
            .with_context(|| format!("Failed to write record: {} into the BufWriter", record))?;
    }
    buf_writer
        .flush()
        .with_context(|| "Failed flushing all bytes".to_string())?;
    Ok(())
}

// This struct corresponds to a line in the file "camera_top_100_YYYYMMDD.txt" file.
#[derive(PartialEq, Debug)]
pub(crate) struct CameraBestAvgPicsRecord {
    camera_id: u8,
    sessions: [Uuid; 100],
    avg_pics: [f32; 100],
}

impl CameraBestAvgPicsRecord {
    pub fn new(camera_id: u8, camera_best_avg_pics: CameraBestAvgPics) -> Self {
        Self {
            camera_id,
            sessions: camera_best_avg_pics.sessions,
            avg_pics: camera_best_avg_pics.avg_pics,
        }
    }
}

impl Display for CameraBestAvgPicsRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // There must be a more efficient way to do this!
        let write_string = std::iter::once(format!("{}|", self.camera_id))
            .chain(
                self.sessions
                    .iter()
                    .zip(self.avg_pics.iter())
                    .map(|(id, pic_score)| format!("{}:{},", id, pic_score)),
            )
            .join("");
        write!(f, "{}", write_string)
    }
}
