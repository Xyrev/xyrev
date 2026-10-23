use super::storage::{self, EncryptionSettings, Paths};
use super::template::{self, Engine};
use super::{crypto, tools};
use crate::cli::dotfile::{GpgArgs, GpgBackupArgs, GpgCommands, GpgRestoreArgs};
use age::secrecy::{ExposeSecret, SecretString, SecretVec};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const FILES: [&str; 4] = [
    "public.asc",
    "secret.asc.age",
    "ownertrust.txt.age",
    "backup-metadata.sh",
];

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    enabled: bool,
    directory: PathBuf,
    program: Option<PathBuf>,
    identity: Option<PathBuf>,
    passphrase_file: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: ".stooltemplates/gpg".into(),
            program: None,
            identity: None,
            passphrase_file: None,
        }
    }
}

#[derive(Clone)]
struct Metadata {
    mode: String,
    fingerprints: BTreeSet<String>,
    recipient: Option<String>,
    hashes: [String; 3],
}

struct Bundle {
    metadata: Metadata,
    public: Vec<u8>,
    secret: Vec<u8>,
    ownertrust: Vec<u8>,
    manifest: Vec<u8>,
}

/// An immutable encrypted snapshot prepared without running GPG or asking for secrets.
pub struct RestorePlan {
    pub digest: String,
    settings: Settings,
    directory: PathBuf,
    bundle: Bundle,
}

fn data_path(engine: &Engine, key: &str) -> Option<PathBuf> {
    engine
        .data
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

fn settings(paths: &Paths, engine: &Engine) -> Result<(Settings, bool)> {
    let root = template::source_root(paths)?;
    let mut found = None;
    for name in [".stoolgpg.toml", ".stoolgpg.toml.tmpl"] {
        let path = root.join(name);
        if fs::symlink_metadata(&path).is_ok() {
            if found.is_some() {
                bail!("Use only one .stoolgpg.toml or .stoolgpg.toml.tmpl declaration");
            }
            template::regular(&path)?;
            let bytes = fs::read(path)?;
            let bytes = if name.ends_with(".tmpl") {
                engine.render(name, &bytes)?
            } else {
                bytes
            };
            found = Some(
                toml::from_str::<Settings>(std::str::from_utf8(&bytes)?)
                    .context("Invalid GPG declaration")?,
            );
        }
    }
    let declared = found.is_some();
    let mut value = found.unwrap_or_default();
    if value.program.as_ref().is_none_or_empty() {
        value.program = data_path(engine, "gpg_program");
    }
    if value.identity.as_ref().is_none_or_empty() {
        value.identity = data_path(engine, "gpg_identity");
    }
    if value.passphrase_file.as_ref().is_none_or_empty() {
        value.passphrase_file = data_path(engine, "gpg_passphrase_file");
    }
    Ok((value, declared))
}

trait EmptyPath {
    fn is_none_or_empty(&self) -> bool;
}
impl EmptyPath for Option<&PathBuf> {
    fn is_none_or_empty(&self) -> bool {
        self.map_or(true, |p| p.as_os_str().is_empty())
    }
}

fn source_directory(paths: &Paths, settings: &Settings) -> Result<PathBuf> {
    let root = template::source_root(paths)?;
    let relative = storage::relative_path(&settings.directory)?;
    let path = root.join(relative);
    check_descendants(&root, &path)?;
    Ok(path)
}

fn check_descendants(root: &Path, path: &Path) -> Result<()> {
    let mut current = root.to_owned();
    for part in path
        .strip_prefix(root)
        .context("GPG backup path escapes source")?
        .components()
    {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                bail!("GPG backup path cannot be a symlink: {}", current.display())
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e).context("Inspect GPG backup path"),
        }
    }
    Ok(())
}

fn explicit_directory(path: &Path, home: &Path) -> Result<PathBuf> {
    let path = storage::expand_home(path, home);
    if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
        bail!("GPG backup directory cannot be a symlink");
    }
    storage::resolve(&path)
}

fn read_regular(path: &Path) -> Result<Vec<u8>> {
    template::regular(path)
        .with_context(|| format!("GPG backup file must be regular: {}", path.display()))?;
    fs::read(path).with_context(|| format!("Read GPG backup {}", path.display()))
}

