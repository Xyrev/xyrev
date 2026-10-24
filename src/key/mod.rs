mod gpg;
mod ssh;

use crate::cli::key::{BackupArgs, Commands, KeyCommand, RestoreArgs};
use crate::dotfile::{storage, tools};
use age::secrecy::{ExposeSecret, SecretString, SecretVec, Zeroize};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const LIMIT: u64 = 64 * 1024 * 1024;
type Files = BTreeMap<String, SecretVec<u8>>;

#[derive(Default)]
struct Sensitive(Vec<u8>);
impl Sensitive {
    fn finish(mut self) -> SecretVec<u8> {
        SecretVec::new(std::mem::take(&mut self.0))
    }
}
impl Drop for Sensitive {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
impl Write for Sensitive {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > LIMIT as usize {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Key backup exceeds the size limit",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    version: u32,
    created_unix: u64,
    gpg: Option<gpg::Inventory>,
    ssh: Vec<ssh::Key>,
    files: BTreeMap<String, String>,
    warnings: Vec<String>,
}

pub fn run(command: &KeyCommand) -> Result<()> {
    match &command.command {
        Commands::Backup(args) => backup(args),
        Commands::Restore(args) => restore(args),
    }
}

fn home() -> Result<PathBuf> {
    let path = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .context("Cannot determine user home directory")?;
    checked(Path::new(&path))
}
fn expand(path: &Path) -> Result<PathBuf> {
    if path == Path::new("~") {
        return home();
    }
    if let Ok(relative) = path.strip_prefix("~/") {
        return Ok(home()?.join(relative));
    }
    Ok(path.to_owned())
}

/// Inspect the original spelling before canonicalization, including every existing parent.
fn checked(path: &Path) -> Result<PathBuf> {
    let path = expand_absolute(path)?;
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => bail!("Parent traversal is not allowed in key paths"),
            Component::CurDir => continue,
            _ => current.push(component.as_os_str()),
        }
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                bail!("Key path cannot contain a symlink: {}", current.display())
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e).context("Inspect key path"),
        }
    }
    storage::resolve(&path)
}
fn expand_absolute(path: &Path) -> Result<PathBuf> {
    // Do not call home() from here: its path uses this same validation.
    let path = if path == Path::new("~") || path.starts_with("~/") {
        let root = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .context("Cannot determine user home directory")?;
        if path == Path::new("~") {
            PathBuf::from(root)
        } else {
            PathBuf::from(root).join(path.strip_prefix("~/")?)
        }
    } else {
        path.to_owned()
    };
    Ok(if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(path)
    })
}
fn outside_git(path: &Path) -> Result<()> {
    for parent in path.ancestors() {
        if parent.join(".git").exists()
            || fs::symlink_metadata(parent.join(".git")).is_ok()
            || (parent.join("HEAD").is_file()
                && parent.join("objects").is_dir()
                && parent.join("refs").is_dir())
        {
            bail!(
                "Keep key backups and restored keys outside Git workspaces: {}",
                path.display()
            );
        }
    }
    Ok(())
}
fn relative(value: &str) -> Result<PathBuf> {
    if value.is_empty() || value.contains(['\\', ':', '\0']) || value.starts_with('/') {
        bail!("Invalid key archive path");
    }
    for part in value.split('/') {
        let upper = part
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.eq_ignore_ascii_case(".git")
            || part.ends_with(['.', ' '])
            || ["CON", "PRN", "AUX", "NUL"].contains(&upper.as_str())
            || (upper.len() == 4
                && (upper.starts_with("COM") || upper.starts_with("LPT"))
                && matches!(upper.as_bytes()[3], b'1'..=b'9'))
        {
            bail!("Invalid key archive path");
        }
    }
    Ok(PathBuf::from(value))
}
fn read(path: &Path) -> Result<SecretVec<u8>> {
    let path = checked(path)?;
    let metadata = fs::metadata(&path)?;
    if !metadata.is_file() || metadata.len() > LIMIT {
        bail!("Key input must be a bounded regular file");
    }
    let mut bytes = Sensitive::default();
    let mut file = fs::File::open(&path)?.take(LIMIT + 1);
    std::io::copy(&mut file, &mut bytes)?;
    Ok(bytes.finish())
}
fn add(files: &mut Files, name: String, bytes: SecretVec<u8>) -> Result<()> {
    relative(&name)?;
    let folded = name.to_ascii_lowercase();
    if files.keys().any(|old| {
        let old = old.to_ascii_lowercase();
        old == folded
            || old.starts_with(&format!("{folded}/"))
            || folded.starts_with(&format!("{old}/"))
    }) {
        bail!("Duplicate or conflicting key archive path");
    }
    if files
        .values()
        .map(|v| v.expose_secret().len())
        .sum::<usize>()
        .saturating_add(bytes.expose_secret().len())
        > LIMIT as usize
    {
        bail!("Key backup exceeds the size limit");
    }
    files.insert(name, bytes);
    Ok(())
}

