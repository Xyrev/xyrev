use anyhow::{bail, Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub repo: String,
    pub branch: String,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub recurse_submodules: bool,
    #[serde(default = "empty_object")]
    pub data: serde_json::Value,
    #[serde(default)]
    pub encryption: EncryptionSettings,
    #[serde(default)]
    pub git: GitSettings,
    #[serde(default)]
    pub editor: Option<String>,
}

pub fn empty_object() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EncryptionSettings {
    pub backend: String,
    pub recipients: Vec<String>,
    pub identities: Vec<PathBuf>,
    pub gpg_recipient: Option<String>,
}
impl Default for EncryptionSettings {
    fn default() -> Self {
        Self {
            backend: "age".into(),
            recipients: Vec::new(),
            identities: Vec::new(),
            gpg_recipient: None,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GitSettings {
    pub builtin: bool,
    pub lfs: bool,
}

pub struct Paths {
    pub home: PathBuf,
    pub root: PathBuf,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub state_file: PathBuf,
    pub config: Option<Config>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Fingerprint {
    File { hash: String, mode: u32 },
    Directory { mode: u32 },
    Symlink { target: PathBuf },
}

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub entries: BTreeMap<String, Fingerprint>,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Resolve CLI roots once; descendant paths are subsequently checked without following symlinks.
pub fn resolve(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for part in absolute.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    let mut ancestor = normalized.as_path();
    let mut missing = Vec::new();
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                missing.push(
                    ancestor
                        .file_name()
                        .context("Invalid absolute path")?
                        .to_owned(),
                );
                ancestor = ancestor.parent().context("Cannot resolve path")?;
            }
            Err(e) => return Err(e).with_context(|| format!("Inspect {}", ancestor.display())),
        }
    }
    let mut result =
        fs::canonicalize(ancestor).with_context(|| format!("Resolve {}", ancestor.display()))?;
    for part in missing.into_iter().rev() {
        result.push(part);
    }
    Ok(result)
}

impl Paths {
    pub fn load(source: Option<&Path>, destination: Option<&Path>) -> Result<Self> {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .context("Cannot determine user home directory")?;
        let home = resolve(Path::new(&home))?;
        let root = resolve(&home.join(".xyrev/dotfile"))?;
        let config: Option<Config> = match fs::read_to_string(root.join("config.toml")) {
            Ok(text) => {
                let cfg: Config = toml::from_str(&text).context("Invalid dotfile config.toml")?;
                if cfg.version != 1 {
                    bail!("Unsupported dotfile configuration version {}", cfg.version);
                }
                if !cfg.data.is_object() {
                    bail!("Configuration data must be an object");
                }
                Some(cfg)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e).context("Read dotfile configuration"),
        };
        let default_source = root.join("source");
        let source = resolve(source.unwrap_or_else(|| {
            config
                .as_ref()
                .map(|c| c.source.as_path())
                .unwrap_or(&default_source)
        }))?;
        let destination = resolve(destination.unwrap_or_else(|| {
            config
                .as_ref()
                .map(|c| c.destination.as_path())
                .unwrap_or(&home)
        }))?;
        if destination.starts_with(&source) || root.starts_with(&source) {
            bail!("Source must not contain the destination or xyrev control directory");
        }
        if destination.starts_with(&root) {
            bail!("Destination must not be inside the xyrev control directory");
        }
        let key = digest(&serde_json::to_vec(&(&source, &destination))?);
        Ok(Self {
            home,
            state_file: root.join("state").join(format!("{key}.json")),
            root,
            source,
            destination,
            config,
        })
    }

