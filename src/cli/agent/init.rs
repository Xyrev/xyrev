use clap::Args;

#[derive(Args, Debug)]
pub struct InitCommand {
    #[arg(help = "Agent name")]
    pub name: String,
}