/// Feed borrowed secret input without leaving an unzeroized writer-thread clone.
fn capture(mut command: Command, input: Option<&[u8]>, executable: &str) -> Result<SecretVec<u8>> {
    command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("STOOL_AGE_PASSPHRASE");
    let mut child = command
        .spawn()
        .with_context(|| format!("Start {executable}; install it and make it available on PATH"))?;
    let output = std::thread::scope(|scope| -> Result<_> {
        let writer = input.map(|bytes| {
            let mut stdin = child.stdin.take().unwrap();
            scope.spawn(move || stdin.write_all(bytes))
        });
        let output = child.wait_with_output()?;
        if let Some(writer) = writer {
            let result = writer
                .join()
                .map_err(|_| anyhow::anyhow!("Key input writer failed"))?;
            if output.status.success() {
                result?;
            }
        }
        Ok(output)
    })?;
    let stdout = SecretVec::new(output.stdout);
    let _stderr = SecretVec::new(output.stderr);
    if !output.status.success() {
        bail!("{executable} operation failed ({}); check the selected keys, agent and original key passphrases",output.status);
    }
    if stdout.expose_secret().len() > LIMIT as usize {
        bail!("Key command output exceeds the size limit");
    }
    Ok(stdout)
}

fn password(path: Option<&Path>, confirm: bool) -> Result<SecretString> {
    let password = if let Some(path) = path {
        let path = expand(path)?;
        let pipe = confirm && (path.starts_with("/dev/fd") || path.starts_with("/proc/self/fd"));
        let bytes = if pipe {
            SecretVec::new(fs::read(&path).context("Read password pipe")?)
        } else {
            let path = checked(&path)?;
            outside_git(&path)?;
            let metadata = fs::metadata(&path)?;
            if !metadata.is_file() || storage::path_mode(&path, &metadata)? != 0o600 {
                bail!("Password file must be private (0600)");
            }
            read(&path)?
        };
        let mut text =
            std::str::from_utf8(bytes.expose_secret()).context("Backup password must be UTF-8")?;
        if let Some(stripped) = text.strip_suffix('\n') {
            text = stripped.strip_suffix('\r').unwrap_or(stripped);
        }
        SecretString::new(text.to_owned())
    } else {
        let password = tools::prompt_passphrase("Backup password: ")
            .context("Read backup password privately from a terminal, or use --passphrase-file")?;
        if confirm
            && tools::prompt_passphrase("Confirm backup password: ")?.expose_secret()
                != password.expose_secret()
        {
            bail!("Backup passwords did not match");
        }
        password
    };
    if password.expose_secret().is_empty() || password.expose_secret().contains(['\r', '\n', '\0'])
    {
        bail!("Backup password must be nonempty and contain no line breaks or NUL bytes");
    }
    Ok(password)
}

