mod preferences;

use crate::cli::apple::{AppleCommand, Commands};
use anyhow::Result;

pub fn run(command: &AppleCommand, verbose: bool) -> Result<()> {
    match &command.command {
        Commands::Init(args) => preferences::init(args.dry_run, verbose),
    }
}
