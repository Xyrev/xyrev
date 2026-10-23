mod common;
use common::gpg::{fake_gpg, private_write};
use common::*;
use redb::{Database, ReadableTable, TableDefinition};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const SUCCESS: TableDefinition<&str, &str> = TableDefinition::new("script_success");
const FINGERPRINT: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const BACKUP_FILES: [&str; 4] = [
    "public.asc",
    "secret.asc.age",
    "ownertrust.txt.age",
    "backup-metadata.sh",
];

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn namespace(source: &Path, destination: &Path) -> String {
    digest(&serde_json::to_vec(&(source, destination)).unwrap())
}

struct Fixture {
    s: Sandbox,
    old: PathBuf,
    source: PathBuf,
    destination: PathBuf,
    config: Value,
}

impl Fixture {
    fn new(custom_source: bool, custom_destination: bool) -> Self {
        let s = Sandbox::new(&[("dot_seed", "seed\n"), (".stoolversion", "0.6.0\n")]);
        let old = s.home.join(".xyrev/dotfile");
        fs::create_dir_all(&old).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&old, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let old = fs::canonicalize(old).unwrap();
        let source = if custom_source {
            s.work.clone()
        } else {
            let source = old.join("source");
            git(
                &s.root,
                &[
                    "clone",
                    "--quiet",
                    s.work.to_str().unwrap(),
                    source.to_str().unwrap(),
                ],
            );
            source
        };
        let source = fs::canonicalize(source).unwrap();
        let destination = if custom_destination {
            s.destination.clone()
        } else {
            s.home.clone()
        };
        write(&destination.join(".seed"), "seed\n");
        let destination = fs::canonicalize(destination).unwrap();
        let config = json!({
            "version": 1,
            "repo": s.url(),
            "branch": "dotfiles-trunk",
            "source": source,
            "destination": destination,
            "recurse_submodules": false,
            "data": {"name": "Retained User"},
            "encryption": {"backend": "age", "recipients": [], "identities": []},
            "git": {"builtin": false, "lfs": false}
        });
        let fixture = Self {
            s,
            old,
            source,
            destination,
            config,
        };
        fixture.save_config();
        private_write(&fixture.old.join("lock"), b"");
        fixture.write_state(None);
        fixture
    }

    fn save_config(&self) {
        private_write(
            &self.old.join("config.toml"),
            toml::to_string(&self.config).unwrap().as_bytes(),
        );
    }

    fn old_namespace(&self) -> String {
        namespace(&self.source, &self.destination)
    }

    fn new_root(&self) -> PathBuf {
        fs::canonicalize(&self.s.home)
            .unwrap()
            .join(".stool/dotfile")
    }

    fn new_source(&self) -> PathBuf {
        if self.source.starts_with(&self.old) {
            self.new_root()
                .join(self.source.strip_prefix(&self.old).unwrap())
        } else {
            self.source.clone()
        }
    }

    fn new_namespace(&self) -> String {
        namespace(&self.new_source(), &self.destination)
    }

    fn write_state(&self, gpg_restore: Option<&str>) {
        let mut state = json!({"entries": {".seed": {
            "kind": "file", "hash": digest(b"seed\n"), "mode": 0o644
        }}});
        if let Some(gpg_restore) = gpg_restore {
            state["gpg_restore"] = gpg_restore.into();
        }
        private_write(
            &self
                .old
                .join("state")
                .join(format!("{}.json", self.old_namespace())),
            &serde_json::to_vec_pretty(&state).unwrap(),
        );
    }

