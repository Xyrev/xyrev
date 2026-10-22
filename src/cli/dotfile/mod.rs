use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
#[command(about = "Manage dotfiles from a Git repository")]
pub struct DotfileCommand {
    /// Local source repository (saved by init)
    #[arg(long, global = true)]
    pub source: Option<PathBuf>,
    /// Destination directory (defaults to the user's home)
    #[arg(long, global = true)]
    pub destination: Option<PathBuf>,
    /// Override template data (key=value; JSON values are accepted)
    #[arg(long, global = true)]
    pub data: Vec<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Clone a Git URL and save configuration
    Init(InitArgs),
    /// Preview content and permission changes without writing
    Diff,
    /// List pending changes and conflicts
    Status,
    /// Deploy the local source repository
    Apply(ApplyArgs),
    /// Fast-forward the source repository and deploy changes
    Update,
    /// List managed destination paths
    Managed,
    /// Import existing destination files into source state
    Add(AddArgs),
    /// Edit source files (encrypted files are decrypted only in a private temporary directory)
    Edit(EditArgs),
    /// List destination files not managed by source
    Unmanaged,
    /// Return nonzero when the destination differs from source
    Verify,
    /// Diagnose repository, configuration, state and optional executables
    Doctor,
    /// Export rendered target files as tar, tar.gz, or zip
    Archive(ArchiveArgs),
    /// Evaluate a Jinja template using repository and local data
    ExecuteTemplate(TemplateArgs),
    /// Print merged template data as JSON
    Data,
    /// Back up or restore GPG keys using encrypted dotfile storage
    Gpg(GpgArgs),
    /// Generate an age identity and print its public recipient
    Keygen {
        #[arg(long)]
        output: PathBuf,
    },
    /// Start a shell in the source repository, or print its path
    Cd {
        #[arg(long)]
        print: bool,
    },
}

#[derive(Args, Debug)]
pub struct AddArgs {
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,
    #[arg(long)]
    pub template: bool,
    #[arg(long)]
    pub encrypt: bool,
}
#[derive(Args, Debug)]
pub struct EditArgs {
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,
    #[arg(long)]
    pub apply: bool,
}
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum ArchiveFormat {
    Tar,
    Tgz,
    Zip,
}
#[derive(Args, Debug)]
pub struct ArchiveArgs {
    #[arg(long, value_enum, default_value = "tar")]
    pub format: ArchiveFormat,
    #[arg(short, long)]
    pub output: PathBuf,
}
#[derive(Args, Debug)]
pub struct TemplateArgs {
    pub template: Option<String>,
    #[arg(long, conflicts_with = "template")]
    pub file: Option<PathBuf>,
    #[arg(long)]
    pub init: bool,
}
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

#[derive(Args, Debug, Default)]
pub struct ApplyArgs {
    #[arg(long)]
    pub dry_run: bool,
    /// Back up and overwrite conflicting destination entries
    #[arg(long)]
    pub force: bool,
    /// Interactive conflict handling (auto only prompts on a terminal)
    #[arg(long, value_enum, default_value = "auto")]
    pub prompt: Prompt,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default)]
pub enum Prompt {
    #[default]
    Auto,
    Always,
    Never,
}

impl DotfileCommand {
    pub fn run(&self, verbose: bool) -> anyhow::Result<()> {
        crate::dotfile::run(self, verbose)
    }
}

#[derive(Args, Debug)]
pub struct GpgArgs {
    #[command(subcommand)]
    pub command: GpgCommands,
}

#[derive(Subcommand, Debug)]
pub enum GpgCommands {
    /// Export selected GPG keys and ownertrust into an encrypted backup
    Backup(GpgBackupArgs),
    /// Validate and restore an encrypted GPG key backup
    Restore(GpgRestoreArgs),
}

#[derive(Args, Debug)]
pub struct GpgBackupArgs {
    /// Complete primary GPG fingerprint; repeat to select multiple keys
    #[arg(long, required = true)]
    pub fingerprint: Vec<String>,
    /// Backup directory (defaults to .xyrevtemplates/gpg in the source)
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Separate age identity kept outside the repository
    #[arg(long, conflicts_with = "passphrase_file")]
    pub identity: Option<PathBuf>,
    /// Read the password from private external storage or an inherited pipe
    #[arg(long)]
    pub passphrase_file: Option<PathBuf>,
    /// GPG executable (GPG Suite is detected on macOS)
    #[arg(long)]
    pub gpg_program: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct GpgRestoreArgs {
    /// Backup directory (defaults to .xyrevtemplates/gpg in the source)
    #[arg(long)]
    pub input: Option<PathBuf>,
    /// Separate age identity for identity-encrypted backups
    #[arg(long, conflicts_with = "passphrase_file")]
    pub identity: Option<PathBuf>,
    /// Read the password from a private file outside the repository
    #[arg(long)]
    pub passphrase_file: Option<PathBuf>,
    /// GPG executable (GPG Suite is detected on macOS)
    #[arg(long)]
    pub gpg_program: Option<PathBuf>,
}
