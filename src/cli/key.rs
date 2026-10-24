use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
#[command(about = "Back up and restore GPG and SSH keys independently of dotfiles")]
pub struct KeyCommand {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create one password-encrypted offline key backup outside Git workspaces
    Backup(BackupArgs),
    /// Validate and manually restore an offline key backup
    Restore(RestoreArgs),
}

#[derive(Args, Debug, Default)]
pub struct Selection {
    /// Only GPG keys
    #[arg(long, conflicts_with_all = ["ssh", "all"])]
    pub gpg: bool,
    /// Only SSH keys
    #[arg(long, conflicts_with_all = ["gpg", "all"])]
    pub ssh: bool,
    /// All available key categories (the default)
    #[arg(long, conflicts_with_all = ["gpg", "ssh"])]
    pub all: bool,
}
impl Selection {
    pub fn gpg(&self) -> bool {
        !self.ssh
    }
    pub fn ssh(&self) -> bool {
        !self.gpg
    }
}

#[derive(Args, Debug)]
pub struct BackupArgs {
    #[command(flatten)]
    pub selection: Selection,
    /// Source user directory (defaults to the current user environment)
    #[arg(long)]
    pub input: Option<PathBuf>,
    /// New backup file (defaults to stool-keys-<timestamp>.tar.age in the current directory)
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Read the password from a private external file or inherited pipe
    #[arg(long)]
    pub passphrase_file: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct RestoreArgs {
    #[command(flatten)]
    pub selection: Selection,
    /// Password-encrypted stool key backup file
    #[arg(long)]
    pub input: PathBuf,
    /// Target user directory (defaults to the current user's home)
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Read the password from a private external file
    #[arg(long)]
    pub passphrase_file: Option<PathBuf>,
}
