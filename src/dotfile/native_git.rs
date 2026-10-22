//! Built-in Git plumbing. Checkout updates are prepared in a sibling directory,
//! including preserved local Git metadata, then installed by rename after a clean check.
use super::storage::{self, Paths};
use anyhow::{bail, Context, Result};
use gix::bstr::ByteSlice;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub fn enabled(source: &Path) -> bool {
    source.join(".git/xyrev-builtin").is_file()
}
pub fn mark(source: &Path) -> Result<()> {
    storage::atomic_write(&source.join(".git/xyrev-builtin"), b"1\n", 0o600)
}

pub fn clone(url: &str, destination: &Path, branch: Option<&str>) -> Result<()> {
    let mut fetch = gix::prepare_clone(url, destination)
        .context("Prepare built-in Git clone")?
        .with_in_memory_config_overrides(["user.name=xyrev", "user.email=xyrev@localhost"]);
    let interrupt = AtomicBool::new(false);
    let (mut checkout, _) = fetch
        .fetch_then_checkout(gix::progress::Discard, &interrupt)
        .context("Fetch with built-in Git")?;
    if let Some(branch) = branch {
        let full = format!("refs/heads/{branch}");
        let repo = checkout.repo();
        let id = repo
            .find_reference(&format!("refs/remotes/origin/{branch}"))?
            .peel_to_id_in_place()?
            .detach();
        repo.reference(
            full.as_str(),
            id,
            gix::refs::transaction::PreviousValue::Any,
            "xyrev clone branch",
        )?;
        storage::atomic_write(
            &repo.git_dir().join("HEAD"),
            format!("ref: {full}\n").as_bytes(),
            0o644,
        )?;
    }
    let (repo, outcome) = checkout
        .main_worktree(gix::progress::Discard, &interrupt)
        .context("Checkout with built-in Git")?;
    if !outcome.errors.is_empty() || !outcome.collisions.is_empty() {
        bail!("Built-in Git checkout encountered errors or path collisions");
    }
    repo.head_id().context("Repository must contain a commit")?;
    mark(destination)?;
    Ok(())
}

pub fn run(source: &Path, args: &[String]) -> Result<Option<String>> {
    let repo = gix::open(source).context("Open built-in Git repository")?;
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    Ok(Some(match words.as_slice() {
        ["rev-parse", "--show-toplevel"] => repo
            .work_dir()
            .context("Source has no worktree")?
            .to_string_lossy()
            .into_owned(),
        ["rev-parse", "--verify", "HEAD"] | ["rev-parse", "HEAD"] => repo.head_id()?.to_string(),
        ["rev-parse", "--short", "HEAD"] => repo.head_id()?.to_string()[..7].to_owned(),
        ["symbolic-ref", "--short", "HEAD"] => repo
            .head_name()?
            .context("Source HEAD is detached")?
            .as_bstr()
            .to_str()?
            .strip_prefix("refs/heads/")
            .context("Source HEAD is not a branch")?
            .to_owned(),
        ["remote", "get-url", "origin"] => repo
            .find_remote("origin")?
            .url(gix::remote::Direction::Fetch)
            .context("Missing origin URL")?
            .to_bstring()
            .to_str()?
            .to_owned(),
        ["status", ..] => {
            if clean(source).is_ok() {
                String::new()
            } else {
                " M source".into()
            }
        }
        ["submodule", "status", "--recursive"] if !source.join(".gitmodules").exists() => {
            String::new()
        }
        _ => return Ok(None),
    }))
}

pub fn clean(source: &Path) -> Result<()> {
    let repo = gix::open(source)?;
    if repo.is_dirty()? {
        bail!("Source has local changes; commit or move them before update");
    }
    let files = repo
        .head_commit()?
        .tree()?
        .traverse()
        .breadthfirst
        .files()?;
    let index = repo.index()?;
    let mut known = BTreeSet::new();
    for entry in files {
        let relative = storage::relative_path(Path::new(entry.filepath.to_str()?))?;
        if entry.mode.is_tree() {
            continue;
        }
        if entry.mode.is_commit() {
            bail!("Built-in update with submodules requires the system Git fallback");
        }
        let indexed = index
            .entries()
            .iter()
            .find(|indexed| indexed.path(&index) == entry.filepath.as_bstr())
            .context("Source index has local changes")?;
        if indexed.id != entry.oid || indexed.mode.bits() != u32::from(entry.mode.0) {
            bail!("Source index has local changes");
        }
        known.insert(relative);
    }
    if known.len() != index.entries().len() {
        bail!("Source index has local changes");
    }
    fn walk(source: &Path, base: &Path, known: &BTreeSet<std::path::PathBuf>) -> Result<()> {
        for child in fs::read_dir(source.join(base))? {
            let child = child?;
            if base.as_os_str().is_empty() && child.file_name() == ".git" {
                continue;
            }
            let relative = base.join(child.file_name());
            let meta = fs::symlink_metadata(child.path())?;
            if meta.is_dir() && !meta.file_type().is_symlink() {
                if !known.iter().any(|path| path.starts_with(&relative)) {
                    bail!("Source has untracked files or directories");
                }
                walk(source, &relative, known)?;
            } else if !known.contains(&relative) {
                bail!("Source has untracked files");
            }
        }
        Ok(())
    }
    walk(source, Path::new(""), &known)
}

fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(source)?;
    if meta.file_type().is_symlink() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(fs::read_link(source)?, target)?;
        #[cfg(windows)]
        {
            if source.is_dir() {
                std::os::windows::fs::symlink_dir(fs::read_link(source)?, target)?;
            } else {
                std::os::windows::fs::symlink_file(fs::read_link(source)?, target)?;
            }
        }
    } else if meta.is_dir() {
        fs::create_dir_all(target)?;
        for child in fs::read_dir(source)? {
            let child = child?;
            copy_tree(&child.path(), &target.join(child.file_name()))?;
        }
    } else {
        fs::copy(source, target)?;
    }
    Ok(())
}

pub fn update(paths: &Paths) -> Result<()> {
    let config = paths
        .config
        .as_ref()
        .context("Initialize source before update")?;
    clean(&paths.source)?;
    let old = gix::open(&paths.source)?.head_id()?.detach();
    let temp = tempfile::Builder::new()
        .prefix(".xyrev-update-")
        .tempdir_in(paths.source.parent().context("Source has no parent")?)?;
    let stage = temp.path().join("repository");
    clone(&config.repo, &stage, Some(&config.branch))?;
    let repo = gix::open(&stage)?;
    let new = repo.head_id()?.detach();
    let mut ancestor = false;
    for commit in repo.head_commit()?.ancestors().all()? {
        if commit?.id == old {
            ancestor = true;
            break;
        }
    }
    if !ancestor {
        bail!("Source has local commits or has diverged from origin; only fast-forward updates are allowed");
    }
    if old == new {
        println!("Source is already up to date");
        return Ok(());
    }
    drop(repo);
    // Preserve configuration, hooks, other local branches, tags and reflogs. Copying
    // objects is safe: their names are content hashes and existing objects are immutable.
    let metadata = temp.path().join("preserved.git");
    copy_tree(&paths.source.join(".git"), &metadata)?;
    copy_tree(&stage.join(".git/objects"), &metadata.join("objects"))?;
    fs::copy(stage.join(".git/index"), metadata.join("index"))?;
    fs::remove_dir_all(stage.join(".git"))?;
    fs::rename(metadata, stage.join(".git"))?;
    let mut repo = gix::open(&stage)?;
    if repo.committer().is_none() {
        let mut snapshot = repo.config_snapshot_mut();
        snapshot.set_value(&gix::config::tree::User::NAME, "xyrev")?;
        snapshot.set_value(&gix::config::tree::User::EMAIL, "xyrev@localhost")?;
        snapshot.commit()?;
    }
    repo.reference(
        format!("refs/heads/{}", config.branch).as_str(),
        new,
        gix::refs::transaction::PreviousValue::MustExistAndMatch(old.into()),
        "xyrev fast-forward",
    )?;
    repo.reference(
        format!("refs/remotes/origin/{}", config.branch).as_str(),
        new,
        gix::refs::transaction::PreviousValue::Any,
        "xyrev fetch",
    )?;
    drop(repo);
    clean(&paths.source)?;
    if gix::open(&paths.source)?.head_id()?.detach() != old {
        bail!("Source changed while preparing update");
    }
    let previous = temp.path().join("previous");
    fs::rename(&paths.source, &previous)?;
    if let Err(error) = fs::rename(&stage, &paths.source) {
        if let Err(rollback) = fs::rename(&previous, &paths.source) {
            let retained = temp.into_path();
            bail!("Install checkout failed: {error}; restore failed: {rollback}; original checkout retained in {}",retained.display());
        }
        return Err(error).context("Install updated checkout; original source restored");
    }
    println!("Updated source to {}", &new.to_string()[..7]);
    Ok(())
}
