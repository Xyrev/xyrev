//! One-shot legacy migration. Ordinary stool commands deliberately have no legacy aliases.
use super::source::Condition;
use super::storage::{self, Config, Paths, State};
use super::{gpg, source, template};
use crate::cli::dotfile::MigrateArgs;
use crate::common::constants::ROOT_DIR;
use anyhow::{bail, Context, Result};
use redb::{Database, ReadableTable, StorageBackend, TableDefinition};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const RECEIPT: &str = "migration-from-xyrev.json";
const SUCCESS: TableDefinition<&str, &str> = TableDefinition::new("script_success");

#[derive(Serialize, Deserialize)]
struct Receipt {
    version: u32,
    from: PathBuf,
    snapshot: String,
}

// This is the published v0.5 settings serialization, including its field order.
#[derive(Serialize, Deserialize)]
struct LegacyGpgSettings {
    enabled: bool,
    directory: PathBuf,
    program: Option<PathBuf>,
    identity: Option<PathBuf>,
    passphrase_file: Option<PathBuf>,
}

fn regular(path: &Path) -> Result<Vec<u8>> {
    template::regular(path)?;
    Ok(fs::read(path)?)
}

fn exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn namespace(source: &Path, destination: &Path) -> Result<String> {
    Ok(storage::digest(&serde_json::to_vec(&(
        source,
        destination,
    ))?))
}

fn remap(path: &Path, from: &Path, to: &Path) -> PathBuf {
    path.strip_prefix(from).map_or_else(
        |_| path.to_owned(),
        |tail| {
            if tail.as_os_str().is_empty() {
                to.to_owned()
            } else {
                to.join(tail)
            }
        },
    )
}

fn legacy_path(path: &Path) -> PathBuf {
    path.components()
        .map(|part| {
            let text = part.as_os_str().to_string_lossy();
            if let Some(tail) = text.strip_prefix(".stool") {
                std::ffi::OsString::from(format!(".xyrev{tail}"))
            } else {
                part.as_os_str().to_owned()
            }
        })
        .collect()
}

// Settings are serialized before GPG path validation normalizes separators.
// Preserve their original spelling, including '/' on Windows, for the old hash.
fn legacy_setting_path(path: &Path) -> Option<PathBuf> {
    let text = path.to_str()?;
    let mut output = String::new();
    let mut start = 0;
    for (index, separator) in text
        .char_indices()
        .filter(|(_, c)| *c == '/' || (cfg!(windows) && *c == '\\'))
    {
        let component = &text[start..index];
        output.push_str(&component.replacen(
            ".stool",
            ".xyrev",
            usize::from(component.starts_with(".stool")),
        ));
        output.push(separator);
        start = index + separator.len_utf8();
    }
    let component = &text[start..];
    output.push_str(&component.replacen(
        ".stool",
        ".xyrev",
        usize::from(component.starts_with(".stool")),
    ));
    Some(output.into())
}

fn source_protocol(directory: &Path) -> Result<()> {
    for child in fs::read_dir(directory)? {
        let child = child?;
        let name = child.file_name();
        if name == ".git" {
            continue;
        }
        if name.to_string_lossy().starts_with(".xyrev") {
            bail!("Update the source to the stool protocol before migration (for example, git pull --ff-only); the working tree was not changed");
        }
        let meta = fs::symlink_metadata(child.path())?;
        if meta.is_dir() && !meta.file_type().is_symlink() {
            source_protocol(&child.path())?;
        }
    }
    Ok(())
}

fn snapshot(root: &Path) -> Result<String> {
    let mut data = regular(&root.join("config.toml"))?;
    let directory = root.join("state");
    if exists(&directory) {
        let meta = fs::symlink_metadata(&directory)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            bail!("Legacy state must be a regular directory");
        }
        let mut files = fs::read_dir(&directory)?.collect::<std::io::Result<Vec<_>>>()?;
        files.sort_by_key(|file| file.file_name());
        for file in files {
            if file
                .path()
                .extension()
                .is_some_and(|ext| ext == "json" || ext == "redb")
            {
                data.extend_from_slice(file.file_name().to_string_lossy().as_bytes());
                data.extend_from_slice(&regular(&file.path())?);
            }
        }
    }
    Ok(storage::digest(&data))
}

