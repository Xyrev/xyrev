mod common;
use common::*;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::process::Stdio;
use std::sync::OnceLock;

const PERSONAL: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const WORK: &str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";
const OTHER: &str = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC";
const SUBKEY: &str = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD";
const PASSWORD: &str = "  fixture's password $(unexecuted)  ";
const FILES: [&str; 4] = [
    "public.asc",
    "secret.asc.age",
    "ownertrust.txt.age",
    "backup-metadata.sh",
];

// A real executable rather than a shell script exercises Windows process
// creation, argument passing and stdin without requiring another runtime.
fn fake_gpg() -> &'static Path {
    static FIXTURE: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    &FIXTURE
        .get_or_init(|| {
            let directory = tempfile::tempdir().unwrap();
            let source = directory.path().join("fixture.rs");
            fs::write(&source, FAKE_GPG).unwrap();
            let binary = git_path(&directory.path().join(if cfg!(windows) {
                "gpg fixture.exe"
            } else {
                "gpg fixture"
            }));
            let output = Command::new("rustc")
                .args(["--edition=2021", "-O"])
                .arg(&source)
                .arg("-o")
                .arg(&binary)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            (directory, binary)
        })
        .1
}

const FAKE_GPG: &str = r#"
use std::{env, fs, io::{self, Read, Write}, path::PathBuf};
const A: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const B: &str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";
const C: &str = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC";
const D: &str = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD";
fn append(path: PathBuf, value: &str) { let mut f=fs::OpenOptions::new().create(true).append(true).open(path).unwrap(); writeln!(f,"{value}").unwrap(); }
fn die() -> ! { eprintln!("fixture GPG failed; private output intentionally omitted"); std::process::exit(19); }
fn packet(kind: &str, keys: &[String]) -> String { format!("XYREV-FAKE-{kind}\n{}\n",keys.join("\n")) }
fn main() {
 let args:Vec<String>=env::args().skip(1).collect();
 let root=PathBuf::from(env::var_os("XYREV_TEST_GPG_ROOT").unwrap());
 if let Some(i)=args.iter().position(|a| a=="--xyrev-hook") {
   let phase=&args[i+1]; append(root.join("events"),&format!("hook:{phase}"));
   if phase=="before" { fs::write(root.join("before"),"").unwrap(); }
   if phase=="after" { if !root.join("imports").exists() { die(); } fs::write(root.join("after"),"").unwrap(); }
   return;
 }
 append(root.join("events"),&format!("gpg:{}",args.join(" ")));
 let has=|needle:&str| args.iter().any(|a| a==needle);
 if has("--list-keys") {
   let key=args.last().unwrap();
   let key=if key==D { A } else {key};
   if ![A,B,C].contains(&key) {die();}
   println!("pub:u:256:22:0000:0:::u:::scSC:"); println!("fpr:::::::::{key}:");
   println!("sub:u:256:22:0001:0:::::s:"); println!("fpr:::::::::{D}:"); return;
 }
 for (flag,kind) in [("--export-secret-keys","SECRET"),("--export","PUBLIC")] {
   if let Some(i)=args.iter().position(|a| a==flag) {
     let mut keys=args[i+1..].to_vec();
     if root.join("partial-export").exists() && kind=="SECRET" {keys.retain(|k| k!=B);}
     print!("{}",packet(kind,&keys)); return;
   }
 }
 if has("--export-ownertrust") { for key in [A,B,C] {println!("{key}:6:");} return; }
 let mut input=String::new(); io::stdin().read_to_string(&mut input).unwrap();
 if has("show-only") {
   let mut lines=input.lines();
   let kind=match lines.next() {Some("XYREV-FAKE-SECRET")=>"sec",Some("XYREV-FAKE-PUBLIC")=>"pub",_=>die()};
   for key in lines {if ![A,B,C].contains(&key){die();} println!("{kind}:u:256:22:0000:0:::u:::scSC:");println!("fpr:::::::::{key}:");println!("uid:u::::::::Fixture:");}
   return;
 }
 if has("--import") || has("--import-ownertrust") {
   if root.join("fail-import").exists() {die();}
   let deployed=PathBuf::from(env::var_os("HOME").unwrap()).join(".seed").exists();
   let kind=if has("--import-ownertrust") {"trust"} else if input.starts_with("XYREV-FAKE-SECRET") {"secret"} else {"public"};
   append(root.join("imports"),&format!("{kind}:deployed={deployed}")); append(root.join("events"),&format!("import:{kind}")); return;
 }
 die();
}
"#;

