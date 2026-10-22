#![allow(dead_code)]
use std::collections::BTreeMap;
pub use std::fs;
pub use std::path::{Path, PathBuf};
pub use std::process::{Command, Output};
use tempfile::TempDir;

pub struct Sandbox {
    _temporary: TempDir,
    pub root: PathBuf,
    pub home: PathBuf,
    pub work: PathBuf,
    pub remote: PathBuf,
    pub source: PathBuf,
    pub destination: PathBuf,
}

pub fn git(directory: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(args)
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_ALLOW_PROTOCOL", "file")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
    success(output)
}

pub fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

pub fn file_url(path: &Path) -> String {
    let path = path.to_str().unwrap().replace('\\', "/");
    let path = path.strip_prefix("//?/").unwrap_or(&path);
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

pub fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

pub fn failure(output: Output, expected: &str) {
    assert!(
        !output.status.success(),
        "Unexpected success: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "Expected {expected:?} in:\n{stderr}"
    );
}

impl Sandbox {
    pub fn new(files: &[(&str, &str)]) -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temporary.path()).unwrap();
        let home = root.join("home");
        let work = root.join("upstream work");
        let remote = root.join("remote repo.git");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&work).unwrap();
        git(&work, &["init"]);
        git(
            &work,
            &["symbolic-ref", "HEAD", "refs/heads/dotfiles-trunk"],
        );
        git(&work, &["config", "user.name", "Dotfile Test"]);
        git(&work, &["config", "user.email", "test@example.invalid"]);
        git(&work, &["config", "commit.gpgsign", "false"]);
        for (path, contents) in files {
            write(&work.join(path), contents);
        }
        git(&work, &["add", "--all"]);
        git(&work, &["commit", "--allow-empty", "-m", "initial"]);
        git(
            &root,
            &[
                "clone",
                "--bare",
                work.to_str().unwrap(),
                remote.to_str().unwrap(),
            ],
        );
        git(
            &work,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        Self {
            source: root.join("source checkout"),
            destination: root.join("destination"),
            _temporary: temporary,
            root,
            home,
            work,
            remote,
        }
    }

    pub fn url(&self) -> String {
        file_url(&self.remote)
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_xyrev"));
        command
            .current_dir(&self.root)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_ALLOW_PROTOCOL", "file")
            .arg("dotfile");
        command
    }

    pub fn init(&self, extra: &[&str]) -> Output {
        self.command()
            .arg("--source")
            .arg(&self.source)
            .arg("--destination")
            .arg(&self.destination)
            .arg("init")
            .arg(self.url())
            .args(extra)
            .output()
            .unwrap()
    }

    pub fn cli(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    pub fn push(&self) {
        git(&self.work, &["add", "--all"]);
        git(&self.work, &["commit", "-m", "update"]);
        git(&self.work, &["push", "origin", "HEAD"]);
    }

    pub fn control(&self) -> PathBuf {
        self.home.join(".xyrev/dotfile")
    }
}

/// Contents, permissions, and modification times detect accidental writes in preview/idempotency checks.
pub fn snapshot(root: &Path) -> BTreeMap<PathBuf, String> {
    fn walk(path: &Path, map: &mut BTreeMap<PathBuf, String>) {
        let meta = fs::symlink_metadata(path).unwrap();
        let contents = if meta.file_type().is_symlink() {
            format!("link:{:?}", fs::read_link(path).unwrap())
        } else if meta.is_file() {
            format!("{:?}", fs::read(path).unwrap())
        } else {
            "directory".into()
        };
        map.insert(
            path.to_owned(),
            format!(
                "{contents}:{:?}:{:?}",
                meta.modified().unwrap(),
                meta.permissions()
            ),
        );
        if meta.is_dir() {
            for child in fs::read_dir(path).unwrap() {
                walk(&child.unwrap().path(), map);
            }
        }
    }
    let mut map = BTreeMap::new();
    if root.exists() {
        walk(root, &mut map);
    }
    map
}
