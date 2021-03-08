use clap::Clap;
use std::path::PathBuf;

/// ske-rs, a home for SKE server
///
/// Use this tool test a XNG configuration in Separation Kernel Emulator (SKE).
#[derive(Clap)]
#[clap(version, author)]
pub struct Opts {
    /// SKE library file
    #[clap(long = "libske")]
    pub libfile: Option<PathBuf>,

    /// The XNG configuration to execute
    #[clap(default_value = "module.xml")]
    pub configuration: PathBuf,

    #[clap(subcommand)]
    pub cmd: Cmd,
}

#[derive(Clap)]
pub enum Cmd {
    /// Run the given configuration
    Run {
        /// If given, the configuration runs for <duration> seconds. Scientific notation is
        /// supported, e.g. `1e-3` results in 1 ms run time.
        #[clap()]
        duration: Option<f64>,
    },

    /// Validate a configuration. This will try to load the configuration in SKE.
    Check,
}
