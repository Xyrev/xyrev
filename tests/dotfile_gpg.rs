mod common;
use common::gpg::{fake_gpg, private_write};
use common::*;
use sha2::{Digest, Sha256};

const PERSONAL: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const WORK: &str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";
const OTHER: &str = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC";
const SUBKEY: &str = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD";
const WORK_SUBKEY: &str = "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE";
const PASSWORD: &str = "  fixture's password $(unexecuted)  ";
const FILES: [&str; 4] = [
    "public.asc",
    "secret.asc.age",
    "ownertrust.txt.age",
    "backup-metadata.sh",
];

struct Fixture {
    s: Sandbox,
    password: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let s = Sandbox::new(&[("dot_seed", "seed\n")]);
        let password = s.root.join("password file's private.txt");
        private_write(&password, format!("{PASSWORD}\r\n").as_bytes());
        success(
            s.command()
                .arg("--source")
                .arg(&s.source)
                .arg("--destination")
                .arg(&s.home)
                .arg("init")
                .arg(s.url())
                .output()
                .unwrap(),
        );
        Self { s, password }
    }
    fn command(&self) -> Command {
        let mut command = self.s.command();
        command.env("STOOL_TEST_GPG_ROOT", &self.s.root);
        command
    }
    fn backup_to(&self, keys: &[&str], output: Option<&Path>) -> Output {
        let mut command = self.command();
        command
            .args(["gpg", "backup", "--gpg-program"])
            .arg(fake_gpg())
            .arg("--passphrase-file")
            .arg(&self.password);
        for key in keys {
            command.arg("--fingerprint").arg(key);
        }
        if let Some(output) = output {
            command.arg("--output").arg(output);
        }
        command.output().unwrap()
    }
    fn backup(&self, keys: &[&str]) {
        success(self.backup_to(keys, None));
    }
    fn restore(&self) -> Output {
        self.command()
            .args(["gpg", "restore", "--gpg-program"])
            .arg(fake_gpg())
            .arg("--passphrase-file")
            .arg(&self.password)
            .output()
            .unwrap()
    }
    fn directory(&self) -> PathBuf {
        self.s.source.join(".stooltemplates/gpg")
    }
    fn files(&self) -> Vec<Vec<u8>> {
        FILES
            .iter()
            .map(|name| fs::read(self.directory().join(name)).unwrap())
            .collect()
    }
    fn backup_snapshot(&self) -> std::collections::BTreeMap<PathBuf, String> {
        let directory = self.directory();
        let mut contents = snapshot(&directory);
        // Failed staging can update the containing directory's mtime while the
        // existing backup files, permissions, and modification times stay intact.
        contents.remove(&directory);
        contents
    }
    fn imports(&self) -> String {
        fs::read_to_string(self.s.root.join("imports")).unwrap_or_default()
    }
    fn declaration_at(&self, root: &Path) {
        write(
            &root.join(".stoolgpg.toml"),
            &format!(
                "enabled=true\ndirectory='.stooltemplates/gpg'\nprogram={}\npassphrase_file={}\n",
                serde_json::to_string(fake_gpg()).unwrap(),
                serde_json::to_string(&self.password).unwrap()
            ),
        );
    }
    fn declare(&self) {
        self.declaration_at(&self.s.source);
    }
    fn replace(&self, name: &str, bytes: &[u8]) {
        let path = self.directory().join(name);
        let before = format!("{:x}", Sha256::digest(fs::read(&path).unwrap()));
        let after = format!("{:x}", Sha256::digest(bytes));
        fs::write(path, bytes).unwrap();
        let metadata = self.directory().join("backup-metadata.sh");
        let text = fs::read_to_string(&metadata)
            .unwrap()
            .replace(&before, &after);
        fs::write(metadata, text).unwrap();
    }
    fn assert_failed(&self, out: Output) {
        assert!(
            !out.status.success(),
            "Unexpected success: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        for stream in [&out.stdout, &out.stderr] {
            assert!(!String::from_utf8_lossy(stream).contains(PASSWORD));
            assert!(!String::from_utf8_lossy(stream).contains("STOOL-FAKE-SECRET"));
        }
    }
}

