mod common;
use common::*;
use fs2::FileExt;

#[test]
fn standalone_crypto_works_under_deployment_lock_but_writers_remain_locked() {
    let sandbox = Sandbox::new(&[("dot_seed", "seed")]);
    success(sandbox.init(&[]));
    let identity = sandbox.home.join("identity.txt");
    let recipient = success(
        sandbox
            .command()
            .args(["keygen", "--output"])
            .arg(&identity)
            .output()
            .unwrap(),
    )
    .trim()
    .to_owned();
    let plaintext = sandbox.root.join("synthetic backup");
    write(&plaintext, "synthetic private backup\n");
    let ciphertext = sandbox.root.join("synthetic backup.age");
    let before = snapshot(&sandbox.control());
    let source_before = snapshot(&sandbox.source);
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(sandbox.control().join("lock"))
        .unwrap();
    lock.lock_exclusive().unwrap();
    success(
        sandbox
            .command()
            .arg("encrypt")
            .arg(&plaintext)
            .arg("--recipient")
            .arg(&recipient)
            .arg("--output")
            .arg(&ciphertext)
            .output()
            .unwrap(),
    );
    assert!(!fs::read_to_string(&ciphertext)
        .unwrap()
        .contains("synthetic private backup"));
    assert_eq!(
        success(
            sandbox
                .command()
                .arg("decrypt")
                .arg(&ciphertext)
                .arg("--identity")
                .arg(&identity)
                .output()
                .unwrap(),
        ),
        "synthetic private backup\n"
    );
    failure(
        sandbox.cli(&["apply"]),
        "Another dotfile command is running",
    );
    failure(
        sandbox.cli(&["add", ".seed"]),
        "Another dotfile command is running",
    );
    // Windows byte-range locks also prevent reading the lock file through a
    // different handle. Inspect the unchanged control files after unlocking.
    drop(lock);
    assert_eq!(snapshot(&sandbox.control()), before);
    assert_eq!(snapshot(&sandbox.source), source_before);
    assert!(!sandbox.destination.exists());
}

#[cfg(unix)]
fn executable(path: &Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt;
    write(path, contents);
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn after_hook_streams_ciphertext_with_the_running_binary_without_deploying_secrets() {
    let sandbox = Sandbox::new(&[
        ("dot_seed", "seed"),
        (
            "run_once_after_import.sh",
            "#!/bin/sh\nset -eu\nif command -v xyrev >/dev/null 2>&1; then exit 83; fi\n\"$XYREV_EXECUTABLE\" dotfile decrypt \"$XYREV_SOURCE_DIR/.xyrevtemplates/synthetic-backup.age\" | \"$XYREV_TEST_IMPORTER\"\n",
        ),
    ]);
    let identity = sandbox.home.join("identity.txt");
    let recipient = success(
        sandbox
            .command()
            .args(["keygen", "--output"])
            .arg(&identity)
            .output()
            .unwrap(),
    )
    .trim()
    .to_owned();
    write(
        &sandbox.work.join(".xyrev.toml.tmpl"),
        &format!(
            "[encryption]\nidentities=[{}]\n",
            serde_json::to_string(&identity).unwrap()
        ),
    );
    let plaintext = sandbox.root.join("synthetic private backup");
    write(&plaintext, "synthetic private backup\n");
    let ciphertext = sandbox.work.join(".xyrevtemplates/synthetic-backup.age");
    success(
        sandbox
            .command()
            .arg("encrypt")
            .arg(&plaintext)
            .arg("--recipient")
            .arg(&recipient)
            .arg("--output")
            .arg(&ciphertext)
            .output()
            .unwrap(),
    );
    sandbox.push();
    success(sandbox.init(&[]));
    let preview = success(sandbox.cli(&["diff"]));
    assert!(!preview.contains("synthetic private backup"));
    assert!(!preview.contains("synthetic-backup.age"));
    let bin = sandbox.root.join("hook tools");
    executable(
        &bin.join("git"),
        "#!/bin/sh\nPATH=\"$XYREV_TEST_REAL_PATH\" exec git \"$@\"\n",
    );
    let importer = bin.join("synthetic-importer");
    executable(
        &importer,
        "#!/bin/sh\nset -eu\ntest -f \"$XYREV_DEST_DIR/.seed\"\n/usr/bin/cmp - \"$XYREV_TEST_EXPECTED\"\nprintf 'imported\\n' >> \"$XYREV_DEST_DIR/import.log\"\n",
    );
    success(
        sandbox
            .command()
            .arg("apply")
            .env("PATH", &bin)
            .env("XYREV_TEST_REAL_PATH", std::env::var_os("PATH").unwrap())
            .env("XYREV_TEST_EXPECTED", &plaintext)
            .env("XYREV_TEST_IMPORTER", &importer)
            // The hook must receive the running binary, not an inherited path.
            .env("XYREV_EXECUTABLE", "/missing/older-xyrev")
            .output()
            .unwrap(),
    );
    assert_eq!(
        fs::read_to_string(sandbox.destination.join("import.log")).unwrap(),
        "imported\n"
    );
    let mut targets: Vec<_> = fs::read_dir(&sandbox.destination)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    targets.sort();
    assert_eq!(targets, [".seed", "import.log"]);
    success(sandbox.cli(&["verify"]));
    assert!(success(sandbox.cli(&["apply"])).contains("Applied 0 change(s)"));
    assert_eq!(
        fs::read_to_string(sandbox.destination.join("import.log")).unwrap(),
        "imported\n"
    );
}
