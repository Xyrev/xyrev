mod copilot;
mod init;

use crate::cli::agent::{AgentCommand, Commands};

impl AgentCommand {
    pub fn run(&self, verbose: bool) {
        match &self.command {
            Commands::Init(cmd) => cmd.run(verbose),
            Commands::Copilot(cmd) => cmd.run(verbose),
        }
    }
}
