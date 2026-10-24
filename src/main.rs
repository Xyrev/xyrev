mod agent;
#[cfg(target_os = "macos")]
mod apple;
mod cli;
mod common;
mod dotfile;
mod help;
mod init;
mod key;

use clap::Parser;
use cli::Cli;

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        cli::Commands::Init(cmd) => init::run(&cmd, cli.verbose)?,
        cli::Commands::Agent(cmd) => cmd.run(cli.verbose),
        #[cfg(target_os = "macos")]
        cli::Commands::Apple(cmd) => apple::run(&cmd, cli.verbose)?,
        cli::Commands::Dotfile(cmd) => dotfile::run(&cmd, cli.verbose)?,
        cli::Commands::Key(cmd) => key::run(&cmd)?,
        cli::Commands::Completions { shell } => help::completions(shell),
        cli::Commands::Man { output } => help::man(output.as_deref())?,
    }
    Ok(())
}
