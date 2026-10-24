//! Command and argument definitions; execution lives in the feature modules.

pub mod agent;
#[cfg(target_os = "macos")]
pub mod apple;
pub mod dotfile;
pub mod init;
pub mod key;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "stool",
    about = "Developer environment, dotfiles and offline key backups",
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
    Key(key::KeyCommand),
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
