use clap::Args;

#[derive(Args, Debug)]
pub struct DotfileCommand {
    #[arg(
        short,
        long,
        action = clap::ArgAction::Count,
        help = "Use verbose output (-vv for very verbose)"
    )]
    verbose: u8,

    #[arg(help = "Dotfile name")]
    pub name: String,
}