fn pack(manifest: &Manifest, files: &Files) -> Result<SecretVec<u8>> {
    let mut builder = tar::Builder::new(Sensitive::default());
    let metadata = SecretVec::new(serde_json::to_vec(manifest)?);
    for (name, bytes) in std::iter::once(("manifest.json", metadata.expose_secret().as_slice()))
        .chain(
            files
                .iter()
                .map(|(name, bytes)| (name.as_str(), bytes.expose_secret().as_slice())),
        )
    {
        let mut header = tar::Header::new_ustar();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o600);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        builder.append_data(&mut header, name, bytes)?;
    }
    Ok(builder.into_inner()?.finish())
}
fn unpack(payload: &[u8]) -> Result<(Manifest, Files)> {
    let mut files = Files::new();
    let mut manifest = None;
    let mut names = BTreeSet::new();
    let mut archive = tar::Archive::new(payload);
    for entry in archive.entries()? {
        let entry = entry?;
        if !entry.header().entry_type().is_file() || entry.size() > LIMIT || files.len() > 4096 {
            bail!("Key archive contains an unsupported entry");
        }
        let name = std::str::from_utf8(&entry.path_bytes())?.to_owned();
        relative(&name)?;
        if !names.insert(name.to_ascii_lowercase()) {
            bail!("Key archive contains duplicate paths");
        }
        let mut bytes = Sensitive::default();
        std::io::copy(&mut entry.take(LIMIT + 1), &mut bytes)?;
        let bytes = bytes.finish();
        if name == "manifest.json" {
            manifest = Some(serde_json::from_slice::<Manifest>(bytes.expose_secret())?);
        } else {
            add(&mut files, name, bytes)?;
        }
    }
    let manifest = manifest.context("Key archive manifest is missing")?;
    if manifest.format != "stool-key-backup" || manifest.version != 1 {
        bail!("Unsupported key backup format or version");
    }
    if files.len() != manifest.files.len()
        || files.iter().any(|(name, bytes)| {
            manifest.files.get(name) != Some(&storage::digest(bytes.expose_secret()))
        })
    {
        bail!("Key backup checksum or inventory mismatch");
    }
    let mut expected = BTreeSet::new();
    if let Some(gpg) = &manifest.gpg {
        expected.extend(gpg::paths(gpg)?);
    }
    let mut ssh_names = BTreeSet::new();
    for key in &manifest.ssh {
        if !ssh_names.insert(key.path.to_ascii_lowercase()) {
            bail!("Duplicate SSH identity in manifest");
        }
        relative(&key.path)?;
        expected.insert(format!("ssh/{}", key.path));
        expected.insert(format!("ssh/{}.pub", key.path));
        ssh::validate(key, &files)?;
    }
    if expected.len() != files.len() || !expected.iter().eq(files.keys()) {
        bail!("Key backup contains missing or unexpected files");
    }
    if manifest.gpg.is_none() && manifest.ssh.is_empty() {
        bail!("Key backup is empty");
    }
    Ok((manifest, files))
}
fn encrypt(password: &SecretString, bytes: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut writer =
        age::Encryptor::with_user_passphrase(password.clone()).wrap_output(&mut output)?;
    writer.write_all(bytes)?;
    writer.finish()?;
    Ok(output)
}
fn decrypt(password: &SecretString, bytes: &[u8]) -> Result<SecretVec<u8>> {
    let decryptor = age::Decryptor::new(bytes).context("Invalid encrypted key backup")?;
    let age::Decryptor::Passphrase(decryptor) = decryptor else {
        bail!("Only password-encrypted stool key backups are supported");
    };
    let mut reader = decryptor
        .decrypt(password, Some(20))
        .context("Unable to decrypt key backup; check the password")?
        .take(LIMIT + 1);
    let mut output = Sensitive::default();
    std::io::copy(&mut reader, &mut output).context("Invalid or damaged encrypted key backup")?;
    Ok(output.finish())
}