    pub fn state(&self) -> Result<State> {
        match fs::read(&self.state_file) {
            Ok(data) => serde_json::from_slice(&data).context("Invalid dotfile state"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(State::default()),
            Err(e) => Err(e).context("Read dotfile state"),
        }
    }

    pub fn save_state(&self, state: &State) -> Result<()> {
        private_dir(self.state_file.parent().unwrap())?;
        atomic_write(&self.state_file, &serde_json::to_vec_pretty(state)?, 0o600)
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        atomic_write(
            &self.root.join("config.toml"),
            toml::to_string_pretty(config)?.as_bytes(),
            0o600,
        )
    }

    pub fn check_target(&self, target: &Path) -> Result<()> {
        if !target.starts_with(&self.destination)
            || target == self.destination
            || target.starts_with(&self.source)
            || self.source.starts_with(target)
            || target.starts_with(&self.root)
            || self.root.starts_with(target)
        {
            bail!(
                "Target overlaps source/control storage or escapes destination: {}",
                target.display()
            );
        }
        Ok(())
    }

    pub fn encryption(&self) -> EncryptionSettings {
        self.config
            .as_ref()
            .map(|cfg| cfg.encryption.clone())
            .unwrap_or_default()
    }
}

pub fn relative_path(raw: &Path) -> Result<PathBuf> {
    let mut result = PathBuf::new();
    for part in raw.components() {
        match part {
            Component::Normal(name) => {
                #[cfg(windows)]
                {
                    let text = name.to_str().context("Path must be UTF-8")?;
                    let base = text.split('.').next().unwrap_or("").to_ascii_uppercase();
                    if text.contains([':', '<', '>', '"', '|', '?', '*'])
                        || text.ends_with(['.', ' '])
                        || ["CON", "PRN", "AUX", "NUL"].contains(&base.as_str())
                        || (base.len() == 4
                            && (base.starts_with("COM") || base.starts_with("LPT"))
                            && matches!(base.as_bytes()[3], b'1'..=b'9'))
                    {
                        bail!("Invalid Windows destination name: {text}");
                    }
                }
                result.push(name)
            }
            Component::CurDir => {}
            _ => bail!("Expected a path inside the destination: {}", raw.display()),
        }
    }
    if result.as_os_str().is_empty() {
        bail!("A nonempty relative path is required");
    }
    Ok(result)
}

pub fn expand_home(path: &Path, home: &Path) -> PathBuf {
    if path == Path::new("~") {
        home.to_owned()
    } else if let Ok(relative) = path.strip_prefix("~") {
        home.join(relative)
    } else {
        path.to_owned()
    }
}

/// Keep the file open for the command lifetime; OS locks are released even after a crash.
pub fn lock(root: &Path, write: bool) -> Result<Option<File>> {
    if write {
        private_dir(root)?;
    }
    let file = match OpenOptions::new()
        .read(true)
        .write(write)
        .create(write)
        .truncate(false)
        .open(root.join("lock"))
    {
        Ok(file) => file,
        Err(e) if !write && e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).context("Open dotfile lock"),
    };
    let result = if write {
        file.try_lock_exclusive()
    } else {
        FileExt::try_lock_shared(&file)
    };
    result.context("Another dotfile command is running; retry after it finishes")?;
    Ok(Some(file))
}

pub fn private_dir(path: &Path) -> Result<()> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() || !meta.is_dir() {
            bail!(
                "Control directory is not a regular directory: {}",
                path.display()
            );
        }
    }
    fs::create_dir_all(path).with_context(|| format!("Create {}", path.display()))?;
    set_mode(path, 0o700)
}

pub fn atomic_write(path: &Path, data: &[u8], mode: u32) -> Result<()> {
    let mut temp =
        tempfile::NamedTempFile::new_in(path.parent().context("File has no parent directory")?)?;
    temp.write_all(data)?;
    // Windows persistence changes file attributes, so retain owner write access
    // until the replacement is complete; private access rules still apply.
    #[cfg(windows)]
    let staged_mode = mode | 0o200;
    #[cfg(not(windows))]
    let staged_mode = mode;
    set_mode(temp.path(), staged_mode)?;
    temp.as_file().sync_all()?;
    #[cfg(windows)]
    if path.is_file() {
        set_mode(path, mode | 0o200)?;
    }
    temp.persist(path)
        .map_err(|e| e.error)
        .with_context(|| format!("Replace {}", path.display()))?;
    // tempfile clears FILE_ATTRIBUTE_READONLY while persisting on Windows.
    // Set read-only permissions only after its atomic replacement completes.
    #[cfg(windows)]
    if mode & 0o200 == 0 {
        set_mode(path, mode)?;
    }
    Ok(())
}