fn private_write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    #[cfg(windows)]
    {
        // Match a real owner-only password file, rather than relying on the
        // runner's inherited temporary-directory ACL.
        let script = r#"
$ErrorActionPreference = 'Stop'
$path = $env:XYREV_TEST_PASSWORD_PATH
$acl = Get-Acl -LiteralPath $path
$acl.SetAccessRuleProtection($true, $false)
foreach ($rule in @($acl.Access)) { $acl.RemoveAccessRuleSpecific($rule) }
$allow = [System.Security.AccessControl.AccessControlType]::Allow
$rights = [System.Security.AccessControl.FileSystemRights]::FullControl
$me = [System.Security.Principal.WindowsIdentity]::GetCurrent().User
$acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($me, $rights, $allow))
foreach ($sid in @('S-1-5-18', 'S-1-5-32-544')) {
    $identity = [System.Security.Principal.SecurityIdentifier]::new($sid)
    $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($identity, $rights, $allow))
}
Set-Acl -LiteralPath $path -AclObject $acl
"#;
        let powershell = std::env::var_os("SystemRoot")
            .map(|root| PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe"))
            .unwrap_or_else(|| "powershell.exe".into());
        let out = Command::new(powershell)
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .env("XYREV_TEST_PASSWORD_PATH", path)
            .env_remove("PSModulePath")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

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
        command.env("XYREV_TEST_GPG_ROOT", &self.s.root);
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
        self.s.source.join(".xyrevtemplates/gpg")
    }
    fn files(&self) -> Vec<Vec<u8>> {
        FILES
            .iter()
            .map(|name| fs::read(self.directory().join(name)).unwrap())
            .collect()
    }
    fn imports(&self) -> String {
        fs::read_to_string(self.s.root.join("imports")).unwrap_or_default()
    }
    fn declaration_at(&self, root: &Path) {
        write(
            &root.join(".xyrevgpg.toml"),
            &format!(
                "enabled=true\ndirectory='.xyrevtemplates/gpg'\nprogram={}\npassphrase_file={}\n",
                serde_json::to_string(fake_gpg()).unwrap(),
                serde_json::to_string(&self.password).unwrap()
            ),
        );
    }
    fn declare(&self) {
        self.declaration_at(&self.s.source);
    }
    fn transform(&self, operation: &str, bytes: &[u8]) -> Vec<u8> {
        let mut child = self
            .command()
            .arg(operation)
            .arg("-")
            .arg("--passphrase-file")
            .arg(&self.password)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(bytes).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        out.stdout
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
            assert!(!String::from_utf8_lossy(stream).contains("XYREV-FAKE-SECRET"));
        }
    }
}

#[test]
fn native_password_backup_and_explicit_restore_preserve_all_selected_keys() {
    let f = Fixture::new();
    f.backup(&[WORK, PERSONAL, &WORK.to_ascii_lowercase()]);
    let metadata = fs::read_to_string(f.directory().join("backup-metadata.sh")).unwrap();
    assert!(metadata.contains(&format!("gpg_backup_fingerprints='{PERSONAL} {WORK}'")));
    let private = f.transform(
        "decrypt",
        &fs::read(f.directory().join("secret.asc.age")).unwrap(),
    );
    assert_eq!(
        String::from_utf8(private).unwrap(),
        format!("XYREV-FAKE-SECRET\n{PERSONAL}\n{WORK}\n")
    );
    let trust = f.transform(
        "decrypt",
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
        assert!(!String::from_utf8_lossy(&bytes).contains("XYREV-FAKE-SECRET"));
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
                "#!{} --xyrev-hook {phase}\n",
                serde_json::to_string(fake_gpg()).unwrap()
            ),
        );
    }
    fs::remove_file(f.s.root.join("events")).unwrap();
    let before = snapshot(&f.s.control());
    let source = snapshot(&f.s.source);
    for args in [vec!["diff"], vec!["status"], vec!["apply", "--dry-run"]] {
        let out = success(f.command().args(args).output().unwrap());
        assert!(!out.contains("XYREV-FAKE-SECRET"));
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
        assert!(!name.contains(".xyrev"));
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
    let output = f.s.work.join(".xyrevtemplates/gpg");
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
        ("secret.asc.age", format!("XYREV-FAKE-SECRET\n{PERSONAL}\n")),
        (
            "secret.asc.age",
            format!("XYREV-FAKE-SECRET\n{PERSONAL}\n{WORK}\n{OTHER}\n"),
        ),
        (
            "secret.asc.age",
            format!("XYREV-FAKE-PUBLIC\n{PERSONAL}\n{WORK}\n"),
        ),
        (
            "public.asc",
            format!("XYREV-FAKE-PUBLIC\n{PERSONAL}\n{OTHER}\n"),
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
            f.transform("encrypt", plain.as_bytes())
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
