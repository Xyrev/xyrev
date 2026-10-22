
use builtin;
use str;

set edit:completion:arg-completer[xyrev] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'xyrev'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'xyrev'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand init 'Create a new configuration folder or reinitialize the existing one'
            cand agent 'agent'
            cand apple 'apple'
            cand dotfile 'Manage dotfiles from a Git repository using chezmoi-style names'
            cand completions 'Generate shell completions from the current command definitions'
            cand man 'Render the manual, or write all command manuals to a directory'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'xyrev;init'= {
            cand -f 'Force reinitialize configuration (overwrite existing)'
            cand --force 'Force reinitialize configuration (overwrite existing)'
            cand -C 'Clean and reinitialize (remove all existing data)'
            cand --clean 'Clean and reinitialize (remove all existing data)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;agent'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
            cand init 'init'
            cand copilot 'copilot'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'xyrev;agent;init'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;agent;copilot'= {
            cand -m 'Install method (auto, brew, npm, winget, script)'
            cand --method 'Install method (auto, brew, npm, winget, script)'
            cand --version 'Specific version to install (e.g., v0.0.369)'
            cand --prefix 'Custom installation prefix (for script method)'
            cand --prerelease 'Install prerelease version'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;agent;help'= {
            cand init 'init'
            cand copilot 'copilot'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'xyrev;agent;help;init'= {
        }
        &'xyrev;agent;help;copilot'= {
        }
        &'xyrev;agent;help;help'= {
        }
        &'xyrev;apple'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
            cand init 'Clone a Git URL and save configuration'
            cand diff 'Preview content and permission changes without writing'
            cand status 'List pending changes and conflicts'
            cand apply 'Deploy the local source repository'
            cand update 'Fast-forward the source repository and deploy changes'
            cand managed 'List managed destination paths'
            cand add 'Import existing destination files into source state'
            cand edit 'Edit source files (encrypted files are decrypted only in a private temporary directory)'
            cand unmanaged 'List destination files not managed by source'
            cand verify 'Return nonzero when the destination differs from source'
            cand doctor 'Diagnose repository, configuration, state and optional executables'
            cand archive 'Export rendered target files as tar, tar.gz, or zip'
            cand execute-template 'Evaluate a Jinja template using repository and local data'
            cand data 'Print merged template data as JSON'
            cand encrypt 'Encrypt a file using configured age or gpg'
            cand decrypt 'Decrypt a file using configured identities'
            cand gpg 'Back up or restore GPG keys using encrypted dotfile storage'
            cand keygen 'Generate an age identity and print its public recipient'
            cand cd 'Start a shell in the source repository, or print its path'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'xyrev;dotfile;init'= {
            cand --branch 'branch'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --apply 'Deploy after cloning'
            cand --recurse-submodules 'recurse-submodules'
            cand --builtin 'Clone using the built-in Git backend'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;diff'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;status'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;apply'= {
            cand --prompt 'Interactive conflict handling (auto only prompts on a terminal)'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --dry-run 'dry-run'
            cand --force 'Back up and overwrite conflicting destination entries'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;update'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;managed'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;add'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --template 'template'
            cand --encrypt 'encrypt'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;edit'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --apply 'apply'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;unmanaged'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;verify'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;doctor'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;archive'= {
            cand --format 'format'
            cand -o 'o'
            cand --output 'output'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;execute-template'= {
            cand --file 'file'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --init 'init'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;data'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;encrypt'= {
            cand -o 'o'
            cand --output 'output'
            cand --backend 'backend'
            cand --recipient 'recipient'
            cand --identity 'identity'
            cand --passphrase-file 'Read one passphrase line from a file or pipe (such as /dev/fd/3)'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --passphrase 'Use age passphrase encryption; read the password privately from the terminal'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;decrypt'= {
            cand -o 'o'
            cand --output 'output'
            cand --backend 'backend'
            cand --recipient 'recipient'
            cand --identity 'identity'
            cand --passphrase-file 'Read one passphrase line from a file or pipe (such as /dev/fd/3)'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --passphrase 'Use age passphrase encryption; read the password privately from the terminal'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;gpg'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
            cand backup 'Export selected GPG keys and ownertrust into an encrypted backup'
            cand restore 'Validate and restore an encrypted GPG key backup'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'xyrev;dotfile;gpg;backup'= {
            cand --fingerprint 'Complete primary GPG fingerprint; repeat to select multiple keys'
            cand --output 'Backup directory (defaults to .xyrevtemplates/gpg in the source)'
            cand --identity 'Separate age identity kept outside the repository'
            cand --passphrase-file 'Read the password from private external storage or an inherited pipe'
            cand --gpg-program 'GPG executable (GPG Suite is detected on macOS)'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;gpg;restore'= {
            cand --input 'Backup directory (defaults to .xyrevtemplates/gpg in the source)'
            cand --identity 'Separate age identity for identity-encrypted backups'
            cand --passphrase-file 'Read the password from a private file outside the repository'
            cand --gpg-program 'GPG executable (GPG Suite is detected on macOS)'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;gpg;help'= {
            cand backup 'Export selected GPG keys and ownertrust into an encrypted backup'
            cand restore 'Validate and restore an encrypted GPG key backup'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'xyrev;dotfile;gpg;help;backup'= {
        }
        &'xyrev;dotfile;gpg;help;restore'= {
        }
        &'xyrev;dotfile;gpg;help;help'= {
        }
        &'xyrev;dotfile;keygen'= {
            cand --output 'output'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;cd'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --print 'print'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile;help'= {
            cand init 'Clone a Git URL and save configuration'
            cand diff 'Preview content and permission changes without writing'
            cand status 'List pending changes and conflicts'
            cand apply 'Deploy the local source repository'
            cand update 'Fast-forward the source repository and deploy changes'
            cand managed 'List managed destination paths'
            cand add 'Import existing destination files into source state'
            cand edit 'Edit source files (encrypted files are decrypted only in a private temporary directory)'
            cand unmanaged 'List destination files not managed by source'
            cand verify 'Return nonzero when the destination differs from source'
            cand doctor 'Diagnose repository, configuration, state and optional executables'
            cand archive 'Export rendered target files as tar, tar.gz, or zip'
            cand execute-template 'Evaluate a Jinja template using repository and local data'
            cand data 'Print merged template data as JSON'
            cand encrypt 'Encrypt a file using configured age or gpg'
            cand decrypt 'Decrypt a file using configured identities'
            cand gpg 'Back up or restore GPG keys using encrypted dotfile storage'
            cand keygen 'Generate an age identity and print its public recipient'
            cand cd 'Start a shell in the source repository, or print its path'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'xyrev;dotfile;help;init'= {
        }
        &'xyrev;dotfile;help;diff'= {
        }
        &'xyrev;dotfile;help;status'= {
        }
        &'xyrev;dotfile;help;apply'= {
        }
        &'xyrev;dotfile;help;update'= {
        }
        &'xyrev;dotfile;help;managed'= {
        }
        &'xyrev;dotfile;help;add'= {
        }
        &'xyrev;dotfile;help;edit'= {
        }
        &'xyrev;dotfile;help;unmanaged'= {
        }
        &'xyrev;dotfile;help;verify'= {
        }
        &'xyrev;dotfile;help;doctor'= {
        }
        &'xyrev;dotfile;help;archive'= {
        }
        &'xyrev;dotfile;help;execute-template'= {
        }
        &'xyrev;dotfile;help;data'= {
        }
        &'xyrev;dotfile;help;encrypt'= {
        }
        &'xyrev;dotfile;help;decrypt'= {
        }
        &'xyrev;dotfile;help;gpg'= {
            cand backup 'Export selected GPG keys and ownertrust into an encrypted backup'
            cand restore 'Validate and restore an encrypted GPG key backup'
        }
        &'xyrev;dotfile;help;gpg;backup'= {
        }
        &'xyrev;dotfile;help;gpg;restore'= {
        }
        &'xyrev;dotfile;help;keygen'= {
        }
        &'xyrev;dotfile;help;cd'= {
        }
        &'xyrev;dotfile;help;help'= {
        }
        &'xyrev;completions'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;man'= {
            cand --output 'output'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;help'= {
            cand init 'Create a new configuration folder or reinitialize the existing one'
            cand agent 'agent'
            cand apple 'apple'
            cand dotfile 'Manage dotfiles from a Git repository using chezmoi-style names'
            cand completions 'Generate shell completions from the current command definitions'
            cand man 'Render the manual, or write all command manuals to a directory'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'xyrev;help;init'= {
        }
        &'xyrev;help;agent'= {
            cand init 'init'
            cand copilot 'copilot'
        }
        &'xyrev;help;agent;init'= {
        }
        &'xyrev;help;agent;copilot'= {
        }
        &'xyrev;help;apple'= {
        }
        &'xyrev;help;dotfile'= {
            cand init 'Clone a Git URL and save configuration'
            cand diff 'Preview content and permission changes without writing'
            cand status 'List pending changes and conflicts'
            cand apply 'Deploy the local source repository'
            cand update 'Fast-forward the source repository and deploy changes'
            cand managed 'List managed destination paths'
            cand add 'Import existing destination files into source state'
            cand edit 'Edit source files (encrypted files are decrypted only in a private temporary directory)'
            cand unmanaged 'List destination files not managed by source'
            cand verify 'Return nonzero when the destination differs from source'
            cand doctor 'Diagnose repository, configuration, state and optional executables'
            cand archive 'Export rendered target files as tar, tar.gz, or zip'
            cand execute-template 'Evaluate a Jinja template using repository and local data'
            cand data 'Print merged template data as JSON'
            cand encrypt 'Encrypt a file using configured age or gpg'
            cand decrypt 'Decrypt a file using configured identities'
            cand gpg 'Back up or restore GPG keys using encrypted dotfile storage'
            cand keygen 'Generate an age identity and print its public recipient'
            cand cd 'Start a shell in the source repository, or print its path'
        }
        &'xyrev;help;dotfile;init'= {
        }
        &'xyrev;help;dotfile;diff'= {
        }
        &'xyrev;help;dotfile;status'= {
        }
        &'xyrev;help;dotfile;apply'= {
        }
        &'xyrev;help;dotfile;update'= {
        }
        &'xyrev;help;dotfile;managed'= {
        }
        &'xyrev;help;dotfile;add'= {
        }
        &'xyrev;help;dotfile;edit'= {
        }
        &'xyrev;help;dotfile;unmanaged'= {
        }
        &'xyrev;help;dotfile;verify'= {
        }
        &'xyrev;help;dotfile;doctor'= {
        }
        &'xyrev;help;dotfile;archive'= {
        }
        &'xyrev;help;dotfile;execute-template'= {
        }
        &'xyrev;help;dotfile;data'= {
        }
        &'xyrev;help;dotfile;encrypt'= {
        }
        &'xyrev;help;dotfile;decrypt'= {
        }
        &'xyrev;help;dotfile;gpg'= {
            cand backup 'Export selected GPG keys and ownertrust into an encrypted backup'
            cand restore 'Validate and restore an encrypted GPG key backup'
        }
        &'xyrev;help;dotfile;gpg;backup'= {
        }
        &'xyrev;help;dotfile;gpg;restore'= {
        }
        &'xyrev;help;dotfile;keygen'= {
        }
        &'xyrev;help;dotfile;cd'= {
        }
        &'xyrev;help;completions'= {
        }
        &'xyrev;help;man'= {
        }
        &'xyrev;help;help'= {
        }
    ]
    $completions[$command]
}
