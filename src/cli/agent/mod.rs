mod copilot;
mod init;

use clap::{Args, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(init::InitCommand),

    Copilot(copilot::CopilotCommand),
}

#[derive(Args, Debug)]
pub struct AgentCommand {
    #[command(subcommand)]
    pub command: Commands,
}

impl AgentCommand {
    pub fn run(&self, verbose: u8) {
        match &self.command {
            Commands::Init(cmd) => cmd.run(verbose),
            Commands::Copilot(cmd) => cmd.run(verbose),
        }
    }
}
