use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cmd {
    #[clap(flatten)]
    pub opt: Opt,
    #[clap(subcommand)]
    pub sub: SubCommand,
}

/// Utility to store encrypted secrets in version trackable plain text files.
#[derive(Parser, Debug)]
pub struct Opt {
    /// Turn on verbose output
    #[clap(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Display Weather Information
    Info {
        /// Station code
        #[clap(long, default_value = "VOBL")]
        station_id: String,
    },
}

pub(crate) fn init() -> Cmd {
    Cmd::parse()
}
