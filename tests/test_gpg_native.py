"""Native GPG integration; every private key is disposable and generated here.

Run after `cargo build`, with XYREV_BIN if the executable is elsewhere.
"""
import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import sys
import tempfile
import time
import unittest

try:
    import pty
except ImportError:
    pty = None


FILES = ('public.asc', 'secret.asc.age', 'ownertrust.txt.age', 'backup-metadata.sh')


def primary_fingerprints(listing):
    result, primary = [], False
    for line in listing.decode().splitlines():
        fields = line.split(':')
        if fields[0] in ('sec', 'pub'):
            primary = True
        elif fields[0] == 'fpr' and primary:
            result.append(fields[9])
            primary = False
    return sorted(result)


class NativeGpgTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.name != 'posix':
            raise unittest.SkipTest('Real GPG integration uses Unix disposable keyrings')
        default = Path(__file__).resolve().parents[1] / 'target/debug/xyrev'
        binary = Path(os.environ.get('XYREV_BIN', str(default)))
        if not binary.is_file():
            raise unittest.SkipTest('Build xyrev or set XYREV_BIN')
        cls.binary = str(binary.resolve())
        suite = Path('/usr/local/MacGPG2/bin/gpg')
        cls.gpg_program = str(suite if suite.is_file() else shutil.which('gpg') or '')
        if not cls.gpg_program:
            raise unittest.SkipTest('Install GPG to run real disposable-key integration')
        cls.gpgconf = str(Path(cls.gpg_program).with_name('gpgconf'))

    def setUp(self):
        # Short socket paths are necessary for gpg-agent on macOS.
        self.temporary = tempfile.TemporaryDirectory(prefix='xyrev-ngpg-', dir='/tmp')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.home, self.origin = self.root / 'home', self.root / 'origin'
        self.source = self.root / "source with ' spaces"
        for directory in (self.home, self.origin, self.source):
            directory.mkdir(mode=0o700)
        self.restored = self.home / '.gnupg'
        self.env = os.environ.copy()
        self.env.update(HOME=str(self.home), USERPROFILE=str(self.home),
                        GNUPGHOME=str(self.origin), GIT_CONFIG_GLOBAL=os.devnull,
                        GIT_CONFIG_NOSYSTEM='1', GIT_ALLOW_PROTOCOL='file',
                        GIT_TERMINAL_PROMPT='0')
        self.env.pop('GIT_DIR', None)
        self.env.pop('GIT_WORK_TREE', None)
        self.env.pop('XYREV_AGE_PASSPHRASE', None)
        subprocess.run(['git', '-C', str(self.source), 'init', '--quiet'], env=self.env,
                       capture_output=True, check=True, timeout=30)
        subprocess.run(['git', '-C', str(self.source), '-c', 'user.name=Native Test',
                        '-c', 'user.email=native@example.invalid', '-c', 'commit.gpgsign=false',
                        'commit', '--quiet', '--allow-empty', '-m', 'empty source fixture'],
                       env=self.env, capture_output=True, check=True, timeout=30)
        self.addCleanup(self.kill_agent, self.origin)
        self.addCleanup(self.kill_agent, self.restored)
        self.password = "  native's synthetic password $(unexecuted)  "
        self.password_file = self.root / "private password's file"
        self.password_file.write_bytes(self.password.encode() + b'\r\n')
        self.password_file.chmod(0o600)
        self.identity = self.root / 'keys' / 'restore.agekey'
        self.identity.parent.mkdir(mode=0o700)
        self.backup_dir = self.source / '.xyrevtemplates/gpg'
        self.personal = self.generate('Native Personal <personal@example.invalid>')

    def kill_agent(self, home):
        if Path(self.gpgconf).is_file():
            subprocess.run([self.gpgconf, '--homedir', str(home), '--kill', 'all'],
                           env=dict(self.env, GNUPGHOME=str(home)),
                           capture_output=True, timeout=15)

    def gpg(self, home, *args, input=None, ok=True):
        result = subprocess.run([self.gpg_program, '--no-options', '--homedir', str(home),
                                 '--batch', '--pinentry-mode', 'loopback', *args],
                                # GPG 2.2 also passes GNUPGHOME to its agent;
                                # keep it consistent with the explicit keyring.
                                env=dict(self.env, GNUPGHOME=str(home)), input=input,
                                capture_output=True, timeout=40)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))
        else:
            self.assertNotEqual(result.returncode, 0)
        return result

    def generate(self, identity, passphrase=''):
        self.gpg(self.origin, '--passphrase', passphrase, '--quick-generate-key', identity,
                 'ed25519', 'sign', '0')
        result = self.gpg(self.origin, '--with-colons', '--list-secret-keys', identity)
        return primary_fingerprints(result.stdout)[0]

    def command(self, *args):
        return [self.binary, 'dotfile', '--source', str(self.source),
                '--destination', str(self.home), *args]

    def xyrev(self, *args, input=None, ok=True):
        result = subprocess.run(self.command(*args), env=self.env, cwd=self.root,
                                input=input, capture_output=True, timeout=90)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout.decode(errors='replace'))
            self.assertNotIn(self.password.encode(), result.stdout + result.stderr)
            self.assertNotIn(b'BEGIN PGP PRIVATE KEY BLOCK', result.stdout + result.stderr)
        return result

    def backup(self, keys, identity=False, ok=True):
        args = ['gpg', 'backup', '--gpg-program', self.gpg_program]
        for key in keys:
            args += ['--fingerprint', key]
        args += ['--identity', str(self.identity)] if identity else [
            '--passphrase-file', str(self.password_file)]
        return self.xyrev(*args, ok=ok)

    def restore(self, identity=False, ok=True):
        args = ['gpg', 'restore', '--gpg-program', self.gpg_program]
        args += ['--identity', str(self.identity)] if identity else [
            '--passphrase-file', str(self.password_file)]
        return self.xyrev(*args, ok=ok)

    def restored_keys(self):
        if not self.restored.exists():
            return []
        return primary_fingerprints(self.gpg(self.restored, '--with-colons',
                                              '--list-secret-keys').stdout)

    def assert_signing(self, keys):
        self.assertEqual(self.restored_keys(), sorted(keys))
        for key in keys:
            signed = self.gpg(self.restored, '--passphrase', '', '--local-user', key,
                              '--clearsign', input=b'native restored signing check\n').stdout
            result = self.gpg(self.restored, '--status-fd', '1', '--verify', input=signed)
            self.assertIn(b'VALIDSIG ', result.stdout)
            self.assertIn(key.encode(), result.stdout)
        trust = self.gpg(self.restored, '--export-ownertrust').stdout
        selected = sorted(line.split(b':')[0].decode() for line in trust.splitlines()
                          if line and not line.startswith(b'#'))
        self.assertEqual(selected, sorted(keys))
        for key in keys:
            self.assertIn((key + ':6:').encode(), trust)

    def legacy_backup(self, keys, identity=False, singular=False):
        """Produce the old script's format independently of native `gpg backup`."""
        keys = sorted(keys)
        self.backup_dir.mkdir(parents=True)
        public = self.gpg(self.origin, '--armor', '--export', *keys).stdout
        private = self.gpg(self.origin, '--armor', '--export-secret-keys', *keys).stdout
        trust = self.gpg(self.origin, '--export-ownertrust').stdout
        trust = b'\n'.join(line for line in trust.splitlines()
                           if line.split(b':')[0].decode() in keys) + b'\n'
        (self.backup_dir / 'public.asc').write_bytes(public)
        if identity:
            recipient = self.xyrev('keygen', '--output', str(self.identity)).stdout.decode().strip()
            encryption = ['--recipient', recipient]
        else:
            recipient = ''
            encryption = ['--passphrase-file', str(self.password_file)]
        for name, content in [('secret.asc.age', private), ('ownertrust.txt.age', trust)]:
            self.xyrev('encrypt', '-', '--backend', 'age', *encryption,
                       '--output', str(self.backup_dir / name), input=content)
        metadata = "# Generated by the legacy shell producer.\n"
        if not identity:
            metadata += "gpg_backup_mode='passphrase'\n"
        metadata += "gpg_backup_fingerprint='" + keys[0] + "'\n"
        if not singular:
            metadata += "gpg_backup_fingerprints='" + ' '.join(keys) + "'\n"
        if identity:
            metadata += "gpg_backup_recipient='" + recipient + "'\n"
        for field, filename in [('public', 'public.asc'), ('secret', 'secret.asc.age'),
                                ('ownertrust', 'ownertrust.txt.age')]:
            digest = hashlib.sha256((self.backup_dir / filename).read_bytes()).hexdigest()
            metadata += 'gpg_backup_' + field + "_sha256='" + digest + "'\n"
        (self.backup_dir / 'backup-metadata.sh').write_text(metadata)

    def initialize_repository(self):
        (self.source / 'dot_seed').write_text('managed seed\n')
        settings = ('enabled=true\ndirectory=".xyrevtemplates/gpg"\nprogram=' +
                    json.dumps(self.gpg_program) + '\npassphrase_file=' +
                    json.dumps(str(self.password_file)) + '\n')
        (self.source / '.xyrevgpg.toml').write_text(settings)
        def git(*args):
            subprocess.run(['git', '-C', str(self.source), *args], env=self.env,
                           capture_output=True, check=True, timeout=30)
        git('init', '--quiet')
        git('-c', 'user.name=Native Test', '-c', 'user.email=native@example.invalid',
            '-c', 'commit.gpgsign=false', 'add', '--all')
        git('-c', 'user.name=Native Test', '-c', 'user.email=native@example.invalid',
            '-c', 'commit.gpgsign=false', 'commit', '--quiet', '-m', 'legacy fixture')
        upstream = self.source
        self.source = self.root / 'cloned source'
        self.backup_dir = self.source / '.xyrevtemplates/gpg'
        self.xyrev('init', upstream.as_uri())

    def test_native_multiple_key_backup_restores_and_signs_in_both_modes(self):
        work = self.generate('Native Work <work@example.invalid>')
        other = self.generate('Native Unselected <unselected@example.invalid>')
        keys = sorted([self.personal, work])
        for identity in (True, False):
            with self.subTest(identity=identity):
                result = self.backup([work.lower(), self.personal, work], identity=identity)
                self.assertNotIn(self.password.encode(), result.stdout + result.stderr)
                metadata = (self.backup_dir / 'backup-metadata.sh').read_text()
                self.assertIn("gpg_backup_fingerprints='" + ' '.join(keys) + "'", metadata)
                self.assertNotIn(other, metadata)
                for name in FILES:
                    content = (self.backup_dir / name).read_bytes()
                    self.assertNotIn(b'BEGIN PGP PRIVATE KEY BLOCK', content)
                    self.assertNotIn(b'AGE-SECRET-KEY-', content)
                    self.assertNotIn(self.password.encode(), content)
                self.restore(identity=identity)
                self.assert_signing(keys)
                if identity:
                    self.kill_agent(self.restored)
                    shutil.rmtree(self.restored)

    def test_existing_password_bundle_auto_restores_without_reencrypting(self):
        work = self.generate('Legacy Work <legacy-work@example.invalid>')
        keys = [self.personal, work]
        self.legacy_backup(keys)
        before = {name: (self.backup_dir / name).read_bytes() for name in FILES}
        self.initialize_repository()
        for command in [('diff',), ('status',), ('apply', '--dry-run')]:
            result = self.xyrev(*command)
            self.assertNotIn(b'BEGIN PGP PRIVATE KEY BLOCK', result.stdout)
        self.assertEqual(self.restored_keys(), [])
        self.xyrev('apply')
        self.assert_signing(keys)
        self.assertEqual({name: (self.backup_dir / name).read_bytes() for name in FILES}, before)
        state = list((self.home / '.xyrev/dotfile/state').glob('*.json'))
        self.assertEqual(len(state), 1)
        previous = state[0].read_bytes()
        self.xyrev('apply')
        self.xyrev('verify')
        self.assertEqual(state[0].read_bytes(), previous)

    def test_legacy_identity_singular_without_mode_still_restores(self):
        self.legacy_backup([self.personal], identity=True, singular=True)
        self.restore(identity=True)
        self.assert_signing([self.personal])

    def test_legacy_password_singular_and_wrong_password_retry(self):
        self.legacy_backup([self.personal], singular=True)
        self.initialize_repository()
        self.password_file.write_text('incorrect synthetic password')
        self.xyrev('apply', ok=False)
        self.assertEqual(self.restored_keys(), [])
        self.password_file.write_bytes(self.password.encode() + b'\r\n')
        self.xyrev('apply')
        self.assert_signing([self.personal])

    def test_missing_private_work_key_does_not_replace_a_complete_backup(self):
        work = self.generate('Native Work Missing <missing@example.invalid>')
        self.backup([self.personal, work])
        before = {name: (self.backup_dir / name).read_bytes() for name in FILES}
        self.backup([self.personal], ok=False)
        self.gpg(self.origin, '--yes', '--delete-secret-key', work)
        self.backup([self.personal, work], ok=False)
        self.assertEqual({name: (self.backup_dir / name).read_bytes() for name in FILES}, before)

    def test_protected_gpg_key_uses_pinentry_without_sending_its_passphrase_to_xyrev(self):
        # These are generated test credentials, never the user's GPG passphrase.
        original = 'synthetic-original-gpg-key-password'
        key = self.generate('Protected Work <protected@example.invalid>', original)
        marker, pinentry = self.root / 'pinentry-requested', self.root / 'pinentry-fixture'
        pinentry.write_text('#!' + sys.executable + '\n' +
                            'import pathlib, sys\n'
                            'print("OK disposable pinentry", flush=True)\n'
                            'for command in sys.stdin:\n'
                            '    if command.startswith("GETPIN"):\n'
                            '        pathlib.Path(' + repr(str(marker)) + ').touch()\n'
                            '        print("D " + ' + repr(original) + ', flush=True)\n'
                            '    print("OK", flush=True)\n'
                            '    if command.startswith("BYE"): break\n')
        pinentry.chmod(0o700)
        (self.origin / 'gpg-agent.conf').write_text('pinentry-program ' + str(pinentry) + '\n')
        self.kill_agent(self.origin)  # Ensure key generation's unlock cache is gone.
        result = self.backup([self.personal, key])
        self.assertTrue(marker.is_file(), 'The protected key must really ask pinentry to unlock')
        self.assertNotIn(original.encode(), result.stdout + result.stderr)
        self.restore()
        self.assertEqual(self.restored_keys(), sorted([self.personal, key]))
        self.kill_agent(self.restored)
        self.gpg(self.restored, '--passphrase', 'wrong-original-test-password',
                 '--local-user', key, '--clearsign',
                 input=b'protected restored signing check\n', ok=False)
        signed = self.gpg(self.restored, '--passphrase', original, '--local-user', key,
                          '--clearsign', input=b'protected restored signing check\n').stdout
        self.gpg(self.restored, '--verify', input=signed)

    def terminal(self, command, answers):
        child, terminal = pty.fork()
        if child == 0:
            os.chdir(self.root)
            os.execve(command[0], command, self.env)
        output, sent, status = bytearray(), set(), None
        try:
            deadline = time.monotonic() + 90
            while time.monotonic() < deadline:
                readable, _, _ = select.select([terminal], [], [], 0.2)
                if readable:
                    try:
                        chunk = os.read(terminal, 4096)
                    except OSError:
                        break
                    if not chunk:
                        break
                    output.extend(chunk)
                    for prompt, answer in answers:
                        if prompt in output and prompt not in sent:
                            # The terminal reader enables echo suppression just
                            # after printing its prompt.
                            time.sleep(0.05)
                            os.write(terminal, answer + b'\n')
                            sent.add(prompt)
                ended, code = os.waitpid(child, os.WNOHANG)
                if ended:
                    status = code
                    break
            if status is None:
                ended, code = os.waitpid(child, os.WNOHANG)
                if ended:
                    status = code
        finally:
            os.close(terminal)
            if status is None:
                os.kill(child, 15)
                _, status = os.waitpid(child, 0)
        self.assertEqual(len(sent), len(answers), output.decode(errors='replace'))
        for _, answer in answers:
            self.assertNotIn(answer, output)
        return os.waitstatus_to_exitcode(status), bytes(output)

    @unittest.skipUnless(pty is not None and hasattr(pty, 'fork'), 'Unix terminal required')
    def test_native_password_prompts_are_hidden_confirmed_and_restore_asks_once(self):
        backup = self.command('gpg', 'backup', '--fingerprint', self.personal,
                              '--gpg-program', self.gpg_program)
        password, confirm = b'GPG backup password: ', b'Confirm backup password: '
        code, _ = self.terminal(backup, [(password, b'first wrong value'),
                                        (confirm, b'second wrong value')])
        self.assertNotEqual(code, 0)
        self.assertFalse((self.backup_dir / 'secret.asc.age').exists())
        code, output = self.terminal(backup, [(password, self.password.encode()),
                                            (confirm, self.password.encode())])
        self.assertEqual(code, 0, output.decode(errors='replace'))
        restore = self.command('gpg', 'restore', '--gpg-program', self.gpg_program)
        code, output = self.terminal(restore, [(password, self.password.encode())])
        self.assertEqual(code, 0, output.decode(errors='replace'))
        self.assertEqual(output.count(password), 1)
        self.assert_signing([self.personal])


if __name__ == '__main__':
    unittest.main()
