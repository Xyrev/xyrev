use super::process;
use super::storage::{self, EncryptionSettings};
use age::secrecy::{ExposeSecret, SecretString};
use anyhow::{bail, Context, Result};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

pub fn suffix(settings: &EncryptionSettings) -> Result<&'static str> {
    match settings.backend.as_str() {
        "age" => Ok("age"),
        "gpg" => Ok("asc"),
        _ => bail!("Encryption backend must be age or gpg"),
    }
}

pub fn encrypt(settings: &EncryptionSettings, bytes: &[u8]) -> Result<Vec<u8>> {
    match settings.backend.as_str() {
        "age" => {
            let recipients: Vec<Box<dyn age::Recipient + Send>> = settings
                .recipients
                .iter()
                .map(|text| {
                    text.parse::<age::x25519::Recipient>()
                        .map(|key| Box::new(key) as Box<dyn age::Recipient + Send>)
                        .map_err(|_| anyhow::anyhow!("Invalid age recipient"))
                })
                .collect::<Result<_>>()?;
            let encryptor = age::Encryptor::with_recipients(recipients)
                .context("Configure encryption.recipients before encrypting")?;
            let mut output = Vec::new();
            let armor = age::armor::ArmoredWriter::wrap_output(
                &mut output,
                age::armor::Format::AsciiArmor,
            )?;
            let mut writer = encryptor.wrap_output(armor)?;
            writer.write_all(bytes)?;
            writer.finish()?.finish()?;
            Ok(output)
        }
        "gpg" => {
            let recipient = settings
                .gpg_recipient
                .as_ref()
                .context("Configure encryption.gpg_recipient")?;
            let mut command = Command::new("gpg");
            command.args([
                "--batch",
                "--yes",
                "--armor",
                "--encrypt",
                "--recipient",
                recipient,
            ]);
            let output = process::capture(command, Some(bytes))?;
            if !output.status.success() {
                bail!(
                    "gpg encryption failed ({}); check the recipient and keyring",
                    output.status
                );
            }
            Ok(output.stdout)
        }
        _ => bail!("Encryption backend must be age or gpg"),
    }
}

pub fn encrypt_with_passphrase(passphrase: SecretString, bytes: &[u8]) -> Result<Vec<u8>> {
    if passphrase.expose_secret().is_empty() {
        bail!("Passphrase must not be empty");
    }
    let mut output = Vec::new();
    let armor =
        age::armor::ArmoredWriter::wrap_output(&mut output, age::armor::Format::AsciiArmor)?;
    let mut writer = age::Encryptor::with_user_passphrase(passphrase).wrap_output(armor)?;
    writer.write_all(bytes)?;
    writer.finish()?.finish()?;
    Ok(output)
}

pub fn decrypt_with_passphrase(passphrase: &SecretString, bytes: &[u8]) -> Result<Vec<u8>> {
    if passphrase.expose_secret().is_empty() {
        bail!("Passphrase must not be empty");
    }
    let reader = age::armor::ArmoredReader::new(bytes);
    let decryptor = age::Decryptor::new(reader).context("Invalid age ciphertext")?;
    let age::Decryptor::Passphrase(decryptor) = decryptor else {
        bail!("Age ciphertext requires an identity, not a passphrase");
    };
    let mut output = Vec::new();
    decryptor
        .decrypt(passphrase, None)
        .context("Unable to decrypt age ciphertext; check the passphrase")?
        .read_to_end(&mut output)?;
    Ok(output)
}

pub fn decrypt(
    settings: &EncryptionSettings,
    home: &Path,
    bytes: &[u8],
    extension: &str,
) -> Result<Vec<u8>> {
    if extension == "asc" || (extension != "age" && settings.backend == "gpg") {
        let mut command = Command::new("gpg");
        command.args(["--batch", "--yes", "--decrypt"]);
        let output = process::capture(command, Some(bytes))?;
        if !output.status.success() {
            bail!(
                "gpg decryption failed ({}); check your private key and agent",
                output.status
            );
        }
        return Ok(output.stdout);
    }
    let mut identities = Vec::<age::x25519::Identity>::new();
    for path in &settings.identities {
        let path = storage::expand_home(path, home);
        for line in std::fs::read_to_string(&path)
            .with_context(|| format!("Read age identity {}", path.display()))?
            .lines()
        {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            identities.push(
                line.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid age identity in {}", path.display()))?,
            );
        }
    }
    let reader = age::armor::ArmoredReader::new(bytes);
    let decryptor = age::Decryptor::new(reader).context("Invalid age ciphertext")?;
    let mut output = Vec::new();
    match decryptor {
        age::Decryptor::Recipients(decryptor) => {
            if identities.is_empty() {
                bail!("Configure encryption.identities before decrypting age files");
            }
            decryptor
                .decrypt(identities.iter().map(|key| key as &dyn age::Identity))?
                .read_to_end(&mut output)?;
        }
        age::Decryptor::Passphrase(decryptor) => {
            let passphrase = std::env::var("STOOL_AGE_PASSPHRASE")
                .context("Set STOOL_AGE_PASSPHRASE for passphrase-encrypted age files")?;
            decryptor
                .decrypt(&age::secrecy::Secret::new(passphrase), None)?
                .read_to_end(&mut output)?;
        }
    }
    Ok(output)
}

