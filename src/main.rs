use clap::Clap;

// Necessary to get the path to a file as *const i8

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
            if duration.is_some() {
                unimplemented!("We don't do that atm");
            }
            ske.config(&opts.configuration)?;
            ske.run();
        }
        Check => {
            ske.config(&opts.configuration)?;
        } //_=>{
          //    unimplemented!("Not yet implemented");
          //}
    }

    Ok(())
}