fn valid_fingerprint(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b))
}

fn fingerprint_list(value: &str) -> Result<BTreeSet<String>> {
    let mut keys = BTreeSet::new();
    for key in value.split(' ') {
        if !valid_fingerprint(key) || !keys.insert(key.to_owned()) {
            bail!("GPG backup fingerprint list is invalid or contains a duplicate");
        }
    }
    if keys.is_empty() {
        bail!("GPG backup fingerprint list is empty");
    }
    Ok(keys)
}

fn metadata(bytes: &[u8]) -> Result<Metadata> {
    let text = std::str::from_utf8(bytes).context("GPG backup metadata must be UTF-8")?;
    let mut values = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, literal) = line
            .split_once('=')
            .context("Invalid GPG backup metadata assignment")?;
        if ![
            "gpg_backup_mode",
            "gpg_backup_fingerprint",
            "gpg_backup_fingerprints",
            "gpg_backup_recipient",
            "gpg_backup_public_sha256",
            "gpg_backup_secret_sha256",
            "gpg_backup_ownertrust_sha256",
        ]
        .contains(&name)
        {
            bail!("Unknown GPG backup metadata field");
        }
        let literal = literal
            .strip_prefix('\'')
            .and_then(|v| v.strip_suffix('\''))
            .context("GPG backup metadata must use literal single-quoted values")?;
        if literal.contains(['\'', '\\', '\r', '\n']) || values.insert(name, literal).is_some() {
            bail!("Invalid or duplicate GPG backup metadata field");
        }
    }
    let mode = values.get("gpg_backup_mode").copied().unwrap_or("identity");
    if !matches!(mode, "identity" | "passphrase") {
        bail!("Unsupported GPG backup mode");
    }
    let singular = values.get("gpg_backup_fingerprint").copied();
    let list = values
        .get("gpg_backup_fingerprints")
        .copied()
        .or(singular)
        .context("GPG backup fingerprint metadata is missing")?;
    let keys = fingerprint_list(list)?;
    if let Some(singular) = singular {
        if !valid_fingerprint(singular) || list.split(' ').next() != Some(singular) {
            bail!("GPG backup primary fingerprint metadata is inconsistent");
        }
    }
    let mut hashes = [String::new(), String::new(), String::new()];
    for (index, name) in [
        "gpg_backup_public_sha256",
        "gpg_backup_secret_sha256",
        "gpg_backup_ownertrust_sha256",
    ]
    .iter()
    .enumerate()
    {
        let value = values
            .get(name)
            .context("GPG backup checksum metadata is missing")?;
        if value.len() != 64
            || !value
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            bail!("GPG backup checksum metadata is invalid");
        }
        hashes[index] = (*value).to_owned();
    }
    let recipient = values.get("gpg_backup_recipient").map(|s| (*s).to_owned());
    if mode == "identity" {
        let recipient = recipient
            .as_ref()
            .context("GPG backup age recipient is missing")?;
        recipient
            .parse::<age::x25519::Recipient>()
            .map_err(|_| anyhow::anyhow!("Invalid GPG backup age recipient"))?;
    }
    Ok(Metadata {
        mode: mode.into(),
        fingerprints: keys,
        recipient,
        hashes,
    })
}

fn read_bundle(directory: &Path) -> Result<Bundle> {
    template::regular(&directory.join(FILES[3]))?;
    let manifest = read_regular(&directory.join(FILES[3]))?;
    let metadata = metadata(&manifest)?;
    let mut contents = Vec::new();
    for (name, hash) in FILES[..3].iter().zip(&metadata.hashes) {
        let bytes = read_regular(&directory.join(name))?;
        if storage::digest(&bytes) != *hash {
            bail!("GPG backup checksum failed: {name}");
        }
        contents.push(bytes);
    }
    Ok(Bundle {
        metadata,
        public: contents.remove(0),
        secret: contents.remove(0),
        ownertrust: contents.remove(0),
        manifest,
    })
}