#[test]
fn backup_and_restore_use_gpg_from_path_by_default() {
    let f = Fixture::new();
    let bin = f.s.root.join("GPG on PATH with spaces");
    fs::create_dir(&bin).unwrap();
    fs::copy(
        fake_gpg(),
        bin.join(if cfg!(windows) { "gpg.exe" } else { "gpg" }),
    )
    .unwrap();
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(paths).unwrap();
    for operation in ["backup", "restore"] {
        success(
            f.command()
                .env("PATH", &path)
                .args(["gpg", operation, "--passphrase-file"])
                .arg(&f.password)
                .output()
                .unwrap(),
        );
    }
    let metadata = fs::read_to_string(f.directory().join("backup-metadata.sh")).unwrap();
    assert!(metadata.contains(&format!("gpg_backup_fingerprints='{PERSONAL} {WORK}'")));
    assert_eq!(f.imports().lines().count(), 3);
}

#[test]
fn default_backup_discovers_complete_secret_groups_and_only_their_public_keys_and_trust() {
    let f = Fixture::new();
    f.backup(&[]);
    let metadata = fs::read_to_string(f.directory().join("backup-metadata.sh")).unwrap();
    assert!(metadata.contains(&format!("gpg_backup_fingerprints='{PERSONAL} {WORK}'")));
    assert!(!metadata.contains(OTHER));
    assert_eq!(
        fs::read_to_string(f.directory().join("public.asc")).unwrap(),
        format!("STOOL-FAKE-PUBLIC\n{PERSONAL}\nsub:{SUBKEY}\n{WORK}\nsub:{WORK_SUBKEY}\n")
    );
    assert_eq!(
        decrypt_age_fixture(PASSWORD, &f.files()[1]),
        format!("STOOL-FAKE-SECRET\n{PERSONAL}\nsub:{SUBKEY}\n{WORK}\nsub:{WORK_SUBKEY}\n")
            .as_bytes()
    );
    assert_eq!(
        decrypt_age_fixture(PASSWORD, &f.files()[2]),
        format!("{PERSONAL}:6:\n{WORK}:6:\n").as_bytes()
    );
    let events = fs::read_to_string(f.s.root.join("events")).unwrap();
    assert!(events.contains("--list-secret-keys"));
    success(f.restore());
    assert_eq!(f.imports().lines().count(), 3);
}

#[test]
fn explicit_filter_excludes_other_secret_groups_and_rejects_public_only_keys() {
    let f = Fixture::new();
    f.backup(&[PERSONAL]);
    let metadata = fs::read_to_string(f.directory().join("backup-metadata.sh")).unwrap();
    assert!(metadata.contains(&format!("gpg_backup_fingerprints='{PERSONAL}'")));
    assert!(!metadata.contains(WORK));
    let before = f.backup_snapshot();
    f.assert_failed(f.backup_to(&[PERSONAL, OTHER], None));
    assert_eq!(f.backup_snapshot(), before);
    // An omitted filter discovers the additional key and extends the backup.
    f.backup(&[]);
    let metadata = fs::read_to_string(f.directory().join("backup-metadata.sh")).unwrap();
    assert!(metadata.contains(&format!("gpg_backup_fingerprints='{PERSONAL} {WORK}'")));
}

#[test]
fn empty_discovery_fails_before_password_input_and_preserves_existing_files() {
    let f = Fixture::new();
    fs::write(f.s.root.join("no-secret"), "").unwrap();
    for existing in [false, true] {
        if existing {
            fs::remove_file(f.s.root.join("no-secret")).unwrap();
            f.backup(&[]);
            fs::write(f.s.root.join("no-secret"), "").unwrap();
        }
        let before = f.backup_snapshot();
        fs::remove_file(f.s.root.join("events")).ok();
        let output = f
            .command()
            .args(["gpg", "backup", "--gpg-program"])
            .arg(fake_gpg())
            .output()
            .unwrap();
        let diagnostic = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
        assert!(diagnostic.contains("exportable"), "{diagnostic}");
        assert!(!diagnostic.contains("gpg backup password"), "{diagnostic}");
        f.assert_failed(output);
        assert_eq!(f.backup_snapshot(), before);
        let events = fs::read_to_string(f.s.root.join("events")).unwrap();
        assert!(!events.contains("--export"));
    }
}

