use clap::Args;

#[derive(Args, Debug)]
pub struct AppleCommand {
    #[arg(help = "Dotfile name")]
    pub name: String,
}
