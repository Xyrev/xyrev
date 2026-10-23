pub mod agent;
#[cfg(target_os = "macos")]
pub mod apple;
pub mod dotfile;
pub mod init;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "stool",
    about = "Developer environment and Git-backed dotfile management",
    version,
    disable_help_subcommand = false,
    color = clap::ColorChoice::Auto,
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
    Init(init::InitCommand),

    Agent(agent::AgentCommand),

    #[cfg(target_os = "macos")]
    Apple(apple::AppleCommand),

    Dotfile(dotfile::DotfileCommand),
    /// Generate shell completions from the current command definitions
    Completions {
        shell: clap_complete::Shell,
    },
    /// Render the manual, or write all command manuals to a directory
    Man {
        #[arg(long)]
        output: Option<std::path::PathBuf>,
    },
}
