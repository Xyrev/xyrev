use clap::Args;

#[derive(Args, Debug)]
#[command(
    about = "Create a new configuration folder or reinitialize the existing one",
    after_help = "For example:\n  stool dotfile init https://github.com/YOUR_NAME/dotfiles"
)]
pub struct InitCommand {
    #[arg(
        short,
        long,
        help = "Force reinitialize configuration (overwrite existing)"
    )]
    pub force: bool,

    #[arg(
        short = 'C',
        long,
        help = "Clean and reinitialize (remove all existing data)"
    )]
    pub clean: bool,
}
