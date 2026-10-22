use super::storage::{Config, Paths};
use crate::cli::dotfile::InitArgs;
use anyhow::{bail, Context, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

/// Keep canonical paths internally, adapting only arguments passed to Git.
fn system_path(path: &Path) -> &Path {
    #[cfg(windows)]
    {
        dunce::simplified(path)
    }
    #[cfg(not(windows))]
    {
        path
    }
}

fn command(directory: Option<&Path>) -> Command {
    let mut cmd = Command::new("git");
    if let Some(directory) = directory {
        cmd.arg("-C").arg(system_path(directory));
    }
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
    ] {
        cmd.env_remove(key);
    }
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd
}

fn checked(mut command: Command) -> Result<Output> {
    let output = command
        .output()
        .map_err(super::prerequisites::git_spawn_error)?;
    if !output.status.success() {
        bail!(
            "Git failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output)
}

pub fn run<I, S>(directory: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<String> = args
        .into_iter()
        .map(|arg| arg.as_ref().to_string_lossy().into_owned())
        .collect();
    if super::native_git::enabled(directory) {
        if let Some(output) = super::native_git::run(directory, &args)? {
            return Ok(output);
        }
    }
    let mut cmd = command(Some(directory));
    cmd.args(args);
    let output = checked(cmd)?;
    Ok(String::from_utf8(output.stdout)
        .context("Git returned non-UTF-8 output")?
        .trim_end()
        .to_owned())
}

pub fn validate_url(url: &str) -> Result<()> {
    if url.chars().any(char::is_control) {
        bail!("Git URL contains control characters");
    }
    if let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("ssh://"))
    {
        let (authority, path) = rest
            .split_once('/')
            .context("Git URL needs a host and repository path")?;
        if authority.is_empty() || path.is_empty() {
            bail!("Git URL needs a host and repository path");
        }
        if url.starts_with("https://") && authority.contains('@') {
            bail!("Use a Git credential helper instead of embedding credentials in the URL");
        }
        if let Some((user, _)) = authority.rsplit_once('@') {
            if user.contains(':') {
                bail!("Do not embed passwords in the Git URL");
            }
        }
        return Ok(());
    }
    if let Some(path) = url.strip_prefix("file://") {
        if path.starts_with('/') {
            return Ok(());
        }
    }
    if let Some((host, path)) = url.split_once(':') {
        if let Some((user, hostname)) = host.split_once('@') {
            if !user.is_empty()
                && !hostname.is_empty()
                && !path.is_empty()
                && host
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "@._-".contains(c))
                && !host.starts_with('-')
            {
                return Ok(());
            }
        }
    }
    bail!("Expected an HTTPS, SSH, git@host:path, or absolute file:// Git URL");
}

pub fn clone_repository(
    url: &str,
    destination: &Path,
    branch: Option<&str>,
    recursive: bool,
) -> Result<()> {
    validate_url(url)?;
    let mut cmd = command(None);
    cmd.arg("clone");
    if let Some(branch) = branch {
        cmd.arg("--branch").arg(branch);
    }
    if recursive {
        cmd.arg("--recurse-submodules");
    }
    cmd.arg("--").arg(url).arg(system_path(destination));
    checked(cmd)?;
    ready(&fs::canonicalize(destination)?)
}

pub fn lfs(paths: &Paths) -> Result<()> {
    if paths.config.as_ref().is_some_and(|cfg| cfg.git.lfs) {
        run(&paths.source, ["lfs", "version"]).context("Install git-lfs before using git.lfs")?;
        run(&paths.source, ["lfs", "pull"]).context("Download Git LFS objects")?;
    }
    Ok(())
}

pub fn verify(source: &Path) -> Result<()> {
    if !source.join(".git").exists() {
        bail!(
            "Source is not an initialized Git repository: {}. Run xyrev dotfile init <git-url>",
            source.display()
        );
    }
    let top = run(source, ["rev-parse", "--show-toplevel"])?;
    if fs::canonicalize(top)? != source {
        bail!("Source must be the repository root");
    }
    run(source, ["rev-parse", "--verify", "HEAD"])
        .context("The source repository must contain a commit")?;
    Ok(())
}

pub fn ready(source: &Path) -> Result<()> {
    verify(source)?;
    let status = run(source, ["submodule", "status", "--recursive"])?;
    if status
        .lines()
        .any(|line| matches!(line.as_bytes().first(), Some(b'-' | b'+' | b'U')))
    {
        bail!("Submodules are missing or do not match the recorded commits. Initialize with --recurse-submodules or run git -C '{}' submodule update --init --recursive", source.display());
    }
    Ok(())
}

