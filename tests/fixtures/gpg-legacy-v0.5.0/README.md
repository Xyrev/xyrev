# Public test-only legacy GPG backups

Every key, password and age identity in this directory is a **public test
credential**. The two OpenPGP identities are `XYREV TEST ONLY Legacy Personal
<legacy-personal@example.invalid>` and `XYREV TEST ONLY Legacy Work
<legacy-work@example.invalid>`. Never use these keys for real identities or
protect real data with these credentials. No user keys were read or exported.

These fixed bundles are an independent compatibility anchor for native GPG
restore. The tests copy them without invoking either a generic encryption CLI
or native GPG backup, then restore and sign using disposable test keyrings.

| Bundle | Legacy metadata case |
| --- | --- |
| `password-plural/` | Password encryption, singular and plural fields, two primary keys |
| `password-singular/` | Password encryption, only the old singular fingerprint field |
| `identity-singular/` | Identity encryption, singular field, omitted mode (legacy default) |

Each bundle contains `public.asc`, `secret.asc.age`, `ownertrust.txt.age` and
`backup-metadata.sh`. The last file contains literal metadata, not executable
fixture setup. `test-credentials.json` records the known password and selected
fingerprints; `test-only.agekey` is the public test identity for the identity
case. Tests copy that identity outside the source with private permissions.
The local `.gitattributes` disables checkout newline conversion so the recorded
hashes and ciphertext remain byte-identical.

The fixtures were generated once in an isolated `/tmp` HOME and GNUPGHOME:

1. GnuPG generated two disposable Ed25519 signing keys, without key passwords,
   using a fixed 2024-01-01 creation time and no expiration. Public/private
   exports and selected `:6:` ownertrust rows were captured in memory.
2. The identity fixture used the already released xyrev **v0.5.0** generic age
   producer. The password fixtures used the unmodified
   `crypto::encrypt_with_passphrase` function from the same release commit,
   compiled without optimization against its locked age 0.10.1 dependency.
   This avoids a release producer's higher automatically calibrated scrypt
   cost exceeding the debug test binary's default decryption work limit.
   The fixture work factors are recorded in `test-credentials.json`.
3. Neither producer used native `gpg backup`; the single-quoted legacy metadata
   and SHA-256 fields were assembled independently. No private export was
   written to an intermediate plaintext file.
4. All generated GPG agents were stopped and the temporary keyring removed.

The producer came from the verified `xyrev-v0.5.0-aarch64-apple-darwin.tar.gz`
release archive, SHA-256
`d02572d9b216c9d7aea83a55b4af5c07ad268e589f5108ee52b6ee8a4941b2b2`.
The matching v0.5.0 source commit is
`1ab04c1b7dbe0af7a0b3ad56b8a837f507d29a2e`.
Neither producer is needed to run these tests, nor is a network connection or
an additional age executable. Preserve these bytes when changing native backup
format so the old-format compatibility check remains independent. These known
test credentials and work factors are not guidance for protecting real data.
