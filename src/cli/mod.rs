pub mod agent;
#[cfg(target_os = "macos")]
pub mod apple;
pub mod dotfile;
pub mod init;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "xyrev",
    about = "A helpful utility designed to make xyrev effortless.",
    version,
    disable_help_subcommand = false,
    color = clap::ColorChoice::Always,
    args_conflicts_with_subcommands = false
)]
pub struct Cli {
    #[arg(
        short,
        long,
        global = true,
        action = clap::ArgAction::SetTrue,
        help = "Show more debugging information"
    )]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Create a new configuration folder or reinitialize the existing one",
        after_help = "For example:\n  xyrev build --release"
    )]
    Init(init::InitCommand),

    Agent(agent::AgentCommand),

    #[cfg(target_os = "macos")]
    Apple(apple::AppleCommand),

    Dotfile(dotfile::DotfileCommand),
}