pub fn initialize(paths: &Paths, args: &InitArgs) -> Result<()> {
    validate_url(&args.repo)?;
    if let Some(branch) = &args.branch {
        if branch.starts_with('-') {
            bail!("Invalid branch name");
        }
        gix::refs::FullName::try_from(format!("refs/heads/{branch}"))
            .context("Invalid branch name")?;
    }
    let system_backend = if paths.source.exists() {
        !super::native_git::enabled(&paths.source)
    } else {
        !args.builtin
    };
    if system_backend || args.recurse_submodules || paths.source.join(".gitmodules").exists() {
        super::prerequisites::git().context("Check Git before initializing dotfiles")?;
    }
    if fs::symlink_metadata(&paths.source).is_ok() {
        verify(&paths.source)?;
        if run(&paths.source, ["remote", "get-url", "origin"])? != args.repo {
            bail!(
                "Source origin differs from the requested URL; choose another --source directory"
            );
        }
        let actual_branch = run(&paths.source, ["symbolic-ref", "--short", "HEAD"])?;
        let expected = args.branch.as_ref().or_else(|| {
            paths
                .config
                .as_ref()
                .filter(|cfg| cfg.source == paths.source)
                .map(|cfg| &cfg.branch)
        });
        if expected.is_some_and(|expected| expected != &actual_branch) {
            bail!("Source branch differs from the requested/saved branch; existing files were preserved");
        }
        if args.recurse_submodules {
            ensure_clean(&paths.source)?;
            run(
                &paths.source,
                ["submodule", "update", "--init", "--recursive"],
            )?;
        }
        ready(&paths.source)?;
        println!("Reusing {}", paths.source.display());
    } else {
        let parent = paths.source.parent().context("Source has no parent")?;
        fs::create_dir_all(parent)?;
        let temporary = tempfile::Builder::new()
            .prefix(".xyrev-clone-")
            .tempdir_in(parent)?;
        let checkout = temporary.path().join("repository");
        if args.builtin {
            super::native_git::clone(&args.repo, &checkout, args.branch.as_deref())?;
            if args.recurse_submodules {
                run(&checkout, ["submodule", "update", "--init", "--recursive"])?;
            }
        } else {
            clone_repository(
                &args.repo,
                &checkout,
                args.branch.as_deref(),
                args.recurse_submodules,
            )?;
        }
        ready(&fs::canonicalize(&checkout)?)?;
        fs::rename(&checkout, &paths.source)?;
        println!("Cloned {}", paths.source.display());
    }
    let recurse_submodules = args.recurse_submodules
        || paths
            .config
            .as_ref()
            .filter(|cfg| cfg.source == paths.source)
            .is_some_and(|cfg| cfg.recurse_submodules);
    let config = Config {
        version: 1,
        repo: args.repo.clone(),
        branch: run(&paths.source, ["symbolic-ref", "--short", "HEAD"])?,
        source: paths.source.clone(),
        destination: paths.destination.clone(),
        recurse_submodules,
        data: paths
            .config
            .as_ref()
            .map(|cfg| cfg.data.clone())
            .unwrap_or_else(super::storage::empty_object),
        encryption: paths.encryption(),
        git: paths
            .config
            .as_ref()
            .map(|cfg| cfg.git.clone())
            .unwrap_or_default(),
        editor: paths.config.as_ref().and_then(|cfg| cfg.editor.clone()),
    };
    let mut config = config;
    config.git.builtin |= args.builtin;
    paths.save_config(&config)?;
    println!(
        "Source: {}\nDestination: {}",
        paths.source.display(),
        paths.destination.display()
    );
    if !args.apply {
        println!("Next: xyrev dotfile diff; xyrev dotfile apply");
    }
    Ok(())
}

fn ensure_clean(source: &Path) -> Result<()> {
    if !run(
        source,
        [
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--ignore-submodules=none",
        ],
    )?
    .is_empty()
    {
        bail!("Source has local changes (including untracked files/submodules); commit or move them before update");
    }
    Ok(())
}

pub fn update(paths: &Paths) -> Result<()> {
    let config = paths
        .config
        .as_ref()
        .context("Run xyrev dotfile init <git-url> before update")?;
    validate_url(&config.repo)?;
    if config.source != paths.source {
        bail!("Selected source differs from saved configuration; initialize it first");
    }
    verify(&paths.source)?;
    if run(&paths.source, ["remote", "get-url", "origin"])? != config.repo
        || run(&paths.source, ["symbolic-ref", "--short", "HEAD"])? != config.branch
    {
        bail!("Source origin or branch differs from saved configuration");
    }
    if super::native_git::enabled(&paths.source) && !paths.source.join(".gitmodules").exists() {
        return super::native_git::update(paths);
    }
    ensure_clean(&paths.source)?;
    run(&paths.source, ["fetch", "--prune", "origin"])?;
    let upstream = format!("refs/remotes/origin/{}", config.branch);
    run(&paths.source, ["rev-parse", "--verify", &upstream])?;
    let mut cmd = command(Some(&paths.source));
    cmd.args(["merge-base", "--is-ancestor", "HEAD", &upstream]);
    let output = cmd.output().context("Check fast-forward update")?;
    if output.status.code() == Some(1) {
        bail!("Source has local commits or has diverged from origin; only fast-forward updates are allowed");
    }
    if !output.status.success() {
        bail!(
            "Cannot check fast-forward update: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    run(
        &paths.source,
        ["merge", "--ff-only", "--no-edit", &upstream],
    )?;
    if config.recurse_submodules {
        run(&paths.source, ["submodule", "update", "--init", "--recursive"])
            .context("Source was fast-forwarded, but submodule checkout failed; repair submodules before apply")?;
    }
    ready(&paths.source)?;
    println!(
        "Updated source to {}",
        run(&paths.source, ["rev-parse", "--short", "HEAD"])?
    );
    Ok(())
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;

    #[test]
    fn git_arguments_use_compatible_paths_for_canonical_directories() {
        let canonical = Path::new(r"\\?\C:\dotfiles checkout");
        assert_eq!(system_path(canonical), Path::new(r"C:\dotfiles checkout"));
        let command = command(Some(canonical));
        let arguments: Vec<_> = command.get_args().collect();
        assert_eq!(
            arguments,
            [OsStr::new("-C"), OsStr::new(r"C:\dotfiles checkout")]
        );
    }

    #[test]
    fn git_paths_preserve_unc_long_and_verbatim_only_names() {
        for path in [
            Path::new(r"\\?\UNC\server\share\dotfiles"),
            Path::new(r"\\?\C:\COM1"),
            Path::new(r"\\?\C:\dotfiles."),
        ] {
            assert_eq!(system_path(path), path);
        }
        let long = std::path::PathBuf::from(format!(r"\\?\C:\{}", "segment\\".repeat(40)));
        assert_eq!(system_path(&long), long);
    }
}
