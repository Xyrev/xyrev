
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'stool' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'stool'
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
        'stool' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Create a new configuration folder or reinitialize the existing one')
            [CompletionResult]::new('agent', 'agent', [CompletionResultType]::ParameterValue, 'agent')
            [CompletionResult]::new('apple', 'apple', [CompletionResultType]::ParameterValue, 'Manage macOS preferences independently of dotfiles')
            [CompletionResult]::new('dotfile', 'dotfile', [CompletionResultType]::ParameterValue, 'Manage dotfiles from a Git repository')
            [CompletionResult]::new('key', 'key', [CompletionResultType]::ParameterValue, 'Back up and restore GPG and SSH keys independently of dotfiles')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completions from the current command definitions')
            [CompletionResult]::new('man', 'man', [CompletionResultType]::ParameterValue, 'Render the manual, or write all command manuals to a directory')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;init' {
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
        'stool;agent' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'init')
            [CompletionResult]::new('copilot', 'copilot', [CompletionResultType]::ParameterValue, 'copilot')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;agent;init' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;agent;copilot' {
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
        'stool;agent;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'init')
            [CompletionResult]::new('copilot', 'copilot', [CompletionResultType]::ParameterValue, 'copilot')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;agent;help;init' {
            break
        }
        'stool;agent;help;copilot' {
            break
        }
        'stool;agent;help;help' {
            break
        }
        'stool;apple' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Set the current user''s Dock minimization animation to suck')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;apple;init' {
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Show current preferences and planned changes without writing')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;apple;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Set the current user''s Dock minimization animation to suck')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;apple;help;init' {
            break
        }
        'stool;apple;help;help' {
            break
        }
        'stool;dotfile' {
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
            [CompletionResult]::new('keygen', 'keygen', [CompletionResultType]::ParameterValue, 'Generate an age identity and print its public recipient')
            [CompletionResult]::new('cd', 'cd', [CompletionResultType]::ParameterValue, 'Start a shell in the source repository, or print its path')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;dotfile;init' {
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
        'stool;dotfile;diff' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;dotfile;status' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;dotfile;apply' {
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
        'stool;dotfile;update' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;dotfile;managed' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;dotfile;add' {
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
        'stool;dotfile;edit' {
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
        'stool;dotfile;unmanaged' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;dotfile;verify' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;dotfile;doctor' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;dotfile;archive' {
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
        'stool;dotfile;execute-template' {
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
        'stool;dotfile;data' {
            [CompletionResult]::new('--source', '--source', [CompletionResultType]::ParameterName, 'Local source repository (saved by init)')
            [CompletionResult]::new('--destination', '--destination', [CompletionResultType]::ParameterName, 'Destination directory (defaults to the user''s home)')
            [CompletionResult]::new('--data', '--data', [CompletionResultType]::ParameterName, 'Override template data (key=value; JSON values are accepted)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;dotfile;keygen' {
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
        'stool;dotfile;cd' {
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
        'stool;dotfile;help' {
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
            [CompletionResult]::new('keygen', 'keygen', [CompletionResultType]::ParameterValue, 'Generate an age identity and print its public recipient')
            [CompletionResult]::new('cd', 'cd', [CompletionResultType]::ParameterValue, 'Start a shell in the source repository, or print its path')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;dotfile;help;init' {
            break
        }
        'stool;dotfile;help;diff' {
            break
        }
        'stool;dotfile;help;status' {
            break
        }
        'stool;dotfile;help;apply' {
            break
        }
        'stool;dotfile;help;update' {
            break
        }
        'stool;dotfile;help;managed' {
            break
        }
        'stool;dotfile;help;add' {
            break
        }
        'stool;dotfile;help;edit' {
            break
        }
        'stool;dotfile;help;unmanaged' {
            break
        }
        'stool;dotfile;help;verify' {
            break
        }
        'stool;dotfile;help;doctor' {
            break
        }
        'stool;dotfile;help;archive' {
            break
        }
        'stool;dotfile;help;execute-template' {
            break
        }
        'stool;dotfile;help;data' {
            break
        }
        'stool;dotfile;help;keygen' {
            break
        }
        'stool;dotfile;help;cd' {
            break
        }
        'stool;dotfile;help;help' {
            break
        }
        'stool;key' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('backup', 'backup', [CompletionResultType]::ParameterValue, 'Create one password-encrypted offline key backup outside Git workspaces')
            [CompletionResult]::new('restore', 'restore', [CompletionResultType]::ParameterValue, 'Validate and manually restore an offline key backup')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;key;backup' {
            [CompletionResult]::new('--input', '--input', [CompletionResultType]::ParameterName, 'Source user directory (defaults to the current user environment)')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'New backup file (defaults to stool-keys-<timestamp>.tar.age in the current directory)')
            [CompletionResult]::new('--passphrase-file', '--passphrase-file', [CompletionResultType]::ParameterName, 'Read the password from a private external file or inherited pipe')
            [CompletionResult]::new('--gpg', '--gpg', [CompletionResultType]::ParameterName, 'Only GPG keys')
            [CompletionResult]::new('--ssh', '--ssh', [CompletionResultType]::ParameterName, 'Only SSH keys')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'All available key categories (the default)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;key;restore' {
            [CompletionResult]::new('--input', '--input', [CompletionResultType]::ParameterName, 'Password-encrypted stool key backup file')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Target user directory (defaults to the current user''s home)')
            [CompletionResult]::new('--passphrase-file', '--passphrase-file', [CompletionResultType]::ParameterName, 'Read the password from a private external file')
            [CompletionResult]::new('--gpg', '--gpg', [CompletionResultType]::ParameterName, 'Only GPG keys')
            [CompletionResult]::new('--ssh', '--ssh', [CompletionResultType]::ParameterName, 'Only SSH keys')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'All available key categories (the default)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;key;help' {
            [CompletionResult]::new('backup', 'backup', [CompletionResultType]::ParameterValue, 'Create one password-encrypted offline key backup outside Git workspaces')
            [CompletionResult]::new('restore', 'restore', [CompletionResultType]::ParameterValue, 'Validate and manually restore an offline key backup')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;key;help;backup' {
            break
        }
        'stool;key;help;restore' {
            break
        }
        'stool;key;help;help' {
            break
        }
        'stool;completions' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;man' {
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'output')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Show more debugging information')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'stool;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Create a new configuration folder or reinitialize the existing one')
            [CompletionResult]::new('agent', 'agent', [CompletionResultType]::ParameterValue, 'agent')
            [CompletionResult]::new('apple', 'apple', [CompletionResultType]::ParameterValue, 'Manage macOS preferences independently of dotfiles')
            [CompletionResult]::new('dotfile', 'dotfile', [CompletionResultType]::ParameterValue, 'Manage dotfiles from a Git repository')
            [CompletionResult]::new('key', 'key', [CompletionResultType]::ParameterValue, 'Back up and restore GPG and SSH keys independently of dotfiles')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completions from the current command definitions')
            [CompletionResult]::new('man', 'man', [CompletionResultType]::ParameterValue, 'Render the manual, or write all command manuals to a directory')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'stool;help;init' {
            break
        }
        'stool;help;agent' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'init')
            [CompletionResult]::new('copilot', 'copilot', [CompletionResultType]::ParameterValue, 'copilot')
            break
        }
        'stool;help;agent;init' {
            break
        }
        'stool;help;agent;copilot' {
            break
        }
        'stool;help;apple' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Set the current user''s Dock minimization animation to suck')
            break
        }
        'stool;help;apple;init' {
            break
        }
        'stool;help;dotfile' {
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
            [CompletionResult]::new('keygen', 'keygen', [CompletionResultType]::ParameterValue, 'Generate an age identity and print its public recipient')
            [CompletionResult]::new('cd', 'cd', [CompletionResultType]::ParameterValue, 'Start a shell in the source repository, or print its path')
            break
        }
        'stool;help;dotfile;init' {
            break
        }
        'stool;help;dotfile;diff' {
            break
        }
        'stool;help;dotfile;status' {
            break
        }
        'stool;help;dotfile;apply' {
            break
        }
        'stool;help;dotfile;update' {
            break
        }
        'stool;help;dotfile;managed' {
            break
        }
        'stool;help;dotfile;add' {
            break
        }
        'stool;help;dotfile;edit' {
            break
        }
        'stool;help;dotfile;unmanaged' {
            break
        }
        'stool;help;dotfile;verify' {
            break
        }
        'stool;help;dotfile;doctor' {
            break
        }
        'stool;help;dotfile;archive' {
            break
        }
        'stool;help;dotfile;execute-template' {
            break
        }
        'stool;help;dotfile;data' {
            break
        }
        'stool;help;dotfile;keygen' {
            break
        }
        'stool;help;dotfile;cd' {
            break
        }
        'stool;help;key' {
            [CompletionResult]::new('backup', 'backup', [CompletionResultType]::ParameterValue, 'Create one password-encrypted offline key backup outside Git workspaces')
            [CompletionResult]::new('restore', 'restore', [CompletionResultType]::ParameterValue, 'Validate and manually restore an offline key backup')
            break
        }
        'stool;help;key;backup' {
            break
        }
        'stool;help;key;restore' {
            break
        }
        'stool;help;completions' {
            break
        }
        'stool;help;man' {
            break
        }
        'stool;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