#[test]
fn default_discovery_cannot_drop_a_key_already_in_a_backup() {
    let f = Fixture::new();
    f.backup(&[]);
    let before = f.backup_snapshot();
    fs::write(f.s.root.join("only-personal"), "").unwrap();
    f.assert_failed(f.backup_to(&[], None));
    assert_eq!(f.backup_snapshot(), before);
}

#[test]
fn discovery_export_and_missing_components_fail_without_replacing_a_complete_backup() {
    let f = Fixture::new();
    f.backup(&[]);
    let before = f.backup_snapshot();
    for marker in [
        "fail-discovery",
        "missing-primary-fingerprint",
        "missing-subkey-fingerprint",
        "fail-secret-export",
        "fail-public-export",
        "fail-trust-export",
        "partial-export",
        "partial-secret-primary",
        "partial-secret-subkey",
        "partial-public-subkey",
        "stub-secret-export",
        "inventory-changed",
    ] {
        fs::write(f.s.root.join(marker), "").unwrap();
        let output = f.backup_to(&[], None);
        assert!(!output.status.success(), "{marker} unexpectedly succeeded");
        f.assert_failed(output);
        assert_eq!(
            f.backup_snapshot(),
            before,
            "{marker} changed the existing backup"
        );
        fs::remove_file(f.s.root.join(marker)).unwrap();
    }
}

#[test]
fn partly_available_secret_groups_keep_exportable_primary_or_subkeys_and_warn() {
    for marker in [
        "stub-primary",
        "stub-subkey",
        "token-primary",
        "token-subkey",
    ] {
        let f = Fixture::new();
        fs::write(f.s.root.join(marker), "").unwrap();
        let output = f.backup_to(&[], None);
        let diagnostic = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
        assert!(
            diagnostic.contains(&WORK.to_ascii_lowercase()),
            "{marker}: {diagnostic}"
        );
        assert!(
            diagnostic.contains("cannot be exported"),
            "{marker}: {diagnostic}"
        );
        success(output);
        let metadata = fs::read_to_string(f.directory().join("backup-metadata.sh")).unwrap();
        assert!(metadata.contains(&format!("gpg_backup_fingerprints='{PERSONAL} {WORK}'")));
        let secret = decrypt_age_fixture(PASSWORD, &f.files()[1]);
        assert!(
            String::from_utf8_lossy(&secret).contains(WORK_SUBKEY),
            "{marker}"
        );
    }
}

#[test]
fn wholly_unavailable_groups_are_reported_skipped_or_rejected_when_explicit() {
    for marker in ["stub-only-work", "token-only-work"] {
        let f = Fixture::new();
        fs::write(f.s.root.join(marker), "").unwrap();
        let output = f.backup_to(&[], None);
        let diagnostic = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
        assert!(
            diagnostic.contains(&WORK.to_ascii_lowercase()),
            "{diagnostic}"
        );
        assert!(diagnostic.contains("skip"), "{diagnostic}");
        success(output);
        let metadata = fs::read_to_string(f.directory().join("backup-metadata.sh")).unwrap();
        assert!(metadata.contains(&format!("gpg_backup_fingerprints='{PERSONAL}'")));
        assert!(!metadata.contains(WORK));
        let before = f.backup_snapshot();
        f.assert_failed(f.backup_to(&[PERSONAL, WORK], None));
        assert_eq!(f.backup_snapshot(), before);
    }
    let f = Fixture::new();
    fs::write(f.s.root.join("all-stubs"), "").unwrap();
    let output = f
        .command()
        .args(["gpg", "backup", "--gpg-program"])
        .arg(fake_gpg())
        .output()
        .unwrap();
    let diagnostic = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
    assert!(diagnostic.contains("exportable"), "{diagnostic}");
    assert!(!diagnostic.contains("gpg backup password"), "{diagnostic}");
    f.assert_failed(output);
    assert!(!f.directory().exists());
}