pub fn prepare_restore(paths: &Paths, engine: &Engine) -> Result<Option<RestorePlan>> {
    let (settings, declared) = settings(paths, engine)?;
    if !declared || !settings.enabled || paths.destination != paths.home {
        return Ok(None);
    }
    let directory = source_directory(paths, &settings)?;
    if FILES.iter().all(|name| {
        fs::symlink_metadata(directory.join(name))
            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
    }) {
        return Ok(None);
    }
    restore_plan(paths, settings, directory).map(Some)
}

fn restore_plan(paths: &Paths, settings: Settings, directory: PathBuf) -> Result<RestorePlan> {
    if paths.destination != paths.home {
        bail!("GPG restore requires the home destination");
    }
    if fs::symlink_metadata(&directory)?.file_type().is_symlink() {
        bail!("GPG backup directory cannot be a symlink");
    }
    let bundle = read_bundle(&directory)?;
    let mut digest = serde_json::to_vec(&settings)?;
    digest.extend_from_slice(directory.to_string_lossy().as_bytes());
    digest.extend_from_slice(&bundle.manifest);
    // Actual contents are checksum-validated above; no secret data is persisted in state.
    Ok(RestorePlan {
        digest: storage::digest(&digest),
        settings,
        directory,
        bundle,
    })
}

pub fn run(paths: &Paths, engine: &Engine, args: &GpgArgs) -> Result<()> {
    let (mut settings, _) = settings(paths, engine)?;
    match &args.command {
        GpgCommands::Backup(args) => backup(paths, settings, args),
        GpgCommands::Restore(args) => {
            override_restore(&mut settings, args);
            let directory = if let Some(path) = &args.input {
                explicit_directory(path, &paths.home)?
            } else {
                source_directory(paths, &settings)?
            };
            let plan = restore_plan(paths, settings, directory)?;
            if args.identity.is_some() && plan.bundle.metadata.mode != "identity" {
                bail!("GPG backup uses password encryption; use --passphrase-file or enter its password");
            }
            if args.passphrase_file.is_some() && plan.bundle.metadata.mode != "passphrase" {
                bail!("GPG backup requires its separate age identity; use --identity");
            }
            plan.restore(paths)?;
            let mut state = paths.state()?;
            state.gpg_restore = Some(plan.digest);
            paths.save_state(&state)
        }
    }
}

fn override_restore(settings: &mut Settings, args: &GpgRestoreArgs) {
    if let Some(path) = &args.gpg_program {
        settings.program = Some(path.clone());
    }
    if let Some(path) = &args.identity {
        settings.identity = Some(path.clone());
        settings.passphrase_file = None;
    }
    if let Some(path) = &args.passphrase_file {
        settings.passphrase_file = Some(path.clone());
        settings.identity = None;
    }
}

fn program(settings: &Settings, home: &Path) -> PathBuf {
    if let Some(program) = &settings.program {
        return storage::expand_home(program, home);
    }
    #[cfg(target_os = "macos")]
    if Path::new("/usr/local/MacGPG2/bin/gpg").is_file() {
        return "/usr/local/MacGPG2/bin/gpg".into();
    }
    "gpg".into()
}

fn command(settings: &Settings, home: &Path, restore: bool) -> Command {
    let mut command = Command::new(program(settings, home));
    command
        .args(["--no-options", "--batch"])
        .env_remove("STOOL_AGE_PASSPHRASE");
    if restore {
        command.arg("--homedir").arg(home.join(".gnupg"));
        command.env("GNUPGHOME", home.join(".gnupg"));
    }
    command
}

/// Borrow secret input into the writer thread; never leave an unzeroized clone.
fn capture(mut command: Command, input: Option<&[u8]>) -> Result<SecretVec<u8>> {
    command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .context("Start GPG; install GPG Suite or configure gpg_program")?;
    let output = std::thread::scope(|scope| -> Result<_> {
        let writer = input.map(|bytes| {
            let mut stdin = child.stdin.take().unwrap();
            scope.spawn(move || stdin.write_all(bytes))
        });
        let output = child.wait_with_output().context("Wait for GPG")?;
        if let Some(writer) = writer {
            let result = writer
                .join()
                .map_err(|_| anyhow::anyhow!("GPG input writer failed"))?;
            if output.status.success() {
                result.context("Write GPG input")?;
            }
        }
        Ok(output)
    })?;
    let stdout = SecretVec::new(output.stdout);
    let _stderr = SecretVec::new(output.stderr);
    if !output.status.success() {
        bail!(
            "GPG operation failed ({}); check the selected keys, GPG agent and pinentry",
            output.status
        );
    }
    Ok(stdout)
}

