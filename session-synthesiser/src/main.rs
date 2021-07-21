use anyhow::{Context, Result};
use rand::prelude::*;
use rand_pcg::Pcg64;
use structopt::StructOpt;

/// Produces a weeks worth of synthetic daily Safari session records.
#[derive(StructOpt)]
struct Cli {
    /// The path to the directory to write to
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,

    /// The number of sessions per day
    #[structopt(short, long)]
    number_of_sessions: usize,

    /// Set seed to get reproducible results on consecutive runs
    #[structopt(short = "s", long = "seed", default_value = "1")]
    seed: u64,
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let path = &args.path;
    if !path.as_path().is_dir() {
        println!(
            "The specified directory \"{:?}\" does not exists. We will try to create it!",
            path.as_path().as_os_str()
        );
        std::fs::create_dir(path)
        .with_context(|| format!("Could not create the directory \"{:?}\". Please make sure that the parent directory exists.", path.as_os_str()))?;
        println!("The directory was successfully created!");
    }
    let seed = args.seed;
    let mut rng = Pcg64::seed_from_u64(seed);
    let num_sessions = args.number_of_sessions;
    session_synthesiser::run(path.clone(), num_sessions, &mut rng)
        .with_context(|| "The creation of the synthetic sessions failed")?;
    println!(
        "The files have been successfully written in {}",
        path.to_str().unwrap()
    );

    Ok(())
}
