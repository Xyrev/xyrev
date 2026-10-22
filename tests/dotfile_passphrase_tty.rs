#![cfg(unix)]

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::{Duration, Instant};

fn command(root: &Path, operation: &str) -> Command {
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_xyrev"));
    command
        .current_dir(root)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env_remove("XYREV_AGE_PASSPHRASE")
        .env_remove("XYREV_AGE_PASSPHRASE_FILE")
        .args(["dotfile", operation]);
    command
}

fn termios(fd: RawFd, stage: &str) -> libc::termios {
    let mut settings = std::mem::MaybeUninit::uninit();
    unsafe {
        assert_eq!(
            libc::tcgetattr(fd, settings.as_mut_ptr()),
            0,
            "tcgetattr({fd}) {stage}: {}",
            std::io::Error::last_os_error()
        );
        settings.assume_init()
    }
}

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

struct TerminalOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    transcript: String,
}

fn read_terminal(master: &mut File, output: &mut Vec<u8>) {
    loop {
        let mut bytes = [0; 8192];
        match master.read(&mut bytes) {
            Ok(0) => break,
            Ok(count) => output.extend_from_slice(&bytes[..count]),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(error) if error.raw_os_error() == Some(libc::EIO) => break,
            Err(error) => panic!("Read terminal: {error}"),
        }
    }
}

fn terminal(mut command: Command, answers: &[&str]) -> TerminalOutput {
    let (mut master, mut slave) = (0, 0);
    unsafe {
        assert_eq!(
            libc::openpty(
                std::ptr::addr_of_mut!(master),
                std::ptr::addr_of_mut!(slave),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ),
            0
        );
    }
    let mut master = unsafe { File::from_raw_fd(master) };
    let slave = unsafe { File::from_raw_fd(slave) };
    let initial_flags = termios(master.as_raw_fd(), "before spawning").c_lflag;
    assert_ne!(initial_flags & libc::ECHO, 0);
    unsafe {
        let flags = libc::fcntl(master.as_raw_fd(), libc::F_GETFL);
        assert!(flags >= 0);
        assert!(libc::fcntl(master.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) >= 0);
    }
    command
        // Password input must use the controlling terminal, independently of
        // stdin, which may hold plaintext/ciphertext or be disconnected.
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(slave.try_clone().unwrap()));
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 || libc::ioctl(2, libc::TIOCSCTTY as _, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = KillOnDrop(command.spawn().unwrap());
    let mut transcript = Vec::new();
    let (mut answered, mut cursor) = (0, 0);
    let start = Instant::now();
    let status = loop {
        read_terminal(&mut master, &mut transcript);
        if answered < answers.len()
            && String::from_utf8_lossy(&transcript[cursor..])
                .to_ascii_lowercase()
                .contains("passphrase")
            && termios(master.as_raw_fd(), "while waiting for a prompt").c_lflag & libc::ECHO == 0
        {
            // Only send an answer once the terminal itself has disabled echo.
            // This avoids racing prompt output against hidden-input setup.
            master.write_all(answers[answered].as_bytes()).unwrap();
            master.write_all(b"\n").unwrap();
            answered += 1;
            cursor = transcript.len();
        }
        if let Some(status) = child.0.try_wait().unwrap() {
            read_terminal(&mut master, &mut transcript);
            break status;
        }
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "Terminal prompt timed out: {}",
            String::from_utf8_lossy(&transcript)
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(
        answered,
        answers.len(),
        "Missing passphrase prompt: {}",
        String::from_utf8_lossy(&transcript)
    );
    assert_eq!(
        // macOS detaches the slave when its controlling session exits. The
        // master retains the terminal settings and remains queryable.
        termios(master.as_raw_fd(), "after child exited").c_lflag & libc::ECHO,
        initial_flags & libc::ECHO,
        "Terminal echo was not restored"
    );
    let mut stdout = Vec::new();
    child
        .0
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut stdout)
        .unwrap();
    TerminalOutput {
        status,
        stdout,
        transcript: String::from_utf8(transcript).unwrap(),
    }
}

#[test]
fn passphrase_terminal_input_is_hidden_confirmed_and_decrypts_without_confirmation() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("plaintext");
    let cipher = temp.path().join("ciphertext.age");
    let restored = temp.path().join("restored");
    let plain = b"fixture-only private configuration\0\n";
    fs::write(&input, plain).unwrap();
    let passphrase = "fixture-terminal-passphrase-42";
    let mut encrypt = command(temp.path(), "encrypt");
    encrypt
        .arg(&input)
        .args(["--passphrase", "--output"])
        .arg(&cipher);
    let encrypted = terminal(encrypt, &[passphrase, passphrase]);
    assert!(encrypted.status.success(), "{}", encrypted.transcript);
    assert!(encrypted.stdout.is_empty());
    assert!(!encrypted.transcript.contains(passphrase));
    assert!(!fs::read(&cipher)
        .unwrap()
        .windows(plain.len())
        .any(|part| part == plain));

    let mut decrypt = command(temp.path(), "decrypt");
    decrypt
        .arg(&cipher)
        .args(["--passphrase", "--output"])
        .arg(&restored);
    let decrypted = terminal(decrypt, &[passphrase]);
    assert!(decrypted.status.success(), "{}", decrypted.transcript);
    assert!(decrypted.stdout.is_empty());
    assert!(!decrypted.transcript.contains(passphrase));
    assert!(!decrypted
        .transcript
        .to_ascii_lowercase()
        .contains("confirm"));
    assert_eq!(fs::read(restored).unwrap(), plain);
}

#[test]
fn passphrase_confirmation_mismatch_does_not_create_ciphertext_or_echo_answers() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("plaintext");
    let cipher = temp.path().join("not-created/ciphertext.age");
    fs::write(&input, "fixture-only private configuration").unwrap();
    let mut encrypt = command(temp.path(), "encrypt");
    encrypt
        .arg(&input)
        .args(["--passphrase", "--output"])
        .arg(&cipher);
    let answers = [
        "fixture-first-passphrase-42",
        "fixture-different-passphrase-43",
    ];
    let output = terminal(encrypt, &answers);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.transcript.to_ascii_lowercase().contains("match"));
    assert!(!output.transcript.contains("BEGIN AGE ENCRYPTED FILE"));
    for answer in answers {
        assert!(!output.transcript.contains(answer));
    }
    assert!(!cipher.parent().unwrap().exists());
}

fn piped_passphrase(mut command: Command) -> Output {
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command
        .args(["--passphrase-file", "/dev/stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"fixture-only-pipe-passphrase-42\n")
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn passphrase_file_accepts_a_pipe_without_a_terminal_or_prompt() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("plaintext");
    let cipher = temp.path().join("ciphertext.age");
    let plain = b"fixture-only piped passphrase content\0\n";
    fs::write(&input, plain).unwrap();
    let mut encrypt = command(temp.path(), "encrypt");
    encrypt.arg(&input).arg("--output").arg(&cipher);
    let encrypted = piped_passphrase(encrypt);
    assert!(
        encrypted.status.success(),
        "{}",
        String::from_utf8_lossy(&encrypted.stderr)
    );
    assert!(encrypted.stdout.is_empty());
    assert!(encrypted.stderr.is_empty());
    let mut decrypt = command(temp.path(), "decrypt");
    decrypt.arg(&cipher);
    let decrypted = piped_passphrase(decrypt);
    assert!(
        decrypted.status.success(),
        "{}",
        String::from_utf8_lossy(&decrypted.stderr)
    );
    assert!(decrypted.stderr.is_empty());
    assert_eq!(decrypted.stdout, plain);
}