#[test]
fn an_unavailable_key_already_in_a_backup_cannot_be_silently_skipped() {
    let f = Fixture::new();
    f.backup(&[]);
    let before = f.backup_snapshot();
    for marker in ["stub-only-work", "token-only-work", "all-stubs"] {
        fs::write(f.s.root.join(marker), "").unwrap();
        f.assert_failed(f.backup_to(&[], None));
        assert_eq!(f.backup_snapshot(), before, "{marker}");
        fs::remove_file(f.s.root.join(marker)).unwrap();
    }
}

#[test]
fn existing_private_primary_or_subkeys_cannot_be_lost_even_when_primary_set_is_unchanged() {
    let f = Fixture::new();
    f.backup(&[]);
    let before = f.backup_snapshot();
    for marker in [
        "stub-primary",
        "stub-subkey",
        "token-primary",
        "token-subkey",
        "dropped-subkey",
    ] {
        fs::write(f.s.root.join(marker), "").unwrap();
        f.assert_failed(f.backup_to(&[], None));
        assert_eq!(f.backup_snapshot(), before, "{marker}");
        fs::remove_file(f.s.root.join(marker)).unwrap();
    }
    // This is a coherent current keyring/export, so a fresh backup can contain it.
    fs::write(f.s.root.join("dropped-subkey"), "").unwrap();
    let fresh = f.s.root.join("fresh backup without the old work subkey");
    success(f.backup_to(&[], Some(&fresh)));
    let public = fs::read_to_string(fresh.join("public.asc")).unwrap();
    assert!(public.contains(WORK));
    assert!(!public.contains(WORK_SUBKEY));
    fs::remove_file(f.s.root.join("dropped-subkey")).unwrap();
    // Existing component evidence is trusted only when its manifest hash agrees.
    let public_path = f.directory().join("public.asc");
    let mut corrupted = fs::read(&public_path).unwrap();
    corrupted.extend_from_slice(b"corrupted public fixture\n");
    fs::write(public_path, corrupted).unwrap();
    let corrupted_before = f.backup_snapshot();
    fs::remove_file(f.s.root.join("events")).unwrap();
    let output = f
        .command()
        .args(["gpg", "backup", "--gpg-program"])
        .arg(fake_gpg())
        .output()
        .unwrap();
    let diagnostic = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
    assert!(diagnostic.contains("checksum"), "{diagnostic}");
    assert!(!diagnostic.contains("gpg backup password"), "{diagnostic}");
    f.assert_failed(output);
    assert_eq!(f.backup_snapshot(), corrupted_before);
    let events = fs::read_to_string(f.s.root.join("events")).unwrap();
    assert!(!events.contains("--export"));
}

