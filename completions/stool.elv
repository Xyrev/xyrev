
use builtin;
use str;

set edit:completion:arg-completer[stool] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'stool'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'stool'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand init 'Create a new configuration folder or reinitialize the existing one'
            cand agent 'agent'
            cand apple 'apple'
            cand dotfile 'Manage dotfiles from a Git repository'
            cand completions 'Generate shell completions from the current command definitions'
            cand man 'Render the manual, or write all command manuals to a directory'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'stool;init'= {
            cand -f 'Force reinitialize configuration (overwrite existing)'
            cand --force 'Force reinitialize configuration (overwrite existing)'
            cand -C 'Clean and reinitialize (remove all existing data)'
            cand --clean 'Clean and reinitialize (remove all existing data)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;agent'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
            cand init 'init'
            cand copilot 'copilot'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'stool;agent;init'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;agent;copilot'= {
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
        &'stool;agent;help'= {
            cand init 'init'
            cand copilot 'copilot'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'stool;agent;help;init'= {
        }
        &'stool;agent;help;copilot'= {
        }
        &'stool;agent;help;help'= {
        }
        &'stool;apple'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
            cand init 'Clone a Git URL and save configuration'
            cand migrate 'Explicitly copy legacy xyrev state into stool without deploying files'
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
            cand gpg 'Back up or restore GPG keys using encrypted dotfile storage'
            cand keygen 'Generate an age identity and print its public recipient'
            cand cd 'Start a shell in the source repository, or print its path'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'stool;dotfile;init'= {
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
        &'stool;dotfile;migrate'= {
            cand --from 'Previous control directory (defaults to ~/.xyrev/dotfile)'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --dry-run 'Inspect the migration without writing files or running hooks'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;diff'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;status'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;apply'= {
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
        &'stool;dotfile;update'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;managed'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;add'= {
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
        &'stool;dotfile;edit'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --apply 'apply'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;unmanaged'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;verify'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;doctor'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;archive'= {
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
        &'stool;dotfile;execute-template'= {
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
        &'stool;dotfile;data'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;gpg'= {
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
        &'stool;dotfile;gpg;backup'= {
            cand --fingerprint 'Complete primary GPG fingerprint; repeat to select multiple keys'
            cand --output 'Backup directory (defaults to .stooltemplates/gpg in the source)'
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
        &'stool;dotfile;gpg;restore'= {
            cand --input 'Backup directory (defaults to .stooltemplates/gpg in the source)'
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
        &'stool;dotfile;gpg;help'= {
            cand backup 'Export selected GPG keys and ownertrust into an encrypted backup'
            cand restore 'Validate and restore an encrypted GPG key backup'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'stool;dotfile;gpg;help;backup'= {
        }
        &'stool;dotfile;gpg;help;restore'= {
        }
        &'stool;dotfile;gpg;help;help'= {
        }
        &'stool;dotfile;keygen'= {
            cand --output 'output'
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;cd'= {
            cand --source 'Local source repository (saved by init)'
            cand --destination 'Destination directory (defaults to the user''s home)'
            cand --data 'Override template data (key=value; JSON values are accepted)'
            cand --print 'print'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;dotfile;help'= {
            cand init 'Clone a Git URL and save configuration'
            cand migrate 'Explicitly copy legacy xyrev state into stool without deploying files'
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
            cand gpg 'Back up or restore GPG keys using encrypted dotfile storage'
            cand keygen 'Generate an age identity and print its public recipient'
            cand cd 'Start a shell in the source repository, or print its path'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'stool;dotfile;help;init'= {
        }
        &'stool;dotfile;help;migrate'= {
        }
        &'stool;dotfile;help;diff'= {
        }
        &'stool;dotfile;help;status'= {
        }
        &'stool;dotfile;help;apply'= {
        }
        &'stool;dotfile;help;update'= {
        }
        &'stool;dotfile;help;managed'= {
        }
        &'stool;dotfile;help;add'= {
        }
        &'stool;dotfile;help;edit'= {
        }
        &'stool;dotfile;help;unmanaged'= {
        }
        &'stool;dotfile;help;verify'= {
        }
        &'stool;dotfile;help;doctor'= {
        }
        &'stool;dotfile;help;archive'= {
        }
        &'stool;dotfile;help;execute-template'= {
        }
        &'stool;dotfile;help;data'= {
        }
        &'stool;dotfile;help;gpg'= {
            cand backup 'Export selected GPG keys and ownertrust into an encrypted backup'
            cand restore 'Validate and restore an encrypted GPG key backup'
        }
        &'stool;dotfile;help;gpg;backup'= {
        }
        &'stool;dotfile;help;gpg;restore'= {
        }
        &'stool;dotfile;help;keygen'= {
        }
        &'stool;dotfile;help;cd'= {
        }
        &'stool;dotfile;help;help'= {
        }
        &'stool;completions'= {
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;man'= {
            cand --output 'output'
            cand -v 'Show more debugging information'
            cand --verbose 'Show more debugging information'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'stool;help'= {
            cand init 'Create a new configuration folder or reinitialize the existing one'
            cand agent 'agent'
            cand apple 'apple'
            cand dotfile 'Manage dotfiles from a Git repository'
            cand completions 'Generate shell completions from the current command definitions'
            cand man 'Render the manual, or write all command manuals to a directory'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'stool;help;init'= {
        }
        &'stool;help;agent'= {
            cand init 'init'
            cand copilot 'copilot'
        }
        &'stool;help;agent;init'= {
        }
        &'stool;help;agent;copilot'= {
        }
        &'stool;help;apple'= {
        }
        &'stool;help;dotfile'= {
            cand init 'Clone a Git URL and save configuration'
            cand migrate 'Explicitly copy legacy xyrev state into stool without deploying files'
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
            cand gpg 'Back up or restore GPG keys using encrypted dotfile storage'
            cand keygen 'Generate an age identity and print its public recipient'
            cand cd 'Start a shell in the source repository, or print its path'
        }
        &'stool;help;dotfile;init'= {
        }
        &'stool;help;dotfile;migrate'= {
        }
        &'stool;help;dotfile;diff'= {
        }
        &'stool;help;dotfile;status'= {
        }
        &'stool;help;dotfile;apply'= {
        }
        &'stool;help;dotfile;update'= {
        }
        &'stool;help;dotfile;managed'= {
        }
        &'stool;help;dotfile;add'= {
        }
        &'stool;help;dotfile;edit'= {
        }
        &'stool;help;dotfile;unmanaged'= {
        }
        &'stool;help;dotfile;verify'= {
        }
        &'stool;help;dotfile;doctor'= {
        }
        &'stool;help;dotfile;archive'= {
        }
        &'stool;help;dotfile;execute-template'= {
        }
        &'stool;help;dotfile;data'= {
        }
        &'stool;help;dotfile;gpg'= {
            cand backup 'Export selected GPG keys and ownertrust into an encrypted backup'
            cand restore 'Validate and restore an encrypted GPG key backup'
        }
        &'stool;help;dotfile;gpg;backup'= {
        }
        &'stool;help;dotfile;gpg;restore'= {
        }
        &'stool;help;dotfile;keygen'= {
        }
        &'stool;help;dotfile;cd'= {
        }
        &'stool;help;completions'= {
        }
        &'stool;help;man'= {
        }
        &'stool;help;help'= {
        }
    ]
    $completions[$command]
}
