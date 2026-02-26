use clap::Args;

#[derive(Args, Debug)]
pub struct DotfileCommand {
    #[arg(help = "Dotfile name")]
    pub name: String,
}
