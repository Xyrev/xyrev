
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'xyrev' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'xyrev'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'xyrev' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Create a new configuration folder or reinitialize the existing one')
            [CompletionResult]::new('agent', 'agent', [CompletionResultType]::ParameterValue, 'agent')
            [CompletionResult]::new('apple', 'apple', [CompletionResultType]::ParameterValue, 'apple')
            [CompletionResult]::new('dotfile', 'dotfile', [CompletionResultType]::ParameterValue, 'Manage dotfiles from a Git repository using chezmoi-style names')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completions from the current command definitions')
            [CompletionResult]::new('man', 'man', [CompletionResultType]::ParameterValue, 'Render the manual, or write all command manuals to a directory')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'xyrev;init' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Force reinitialize configuration (overwrite existing)')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Force reinitialize configuration (overwrite existing)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Clean and reinitialize (remove all existing data)')
            [CompletionResult]::new('--clean', '--clean', [CompletionResultType]::ParameterName, 'Clean and reinitialize (remove all existing data)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;agent' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'init')
            [CompletionResult]::new('copilot', 'copilot', [CompletionResultType]::ParameterValue, 'copilot')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'xyrev;agent;init' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;agent;copilot' {
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'Install method (auto, brew, npm, winget, script)')
            [CompletionResult]::new('--method', '--method', [CompletionResultType]::ParameterName, 'Install method (auto, brew, npm, winget, script)')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Specific version to install (e.g., v0.0.369)')
            [CompletionResult]::new('--prefix', '--prefix', [CompletionResultType]::ParameterName, 'Custom installation prefix (for script method)')
            [CompletionResult]::new('--prerelease', '--prerelease', [CompletionResultType]::ParameterName, 'Install prerelease version')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;agent;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'init')
            [CompletionResult]::new('copilot', 'copilot', [CompletionResultType]::ParameterValue, 'copilot')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'xyrev;agent;help;init' {
            break
        }
        'xyrev;agent;help;copilot' {
            break
        }
        'xyrev;agent;help;help' {
            break
        }
        'xyrev;apple' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Clone a Git URL and save configuration')
            [CompletionResult]::new('diff', 'diff', [CompletionResultType]::ParameterValue, 'Preview content and permission changes without writing')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'List pending changes and conflicts')
            [CompletionResult]::new('apply', 'apply', [CompletionResultType]::ParameterValue, 'Deploy the local source repository')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Fast-forward the source repository and deploy changes')
            [CompletionResult]::new('managed', 'managed', [CompletionResultType]::ParameterValue, 'List managed destination paths')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Import existing destination files into source state')
            [CompletionResult]::new('edit', 'edit', [CompletionResultType]::ParameterValue, 'Edit source files (encrypted files are decrypted only in a private temporary directory)')
            [CompletionResult]::new('unmanaged', 'unmanaged', [CompletionResultType]::ParameterValue, 'List destination files not managed by source')
            [CompletionResult]::new('verify', 'verify', [CompletionResultType]::ParameterValue, 'Return nonzero when the destination differs from source')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Diagnose repository, configuration, state and optional executables')
            [CompletionResult]::new('archive', 'archive', [CompletionResultType]::ParameterValue, 'Export rendered target files as tar, tar.gz, or zip')
            [CompletionResult]::new('execute-template', 'execute-template', [CompletionResultType]::ParameterValue, 'Evaluate a Jinja template using repository and local data')
            [CompletionResult]::new('data', 'data', [CompletionResultType]::ParameterValue, 'Print merged template data as JSON')
            [CompletionResult]::new('encrypt', 'encrypt', [CompletionResultType]::ParameterValue, 'Encrypt a file using configured age or gpg')
            [CompletionResult]::new('decrypt', 'decrypt', [CompletionResultType]::ParameterValue, 'Decrypt a file using configured identities')
            [CompletionResult]::new('keygen', 'keygen', [CompletionResultType]::ParameterValue, 'Generate an age identity and print its public recipient')
            [CompletionResult]::new('cd', 'cd', [CompletionResultType]::ParameterValue, 'Start a shell in the source repository, or print its path')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'xyrev;dotfile;init' {
            [CompletionResult]::new('--branch', '--branch', [CompletionResultType]::ParameterName, 'branch')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('--apply', '--apply', [CompletionResultType]::ParameterName, 'Deploy after cloning')
            [CompletionResult]::new('--recurse-submodules', '--recurse-submodules', [CompletionResultType]::ParameterName, 'recurse-submodules')
            [CompletionResult]::new('--builtin', '--builtin', [CompletionResultType]::ParameterName, 'Clone using the built-in Git backend')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;diff' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;status' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;apply' {
            [CompletionResult]::new('--prompt', '--prompt', [CompletionResultType]::ParameterName, 'Interactive conflict handling (auto only prompts on a terminal)')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'dry-run')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Back up and overwrite conflicting destination entries')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;update' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;managed' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;add' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('--template', '--template', [CompletionResultType]::ParameterName, 'template')
            [CompletionResult]::new('--encrypt', '--encrypt', [CompletionResultType]::ParameterName, 'encrypt')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;edit' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('--apply', '--apply', [CompletionResultType]::ParameterName, 'apply')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;unmanaged' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;verify' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;doctor' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;archive' {
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'format')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'o')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'output')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;execute-template' {
            [CompletionResult]::new('--file', '--file', [CompletionResultType]::ParameterName, 'file')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('--init', '--init', [CompletionResultType]::ParameterName, 'init')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;data' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;encrypt' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'o')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'output')
            [CompletionResult]::new('--backend', '--backend', [CompletionResultType]::ParameterName, 'backend')
            [CompletionResult]::new('--recipient', '--recipient', [CompletionResultType]::ParameterName, 'recipient')
            [CompletionResult]::new('--identity', '--identity', [CompletionResultType]::ParameterName, 'identity')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;decrypt' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'o')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'output')
            [CompletionResult]::new('--backend', '--backend', [CompletionResultType]::ParameterName, 'backend')
            [CompletionResult]::new('--recipient', '--recipient', [CompletionResultType]::ParameterName, 'recipient')
            [CompletionResult]::new('--identity', '--identity', [CompletionResultType]::ParameterName, 'identity')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;keygen' {
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'output')
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;cd' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('--print', '--print', [CompletionResultType]::ParameterName, 'print')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Clone a Git URL and save configuration')
            [CompletionResult]::new('diff', 'diff', [CompletionResultType]::ParameterValue, 'Preview content and permission changes without writing')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'List pending changes and conflicts')
            [CompletionResult]::new('apply', 'apply', [CompletionResultType]::ParameterValue, 'Deploy the local source repository')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Fast-forward the source repository and deploy changes')
            [CompletionResult]::new('managed', 'managed', [CompletionResultType]::ParameterValue, 'List managed destination paths')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Import existing destination files into source state')
            [CompletionResult]::new('edit', 'edit', [CompletionResultType]::ParameterValue, 'Edit source files (encrypted files are decrypted only in a private temporary directory)')
            [CompletionResult]::new('unmanaged', 'unmanaged', [CompletionResultType]::ParameterValue, 'List destination files not managed by source')
            [CompletionResult]::new('verify', 'verify', [CompletionResultType]::ParameterValue, 'Return nonzero when the destination differs from source')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Diagnose repository, configuration, state and optional executables')
            [CompletionResult]::new('archive', 'archive', [CompletionResultType]::ParameterValue, 'Export rendered target files as tar, tar.gz, or zip')
            [CompletionResult]::new('execute-template', 'execute-template', [CompletionResultType]::ParameterValue, 'Evaluate a Jinja template using repository and local data')
            [CompletionResult]::new('data', 'data', [CompletionResultType]::ParameterValue, 'Print merged template data as JSON')
            [CompletionResult]::new('encrypt', 'encrypt', [CompletionResultType]::ParameterValue, 'Encrypt a file using configured age or gpg')
            [CompletionResult]::new('decrypt', 'decrypt', [CompletionResultType]::ParameterValue, 'Decrypt a file using configured identities')
            [CompletionResult]::new('keygen', 'keygen', [CompletionResultType]::ParameterValue, 'Generate an age identity and print its public recipient')
            [CompletionResult]::new('cd', 'cd', [CompletionResultType]::ParameterValue, 'Start a shell in the source repository, or print its path')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'xyrev;dotfile;help;init' {
            break
        }
        'xyrev;dotfile;help;diff' {
            break
        }
        'xyrev;dotfile;help;status' {
            break
        }
        'xyrev;dotfile;help;apply' {
            break
        }
        'xyrev;dotfile;help;update' {
            break
        }
        'xyrev;dotfile;help;managed' {
            break
        }
        'xyrev;dotfile;help;add' {
            break
        }
        'xyrev;dotfile;help;edit' {
            break
        }
        'xyrev;dotfile;help;unmanaged' {
            break
        }
        'xyrev;dotfile;help;verify' {
            break
        }
        'xyrev;dotfile;help;doctor' {
            break
        }
        'xyrev;dotfile;help;archive' {
            break
        }
        'xyrev;dotfile;help;execute-template' {
            break
        }
        'xyrev;dotfile;help;data' {
            break
        }
        'xyrev;dotfile;help;encrypt' {
            break
        }
        'xyrev;dotfile;help;decrypt' {
            break
        }
        'xyrev;dotfile;help;keygen' {
            break
        }
        'xyrev;dotfile;help;cd' {
            break
        }
        'xyrev;dotfile;help;help' {
            break
        }
        'xyrev;completions' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;man' {
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'output')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Create a new configuration folder or reinitialize the existing one')
            [CompletionResult]::new('agent', 'agent', [CompletionResultType]::ParameterValue, 'agent')
            [CompletionResult]::new('apple', 'apple', [CompletionResultType]::ParameterValue, 'apple')
            [CompletionResult]::new('dotfile', 'dotfile', [CompletionResultType]::ParameterValue, 'Manage dotfiles from a Git repository using chezmoi-style names')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completions from the current command definitions')
            [CompletionResult]::new('man', 'man', [CompletionResultType]::ParameterValue, 'Render the manual, or write all command manuals to a directory')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'xyrev;help;init' {
            break
        }
        'xyrev;help;agent' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'init')
            [CompletionResult]::new('copilot', 'copilot', [CompletionResultType]::ParameterValue, 'copilot')
            break
        }
        'xyrev;help;agent;init' {
            break
        }
        'xyrev;help;agent;copilot' {
            break
        }
        'xyrev;help;apple' {
            break
        }
        'xyrev;help;dotfile' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Clone a Git URL and save configuration')
            [CompletionResult]::new('diff', 'diff', [CompletionResultType]::ParameterValue, 'Preview content and permission changes without writing')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'List pending changes and conflicts')
            [CompletionResult]::new('apply', 'apply', [CompletionResultType]::ParameterValue, 'Deploy the local source repository')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Fast-forward the source repository and deploy changes')
            [CompletionResult]::new('managed', 'managed', [CompletionResultType]::ParameterValue, 'List managed destination paths')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Import existing destination files into source state')
            [CompletionResult]::new('edit', 'edit', [CompletionResultType]::ParameterValue, 'Edit source files (encrypted files are decrypted only in a private temporary directory)')
            [CompletionResult]::new('unmanaged', 'unmanaged', [CompletionResultType]::ParameterValue, 'List destination files not managed by source')
            [CompletionResult]::new('verify', 'verify', [CompletionResultType]::ParameterValue, 'Return nonzero when the destination differs from source')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Diagnose repository, configuration, state and optional executables')
            [CompletionResult]::new('archive', 'archive', [CompletionResultType]::ParameterValue, 'Export rendered target files as tar, tar.gz, or zip')
            [CompletionResult]::new('execute-template', 'execute-template', [CompletionResultType]::ParameterValue, 'Evaluate a Jinja template using repository and local data')
            [CompletionResult]::new('data', 'data', [CompletionResultType]::ParameterValue, 'Print merged template data as JSON')
            [CompletionResult]::new('encrypt', 'encrypt', [CompletionResultType]::ParameterValue, 'Encrypt a file using configured age or gpg')
            [CompletionResult]::new('decrypt', 'decrypt', [CompletionResultType]::ParameterValue, 'Decrypt a file using configured identities')
            [CompletionResult]::new('keygen', 'keygen', [CompletionResultType]::ParameterValue, 'Generate an age identity and print its public recipient')
            [CompletionResult]::new('cd', 'cd', [CompletionResultType]::ParameterValue, 'Start a shell in the source repository, or print its path')
            break
        }
        'xyrev;help;dotfile;init' {
            break
        }
        'xyrev;help;dotfile;diff' {
            break
        }
        'xyrev;help;dotfile;status' {
            break
        }
        'xyrev;help;dotfile;apply' {
            break
        }
        'xyrev;help;dotfile;update' {
            break
        }
        'xyrev;help;dotfile;managed' {
            break
        }
        'xyrev;help;dotfile;add' {
            break
        }
        'xyrev;help;dotfile;edit' {
            break
        }
        'xyrev;help;dotfile;unmanaged' {
            break
        }
        'xyrev;help;dotfile;verify' {
            break
        }
        'xyrev;help;dotfile;doctor' {
            break
        }
        'xyrev;help;dotfile;archive' {
            break
        }
        'xyrev;help;dotfile;execute-template' {
            break
        }
        'xyrev;help;dotfile;data' {
            break
        }
        'xyrev;help;dotfile;encrypt' {
            break
        }
        'xyrev;help;dotfile;decrypt' {
            break
        }
        'xyrev;help;dotfile;keygen' {
            break
        }
        'xyrev;help;dotfile;cd' {
            break
        }
        'xyrev;help;completions' {
            break
        }
        'xyrev;help;man' {
            break
        }
        'xyrev;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