fn backup(args: &BackupArgs) -> Result<()> {
    let source = if let Some(path) = &args.input {
        checked(path)?
    } else {
        home()?
    };
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let output = checked(
        args.output
            .as_deref()
            .unwrap_or(&PathBuf::from(format!("stool-keys-{now}.tar.age"))),
    )?;
    outside_git(&output)?;
    if fs::symlink_metadata(&output).is_ok() {
        bail!("Backup output already exists; choose a new file");
    }
    let mut files = Files::new();
    let mut warnings = Vec::new();
    let gpg_home = if args.input.is_none() {
        std::env::var_os("GNUPGHOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| source.join(".gnupg"))
    } else {
        source.join(".gnupg")
    };
    let gpg = if args.selection.gpg() {
        gpg::export(&checked(&gpg_home)?, &mut files, &mut warnings)?
    } else {
        None
    };
    let ssh = if args.selection.ssh() {
        ssh::export(&source, args.input.is_none(), &mut files, &mut warnings)?
    } else {
        Vec::new()
    };
    if (args.selection.gpg && gpg.is_none())
        || (args.selection.ssh && ssh.is_empty())
        || (gpg.is_none() && ssh.is_empty())
    {
        bail!("No exportable keys found for the selected categories; no backup was written");
    }
    if args.selection.gpg() && gpg.is_none() {
        warnings.push("No exportable local GPG keys found".into());
    }
    if args.selection.ssh() && ssh.is_empty() {
        warnings.push("No local SSH private keys found".into());
    }
    let manifest = Manifest {
        format: "stool-key-backup".into(),
        version: 1,
        created_unix: now,
        gpg,
        ssh,
        files: files
            .iter()
            .map(|(name, bytes)| (name.clone(), storage::digest(bytes.expose_secret())))
            .collect(),
        warnings,
    };
    let payload = pack(&manifest, &files)?;
    let password = password(args.passphrase_file.as_deref(), true)?;
    let ciphertext = encrypt(&password, payload.expose_secret())?;
    // Age framing adds overhead beyond the plaintext tar. Never publish an
    // archive that the bounded input reader would reject during restoration.
    if ciphertext.len() as u64 > LIMIT {
        bail!("Encrypted key backup exceeds the 64 MiB size limit; no backup was written");
    }
    let recovered = decrypt(&password, &ciphertext)?;
    unpack(recovered.expose_secret())?;
    let parent = output.parent().context("Backup file has no parent")?;
    checked(parent)?;
    outside_git(parent)?;
    if !parent.exists() {
        storage::private_dir(parent)?;
    }
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    storage::set_mode(staged.path(), 0o600)?;
    staged.write_all(&ciphertext)?;
    staged.as_file().sync_all()?;
    staged
        .persist_noclobber(&output)
        .map_err(|e| e.error)
        .context("Save new key backup without overwriting an existing file")?;
    report_warnings(&manifest);
    println!(
        "Encrypted key backup created successfully: {}",
        output.display()
    );
    println!(
        "GPG primary keys: {}; SSH private keys: {}",
        manifest.gpg.as_ref().map_or(0, |v| v.groups.len()),
        manifest.ssh.len()
    );
    Ok(())
}
fn report_warnings(manifest: &Manifest) {
    for warning in &manifest.warnings {
        eprintln!("Warning: {warning}");
    }
}

fn target_files(
    manifest: &Manifest,
    files: &Files,
    gpg: bool,
    ssh: bool,
) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    if gpg {
        for name in files
            .keys()
            .filter(|name| name.starts_with("gpg/revocations/"))
        {
            result.insert(
                name.clone(),
                format!(
                    ".gnupg/openpgp-revocs.d/{}",
                    name.trim_start_matches("gpg/revocations/")
                ),
            );
        }
    }
    if ssh {
        for key in &manifest.ssh {
            for suffix in ["", ".pub"] {
                result.insert(
                    format!("ssh/{}{suffix}", key.path),
                    format!(".ssh/{}{suffix}", key.path),
                );
            }
        }
    }
    result
}
fn preflight(target: &Path, names: &BTreeMap<String, String>, files: &Files) -> Result<()> {
    checked(target)?;
    outside_git(target)?;
    for (source, name) in names {
        let path = checked(&target.join(relative(name)?))?;
        outside_git(&path)?;
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if !metadata.is_file() || read(&path)?.expose_secret() != files[source].expose_secret()
            {
                bail!(
                    "Existing key conflicts with backup: {}; no files were overwritten",
                    path.display()
                );
            }
        }
    }
    Ok(())
}
fn restore(args: &RestoreArgs) -> Result<()> {
    let input = checked(&args.input)?;
    let target = if let Some(path) = &args.output {
        checked(path)?
    } else {
        home()?
    };
    outside_git(&target)?;
    let encrypted = read(&input)?;
    let password = password(args.passphrase_file.as_deref(), false)?;
    let payload = decrypt(&password, encrypted.expose_secret())?;
    let (manifest, files) = unpack(payload.expose_secret())?;
    let do_gpg = args.selection.gpg() && manifest.gpg.is_some();
    let do_ssh = args.selection.ssh() && !manifest.ssh.is_empty();
    if !do_gpg && !do_ssh {
        bail!("Backup contains no keys of the selected category");
    }
    let names = target_files(&manifest, &files, do_gpg, do_ssh);
    preflight(&target, &names, &files)?;
    if do_gpg {
        gpg::preflight(&target)?;
        gpg::validate(manifest.gpg.as_ref().unwrap(), &files)?;
    }
    if do_gpg {
        gpg::restore(&target,manifest.gpg.as_ref().unwrap(),&files)
            .context("GPG restore failed and may have imported some keys; retry after fixing the cause; SSH keys have not been written")?;
        println!(
            "Restored GPG keys and ownertrust ({} primary keys).",
            manifest.gpg.as_ref().unwrap().groups.len()
        );
    }
    for (source, name) in &names {
        // Check again immediately before each write; identical files are retained.
        let path = checked(&target.join(relative(name)?))?;
        outside_git(&path)?;
        let category = if name.starts_with(".ssh/") {
            "SSH"
        } else {
            "GPG revocation"
        };
        let mut parents = Vec::new();
        let mut parent = path.parent().unwrap();
        while parent.starts_with(&target) && parent != target {
            parents.push(parent.to_owned());
            parent = parent.parent().unwrap();
        }
        if !target.exists() {
            storage::private_dir(&target)?;
        }
        for directory in parents.into_iter().rev() {
            checked(&directory)?;
            storage::private_dir(&directory)?;
        }
        if fs::symlink_metadata(&path).is_ok() {
            if read(&path)?.expose_secret() != files[source].expose_secret() {
                bail!(
                    "{category} key changed during restore: {}; retry after resolving the conflict",
                    path.display()
                );
            }
            storage::set_mode(&path, 0o600)?;
            continue;
        }
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut output = options
            .open(&path)
            .with_context(|| format!("Create restored {category} key without replacement"))?;
        let result = (|| -> Result<()> {
            storage::set_mode(&path, 0o600)?;
            output.write_all(files[source].expose_secret())?;
            output.sync_all()?;
            Ok(())
        })();
        if let Err(error) = result {
            drop(output);
            let _ = fs::remove_file(&path);
            return Err(error).with_context(||format!("Save restored {category} key; completed imports/files are retained and can be retried"));
        }
    }
    if do_ssh {
        println!("Restored SSH key pairs: {}", manifest.ssh.len());
    }
    report_warnings(&manifest);
    println!("Key restore completed: {}", target.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn raw_tar(entries: &[(&str, &[u8], tar::EntryType)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (name, bytes, kind) in entries {
            let mut header = tar::Header::new_ustar();
            header.as_mut_bytes()[..100].fill(0);
            header.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
            header.set_size(bytes.len() as u64);
            header.set_entry_type(*kind);
            header.set_mode(0o600);
            header.set_cksum();
            builder.append(&header, *bytes).unwrap();
        }
        builder.into_inner().unwrap()
    }
    #[test]
    fn file_directory_collisions_are_rejected_case_insensitively() {
        for (first, second) in [("ssh/key", "ssh/key/child"), ("ssh/KEY/child", "ssh/key")] {
            let mut files = Files::new();
            add(&mut files, first.into(), SecretVec::new(vec![1])).unwrap();
            assert!(add(&mut files, second.into(), SecretVec::new(vec![2])).is_err());
        }
    }
    #[test]
    fn untrusted_archive_paths_duplicates_and_special_files_are_rejected() {
        for name in [
            "../secret",
            "/absolute",
            "ssh/../../escape",
            "ssh/\\windows",
            "ssh/C:drive",
            "ssh/NUL",
            "ssh/.git/key",
            "ssh/a.",
        ] {
            let bytes = raw_tar(&[(name, b"test", tar::EntryType::Regular)]);
            assert!(unpack(&bytes).is_err(), "{name}");
        }
        let bytes = raw_tar(&[
            ("ssh/key", b"x", tar::EntryType::Regular),
            ("SSH/KEY", b"y", tar::EntryType::Regular),
        ]);
        assert!(unpack(&bytes)
            .err()
            .unwrap()
            .to_string()
            .contains("duplicate"));
        for kind in [
            tar::EntryType::Symlink,
            tar::EntryType::Link,
            tar::EntryType::Fifo,
            tar::EntryType::Char,
            tar::EntryType::Block,
        ] {
            assert!(unpack(&raw_tar(&[("ssh/key", b"", kind)])).is_err());
        }
    }
    #[test]
    fn unsupported_versions_and_unlisted_payloads_are_rejected() {
        let bytes=raw_tar(&[("manifest.json",br#"{"format":"stool-key-backup","version":999,"created_unix":0,"gpg":null,"ssh":[],"files":{},"warnings":[]}"#,tar::EntryType::Regular)]);
        assert!(unpack(&bytes)
            .err()
            .unwrap()
            .to_string()
            .contains("version"));
        let bytes=raw_tar(&[("manifest.json",br#"{"format":"stool-key-backup","version":1,"created_unix":0,"gpg":null,"ssh":[],"files":{},"warnings":[]}"#,tar::EntryType::Regular),("ssh/key",b"unlisted private data",tar::EntryType::Regular)]);
        assert!(unpack(&bytes)
            .err()
            .unwrap()
            .to_string()
            .contains("checksum"));
    }
}
