use super::storage::Paths;
use super::{crypto, git, scripts, storage, template};
use crate::cli::dotfile::{CryptoArgs, TemplateArgs};
use age::secrecy::{ExposeSecret, SecretString};
use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;

pub fn crypt(paths: &Paths, args: &CryptoArgs, decrypt: bool) -> Result<()> {
    let mut settings = paths.encryption();
    if let Some(backend) = &args.backend {
        settings.backend = backend.clone();
    }
    if !args.recipient.is_empty() {
        settings.recipients = args.recipient.clone();
        settings.gpg_recipient = args.recipient.first().cloned();
    }
    if !args.identity.is_empty() {
        settings.identities = args.identity.clone();
    }
    let passphrase = if args.passphrase || args.passphrase_file.is_some() {
        if args.backend.as_deref() == Some("gpg") {
            bail!("Passphrase options require the age backend");
        }
        settings.backend = "age".into();
        Some(read_passphrase(paths, args, decrypt)?)
    } else {
        None
    };
    let bytes = if args.input == Path::new("-") {
        let mut bytes = Vec::new();
        io::stdin().read_to_end(&mut bytes)?;
        bytes
    } else {
        fs::read(storage::expand_home(&args.input, &paths.home))?
    };
    let output = if let Some(passphrase) = passphrase {
        if decrypt {
            crypto::decrypt_with_passphrase(&passphrase, &bytes)?
        } else {
            crypto::encrypt_with_passphrase(passphrase, &bytes)?
        }
    } else if decrypt {
        let extension = args
            .input
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or(&settings.backend);
        crypto::decrypt(&settings, &paths.home, &bytes, extension)?
    } else {
        crypto::encrypt(&settings, &bytes)?
    };
    if let Some(file) = &args.output {
        let file = storage::expand_home(file, &paths.home);
        if let Some(parent) = file
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        storage::atomic_write(&file, &output, 0o600)?;
    } else {
        io::stdout().write_all(&output)?;
    }
    Ok(())
}

fn read_passphrase(paths: &Paths, args: &CryptoArgs, decrypt: bool) -> Result<SecretString> {
    let passphrase = if let Some(path) = &args.passphrase_file {
        let path = storage::expand_home(path, &paths.home);
        let mut text = fs::read_to_string(&path)
            .with_context(|| format!("Read passphrase file {}", path.display()))?;
        if text.ends_with('\n') {
            text.pop();
            if text.ends_with('\r') {
                text.pop();
            }
        }
        SecretString::new(text)
    } else {
        prompt_passphrase("Enter passphrase: ").context(
            "Read passphrase privately from the terminal; use --passphrase-file for a pipe",
        )?
    };
    if passphrase.expose_secret().is_empty() {
        bail!("Passphrase must not be empty");
    }
    if passphrase.expose_secret().contains(['\r', '\n']) {
        bail!("Passphrase must contain exactly one line");
    }
    if passphrase.expose_secret().contains('\0') {
        bail!("Passphrase must not contain NUL bytes");
    }
    if args.passphrase && !decrypt {
        let confirmation = prompt_passphrase("Confirm passphrase: ")
            .context("Confirm passphrase privately from the terminal")?;
        if passphrase.expose_secret() != confirmation.expose_secret() {
            bail!("Passphrases do not match");
        }
    }
    Ok(passphrase)
}

#[cfg(not(windows))]
pub(super) fn prompt_passphrase(prompt: &str) -> io::Result<SecretString> {
    rpassword::prompt_password(prompt).map(SecretString::new)
}

#[cfg(windows)]
pub(super) fn prompt_passphrase(prompt: &str) -> io::Result<SecretString> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, SetConsoleMode, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT,
        ENABLE_PROCESSED_INPUT,
    };

    struct ConsoleInput {
        file: fs::File,
        mode: u32,
    }
    impl Drop for ConsoleInput {
        fn drop(&mut self) {
            unsafe { SetConsoleMode(self.file.as_raw_handle() as _, self.mode) };
        }
    }

    // rpassword 7.3.1 writes a newline to stdout on Windows. Use its buffered
    // reader with explicit console handles so prompts cannot alter crypto output.
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("CONIN$")?;
    let mut mode = 0;
    let handle = file.as_raw_handle() as _;
    if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let hidden_mode = (mode | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT) & !ENABLE_ECHO_INPUT;
    if unsafe { SetConsoleMode(handle, hidden_mode) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let input = ConsoleInput { file, mode };
    let mut terminal = fs::OpenOptions::new().write(true).open("CONOUT$")?;
    let mut reader = io::BufReader::new(&input.file);
    let password = SecretString::new(rpassword::prompt_password_from_bufread(
        &mut reader,
        &mut terminal,
        prompt,
    )?);
    terminal.write_all(b"\r\n")?;
    Ok(password)
}

