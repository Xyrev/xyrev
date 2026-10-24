mod copilot;
mod init;

pub use copilot::CopilotCommand;
pub use init::InitCommand;

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct AgentCommand {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(InitCommand),

    Copilot(CopilotCommand),
}