fn script_records(root: &Path) -> Result<BTreeMap<String, String>> {
    let path = root.join("state/scripts.redb");
    let mut records = BTreeMap::new();
    if !exists(&path) {
        return Ok(records);
    }
    // redb 2.1 opens files read/write; an in-memory backend keeps even dry-run
    // entirely free of file writes and never writes open/recovery bits to the old DB.
    let bytes = regular(&path)?;
    let memory = redb::backends::InMemoryBackend::new();
    memory.set_len(bytes.len() as u64)?;
    memory.write(0, &bytes)?;
    let db = Database::builder().create_with_backend(memory)?;
    let tx = db.begin_read()?;
    if let Ok(table) = tx.open_table(SUCCESS) {
        for record in table.iter()? {
            let (key, value) = record?;
            records.insert(key.value().to_owned(), value.value().to_owned());
        }
    }
    Ok(records)
}

fn path_boundary(text: &str, index: usize, before: bool) -> bool {
    let character = if before {
        text[..index].chars().next_back()
    } else {
        text[index..].chars().next()
    };
    character.map_or(true, |c| {
        !c.is_alphanumeric()
            && !matches!(c, '_' | '-' | '.' | '~')
            && (!before || !matches!(c, '/' | '\\'))
    })
}

fn moved_paths(text: &str, pairs: &[(&str, &str)]) -> String {
    let mut output = String::new();
    let mut position = 0;
    while position < text.len() {
        if let Some((old, new)) = pairs.iter().find(|(old, new)| {
            old != new
                && text[position..].starts_with(old)
                && path_boundary(text, position, true)
                && path_boundary(text, position + old.len(), false)
        }) {
            output.push_str(new);
            position += old.len();
        } else {
            let character = text[position..].chars().next().unwrap();
            output.push(character);
            position += character.len_utf8();
        }
    }
    output
}

#[derive(Clone, Copy, PartialEq)]
enum ScriptLanguage {
    Shell,
    Powershell,
    Other,
}

fn script_language(name: &str, text: &str) -> ScriptLanguage {
    if name.ends_with(".ps1") || name.ends_with(".ps1.tmpl") {
        return ScriptLanguage::Powershell;
    }
    let Some(shebang) = text.lines().next().unwrap_or_default().strip_prefix("#!") else {
        return if cfg!(windows) {
            ScriptLanguage::Powershell
        } else {
            ScriptLanguage::Shell
        };
    };
    let Ok(arguments) = shell_words::split(shebang) else {
        return ScriptLanguage::Other;
    };
    if arguments.iter().any(|argument| {
        matches!(
            Path::new(argument)
                .file_name()
                .and_then(|name| name.to_str()),
            Some("sh" | "bash" | "dash" | "zsh" | "ksh")
        )
    }) {
        ScriptLanguage::Shell
    } else {
        ScriptLanguage::Other
    }
}

