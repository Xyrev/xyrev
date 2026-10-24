use clap::Args;

#[derive(Args, Debug)]
pub struct CopilotCommand {
    /// Install method (auto, brew, npm, winget, script)
    #[arg(short, long, default_value = "auto")]
    pub method: String,

    /// Install prerelease version
    #[arg(long)]
    pub prerelease: bool,

    /// Specific version to install (e.g., v0.0.369)
    #[arg(long)]
    pub version: Option<String>,

    /// Custom installation prefix (for script method)
    #[arg(long)]
    pub prefix: Option<String>,
}
