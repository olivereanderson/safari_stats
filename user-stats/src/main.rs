use anyhow::{Context, Result};
use structopt::StructOpt;

/// Produces a text file containing the top 10 number of pics per user in sessions over the last seven days.
#[derive(StructOpt)]
struct Cli {
    /// The path to the directory where session log files can be found.
    #[structopt(parse(from_os_str))]
    from_path: std::path::PathBuf,

    /// The path to the directory where user_top_10_YYYYMMDD.txt is to be written.
    /// We will attempt to write this directory if it does not already exist.
    #[structopt(parse(from_os_str))]
    to_path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let (from_path, to_path) = (args.from_path, args.to_path);
    if !to_path.exists() {
        std::fs::create_dir_all(to_path.as_path()).with_context(|| {
            format!(
                "Failed creating directory: {:?}",
                to_path.as_path().as_os_str()
            )
        })?;
    }

    // This is the directory where the top 10 number of pics per user found in a single log file is stored.
    let storage_directory =
        user_stats::configuration::SavedFstSetFilesConfig::storage_directory();
    if !storage_directory.exists() {
        std::fs::create_dir_all(storage_directory.as_path()).with_context(|| {
            format!(
                "Could not create directory: {:?}",
                storage_directory.as_os_str()
            )
        })?;
    }
    user_stats::run(from_path, to_path)
}