    fn new_state(&self) -> Value {
        serde_json::from_slice(
            &fs::read(
                self.s
                    .control()
                    .join("state")
                    .join(format!("{}.json", self.new_namespace())),
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn command(&self) -> Command {
        let mut command = self.s.command();
        command.args(["migrate", "--from"]).arg(&self.old);
        command
    }

    fn migrate(&self) -> String {
        success(self.command().output().unwrap())
    }

    fn write_scripts(&self, keys: &[String]) {
        let db = Database::create(self.old.join("state/scripts.redb")).unwrap();
        let transaction = db.begin_write().unwrap();
        {
            let mut table = transaction.open_table(SUCCESS).unwrap();
            for key in keys {
                table.insert(key.as_str(), "success").unwrap();
            }
        }
        transaction.commit().unwrap();
    }

    fn migrated_script_keys(&self) -> Vec<String> {
        let db = Database::open(self.s.control().join("state/scripts.redb")).unwrap();
        let transaction = db.begin_read().unwrap();
        let table = transaction.open_table(SUCCESS).unwrap();
        table
            .iter()
            .unwrap()
            .map(|entry| entry.unwrap().0.value().to_owned())
            .collect()
    }

    fn write_bundle(&self) -> Vec<Vec<u8>> {
        let directory = self.source.join(".stooltemplates/gpg");
        let contents = [
            b"public migration fixture\n".as_slice(),
            b"opaque encrypted secret fixture\0\n".as_slice(),
            b"opaque encrypted trust fixture\0\n".as_slice(),
        ];
        for (name, bytes) in BACKUP_FILES.iter().zip(contents) {
            private_write(&directory.join(name), bytes);
        }
        let metadata = format!(
            "gpg_backup_mode='passphrase'\ngpg_backup_fingerprint='{FINGERPRINT}'\ngpg_backup_fingerprints='{FINGERPRINT}'\ngpg_backup_public_sha256='{}'\ngpg_backup_secret_sha256='{}'\ngpg_backup_ownertrust_sha256='{}'\n",
            digest(contents[0]), digest(contents[1]), digest(contents[2])
        );
        private_write(&directory.join(BACKUP_FILES[3]), metadata.as_bytes());
        write(
            &self.source.join(".stoolgpg.toml"),
            "enabled=true\ndirectory='.stooltemplates/gpg'\n",
        );
        BACKUP_FILES
            .iter()
            .map(|name| fs::read(directory.join(name)).unwrap())
            .collect()
    }
}

#[test]
fn dry_run_does_not_create_the_new_root_or_modify_any_legacy_data() {
    let f = Fixture::new(false, true);
    f.write_scripts(&[format!("unrelated:once:{}", digest(b"unknown script"))]);
    private_write(&f.old.join("backups/retained.bin"), b"retained backup\0");
    let before = snapshot(&f.s.root);
    success(f.command().arg("--dry-run").output().unwrap());
    assert!(!f.s.control().exists());
    assert_eq!(snapshot(&f.s.root), before);
}

#[test]
fn default_source_is_copied_with_rebased_state_and_repeat_migration_is_inert() {
    let f = Fixture::new(false, true);
    let unknown_state = b"unrelated namespace must remain byte-for-byte\0";
    private_write(&f.old.join("state/unrelated.json"), unknown_state);
    private_write(&f.old.join("backups/apply-old/retained.bin"), b"backup\0");
    private_write(&f.old.join("cache/retained.bin"), b"cache\0");
    f.write_scripts(&["unrelated:once:fixture".into()]);
    let old = snapshot(&f.old);
    let targets = snapshot(&f.destination);
    f.migrate();
    let config: Value =
        toml::from_str(&fs::read_to_string(f.s.control().join("config.toml")).unwrap()).unwrap();
    assert_eq!(
        config["source"].as_str().unwrap(),
        f.new_source().to_str().unwrap()
    );
    assert_eq!(
        config["destination"].as_str().unwrap(),
        f.destination.to_str().unwrap()
    );
    assert_eq!(config["data"], f.config["data"]);
    assert!(f.new_source().join(".git").is_dir());
    assert_eq!(f.new_state()["entries"][".seed"]["hash"], digest(b"seed\n"));
    assert_eq!(
        fs::read(f.s.control().join("state/unrelated.json")).unwrap(),
        unknown_state
    );
    assert_eq!(
        fs::read(f.s.control().join("backups/apply-old/retained.bin")).unwrap(),
        b"backup\0"
    );
    assert_eq!(
        fs::read(f.s.control().join("cache/retained.bin")).unwrap(),
        b"cache\0"
    );
    assert!(f
        .migrated_script_keys()
        .contains(&"unrelated:once:fixture".into()));
    assert_eq!(snapshot(&f.old), old);
    assert_eq!(snapshot(&f.destination), targets);
    let migrated = snapshot(&f.s.control());
    assert!(f.migrate().contains("Already migrated"));
    assert_eq!(snapshot(&f.s.control()), migrated);
    success(f.s.cli(&["verify"]));
}

#[test]
fn custom_source_destination_and_external_identity_paths_are_preserved() {
    let mut f = Fixture::new(true, true);
    let identity = f.s.home.join(".config/xyrev/keys/external.agekey");
    private_write(&identity, b"external identity fixture, never imported");
    let password = f.s.root.join("external password file");
    private_write(&password, b"fixture password, never read");
    let keyring = f.s.home.join(".gnupg/unrelated-user-data");
    private_write(&keyring, b"unrelated keyring fixture");
    f.config["data"]["gpg_identity"] = json!(identity);
    f.config["data"]["gpg_passphrase_file"] = json!(password);
    f.config["data"]["user_note"] = json!("xyrev in an arbitrary user value stays unchanged");
    f.config["encryption"]["identities"] = json!([identity]);
    f.save_config();
    let config = f.config.clone();
    let tracked = git(&f.source, &["ls-files"]);
    let source = snapshot(&f.source);
    let external = fs::read(&identity).unwrap();
    let targets = snapshot(&f.destination);
    f.migrate();
    let migrated: Value =
        toml::from_str(&fs::read_to_string(f.s.control().join("config.toml")).unwrap()).unwrap();
    assert_eq!(migrated, config);
    assert_eq!(git(&f.source, &["ls-files"]), tracked);
    assert_eq!(snapshot(&f.source), source);
    assert_eq!(fs::read(identity).unwrap(), external);
    assert_eq!(fs::read(password).unwrap(), b"fixture password, never read");
    assert_eq!(fs::read(keyring).unwrap(), b"unrelated keyring fixture");
    assert_eq!(snapshot(&f.destination), targets);
    assert!(!f.s.control().join("source").exists());
    success(f.s.cli(&["verify"]));
}

#[test]
fn existing_new_state_and_invalid_legacy_config_stop_without_merging_or_cleanup() {
    let f = Fixture::new(false, true);
    private_write(&f.s.control().join("keep"), b"existing stool control data");
    let before = snapshot(&f.s.root);
    failure(
        f.command().output().unwrap(),
        "Migration target already exists",
    );
    assert_eq!(snapshot(&f.s.root), before);
    fs::remove_dir_all(f.s.home.join(".stool")).unwrap();
    private_write(&f.old.join("config.toml"), b"invalid = [");
    let old = snapshot(&f.old);
    assert!(!f.command().output().unwrap().status.success());
    assert!(!f.s.control().exists());
    assert_eq!(snapshot(&f.old), old);
    f.save_config();
    write(&f.source.join(".xyrevignore"), "unmigrated protocol\n");
    let old = snapshot(&f.old);
    failure(f.command().output().unwrap(), "Update the source");
    assert!(!f.s.control().exists());
    assert_eq!(snapshot(&f.old), old);
}

#[test]
fn missing_legacy_configuration_and_overrides_do_not_initialize_any_state() {
    let sandbox = Sandbox::new(&[]);
    let before = snapshot(&sandbox.root);
    failure(sandbox.cli(&["migrate"]), "No legacy configuration");
    assert_eq!(snapshot(&sandbox.root), before);
    let f = Fixture::new(false, true);
    let before = snapshot(&f.s.root);
    for args in [
        vec!["--source", f.source.to_str().unwrap()],
        vec!["--destination", f.destination.to_str().unwrap()],
        vec!["--data", "name=override"],
    ] {
        failure(f.command().args(args).output().unwrap(), "do not supply");
        assert!(!f.s.control().exists());
        assert_eq!(snapshot(&f.s.root), before);
    }
}

fn rendered_script(source: &Path, label: &str, legacy: bool) -> Vec<u8> {
    let prefix = if legacy { "XYREV" } else { "STOOL" };
    format!(
        "#!/bin/sh\n# source: {}\nprintf '%s\\n' \"${prefix}_SOURCE_DIR\" > \"${prefix}_DEST_DIR/{label}.log\"\n",
        source.display()
    ).into_bytes()
}

#[test]
fn only_exactly_equivalent_scripts_inherit_success_without_running_providers() {
    let f = Fixture::new(false, true);
    let once_name = "run_once_before_equivalent.sh.tmpl";
    let change_name = "run_onchange_before_equivalent.sh.tmpl";
    let changed_name = "run_onchange_before_changed.sh.tmpl";
    for (name, label) in [
        (once_name, "once"),
        (change_name, "change"),
        (changed_name, "new-behavior"),
    ] {
        let text = String::from_utf8(rendered_script(
            Path::new("{{ stool.source_dir }}"),
            label,
            false,
        ))
        .unwrap();
        write(&f.source.join(name), &text);
    }
    write(
        &f.source.join("run_once_before_secret.sh.tmpl"),
        "#!/bin/sh\nprintf '%s' \"{{ pass('migration-secret') }}\"\n",
    );
    write(
        &f.source.join("run_once_before_env.sh.tmpl"),
        "#!/bin/sh\nprintf '%s' \"{{ env('STOOL_MIGRATION_TOKEN') }}\"\n",
    );
    let old = f.old_namespace();
    let once_hash = digest(&rendered_script(&f.source, "once", true));
    let change_hash = digest(&rendered_script(&f.source, "change", true));
    let old_keys = vec![
        format!("{old}:once:{once_hash}"),
        format!("{old}:onchange:{change_name}:{change_hash}"),
        format!(
            "{old}:onchange:{changed_name}:{}",
            digest(&rendered_script(&f.source, "old-behavior", true))
        ),
        format!("{old}:once:{}", digest(b"previous secret render")),
        format!("{old}:once:{}", digest(b"previous environment render")),
        "unrelated:once:preserved".into(),
    ];
    f.write_scripts(&old_keys);
    let bin = f.s.root.join("provider tools");
    fs::create_dir_all(&bin).unwrap();
    fs::copy(
        fake_gpg(),
        bin.join(if cfg!(windows) { "pass.exe" } else { "pass" }),
    )
    .unwrap();
    let path = std::env::join_paths(
        std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let output = success(
        f.command()
            .env("PATH", path)
            .env("STOOL_TEST_GPG_ROOT", &f.s.root)
            .env(
                "STOOL_MIGRATION_TOKEN",
                "must not consult runtime environment",
            )
            .output()
            .unwrap(),
    );
    assert!(output.contains("not inherited"));
    assert!(
        !f.s.root.join("events").exists(),
        "migration ran a secret provider"
    );
    let keys = f.migrated_script_keys();
    for key in old_keys {
        assert!(keys.contains(&key));
    }
    let new = f.new_namespace();
    assert!(
        keys.contains(&format!(
            "{new}:once:{}",
            digest(&rendered_script(&f.new_source(), "once", false))
        )),
        "{output}\nMigrated keys: {keys:?}"
    );
    assert!(keys.contains(&format!(
        "{new}:onchange:{change_name}:{}",
        digest(&rendered_script(&f.new_source(), "change", false))
    )));
    assert_eq!(
        keys.iter()
            .filter(|key| key.starts_with(&format!("{new}:")))
            .count(),
        2
    );
    assert!(!f.destination.join("once.log").exists());
    assert!(!f.destination.join("change.log").exists());
}

#[derive(Serialize)]
struct GpgSettings<'a> {
    enabled: bool,
    directory: &'a str,
    program: Option<&'a str>,
    identity: Option<&'a str>,
    passphrase_file: Option<&'a str>,
}

fn restore_digest(source: &Path, legacy: bool, metadata: &[u8]) -> String {
    let directory = if legacy {
        ".xyrevtemplates/gpg"
    } else {
        ".stooltemplates/gpg"
    };
    let settings = GpgSettings {
        enabled: true,
        directory,
        program: None,
        identity: None,
        passphrase_file: None,
    };
    let mut bytes = serde_json::to_vec(&settings).unwrap();
    let relative: PathBuf = Path::new(directory).components().collect();
    bytes.extend_from_slice(source.join(relative).to_string_lossy().as_bytes());
    bytes.extend_from_slice(metadata);
    digest(&bytes)
}

#[test]
fn gpg_success_is_rebased_only_when_its_old_digest_exactly_matches() {
    for matching in [true, false] {
        let f = Fixture::new(false, false);
        let original = f.write_bundle();
        let old_digest = restore_digest(&f.source, true, &original[3]);
        f.write_state(Some(if matching {
            &old_digest
        } else {
            "unrelated old successful plan"
        }));
        let keyring = f.s.home.join(".gnupg/unrelated-user-data");
        private_write(&keyring, b"unrelated keyring fixture");
        let old = snapshot(&f.old);
        let output = f.migrate();
        let state = f.new_state();
        if matching {
            assert_eq!(
                state["gpg_restore"],
                restore_digest(&f.new_source(), false, &original[3])
            );
            success(f.s.cli(&["verify"]));
        } else {
            assert!(state.get("gpg_restore").map_or(true, Value::is_null));
            assert!(output.contains("not inherited"));
            failure(f.s.cli(&["verify"]), "verification failed");
        }
        for (name, expected) in BACKUP_FILES.iter().zip(original) {
            assert_eq!(
                fs::read(f.new_source().join(".stooltemplates/gpg").join(name)).unwrap(),
                expected
            );
        }
        assert_eq!(snapshot(&f.old), old);
        assert_eq!(fs::read(keyring).unwrap(), b"unrelated keyring fixture");
    }
}

#[cfg(unix)]
#[test]
fn symlinked_legacy_state_is_rejected_without_following_or_changing_it() {
    for name in ["known-state", "scripts.redb"] {
        let f = Fixture::new(false, true);
        let external = f.s.root.join("external untrusted configuration");
        private_write(&external, b"external fixture must remain intact");
        let link = if name == "known-state" {
            let path = f
                .old
                .join("state")
                .join(format!("{}.json", f.old_namespace()));
            fs::remove_file(&path).unwrap();
            path
        } else {
            f.old.join("state/scripts.redb")
        };
        std::os::unix::fs::symlink(&external, &link).unwrap();
        let before = snapshot(&f.old);
        assert!(!f.command().output().unwrap().status.success());
        assert!(!f.s.control().exists());
        assert_eq!(snapshot(&f.old), before);
        assert_eq!(
            fs::read(external).unwrap(),
            b"external fixture must remain intact"
        );
    }
}

#[test]
fn built_in_markers_are_preserved_for_internal_and_external_sources_after_dry_run() {
    for external in [false, true] {
        let f = Fixture::new(external, true);
        private_write(&f.source.join(".git/xyrev-builtin"), b"1\n");
        let tracked = git(&f.source, &["ls-files"]);
        let head = git(&f.source, &["rev-parse", "HEAD"]);
        let before = snapshot(&f.s.root);
        success(f.command().arg("--dry-run").output().unwrap());
        assert_eq!(snapshot(&f.s.root), before);
        assert!(!f.source.join(".git/stool-builtin").exists());
        f.migrate();
        assert_eq!(
            fs::read(f.new_source().join(".git/stool-builtin")).unwrap(),
            b"1\n"
        );
        assert_eq!(
            fs::read(f.source.join(".git/xyrev-builtin")).unwrap(),
            b"1\n"
        );
        assert_eq!(git(&f.new_source(), &["ls-files"]), tracked);
        assert_eq!(git(&f.new_source(), &["rev-parse", "HEAD"]), head);
        if !external {
            assert!(!f.source.join(".git/stool-builtin").exists());
        }
    }
}

#[test]
fn historical_namespace_conflicts_and_copy_failures_preserve_retryable_old_state() {
    let f = Fixture::new(false, true);
    let historical = f
        .old
        .join("state")
        .join(format!("{}.json", f.new_namespace()));
    private_write(&historical, b"{\"entries\":{}}");
    let before = snapshot(&f.old);
    failure(
        f.command().output().unwrap(),
        "New namespace already contains historical file state",
    );
    assert!(!f.s.control().exists());
    assert_eq!(snapshot(&f.old), before);
    fs::remove_file(&historical).unwrap();
    f.write_scripts(&[format!("{}:once:historical", f.new_namespace())]);
    let before = snapshot(&f.old);
    failure(
        f.command().output().unwrap(),
        "New namespace already contains historical script state",
    );
    assert!(!f.s.control().exists());
    assert_eq!(snapshot(&f.old), before);
    fs::remove_file(f.old.join("state/scripts.redb")).unwrap();
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let fifo = f.old.join("cache/non-copyable-fifo");
        fs::create_dir_all(fifo.parent().unwrap()).unwrap();
        let cpath = CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(cpath.as_ptr(), 0o600) }, 0);
        let before = snapshot(&f.old);
        failure(f.command().output().unwrap(), "Cannot migrate special file");
        assert!(!f.s.control().exists());
        assert_eq!(snapshot(&f.old), before);
        fs::remove_file(fifo).unwrap();
    }
    f.migrate();
    success(f.s.cli(&["verify"]));
}

