use anyhow::{Context, Result};
use structopt::StructOpt;

/// Produces a text file containing the top 100 number of average pics per camera over the last seven days.
#[derive(StructOpt)]
struct Cli {
    /// The path to the folder where session log files can be found
    #[structopt(parse(from_os_str))]
    from_path: std::path::PathBuf,

    /// The path to the folder where camera_top_100_YYYYMMDD.txt is to be written.
    /// We will attempt to create this directory if it does not already exist.
    #[structopt(parse(from_os_str))]
    to_path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let (from_path, to_path) = (args.from_path, args.to_path);
    if !to_path.exists() {
        std::fs::create_dir_all(to_path.as_path()).with_context(|| {
            format!(
                "could not create directory: {:?}",
                to_path.as_path().as_os_str()
            )
        })?;
    }
    // This is a directory where the best camera stats found in a single log file is stored for reuse.
    let serialization_directory =
        camera_stats::configuration::SerializationFilesConfig::serialization_directory();
    if !serialization_directory.exists() {
        std::fs::create_dir_all(serialization_directory.as_path()).with_context(|| {
            format!(
                "could not create directory: {:?}",
                serialization_directory.as_path().as_os_str()
            )
        })?;
    }
    camera_stats::run(from_path, to_path)
}
