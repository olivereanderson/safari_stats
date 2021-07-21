use anyhow::Result;
use rand::{prelude::SliceRandom, Rng};
use std::{
    fmt,
    fs::File,
    io::{prelude::*, BufWriter},
    iter,
    path::PathBuf,
};
use uuid::Uuid;

use crate::generation::{
    self,
    trip_generation::{ParticipantData, TripData},
};

struct ValidRow {
    user_id: Uuid,
    session_id: Uuid,
    camera_id: u8,
    num_pics: u8,
}

impl ValidRow {
    fn new(session_id: Uuid, participant_data: ParticipantData) -> Self {
        Self {
            user_id: participant_data.user_id,
            session_id,
            camera_id: participant_data.camera_id,
            num_pics: participant_data.num_pics,
        }
    }
}
impl fmt::Display for ValidRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.user_id, self.session_id, self.camera_id, self.num_pics
        )
    }
}

impl TripData {
    fn into_valid_rows(self) -> Vec<ValidRow> {
        let session_id = self.session_id;
        let valid_rows: Vec<ValidRow> = self
            .into_participants_data()
            .into_iter()
            .map(|participant_data| ValidRow::new(session_id, participant_data))
            .collect();
        valid_rows
    }
}

fn write_rows_from_trips<W: Write, T: Rng>(
    trips: &mut Vec<TripData>,
    buf_writer: &mut BufWriter<W>,
    rng: &mut T,
) -> Result<()> {
    const CORRUPTED_ROW: &str = "This row is corrupted";
    const FLUSH_THRESHOLD: usize = 1000; // Flush when the difference between the buffers internal capacity and the buffered data is below this threshold.

    for trip in trips.drain(..) {
        if buf_writer.capacity() - buf_writer.buffer().len() < FLUSH_THRESHOLD {
            buf_writer.flush()?;
        }
        for valid_row in trip.into_valid_rows() {
            writeln!(buf_writer, "{}", valid_row)?;
            if rng.gen_bool(0.001) {
                writeln!(buf_writer, "{}", CORRUPTED_ROW)?;
            }
        }
    }
    buf_writer.flush()?;
    Ok(())
}

pub(crate) fn write_synthetic_data_single_day<T: Rng>(
    path: PathBuf,
    num_sessions: usize,
    rng: &mut T,
) -> Result<()> {
    let file = File::create(path)?;

    const BATCH_SIZE: usize = 10usize.pow(4);
    const BUFFER_SIZE: usize = 10usize.pow(8);

    let mut batch_of_trips: Vec<TripData> = Vec::with_capacity(BATCH_SIZE);
    let mut buf_writer = BufWriter::with_capacity(BUFFER_SIZE, file);
    let mut remaining_number_of_sessions = num_sessions;

    while remaining_number_of_sessions > 0 {
        let num_sessions_to_generate_this_iteration =
            std::cmp::min(BATCH_SIZE, remaining_number_of_sessions);
        remaining_number_of_sessions -= num_sessions_to_generate_this_iteration;
        fill_batch_of_trips(
            &mut batch_of_trips,
            num_sessions_to_generate_this_iteration,
            rng,
        );
        // we assume that the data associated with a trip is written as soon as the trip finishes. Hence the trips for a session cannot appear directly
        // after one another in the resulting file. We shuffle our batch of trips to emulate this effect.
        batch_of_trips.shuffle(rng);
        write_rows_from_trips(&mut batch_of_trips, &mut buf_writer, rng)?;
    }

    Ok(())
}

fn fill_batch_of_trips<T: Rng>(
    batch_of_trips: &mut Vec<TripData>,
    num_sessions_to_generate_this_iteration: usize,
    rng: &mut T,
) {
    for _ in (0..num_sessions_to_generate_this_iteration).into_iter() {
        let synthetic_session = generation::session_generation::generate_session(rng);
        let num_trips_for_session =
            generation::session_generation::generate_number_of_trips(&synthetic_session, rng);
        for trip in iter::repeat_with(|| {
            generation::trip_generation::generate_trip_data(&synthetic_session, rng)
        })
        .take(num_trips_for_session as usize)
        {
            batch_of_trips.push(trip);
        }
    }
}
