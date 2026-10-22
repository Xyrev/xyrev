mod cli;
mod common;
mod dotfile;
mod help;

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
        cli::Commands::Init(cmd) => cmd.run(cli.verbose),
        cli::Commands::Agent(cmd) => cmd.run(cli.verbose),
        #[cfg(target_os = "macos")]
        cli::Commands::Apple(cmd) => println!("Apple {}", cmd.name),
        cli::Commands::Dotfile(cmd) => cmd.run(cli.verbose)?,
        cli::Commands::Completions { shell } => help::completions(shell),
        cli::Commands::Man { output } => help::man(output.as_deref())?,
    }
    Ok(())
}
