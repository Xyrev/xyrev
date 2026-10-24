use clap::Args;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// HTTPS, SSH, or file:// Git URL
    pub repo: String,
    #[arg(long)]
    pub branch: Option<String>,
    /// Deploy after cloning
    #[arg(long)]
    pub apply: bool,
    #[arg(long)]
    pub recurse_submodules: bool,
    /// Clone using the built-in Git backend
    #[arg(long)]
    pub builtin: bool,
}