enum Protection {
    Passphrase(SecretString),
    Identity(age::x25519::Identity),
}
impl Protection {
    fn encrypt(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        match self {
            Self::Passphrase(password) => crypto::encrypt_with_passphrase(password.clone(), bytes),
            Self::Identity(identity) => crypto::encrypt(
                &EncryptionSettings {
                    recipients: vec![identity.to_public().to_string()],
                    ..Default::default()
                },
                bytes,
            ),
        }
    }
    fn decrypt(&self, bytes: &[u8]) -> Result<SecretVec<u8>> {
        match self {
            Self::Passphrase(password) => {
                crypto::decrypt_with_passphrase(password, bytes).map(SecretVec::new)
            }
            Self::Identity(identity) => {
                let decryptor = age::Decryptor::new(age::armor::ArmoredReader::new(bytes))
                    .context("Invalid age ciphertext")?;
                let age::Decryptor::Recipients(decryptor) = decryptor else {
                    bail!("GPG backup requires a password, not an age identity");
                };
                let mut output = Vec::new();
                decryptor
                    .decrypt(std::iter::once(identity as &dyn age::Identity))?
                    .read_to_end(&mut output)?;
                Ok(SecretVec::new(output))
            }
        }
    }
}

fn secret_path(path: &Path, paths: &Paths, directory: &Path, allow_pipe: bool) -> Result<PathBuf> {
    let path = storage::expand_home(path, &paths.home);
    if allow_pipe && is_pipe_path(&path) {
        return Ok(path);
    }
    let meta =
        fs::symlink_metadata(&path).context("Read GPG backup password or age identity file")?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        bail!("GPG backup password or age identity must be a regular file, not a symlink");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o777 != 0o600 {
            bail!("GPG backup password or age identity file must have permission 0600");
        }
    }
    #[cfg(windows)]
    if storage::path_mode(&path, &meta)? != 0o600 {
        bail!("GPG backup password or age identity file must have private access");
    }
    let path = storage::resolve(&path)?;
    if path.starts_with(&paths.source) || path.starts_with(directory) {
        bail!("Keep GPG backup passwords and age identities outside the repository and backup directory");
    }
    Ok(path)
}

fn is_pipe_path(path: &Path) -> bool {
    let text = path.to_string_lossy();
    ["/dev/fd/", "/proc/self/fd/"].iter().any(|prefix| {
        text.strip_prefix(prefix)
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    })
}

fn read_password(
    path: Option<&Path>,
    paths: &Paths,
    directory: &Path,
    confirm: bool,
) -> Result<SecretString> {
    let password = if let Some(path) = path {
        let path = secret_path(path, paths, directory, confirm)?;
        let bytes = SecretVec::new(fs::read(&path).context("Read GPG backup password file")?);
        let mut text = std::str::from_utf8(bytes.expose_secret())
            .context("GPG backup password must be UTF-8")?;
        if let Some(stripped) = text.strip_suffix('\n') {
            text = stripped.strip_suffix('\r').unwrap_or(stripped);
        }
        SecretString::new(text.to_owned())
    } else {
        let password = tools::prompt_passphrase("GPG backup password: ").context(
            "Read GPG backup password privately from a terminal, or use --passphrase-file",
        )?;
        if confirm {
            let confirmation = tools::prompt_passphrase("Confirm backup password: ")?;
            if confirmation.expose_secret() != password.expose_secret() {
                bail!("GPG backup passwords did not match");
            }
        }
        password
    };
    if password.expose_secret().is_empty() || password.expose_secret().contains(['\r', '\n', '\0'])
    {
        bail!("GPG backup password must be nonempty and contain no line breaks or NUL bytes");
    }
    Ok(password)
}