#[cfg(unix)]
pub fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}
#[cfg(windows)]
pub fn set_mode(path: &Path, mode: u32) -> Result<()> {
    // Keep all path values in the environment, never interpolate them as code.
    let script = r#"$ErrorActionPreference='Stop'; $path=$env:XYREV_ACL_PATH; $item=Get-Item -LiteralPath $path -Force; $directory=$item.PSIsContainer; $mode=[Convert]::ToInt32($env:XYREV_ACL_MODE,8); $acl=Get-Acl -LiteralPath $path; $acl.SetAccessRuleProtection($true,$false); foreach($rule in @($acl.Access)){$acl.RemoveAccessRuleSpecific($rule)}; $inherit=if($directory){[System.Security.AccessControl.InheritanceFlags]'ContainerInherit,ObjectInherit'}else{[System.Security.AccessControl.InheritanceFlags]::None}; $prop=[System.Security.AccessControl.PropagationFlags]::None; $allow=[System.Security.AccessControl.AccessControlType]::Allow; $me=[System.Security.Principal.WindowsIdentity]::GetCurrent().User; $rights=if(($mode -band 128) -ne 0){[System.Security.AccessControl.FileSystemRights]::FullControl}else{[System.Security.AccessControl.FileSystemRights]::ReadAndExecute}; $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($me,$rights,$inherit,$prop,$allow)); foreach($sid in @('S-1-5-18','S-1-5-32-544')){$acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new([System.Security.Principal.SecurityIdentifier]::new($sid),[System.Security.AccessControl.FileSystemRights]::FullControl,$inherit,$prop,$allow))}; if(($mode -band 63) -ne 0){$acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new([System.Security.Principal.SecurityIdentifier]::new('S-1-5-32-545'),[System.Security.AccessControl.FileSystemRights]::ReadAndExecute,$inherit,$prop,$allow))}; if(!$directory -and (($mode -band 128) -eq 0)){$item.IsReadOnly=$true}; Set-Acl -LiteralPath $path -AclObject $acl; if(!$directory -and (($mode -band 128) -ne 0)){$item.IsReadOnly=$false}"#;
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("XYREV_ACL_PATH", path)
        .env("XYREV_ACL_MODE", format!("{mode:o}"))
        .output()
        .context("Set Windows ACL")?;
    if !output.status.success() {
        bail!(
            "Cannot set Windows permissions for {} ({}): {}",
            path.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}
#[cfg(not(any(unix, windows)))]
pub fn set_mode(_path: &Path, _mode: u32) -> Result<()> {
    bail!("Unsupported platform permissions")
}

#[cfg(unix)]
pub fn mode(meta: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o777
}
#[cfg(not(unix))]
pub fn mode(meta: &fs::Metadata) -> u32 {
    if meta.is_dir() {
        0o755
    } else {
        0o644
    }
}

/// Windows maps private/read-only attributes to a protected DACL. Unix modes
/// remain native; execute bits have no separate meaning for Windows files.
#[cfg(not(windows))]
pub fn path_mode(_path: &Path, meta: &fs::Metadata) -> Result<u32> {
    Ok(mode(meta))
}

#[cfg(windows)]
fn acl_command(path: &Path, script: &str) -> Result<std::process::Output> {
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("XYREV_ACL_PATH", path)
        .output()
        .context("Run Windows ACL helper")?;
    if !output.status.success() {
        bail!(
            "Cannot read or set Windows permissions for {} ({}): {}",
            path.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output)
}

#[cfg(windows)]
pub fn path_mode(path: &Path, meta: &fs::Metadata) -> Result<u32> {
    let output = acl_command(
        path,
        r#"$ErrorActionPreference='Stop'; $acl=Get-Acl -LiteralPath $env:XYREV_ACL_PATH; $me=[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value; $private=$true; $write=$false; foreach($rule in $acl.Access){if($rule.AccessControlType -ne 'Allow'){continue}; $sid=$rule.IdentityReference.Translate([System.Security.Principal.SecurityIdentifier]).Value; if($sid -notin @($me,'S-1-5-18','S-1-5-32-544')){$private=$false}; if($sid -eq $me -and (($rule.FileSystemRights -band [System.Security.AccessControl.FileSystemRights]::WriteData) -ne 0)){$write=$true}}; @{private=$private;write=$write}|ConvertTo-Json -Compress"#,
    )?;
    let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let mut mode = if meta.is_dir() { 0o755 } else { 0o644 };
    if value["private"] == true {
        mode &= 0o700;
    }
    if value["write"] == false || (!meta.is_dir() && meta.permissions().readonly()) {
        mode &= !0o222;
    }
    Ok(mode)
}

pub fn normalized_mode(mode: u32, directory: bool) -> u32 {
    if cfg!(windows) && !directory {
        mode & !0o111
    } else {
        mode
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;

    #[test]
    fn atomic_private_writes_preserve_readonly_and_can_be_replaced() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("private [folder]");
        private_dir(&directory).unwrap();
        let file = directory.join("identity [test]");
        atomic_write(&file, b"first", 0o600).unwrap();
        assert_eq!(
            path_mode(&file, &fs::metadata(&file).unwrap()).unwrap(),
            0o600
        );
        atomic_write(&file, b"readonly", 0o400).unwrap();
        assert!(fs::metadata(&file).unwrap().permissions().readonly());
        assert_eq!(
            path_mode(&file, &fs::metadata(&file).unwrap()).unwrap(),
            0o400
        );
        atomic_write(&file, b"replacement", 0o600).unwrap();
        assert_eq!(fs::read(&file).unwrap(), b"replacement");
        assert!(!fs::metadata(&file).unwrap().permissions().readonly());
        assert_eq!(
            path_mode(&file, &fs::metadata(&file).unwrap()).unwrap(),
            0o600
        );
    }
}