pub fn execute_template(paths: &Paths, args: &TemplateArgs, overrides: &[String]) -> Result<()> {
    let engine = template::Engine::new(paths, overrides, args.init)?;
    let (name, bytes) = if let Some(file) = &args.file {
        (file.to_string_lossy().into_owned(), fs::read(file)?)
    } else if let Some(text) = &args.template {
        ("<argument>".into(), text.as_bytes().to_vec())
    } else {
        let mut bytes = Vec::new();
        io::stdin().read_to_end(&mut bytes)?;
        ("<stdin>".into(), bytes)
    };
    io::stdout().write_all(&engine.render(&name, &bytes)?)?;
    Ok(())
}

pub fn cd(paths: &Paths, print: bool) -> Result<()> {
    git::verify(&paths.source)?;
    if print {
        println!("{}", paths.source.display());
        return Ok(());
    }
    let shell = std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(windows) {
            "powershell.exe".into()
        } else {
            "sh".into()
        }
    });
    let mut command = Command::new(shell);
    command.current_dir(&paths.source);
    let status = command.status().context("Start source-directory shell")?;
    if !status.success() {
        bail!("Source shell exited with {status}");
    }
    Ok(())
}

pub fn available(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|directory| {
            let path = directory.join(name);
            if path.is_file() {
                return true;
            }
            cfg!(windows)
                && ["exe", "cmd", "bat"]
                    .iter()
                    .any(|extension| directory.join(format!("{name}.{extension}")).is_file())
        })
    })
}

pub fn doctor(paths: &Paths, overrides: &[String]) -> Result<()> {
    let mut failures = 0;
    println!(
        "xyrev {} / {} / {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    if super::native_git::enabled(&paths.source) {
        println!("OK built-in Git (SSH transport, submodules and LFS may need external tools)");
    } else if available("git") {
        println!("OK system Git");
    } else {
        println!("ERROR system Git is missing");
        failures += 1;
    }
    for (name, result) in [
        ("repository", git::verify(&paths.source)),
        ("source/submodules", git::ready(&paths.source)),
        ("applied state", paths.state().map(|_| ())),
        ("script database", scripts::Store::load(paths).map(|_| ())),
        (
            "template/data configuration",
            template::Engine::new(paths, overrides, false).map(|_| ()),
        ),
    ] {
        match result {
            Ok(()) => println!("OK {name}"),
            Err(error) => {
                println!("ERROR {name}: {error:#}");
                failures += 1;
            }
        }
    }
    if let Some(config) = &paths.config {
        if config.git.lfs && !available("git-lfs") {
            println!("ERROR git.lfs requires git-lfs");
            failures += 1;
        }
        if config.encryption.backend == "gpg" && !available("gpg") {
            println!("ERROR configured encryption backend requires gpg");
            failures += 1;
        }
        for identity in &config.encryption.identities {
            let path = storage::expand_home(identity, &paths.home);
            if !path.is_file() {
                println!("ERROR age identity is missing: {}", path.display());
                failures += 1;
            }
        }
        if let Ok(status) = git::run(
            &paths.source,
            ["status", "--porcelain=v1", "--untracked-files=all"],
        ) {
            if !status.is_empty() {
                println!("WARN source has local changes; update will require a clean worktree");
            }
        }
    } else {
        println!("WARN no saved dotfile configuration");
    }
    for executable in [
        "gpg",
        "git-lfs",
        "bw",
        "op",
        "pass",
        "gopass",
        "vault",
        "keepassxc-cli",
    ] {
        if available(executable) {
            println!("OK optional executable {executable}");
        }
    }
    println!("{failures} error(s)");
    if failures > 0 {
        bail!("Doctor found {failures} configuration/dependency error(s)");
    }
    Ok(())
}