#[test]
fn unchanged_external_source_keeps_only_the_latest_onchange_success_for_its_namespace() {
    let f = Fixture::new(true, true);
    let name = "run_onchange_before_equivalent.sh.tmpl";
    let text = String::from_utf8(rendered_script(
        Path::new("{{ stool.source_dir }}"),
        "current",
        false,
    ))
    .unwrap();
    write(&f.source.join(name), &text);
    let ns = f.old_namespace();
    assert_eq!(ns, f.new_namespace());
    let legacy = format!(
        "{ns}:onchange:{name}:{}",
        digest(&rendered_script(&f.source, "current", true))
    );
    let stale = format!(
        "{ns}:onchange:{name}:{}",
        digest(&rendered_script(&f.source, "older", false))
    );
    f.write_scripts(&[
        legacy.clone(),
        stale.clone(),
        "unrelated:once:preserved".into(),
    ]);
    f.migrate();
    let keys = f.migrated_script_keys();
    let current = format!(
        "{ns}:onchange:{name}:{}",
        digest(&rendered_script(&f.source, "current", false))
    );
    assert!(keys.contains(&current));
    assert!(!keys.contains(&legacy));
    assert!(!keys.contains(&stale));
    assert!(keys.contains(&"unrelated:once:preserved".into()));
    assert_eq!(
        keys.iter()
            .filter(|key| key.starts_with(&format!("{ns}:onchange:{name}:")))
            .count(),
        1
    );
}

