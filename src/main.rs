use clap::Clap;

mod bindings;
mod cli;

use bindings::SkeServer;
use cli::Cmd::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = cli::Opts::parse();

    let ske = match opts.libfile {
        None => SkeServer::new()?,
        Some(path) => SkeServer::from_file(&path)?,
    };

    match opts.cmd {
        Run { duration } => {
            ske.config(&opts.configuration)?;
            let duration_us = duration
                .map(|d| d * 1e6)
                .map(|d_us| d_us.round() as i64)
                .unwrap_or(-1);
            ske.run(duration_us);
        }
        Check => {
            ske.config(&opts.configuration)?;
        }
    }

    Ok(())
}