#[test]
fn native_password_backup_and_explicit_restore_preserve_all_selected_keys() {
    let f = Fixture::new();
    f.backup(&[WORK, PERSONAL, &WORK.to_ascii_lowercase()]);
    let metadata = fs::read_to_string(f.directory().join("backup-metadata.sh")).unwrap();
    assert!(metadata.contains(&format!("gpg_backup_fingerprints='{PERSONAL} {WORK}'")));
    let private = decrypt_age_fixture(
        PASSWORD,
        &fs::read(f.directory().join("secret.asc.age")).unwrap(),
    );
    assert_eq!(
        String::from_utf8(private).unwrap(),
        format!("STOOL-FAKE-SECRET\n{PERSONAL}\nsub:{SUBKEY}\n{WORK}\nsub:{WORK_SUBKEY}\n")
    );
    let trust = decrypt_age_fixture(
        PASSWORD,
        &fs::read(f.directory().join("ownertrust.txt.age")).unwrap(),
    );
    assert_eq!(
        String::from_utf8(trust).unwrap(),
        format!("{PERSONAL}:6:\n{WORK}:6:\n")
    );
    assert!(!metadata.contains(OTHER));
    f.declare();
    // Explicit restore and apply share successful native restore state, while
    // an explicit user-requested restore always remains runnable.
    success(f.command().args(["gpg", "restore"]).output().unwrap());
    assert_eq!(f.imports().lines().count(), 3);
    success(f.command().arg("apply").output().unwrap());
    assert_eq!(f.imports().lines().count(), 3);
    success(f.command().args(["gpg", "restore"]).output().unwrap());
    assert_eq!(f.imports().lines().count(), 6);
    for bytes in f.files() {
        assert!(!String::from_utf8_lossy(&bytes).contains(PASSWORD));
        assert!(!String::from_utf8_lossy(&bytes).contains("STOOL-FAKE-SECRET"));
    }
}

#[test]
fn preview_does_not_call_gpg_and_native_restore_runs_between_files_and_after_hooks() {
    let f = Fixture::new();
    f.backup(&[PERSONAL, WORK]);
    f.declare();
    for phase in ["before", "after"] {
        write(
            &f.s.source.join(format!("run_once_{phase}_order")),
            &format!(
                "#!{} --stool-hook {phase}\n",
                serde_json::to_string(fake_gpg()).unwrap()
            ),
        );
    }
    fs::remove_file(f.s.root.join("events")).unwrap();
    let before = snapshot(&f.s.control());
    let source = snapshot(&f.s.source);
    for args in [vec!["diff"], vec!["status"], vec!["apply", "--dry-run"]] {
        let out = success(f.command().args(args).output().unwrap());
        assert!(!out.contains("STOOL-FAKE-SECRET"));
        assert!(!out.contains(PASSWORD));
    }
    f.assert_failed(f.command().arg("verify").output().unwrap());
    let archive = f.s.root.join("preview.tar");
    success(
        f.command()
            .args(["archive", "--output"])
            .arg(&archive)
            .output()
            .unwrap(),
    );
    let mut archive = tar::Archive::new(fs::File::open(archive).unwrap());
    for entry in archive.entries().unwrap() {
        let entry = entry.unwrap();
        let name = entry.path().unwrap().to_string_lossy().into_owned();
        assert!(!name.contains(".stool"));
        assert!(!FILES.iter().any(|file| name.contains(file)));
    }
    assert_eq!(snapshot(&f.s.control()), before);
    assert_eq!(snapshot(&f.s.source), source);
    assert!(!f.s.root.join("events").exists());
    success(f.command().arg("apply").output().unwrap());
    assert!(f
        .imports()
        .lines()
        .all(|line| line.ends_with("deployed=true")));
    let events = fs::read_to_string(f.s.root.join("events")).unwrap();
    assert!(events.find("hook:before").unwrap() < events.find("import:public").unwrap());
    assert!(events.find("import:trust").unwrap() < events.find("hook:after").unwrap());
    let imported = f.imports();
    success(f.command().arg("apply").output().unwrap());
    success(f.command().arg("verify").output().unwrap());
    assert_eq!(f.imports(), imported);
}

#[test]
fn failed_import_is_retried_and_only_success_marks_the_backup_restored() {
    let f = Fixture::new();
    f.backup(&[PERSONAL, WORK]);
    f.declare();
    fs::write(f.s.root.join("fail-import"), "").unwrap();
    f.assert_failed(f.command().arg("apply").output().unwrap());
    assert!(f.imports().is_empty());
    f.assert_failed(f.command().arg("verify").output().unwrap());
    fs::remove_file(f.s.root.join("fail-import")).unwrap();
    success(f.command().arg("apply").output().unwrap());
    assert_eq!(f.imports().lines().count(), 3);
    let imported = f.imports();
    success(f.command().arg("apply").output().unwrap());
    assert_eq!(f.imports(), imported);
}

