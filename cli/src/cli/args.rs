use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "phantomrelayctl")]
#[command(version)]
#[command(about = "PhantomRelay Control CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Start { service: String },

    Stop { service: String },

    Restart { service: String },

    Enable { mode: String },

    Disable { mode: String },

    Status,

    Shutdown,
}