#[test]
fn renamed_application_paths_do_not_inherit_old_once_success_and_are_created_on_apply() {
    let f = Fixture::new(false, false);
    let old_directory = f.s.home.join(".config/xyrev");
    let new_directory = f.s.home.join(".config/stool");
    fs::create_dir_all(&old_directory).unwrap();
    #[cfg(windows)]
    let (name, script) = (
        "run_once_before_application_path.ps1",
        "New-Item -ItemType Directory -Force -Path \"$env:STOOL_HOME/.config/stool\" | Out-Null\n",
    );
    #[cfg(not(windows))]
    let (name, script) = (
        "run_once_before_application_path.sh",
        "#!/bin/sh\n/bin/mkdir -p \"$STOOL_HOME/.config/stool\"\n",
    );
    write(&f.source.join(name), script);
    let old_rendered = script
        .replace("STOOL_HOME", "XYREV_HOME")
        .replace(".config/stool", ".config/xyrev");
    let old_key = format!(
        "{}:once:{}",
        f.old_namespace(),
        digest(old_rendered.as_bytes())
    );
    f.write_scripts(std::slice::from_ref(&old_key));
    let output = f.migrate();
    assert!(output.contains("not inherited"), "{output}");
    assert!(old_directory.is_dir());
    assert!(!new_directory.exists());
    let new_key = format!("{}:once:{}", f.new_namespace(), digest(script.as_bytes()));
    let records = f.migrated_script_keys();
    assert!(records.contains(&old_key));
    assert!(!records.contains(&new_key));
    success(f.s.cli(&["apply"]));
    assert!(new_directory.is_dir());
    assert!(old_directory.is_dir());
    assert!(f.migrated_script_keys().contains(&new_key));
    assert!(!success(f.s.cli(&["apply"])).contains("Running run_once_before_application_path"));
    success(f.s.cli(&["verify"]));
}
