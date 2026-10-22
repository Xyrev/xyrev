mod common;
use common::*;
use fs2::FileExt;

fn encrypt(sandbox: &Sandbox, input: &Path, passphrase: &Path, output: &Path) -> Output {
    sandbox
        .command()
        .arg("encrypt")
        .arg(input)
        .arg("--passphrase-file")
        .arg(passphrase)
        .arg("--output")
        .arg(output)
        .output()
        .unwrap()
}

#[test]
fn passphrase_files_roundtrip_under_deployment_lock_without_saving_credentials() {
    let sandbox = Sandbox::new(&[
        ("dot_seed", "seed"),
        (
            ".xyrev.toml.tmpl",
            "[encryption]\nbackend='gpg'\nidentities=['missing-identity']\nrecipients=['invalid-recipient']\n",
        ),
    ]);
    success(sandbox.init(&[]));
    let before = snapshot(&sandbox.control());
    let source_before = snapshot(&sandbox.source);
    let passphrase = sandbox.root.join("password with spaces.txt");
    write(&passphrase, "  synthetic '密码' passphrase  \r\n");
    let input = sandbox.root.join("synthetic private input");
    fs::write(&input, b"synthetic private backup\0\n").unwrap();
    let cipher = sandbox.root.join("backup.age");
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(sandbox.control().join("lock"))
        .unwrap();
    lock.lock_exclusive().unwrap();
    assert_eq!(success(encrypt(&sandbox, &input, &passphrase, &cipher)), "");
    let encrypted = fs::read_to_string(&cipher).unwrap();
    assert!(encrypted.starts_with("-----BEGIN AGE ENCRYPTED FILE-----"));
    assert!(!encrypted.contains("synthetic private backup"));
    assert!(!encrypted.contains("synthetic '密码' passphrase"));
    let output = sandbox.root.join("recovered");
    success(
        sandbox
            .command()
            .arg("decrypt")
            .arg(&cipher)
            .arg("--passphrase-file")
            .arg(&passphrase)
            .arg("--output")
            .arg(&output)
            .output()
            .unwrap(),
    );
    assert_eq!(fs::read(output).unwrap(), fs::read(input).unwrap());
    failure(
        sandbox.cli(&["apply"]),
        "Another dotfile command is running",
    );
    assert_eq!(snapshot(&sandbox.control()), before);
    assert_eq!(snapshot(&sandbox.source), source_before);
    assert!(!sandbox.destination.exists());
}

#[test]
fn wrong_passphrase_never_outputs_plaintext_or_overwrites_output() {
    let sandbox = Sandbox::new(&[]);
    let passphrase = sandbox.root.join("password");
    write(&passphrase, "  synthetic correct password  \n");
    let input = sandbox.root.join("input");
    write(&input, "synthetic plaintext that must remain private\n");
    let cipher = sandbox.root.join("cipher.age");
    success(encrypt(&sandbox, &input, &passphrase, &cipher));
    // Spaces are part of the passphrase; only the line ending is removed.
    write(&passphrase, "synthetic correct password\n");
    let output = sandbox.root.join("existing output");
    write(&output, "keep existing contents\n");
    let mut command = sandbox.command();
    command
        .arg("decrypt")
        .arg(&cipher)
        .arg("--passphrase-file")
        .arg(&passphrase);
    let failed = command.output().unwrap();
    assert!(failed.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&failed.stderr).contains("synthetic correct password"));
    failure(failed, "Unable to decrypt age ciphertext");
    failure(
        command.arg("--output").arg(&output).output().unwrap(),
        "Unable to decrypt age ciphertext",
    );
    assert_eq!(
        fs::read_to_string(output).unwrap(),
        "keep existing contents\n"
    );
}

#[test]
fn empty_multiline_and_invalid_utf8_passphrases_are_rejected_before_writing() {
    let sandbox = Sandbox::new(&[]);
    let passphrase = sandbox.root.join("password");
    let input = sandbox.root.join("input");
    write(&input, "synthetic input");
    let output = sandbox.root.join("not created/output.age");
    for (password, error) in [
        (&b""[..], "Passphrase must not be empty"),
        (&b"\r\n"[..], "Passphrase must not be empty"),
        (
            &b"one\ntwo\n"[..],
            "Passphrase must contain exactly one line",
        ),
        (
            &b"two newlines\n\n"[..],
            "Passphrase must contain exactly one line",
        ),
        (&b"nul\0byte"[..], "Passphrase must not contain NUL bytes"),
        (&b"\xff"[..], "Read passphrase file"),
    ] {
        fs::write(&passphrase, password).unwrap();
        let failed = encrypt(&sandbox, &input, &passphrase, &output);
        assert!(failed.stdout.is_empty());
        failure(failed, error);
        assert!(!output.parent().unwrap().exists());
    }
}

#[test]
fn passphrase_options_reject_identity_recipient_and_gpg_combinations() {
    let sandbox = Sandbox::new(&[]);
    for args in [
        vec!["--passphrase", "--passphrase-file", "missing"],
        vec!["--passphrase", "--identity", "missing"],
        vec!["--passphrase-file", "missing", "--recipient", "invalid"],
    ] {
        failure(
            sandbox
                .command()
                .args(["encrypt", "missing"])
                .args(args)
                .output()
                .unwrap(),
            "cannot be used with",
        );
    }
    failure(
        sandbox
            .command()
            .args([
                "encrypt",
                "missing",
                "--passphrase-file",
                "missing",
                "--backend",
                "gpg",
            ])
            .output()
            .unwrap(),
        "Passphrase options require the age backend",
    );
}

#[test]
fn legacy_environment_passphrase_decryption_remains_compatible() {
    let sandbox = Sandbox::new(&[]);
    let passphrase = sandbox.root.join("password");
    write(&passphrase, "synthetic legacy password");
    let input = sandbox.root.join("input");
    write(&input, "synthetic legacy data\n");
    let cipher = sandbox.root.join("cipher.age");
    success(encrypt(&sandbox, &input, &passphrase, &cipher));
    assert_eq!(
        success(
            sandbox
                .command()
                .arg("decrypt")
                .arg(&cipher)
                .env("XYREV_AGE_PASSPHRASE", "synthetic legacy password")
                .output()
                .unwrap(),
        ),
        "synthetic legacy data\n"
    );
}

#[cfg(unix)]
#[test]
fn passphrase_pipe_fd_leaves_stdin_available_for_streamed_input() {
    use std::io::Write;
    use std::process::Stdio;
    let sandbox = Sandbox::new(&[]);
    let passphrase = sandbox.root.join("password");
    write(&passphrase, "synthetic pipe password\n");
    let transform = |operation: &str, input: &[u8]| {
        let mut child = Command::new("/bin/bash")
            .args([
                "-c",
                "exec \"$1\" dotfile \"$2\" - --passphrase-file /dev/fd/3 3< <(/bin/cat \"$3\")",
                "pipe-test",
            ])
            .arg(env!("CARGO_BIN_EXE_xyrev"))
            .arg(operation)
            .arg(&passphrase)
            .env("HOME", &sandbox.home)
            .env("USERPROFILE", &sandbox.home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    };
    let plain = b"synthetic streamed backup\0\n";
    let cipher = transform("encrypt", plain);
    assert!(cipher.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----"));
    assert_eq!(transform("decrypt", &cipher), plain);
}