#[test]
fn git_update_adds_the_work_key_and_imports_the_changed_backup_once() {
    let f = Fixture::new();
    let output = f.s.work.join(".stooltemplates/gpg");
    success(f.backup_to(&[PERSONAL], Some(&output)));
    f.declaration_at(&f.s.work);
    f.s.push();
    success(f.command().arg("update").output().unwrap());
    assert_eq!(f.imports().lines().count(), 3);
    success(f.backup_to(&[WORK, PERSONAL], Some(&output)));
    f.s.push();
    success(f.command().arg("update").output().unwrap());
    assert_eq!(f.imports().lines().count(), 6);
    let imported = f.imports();
    success(f.command().arg("apply").output().unwrap());
    assert_eq!(f.imports(), imported);
}

#[test]
fn incomplete_selection_and_partial_exports_never_replace_the_previous_backup() {
    let f = Fixture::new();
    f.backup(&[PERSONAL, WORK]);
    let before = f.files();
    for keys in [
        vec![PERSONAL],
        vec![WORK],
        vec![PERSONAL, WORK, "short"],
        vec![PERSONAL, WORK, SUBKEY],
    ] {
        f.assert_failed(f.backup_to(&keys, None));
        assert_eq!(f.files(), before);
    }
    fs::write(f.s.root.join("partial-export"), "").unwrap();
    f.assert_failed(f.backup_to(&[PERSONAL, WORK], None));
    assert_eq!(f.files(), before);
}

#[test]
fn wrong_password_corruption_and_semantically_wrong_bundles_stop_before_import() {
    let f = Fixture::new();
    f.backup(&[PERSONAL, WORK]);
    let original = f.files();
    private_write(&f.password, b"different password");
    f.assert_failed(f.restore());
    assert!(f.imports().is_empty());
    private_write(&f.password, format!("{PASSWORD}\r\n").as_bytes());
    let mut corrupted = original[1].clone();
    corrupted.extend_from_slice(b"corruption");
    fs::write(f.directory().join("secret.asc.age"), corrupted).unwrap();
    f.assert_failed(f.restore());
    assert!(f.imports().is_empty());
    let cases = [
        ("secret.asc.age", format!("STOOL-FAKE-SECRET\n{PERSONAL}\n")),
        (
            "secret.asc.age",
            format!("STOOL-FAKE-SECRET\n{PERSONAL}\n{WORK}\n{OTHER}\n"),
        ),
        (
            "secret.asc.age",
            format!("STOOL-FAKE-PUBLIC\n{PERSONAL}\n{WORK}\n"),
        ),
        (
            "public.asc",
            format!("STOOL-FAKE-PUBLIC\n{PERSONAL}\n{OTHER}\n"),
        ),
        ("ownertrust.txt.age", format!("{PERSONAL}:6:\n{OTHER}:6:\n")),
        (
            "ownertrust.txt.age",
            format!("{PERSONAL}:6:\n{PERSONAL}:6:\n"),
        ),
    ];
    for (name, plain) in cases {
        for (file, bytes) in FILES.iter().zip(&original) {
            fs::write(f.directory().join(file), bytes).unwrap();
        }
        let bytes = if name.ends_with(".age") {
            encrypt_age_fixture(PASSWORD, plain.as_bytes())
        } else {
            plain.into_bytes()
        };
        f.replace(name, &bytes);
        f.assert_failed(f.restore());
        assert!(f.imports().is_empty(), "{name}");
    }
    for (file, bytes) in FILES.iter().zip(&original) {
        fs::write(f.directory().join(file), bytes).unwrap();
    }
    success(f.restore());
    assert_eq!(f.imports().lines().count(), 3);
}