fn environment_references(text: &str, name: &str, from: &str, to: &str) -> String {
    let language = script_language(name, text);
    // Literal heredoc/here-string bodies require a complete interpreter parser.
    // Leave them unchanged rather than interpreting their variable-looking text.
    if language == ScriptLanguage::Other
        || text.contains("$(")
        || (language == ScriptLanguage::Shell && text.contains("<<"))
        || (language == ScriptLanguage::Shell && text.contains('`'))
        || (language == ScriptLanguage::Powershell
            && (text.contains("@\"")
                || text.contains("@'")
                || text.contains("<#")
                || text.contains("#>")))
    {
        return text.to_owned();
    }
    let mut output = String::new();
    let mut position = 0;
    let mut quote = None;
    let mut comment = false;
    let escape = if language == ScriptLanguage::Powershell {
        '`'
    } else {
        '\\'
    };
    while position < text.len() {
        let remaining = &text[position..];
        let character = remaining.chars().next().unwrap();
        if character == '\n' {
            comment = false;
        }
        if !comment && quote != Some('\'') && character == escape {
            output.push(character);
            position += character.len_utf8();
            if let Some(next) = text[position..].chars().next() {
                output.push(next);
                position += next.len_utf8();
            }
            continue;
        }
        if !comment && quote.is_none() && character == '#' {
            comment = true;
        }
        if !comment && matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
        }
        if !comment && quote != Some('\'') && character == '$' {
            let scoped = if language == ScriptLanguage::Powershell {
                "env:"
            } else {
                ""
            };
            let matched = ["SOURCE_DIR", "DEST_DIR", "HOME", "OS", "ARCH", "EXECUTABLE"]
                .into_iter()
                .flat_map(|suffix| {
                    let old = format!("{scoped}{from}_{suffix}");
                    let new = format!("{scoped}{to}_{suffix}");
                    [
                        (format!("${old}"), format!("${new}")),
                        (format!("${{{old}}}"), format!("${{{new}}}")),
                    ]
                })
                .find(|(old, _)| {
                    remaining.starts_with(old)
                        && (old.ends_with('}')
                            || remaining[old.len()..].chars().next().map_or(true, |next| {
                                !next.is_alphanumeric() && next != '_' && next != ':'
                            }))
                });
            if let Some((old, new)) = matched {
                output.push_str(&new);
                position += old.len();
                continue;
            }
        }
        output.push(character);
        position += character.len_utf8();
    }
    output
}

fn old_bytes(
    new: &[u8],
    name: &str,
    old_source: &Path,
    new_source: &Path,
    from: &Path,
    to: &Path,
) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(new).ok()?;
    let old = moved_paths(
        text,
        &[
            (new_source.to_str()?, old_source.to_str()?),
            (to.to_str()?, from.to_str()?),
        ],
    );
    let old = environment_references(&old, name, "STOOL", "XYREV");
    let roundtrip = moved_paths(
        &old,
        &[
            (old_source.to_str()?, new_source.to_str()?),
            (from.to_str()?, to.to_str()?),
        ],
    );
    let roundtrip = environment_references(&roundtrip, name, "XYREV", "STOOL");
    (roundtrip == text).then(|| old.into_bytes())
}

struct Inheritance {
    state: Option<State>,
    records: BTreeMap<String, String>,
    obsolete: Vec<String>,
}

