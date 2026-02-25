use clap::Args;

#[derive(Args, Debug)]
pub struct DotfileCommand {
    #[arg(
        short,
        long,
        action = clap::ArgAction::SetTrue,
        help = "Use verbose output"
    )]
    verbose: bool,

    #[arg(help = "Dotfile name")]
    pub name: String,
}