pub fn keygen(output: &Path) -> Result<String> {
    if output.exists() {
        bail!("Identity file already exists: {}", output.display());
    }
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let identity = age::x25519::Identity::generate();
    let public = identity.to_public().to_string();
    storage::atomic_write(
        output,
        format!(
            "# public key: {public}\n{}\n",
            identity.to_string().expose_secret()
        )
        .as_bytes(),
        0o600,
    )?;
    Ok(public)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn age_roundtrip_and_wrong_identity_are_verified() {
        let temp = tempfile::tempdir().unwrap();
        let key = temp.path().join("identity");
        let public = keygen(&key).unwrap();
        let mut settings = EncryptionSettings {
            recipients: vec![public],
            identities: vec![key],
            ..Default::default()
        };
        let plain = b"private\0configuration\n";
        let cipher = encrypt(&settings, plain).unwrap();
        assert!(!cipher.windows(plain.len()).any(|part| part == plain));
        assert_eq!(
            decrypt(&settings, temp.path(), &cipher, "age").unwrap(),
            plain
        );
        let other = temp.path().join("wrong");
        keygen(&other).unwrap();
        settings.identities = vec![other];
        assert!(decrypt(&settings, temp.path(), &cipher, "age").is_err());
    }

    #[test]
    fn age_passphrase_roundtrip_preserves_binary_data_and_password_spaces() {
        let password = "  fixture's 密码 password  ";
        let plain = b"private binary configuration\0\n";
        let cipher = encrypt_with_passphrase(SecretString::new(password.into()), plain).unwrap();
        assert!(cipher.starts_with(b"-----BEGIN AGE ENCRYPTED FILE-----"));
        assert!(!cipher.windows(plain.len()).any(|part| part == plain));
        assert!(!String::from_utf8_lossy(&cipher).contains(password));
        assert_eq!(
            decrypt_with_passphrase(&SecretString::new(password.into()), &cipher).unwrap(),
            plain
        );
        let error = decrypt_with_passphrase(&SecretString::new(password.trim().into()), &cipher)
            .unwrap_err()
            .to_string();
        assert!(error.contains("Unable to decrypt age ciphertext"));
        assert!(!error.contains(password.trim()));
        assert!(!error.contains("private binary configuration"));
    }

    #[test]
    fn age_passphrase_rejects_empty_passwords_invalid_ciphertext_and_identity_mode() {
        let empty = SecretString::new(String::new());
        assert!(encrypt_with_passphrase(empty.clone(), b"private")
            .unwrap_err()
            .to_string()
            .contains("must not be empty"));
        assert!(decrypt_with_passphrase(&empty, b"not ciphertext")
            .unwrap_err()
            .to_string()
            .contains("must not be empty"));
        let password = SecretString::new("fixture correct password".into());
        assert!(decrypt_with_passphrase(&password, b"not ciphertext")
            .unwrap_err()
            .to_string()
            .contains("Invalid age ciphertext"));
        let cipher = encrypt_with_passphrase(password.clone(), b"private fixture").unwrap();
        assert!(decrypt_with_passphrase(&password, &cipher[..cipher.len() / 2]).is_err());
        let identity = age::x25519::Identity::generate();
        let settings = EncryptionSettings {
            recipients: vec![identity.to_public().to_string()],
            ..Default::default()
        };
        let cipher = encrypt(&settings, b"identity-protected fixture").unwrap();
        assert!(decrypt_with_passphrase(&password, &cipher)
            .unwrap_err()
            .to_string()
            .contains("requires an identity"));
    }
}