fn inheritance(
    home: &Path,
    from: &Path,
    to: &Path,
    config: &Config,
    new_source: &Path,
    old_ns: &str,
    new_ns: &str,
) -> Result<Inheritance> {
    let state_path = from.join("state").join(format!("{old_ns}.json"));
    let mut state = if exists(&state_path) {
        Some(
            serde_json::from_slice::<State>(&regular(&state_path)?)
                .context("Invalid legacy file state")?,
        )
    } else {
        None
    };
    let records = script_records(from)?;
    if old_ns != new_ns
        && records
            .keys()
            .any(|key| key.starts_with(&format!("{new_ns}:")))
    {
        bail!(
            "New namespace already contains historical script state; resolve it before migration"
        );
    }
    let mut inherited = BTreeMap::new();
    let mut obsolete = Vec::new();
    let paths = Paths {
        home: home.to_owned(),
        root: to.to_owned(),
        source: config.source.clone(),
        destination: config.destination.clone(),
        state_file: to.join("state").join(format!("{new_ns}.json")),
        config: Some(config.clone()),
    };
    let engine = template::Engine::for_migration(&paths).and_then(|mut engine| {
        let root = template::source_root(&paths)?;
        engine.data["stool"]["source_dir"] =
            serde_json::to_value(remap(&root, &config.source, new_source))?;
        Ok(engine)
    });
    if let Some(state) = &mut state {
        let old_digest = state.gpg_restore.take();
        if let Ok(engine) = &engine {
            if let Some(plan) = gpg::prepare_restore(&paths, engine)? {
                let (settings, directory, manifest) = plan.migration_material()?;
                let relative = directory.strip_prefix(&config.source)?;
                let new_directory = new_source.join(relative);
                let old_directory = config.source.join(legacy_path(relative));
                let mut old_settings: LegacyGpgSettings = serde_json::from_slice(&settings)?;
                old_settings.directory = legacy_setting_path(&old_settings.directory)
                    .context("GPG directory is not UTF-8; its success cannot be inherited")?;
                let mut old_material = serde_json::to_vec(&old_settings)?;
                old_material.extend_from_slice(old_directory.to_string_lossy().as_bytes());
                old_material.extend_from_slice(&manifest);
                if old_digest.as_deref() == Some(storage::digest(&old_material).as_str()) {
                    let mut new_material = settings;
                    new_material.extend_from_slice(new_directory.to_string_lossy().as_bytes());
                    new_material.extend_from_slice(&manifest);
                    state.gpg_restore = Some(storage::digest(&new_material));
                    println!(
                        "GPG restoration success inherited without decrypting or importing keys"
                    );
                } else if old_digest.is_some() {
                    println!("GPG restoration success not inherited: no exact legacy digest match");
                }
            } else if old_digest.is_some() {
                println!("GPG restoration success not inherited: no active validated declaration");
            }
        } else if old_digest.is_some() {
            println!(
                "GPG restoration success not inherited: template data needs dynamic evaluation"
            );
        }
    }
    match engine {
        Ok(engine) => {
            match source::scan(&paths, &engine, false) {
                Ok(tree) => {
                    for script in tree.scripts {
                        if script.condition == Condition::Always {
                            continue;
                        }
                        let bytes = regular(&tree.root.join(&script.name))?;
                        let rendered = if script.name.ends_with(".tmpl") {
                            engine.render(&script.name, &bytes)
                        } else {
                            Ok(bytes)
                        };
                        let candidate = rendered.as_ref().ok().and_then(|bytes| {
                            old_bytes(bytes, &script.name, &config.source, new_source, from, to)
                        });
                        let Some(old) = candidate else {
                            println!("Script success not inherited: {} requires dynamic or non-equivalent rendering", script.name);
                            continue;
                        };
                        let old_name = legacy_setting_path(Path::new(&script.name))
                            .context("Script name cannot be represented for legacy comparison")?
                            .to_string_lossy()
                            .into_owned();
                        let old_key = match script.condition {
                            Condition::Once => format!("{old_ns}:once:{}", storage::digest(&old)),
                            Condition::OnChange => {
                                format!("{old_ns}:onchange:{old_name}:{}", storage::digest(&old))
                            }
                            Condition::Always => unreachable!(),
                        };
                        if records
                            .get(&old_key)
                            .is_some_and(|value| value == "success")
                        {
                            let hash = storage::digest(&rendered?);
                            let key = match script.condition {
                                Condition::Once => format!("{new_ns}:once:{hash}"),
                                Condition::OnChange => {
                                    format!("{new_ns}:onchange:{}:{hash}", script.name)
                                }
                                Condition::Always => unreachable!(),
                            };
                            if script.condition == Condition::OnChange {
                                let prefix = format!("{new_ns}:onchange:{}:", script.name);
                                obsolete.extend(
                                    records
                                        .keys()
                                        .filter(|key| key.starts_with(&prefix))
                                        .cloned(),
                                );
                            }
                            inherited.insert(key, "success".into());
                        } else {
                            println!("Script success not inherited: {} has no exact successful legacy key", script.name);
                        }
                    }
                }
                Err(_) => println!(
                    "Script success not inherited: control templates need dynamic evaluation"
                ),
            }
        }
        Err(_) => println!("Script success not inherited: template data needs dynamic evaluation"),
    }
    println!(
        "{} script success record(s) inherited; unknown namespaces retained",
        inherited.len()
    );
    Ok(Inheritance {
        state,
        records: inherited,
        obsolete,
    })
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(from)?;
    if meta.file_type().is_symlink() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(fs::read_link(from)?, to)?;
        #[cfg(windows)]
        if from.is_dir() {
            std::os::windows::fs::symlink_dir(fs::read_link(from)?, to)?;
        } else {
            std::os::windows::fs::symlink_file(fs::read_link(from)?, to)?;
        }
        #[cfg(not(any(unix, windows)))]
        bail!("Cannot preserve legacy symlinks on this platform");
    } else if meta.is_dir() {
        fs::create_dir(to)?;
        for child in fs::read_dir(from)? {
            let child = child?;
            copy_tree(&child.path(), &to.join(child.file_name()))?;
        }
        fs::set_permissions(to, meta.permissions())?;
    } else if meta.is_file() {
        fs::copy(from, to)?;
    } else {
        bail!("Cannot migrate special file: {}", from.display());
    }
    Ok(())
}

