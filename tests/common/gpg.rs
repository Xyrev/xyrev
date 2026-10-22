use super::{fs, git_path, Command, Path, PathBuf};
use std::sync::OnceLock;

// A real executable rather than a shell script exercises Windows process
// creation, argument passing and stdin without requiring another runtime.
pub fn fake_gpg() -> &'static Path {
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

pub fn private_write(path: &Path, bytes: &[u8]) {
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