#[test]
fn old_singular_password_metadata_is_supported_but_metadata_is_never_executed() {
    let f = Fixture::new();
    f.backup(&[PERSONAL]);
    let path = f.directory().join("backup-metadata.sh");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        text.lines()
            .filter(|line| !line.starts_with("gpg_backup_fingerprints="))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .unwrap();
    success(f.restore());
    fs::remove_file(f.s.root.join("imports")).unwrap();
    let original = fs::read_to_string(&path).unwrap();
    for invalid in [
        format!("{original}\ngpg_backup_fingerprints=''\n"),
        format!("{original}\ngpg_backup_fingerprints='{PERSONAL} {PERSONAL}'\n"),
        format!("{original}\n$(touch should-never-be-created)\n"),
    ] {
        fs::write(&path, invalid).unwrap();
        f.assert_failed(f.restore());
        assert!(f.imports().is_empty());
    }
    assert!(!f.s.root.join("should-never-be-created").exists());
}

#[test]
fn explicit_restore_refuses_an_isolated_destination_and_passwords_inside_source() {
    let f = Fixture::new();
    f.backup(&[PERSONAL]);
    f.assert_failed(
        f.command()
            .arg("--destination")
            .arg(&f.s.destination)
            .args(["gpg", "restore", "--gpg-program"])
            .arg(fake_gpg())
            .arg("--passphrase-file")
            .arg(&f.password)
            .output()
            .unwrap(),
    );
    let unsafe_password = f.s.source.join("unsafe password");
    private_write(&unsafe_password, PASSWORD.as_bytes());
    f.assert_failed(
        f.command()
            .args(["gpg", "restore", "--gpg-program"])
            .arg(fake_gpg())
            .arg("--passphrase-file")
            .arg(&unsafe_password)
            .output()
            .unwrap(),
    );
    assert!(f.imports().is_empty());
}

#[cfg(unix)]
#[test]
fn password_permissions_and_symlinked_backup_paths_are_rejected() {
    use std::os::unix::{fs::symlink, fs::PermissionsExt};
    let f = Fixture::new();
    f.backup(&[PERSONAL]);
    let before = f.files();
    fs::set_permissions(&f.password, fs::Permissions::from_mode(0o644)).unwrap();
    f.assert_failed(f.restore());
    fs::set_permissions(&f.password, fs::Permissions::from_mode(0o600)).unwrap();
    let secret = f.directory().join("secret.asc.age");
    let external = f.s.root.join("cipher outside source");
    fs::rename(&secret, &external).unwrap();
    symlink(&external, &secret).unwrap();
    f.assert_failed(f.restore());
    assert!(f.imports().is_empty());
    fs::remove_file(&secret).unwrap();
    fs::rename(&external, &secret).unwrap();
    let link = f.s.root.join("password symlink");
    symlink(&f.password, &link).unwrap();
    f.assert_failed(
        f.command()
            .args(["gpg", "backup", "--fingerprint", PERSONAL, "--gpg-program"])
            .arg(fake_gpg())
            .arg("--passphrase-file")
            .arg(link)
            .output()
            .unwrap(),
    );
    assert_eq!(f.files(), before);
}

#[test]
fn invalid_password_files_and_conflicting_protection_options_fail_before_export() {
    let f = Fixture::new();
    for password in [
        &b""[..],
        &b"\r\n"[..],
        &b"one\ntwo\n"[..],
        &b"two newlines\n\n"[..],
        &b"nul\0byte"[..],
        &b"\xff"[..],
    ] {
        private_write(&f.password, password);
        let failed = f.backup_to(&[PERSONAL], None);
        assert!(failed.stdout.is_empty());
        failure(failed, "GPG backup password");
        assert!(!f.directory().exists());
        assert!(f.imports().is_empty());
    }
    let events = fs::read_to_string(f.s.root.join("events")).unwrap();
    assert!(!events.contains("--export-secret-keys"));
    assert!(!events.contains("--export-ownertrust"));
    let failed = f
        .command()
        .args([
            "gpg",
            "backup",
            "--fingerprint",
            PERSONAL,
            "--identity",
            "missing",
            "--passphrase-file",
        ])
        .arg(&f.password)
        .output()
        .unwrap();
    failure(failed, "cannot be used with");
    assert!(!f.directory().exists());
}
