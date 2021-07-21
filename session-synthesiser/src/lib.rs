use std::path::PathBuf;

use anyhow::Result;
use rand::Rng;

mod generation;
mod writing;

pub fn run<T: Rng>(directory_path: PathBuf, num_sessions: usize, rng: &mut T) -> Result<()> {
    let dates = common_utils::date_utils::last_seven_days_ymd();
    for date in dates {
        let mut file_path = directory_path.clone();
        file_path.push(format!("safari-sessions-{}.log", date.into_string()));
        writing::write_synthetic_data_single_day(file_path, num_sessions, rng)?
    }
    Ok(())
}