fn marker(source: &Path) -> Result<bool> {
    let old = source.join(".git/xyrev-builtin");
    if !exists(&old) {
        return Ok(false);
    }
    let git = fs::symlink_metadata(source.join(".git"))?;
    if !git.is_dir() || git.file_type().is_symlink() {
        bail!("Legacy Git metadata must be a regular directory");
    }
    if !matches!(regular(&old)?.as_slice(), b"1" | b"1\n") {
        bail!("Invalid legacy built-in Git marker");
    }
    let new = source.join(".git/stool-builtin");
    if exists(&new) && !matches!(regular(&new)?.as_slice(), b"1" | b"1\n") {
        bail!("Invalid stool built-in Git marker");
    }
    Ok(!exists(&new))
}

struct MarkerGuard(Option<PathBuf>);
impl Drop for MarkerGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            let _ = fs::remove_file(path);
        }
    }
}

pub fn run(args: &MigrateArgs) -> Result<()> {
    let home = storage::resolve(Path::new(
        &std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .context("Cannot determine home")?,
    ))?;
    let from = storage::resolve(&storage::expand_home(
        args.from.as_deref().unwrap_or(&home.join(".xyrev/dotfile")),
        &home,
    ))?;
    let to = storage::resolve(&home.join(ROOT_DIR).join("dotfile"))?;
    if from.starts_with(&to) || to.starts_with(&from) {
        bail!("Migration roots must not overlap");
    }
    if exists(&to) {
        let receipt = to.join(RECEIPT);
        if exists(&receipt) {
            let receipt: Receipt = serde_json::from_slice(&regular(&receipt)?)?;
            if receipt.version == 1 && receipt.from == from && snapshot(&from)? == receipt.snapshot
            {
                println!(
                    "Already migrated; the legacy directory is retained at {}",
                    from.display()
                );
                return Ok(());
            }
        }
        bail!("Migration target already exists; preserve it and resolve the conflict manually");
    }
    if !exists(&from.join("config.toml")) {
        bail!("No legacy configuration found; use stool dotfile init for a new installation");
    }
    let _old_lock = storage::lock(&from, !args.dry_run)?;
    let before = snapshot(&from)?;
    let mut config: Config =
        toml::from_str(std::str::from_utf8(&regular(&from.join("config.toml"))?)?)?;
    if config.version != 1
        || !config.source.is_absolute()
        || !config.destination.is_absolute()
        || !config.data.is_object()
    {
        bail!("Unsupported legacy configuration or non-absolute saved paths");
    }
    if config.data.get("stool").is_some() {
        bail!("Legacy data collides with the reserved stool namespace");
    }
    config.source = storage::resolve(&config.source)?;
    config.destination = storage::resolve(&config.destination)?;
    if !config.source.is_dir() {
        bail!("Saved legacy source is missing; restore it before migration");
    }
    if config.source == from
        || from.starts_with(&config.source)
        || to.starts_with(&config.source)
        || config.destination.starts_with(&from)
        || config.destination.starts_with(&to)
    {
        bail!("Saved paths overlap migration control storage");
    }
    source_protocol(&config.source)?;
    if config.source.starts_with(&from) && exists(&config.source.join(".git")) {
        let metadata = fs::symlink_metadata(config.source.join(".git"))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            bail!("An internal linked worktree cannot be copied safely; keep it as an external source before migration");
        }
    }
    let old_ns = namespace(&config.source, &config.destination)?;
    let new_source = remap(&config.source, &from, &to);
    let new_ns = namespace(&new_source, &config.destination)?;
    if old_ns != new_ns && exists(&from.join("state").join(format!("{new_ns}.json"))) {
        bail!("New namespace already contains historical file state; resolve it before migration");
    }
    let add_marker = marker(&config.source)?;
    let inherited = inheritance(&home, &from, &to, &config, &new_source, &old_ns, &new_ns)?;
    println!(
        "Source: {} -> {}\nDestination: {}",
        config.source.display(),
        new_source.display(),
        config.destination.display()
    );
    if inherited.state.is_none() {
        println!("No saved file state for this pair; no success will be inferred");
    }
    if add_marker {
        println!("Add stool built-in Git marker; retain the legacy marker");
    }
    if args.dry_run {
        println!("Dry run: no files written; no hooks, GPG or passwords were requested");
        return Ok(());
    }
    let parent = to.parent().context("Migration target has no parent")?;
    // Serialize migrations from different old roots into this same HOME. The
    // old command lock alone only serializes users of one legacy directory.
    let _migration_lock = storage::lock(parent, true)?;
    if exists(&to) {
        bail!("Migration target already exists; preserve it and resolve the conflict manually");
    }
    let staging = tempfile::Builder::new()
        .prefix(".migration-")
        .tempdir_in(parent)?;
    let staged_root = staging.path().join("dotfile");
    copy_tree(&from, &staged_root)?;
    storage::private_dir(&staged_root)?;
    let staged_source = remap(&config.source, &from, &staged_root);
    if config.source.starts_with(&from) {
        let mut path = staged_root.clone();
        for part in config.source.strip_prefix(&from)?.components() {
            path.push(part);
            let meta = fs::symlink_metadata(&path)?;
            if !meta.is_dir() || meta.file_type().is_symlink() {
                bail!("Copied internal source must not contain symlinked parents");
            }
        }
    }
    let mut marker_guard = MarkerGuard(None);
    if add_marker {
        let git = fs::symlink_metadata(staged_source.join(".git"))?;
        if !git.is_dir() || git.file_type().is_symlink() {
            bail!("Copied Git metadata must be a regular directory");
        }
        let path = staged_source.join(".git/stool-builtin");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        if !config.source.starts_with(&from) {
            marker_guard.0 = Some(path.clone());
        }
        file.write_all(b"1\n")?;
        storage::set_mode(&path, 0o600)?;
    }
    config.source = new_source;
    template::regular(&staged_root.join("config.toml"))?;
    storage::atomic_write(
        &staged_root.join("config.toml"),
        toml::to_string_pretty(&config)?.as_bytes(),
        0o600,
    )?;
    if let Some(state) = inherited.state {
        storage::private_dir(&staged_root.join("state"))?;
        let state_path = staged_root.join("state").join(format!("{new_ns}.json"));
        if exists(&state_path) {
            template::regular(&state_path)?;
        }
        storage::atomic_write(&state_path, &serde_json::to_vec_pretty(&state)?, 0o600)?;
    }
    if !inherited.records.is_empty() {
        storage::private_dir(&staged_root.join("state"))?;
        let file = staged_root.join("state/scripts.redb");
        if exists(&file) {
            template::regular(&file)?;
        }
        let db = Database::create(&file)?;
        let tx = db.begin_write()?;
        {
            let mut table = tx.open_table(SUCCESS)?;
            for key in inherited.obsolete {
                table.remove(key.as_str())?;
            }
            for (key, value) in inherited.records {
                table.insert(key.as_str(), value.as_str())?;
            }
        }
        tx.commit()?;
        drop(db);
        storage::set_mode(&file, 0o600)?;
    }
    storage::atomic_write(
        &staged_root.join(RECEIPT),
        &serde_json::to_vec_pretty(&Receipt {
            version: 1,
            from: from.clone(),
            snapshot: before.clone(),
        })?,
        0o600,
    )?;
    // Check all copied control metadata even when no record was inherited.
    // A symlinked empty DB must never become the next apply's writable DB.
    snapshot(&staged_root)?;
    if exists(&to) || snapshot(&from)? != before {
        bail!("Migration input or target changed; no new control directory published");
    }
    fs::rename(&staged_root, &to).context("Publish migration; legacy directory retained")?;
    marker_guard.0 = None;
    println!("Migrated to {}. Legacy rollback copy retained at {}. Inspect stool dotfile status before apply.", to.display(), from.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(name: &str, text: &str) -> String {
        String::from_utf8(
            old_bytes(
                text.as_bytes(),
                name,
                Path::new("/old/source"),
                Path::new("/new/source"),
                Path::new("/old/control"),
                Path::new("/new/control"),
            )
            .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn migration_maps_only_known_variable_references_and_moved_path_boundaries() {
        let shell = "#!/bin/sh\nprintf '%s' \"$STOOL_HOME/${STOOL_OS}/$STOOL_ARCH\"\nprintf '%s' \"$STOOL_SOURCE_DIR/$STOOL_DEST_DIR/$STOOL_EXECUTABLE\"\n";
        assert_eq!(
            candidate("run_once.sh", shell),
            shell.replace("STOOL", "XYREV")
        );
        let powershell = "Write-Output \"$env:STOOL_HOME/${env:STOOL_OS}/$env:STOOL_ARCH\"\n";
        assert_eq!(
            candidate("run_once.ps1", powershell),
            powershell.replace("STOOL", "XYREV")
        );
        assert_eq!(
            candidate(
                "run_once.sh",
                "#!/bin/sh\n# /new/source /new/control/file /new/source-extra /prefix/new/source\n"
            ),
            "#!/bin/sh\n# /old/source /old/control/file /new/source-extra /prefix/new/source\n"
        );
    }

    #[test]
    fn migration_does_not_map_literal_escaped_unknown_or_complex_variable_text() {
        for text in [
            "#!/bin/sh\nprintf '%s' 'STOOL_HOME'\n",
            "#!/bin/sh\nprintf '%s' '$STOOL_HOME'\n",
            "#!/bin/sh\nprintf '%s' \\$STOOL_HOME\n",
            "#!/bin/sh\nprintf '%s' \"$STOOL_HOME_SUFFIX\"\n",
            "#!/bin/sh\n# $STOOL_HOME\n",
            "#!/bin/sh\ncat <<'EOF'\n$STOOL_HOME\nEOF\n",
            "#!/bin/sh\nprintf '%s' \"$(printf '$STOOL_HOME')\"\n",
            "#!/bin/sh\nprintf '%s' `$STOOL_HOME`\n",
        ] {
            assert_eq!(candidate("run_once.sh", text), text);
        }
        for text in [
            "Write-Output '$env:STOOL_HOME'\n",
            "Write-Output `$env:STOOL_HOME\n",
            "Write-Output $env:STOOL_HOME_SUFFIX\n",
            "Write-Output 'STOOL_HOME'\n",
            "<# $env:STOOL_HOME #>\n",
            "@'\n$env:STOOL_HOME\n'@\n",
        ] {
            assert_eq!(candidate("run_once.ps1", text), text);
        }
    }

    #[test]
    fn migration_does_not_treat_application_names_or_log_messages_as_protocol() {
        let text = "#!/bin/sh\nmkdir -p \"$STOOL_HOME/.config/stool\"\necho stool\n";
        assert_eq!(
            candidate("run_once.sh", text),
            "#!/bin/sh\nmkdir -p \"$XYREV_HOME/.config/stool\"\necho stool\n"
        );
        assert_ne!(
            candidate("run_once.sh", text),
            text.replace("STOOL", "XYREV").replace("stool", "xyrev")
        );
    }

    #[test]
    fn legacy_gpg_setting_serialization_keeps_original_separator_spelling() {
        let path = legacy_setting_path(Path::new(".stooltemplates/gpg")).unwrap();
        assert_eq!(path.as_os_str(), ".xyrevtemplates/gpg");
        assert_eq!(
            serde_json::to_vec(&path).unwrap(),
            b"\".xyrevtemplates/gpg\""
        );
        #[cfg(windows)]
        {
            let path = legacy_setting_path(Path::new(r".stooltemplates\.stooldata\gpg")).unwrap();
            assert_eq!(path.as_os_str(), r".xyrevtemplates\.xyrevdata\gpg");
            assert_eq!(
                serde_json::to_vec(&path).unwrap(),
                serde_json::to_vec(r".xyrevtemplates\.xyrevdata\gpg").unwrap()
            );
        }
    }
}