fn read_identity(
    path: &Path,
    paths: &Paths,
    directory: &Path,
    create: bool,
) -> Result<age::x25519::Identity> {
    let path = storage::expand_home(path, &paths.home);
    if create
        && fs::symlink_metadata(&path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
    {
        let resolved = storage::resolve(&path)?;
        if resolved.starts_with(&paths.source) || resolved.starts_with(directory) {
            bail!("Keep the age identity outside the repository and backup directory");
        }
        let parent = resolved.parent().context("Age identity has no parent")?;
        if !parent.exists() {
            storage::private_dir(parent)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(parent)?.permissions().mode() & 0o777 != 0o700 {
                bail!("Age identity directory must have permission 0700");
            }
        }
        crypto::keygen(&resolved)?;
    }
    let path = secret_path(&path, paths, directory, false)?;
    let bytes = SecretVec::new(fs::read(path)?);
    let text = std::str::from_utf8(bytes.expose_secret()).context("Age identity must be UTF-8")?;
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'));
    let key = lines
        .next()
        .context("Age identity is empty")?
        .parse::<age::x25519::Identity>()
        .map_err(|_| anyhow::anyhow!("Invalid age identity"))?;
    if lines.next().is_some() {
        bail!("Use an age identity file containing one X25519 identity");
    }
    Ok(key)
}

fn key_set(bytes: &[u8], kind: &str, expected: &BTreeSet<String>) -> Result<()> {
    let text = std::str::from_utf8(bytes).context("Invalid GPG key listing")?;
    let mut seen = BTreeSet::new();
    let mut pending = false;
    for line in text.lines() {
        let fields: Vec<_> = line.split(':').collect();
        match fields[0] {
            "sec" | "pub" => {
                if pending || fields[0] != kind {
                    bail!("GPG backup must contain exactly the selected {kind} primary keys");
                }
                pending = true;
            }
            "ssb" | "sub" => {
                if pending {
                    bail!("GPG primary fingerprint is missing");
                }
            }
            "fpr" if pending => {
                let key = fields
                    .get(9)
                    .context("GPG primary fingerprint is missing")?;
                if !expected.contains(*key) || !seen.insert((*key).to_owned()) {
                    bail!("GPG backup fingerprint set mismatch");
                }
                pending = false;
            }
            _ => (),
        }
    }
    if pending || &seen != expected {
        bail!("GPG backup fingerprint set mismatch; missing selected private or public keys");
    }
    Ok(())
}

fn ownertrust(bytes: &[u8], expected: &BTreeSet<String>, filter: bool) -> Result<SecretVec<u8>> {
    let text = std::str::from_utf8(bytes).context("Invalid GPG ownertrust data")?;
    let mut output = String::new();
    let mut seen = BTreeSet::new();
    for line in text.lines() {
        if filter && (line.starts_with('#') || line.is_empty()) {
            continue;
        }
        let fields: Vec<_> = line.split(':').collect();
        if filter && fields.first().is_some_and(|key| !expected.contains(*key)) {
            continue;
        }
        if line.is_empty() {
            continue;
        }
        if fields.len() != 3
            || !expected.contains(fields[0])
            || fields[1].len() != 1
            || !matches!(fields[1].as_bytes()[0], b'1'..=b'6')
            || !fields[2].is_empty()
            || !seen.insert(fields[0])
        {
            bail!("GPG backup ownertrust contains an invalid, duplicate or unselected key");
        }
        output.push_str(line);
        output.push('\n');
    }
    Ok(SecretVec::new(output.into_bytes()))
}

fn validate_bundle(
    settings: &Settings,
    paths: &Paths,
    bundle: &Bundle,
    secret: &[u8],
    trust: &[u8],
    restore: bool,
) -> Result<()> {
    for (kind, bytes) in [("pub", bundle.public.as_slice()), ("sec", secret)] {
        let mut command = command(settings, &paths.home, restore);
        command.args(["--with-colons", "--import-options", "show-only", "--import"]);
        key_set(
            capture(command, Some(bytes))?.expose_secret(),
            kind,
            &bundle.metadata.fingerprints,
        )?;
    }
    ownertrust(trust, &bundle.metadata.fingerprints, false)?;
    Ok(())
}

impl RestorePlan {
    /// Public, checksum-validated inputs only. Migration never decrypts a bundle.
    pub(super) fn migration_material(&self) -> Result<(Vec<u8>, PathBuf, Vec<u8>)> {
        Ok((
            serde_json::to_vec(&self.settings)?,
            self.directory.clone(),
            self.bundle.manifest.clone(),
        ))
    }

    pub fn restore(&self, paths: &Paths) -> Result<()> {
        if paths.destination != paths.home {
            bail!("GPG restore requires the home destination");
        }
        let protection = if self.bundle.metadata.mode == "passphrase" {
            Protection::Passphrase(read_password(
                self.settings.passphrase_file.as_deref(),
                paths,
                &self.directory,
                false,
            )?)
        } else {
            let path=self.settings.identity.as_ref().context("Restore the separate age identity and configure gpg_identity, or supply --identity")?;
            Protection::Identity(read_identity(path, paths, &self.directory, false)?)
        };
        let secret = protection.decrypt(&self.bundle.secret)?;
        let trust = protection.decrypt(&self.bundle.ownertrust)?;
        storage::private_dir(&paths.home.join(".gnupg"))?;
        validate_bundle(
            &self.settings,
            paths,
            &self.bundle,
            secret.expose_secret(),
            trust.expose_secret(),
            true,
        )?;
        for (flag, bytes) in [
            ("--import", self.bundle.public.as_slice()),
            ("--import", secret.expose_secret().as_slice()),
            ("--import-ownertrust", trust.expose_secret().as_slice()),
        ] {
            let mut command = command(&self.settings, &paths.home, true);
            command.arg(flag);
            capture(command, Some(bytes))?;
        }
        println!(
            "Restored GPG keys {} and their ownertrust.",
            self.bundle
                .metadata
                .fingerprints
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
        );
        Ok(())
    }
}

fn preserve(directory: &Path, selected: &BTreeSet<String>) -> Result<()> {
    let manifest = directory.join(FILES[3]);
    match fs::symlink_metadata(&manifest) {
        Ok(_) => {
            let old = metadata(&read_regular(&manifest)?)?;
            if !old.fingerprints.is_subset(selected) {
                bail!("Existing GPG backup contains keys omitted from --fingerprint; include every previously backed-up key");
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if FILES[..3]
                .iter()
                .any(|name| fs::symlink_metadata(directory.join(name)).is_ok())
            {
                bail!("Existing GPG backup metadata is missing; restore it before replacing this backup");
            }
        }
        Err(e) => return Err(e).context("Inspect existing GPG backup metadata"),
    }
    Ok(())
}

struct BackupLock {
    path: PathBuf,
}
impl BackupLock {
    fn acquire(directory: &Path) -> Result<Self> {
        let path = directory.join(".backup-lock");
        fs::create_dir(&path).context("Another GPG backup is in progress; remove a stale .backup-lock only after confirming no backup is running")?;
        Ok(Self { path })
    }
}
impl Drop for BackupLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn backup(paths: &Paths, mut settings: Settings, args: &GpgBackupArgs) -> Result<()> {
    if let Some(program) = &args.gpg_program {
        settings.program = Some(program.clone());
    }
    let directory = if let Some(path) = &args.output {
        explicit_directory(path, &paths.home)?
    } else {
        source_directory(paths, &settings)?
    };
    let selected = args
        .fingerprint
        .iter()
        .map(|key| key.to_ascii_uppercase())
        .collect::<BTreeSet<_>>();
    if selected.is_empty() || selected.iter().any(|key| !valid_fingerprint(key)) {
        bail!("Supply complete primary GPG fingerprints, not short key IDs");
    }
    for key in &selected {
        let mut command = command(&settings, &paths.home, false);
        command.args(["--with-colons", "--fingerprint", "--list-keys", key]);
        // Listing a subkey selector still yields its parent; require its exact primary.
        key_set(
            capture(command, None)?.expose_secret(),
            "pub",
            &BTreeSet::from([key.clone()]),
        )?;
    }
    preserve(&directory, &selected)?;
    let protection = if let Some(identity) = &args.identity {
        Protection::Identity(read_identity(identity, paths, &directory, true)?)
    } else {
        Protection::Passphrase(read_password(
            args.passphrase_file.as_deref(),
            paths,
            &directory,
            true,
        )?)
    };
    storage::private_dir(&directory)?;
    let _lock = BackupLock::acquire(&directory)?;
    preserve(&directory, &selected)?;
    for name in FILES {
        if fs::symlink_metadata(directory.join(name)).is_ok() {
            template::regular(&directory.join(name))?;
        }
    }
    let mut public = command(&settings, &paths.home, false);
    public.args(["--armor", "--export"]).args(&selected);
    let public = capture(public, None)?.expose_secret().clone();
    // Preserve pinentry interaction for protected exports, as in the legacy
    // helper; password input belongs to the GPG agent, never stool argv/env.
    let mut secret_command = Command::new(program(&settings, &paths.home));
    secret_command
        .arg("--no-options")
        .env_remove("STOOL_AGE_PASSPHRASE");
    secret_command
        .args(["--armor", "--export-secret-keys"])
        .args(&selected);
    let secret = capture(secret_command, None)?;
    let mut trust_command = command(&settings, &paths.home, false);
    trust_command.arg("--export-ownertrust");
    let trust = ownertrust(
        capture(trust_command, None)?.expose_secret(),
        &selected,
        true,
    )?;
    let encrypted_secret = protection.encrypt(secret.expose_secret())?;
    let encrypted_trust = protection.encrypt(trust.expose_secret())?;
    let mut metadata = Metadata {
        mode: if matches!(protection, Protection::Passphrase(_)) {
            "passphrase"
        } else {
            "identity"
        }
        .into(),
        fingerprints: selected,
        recipient: match &protection {
            Protection::Identity(identity) => Some(identity.to_public().to_string()),
            _ => None,
        },
        hashes: [
            storage::digest(&public),
            storage::digest(&encrypted_secret),
            storage::digest(&encrypted_trust),
        ],
    };
    // Validate the actual ciphertext too, rather than only the pre-encryption export.
    let recovered_secret = protection.decrypt(&encrypted_secret)?;
    let recovered_trust = protection.decrypt(&encrypted_trust)?;
    let mut bundle = Bundle {
        metadata: metadata.clone(),
        public,
        secret: encrypted_secret,
        ownertrust: encrypted_trust,
        manifest: Vec::new(),
    };
    validate_bundle(
        &settings,
        paths,
        &bundle,
        recovered_secret.expose_secret(),
        recovered_trust.expose_secret(),
        false,
    )?;
    let fingerprints = metadata
        .fingerprints
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let first = metadata.fingerprints.iter().next().unwrap();
    let mut manifest=format!("# Public metadata only; managed by stool dotfile gpg.\ngpg_backup_mode='{}'\ngpg_backup_fingerprint='{first}'\ngpg_backup_fingerprints='{fingerprints}'\n",metadata.mode);
    if let Some(recipient) = metadata.recipient.take() {
        manifest.push_str(&format!("gpg_backup_recipient='{recipient}'\n"));
    }
    for (name, hash) in ["public", "secret", "ownertrust"]
        .iter()
        .zip(&metadata.hashes)
    {
        manifest.push_str(&format!("gpg_backup_{name}_sha256='{hash}'\n"));
    }
    bundle.manifest = manifest.into_bytes();
    let staged = tempfile::Builder::new()
        .prefix(".staging.")
        .tempdir_in(&directory)?;
    for (name, bytes) in FILES.iter().zip([
        &bundle.public,
        &bundle.secret,
        &bundle.ownertrust,
        &bundle.manifest,
    ]) {
        storage::atomic_write(&staged.path().join(name), bytes, 0o600)?;
    }
    // Publish metadata last. Interrupted publication cannot validate as a complete backup.
    for name in FILES {
        storage::atomic_write(
            &directory.join(name),
            &fs::read(staged.path().join(name))?,
            0o600,
        )?;
    }
    println!("GPG backup ready for {fingerprints}. Only public keys, ciphertext and metadata were written to {}.",directory.display());
    Ok(())
}
