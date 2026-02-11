
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
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Use verbose output (-vv very verbose/build.rs output)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Use verbose output (-vv very verbose/build.rs output)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Create a new configuration folder or reinitialize the existing one')
            [CompletionResult]::new('dotfile', 'dotfile', [CompletionResultType]::ParameterValue, 'dotfile')
            break
        }
        'xyrev;init' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Force reinitialize configuration (overwrite existing)')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Force reinitialize configuration (overwrite existing)')
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Clean and reinitialize (remove all existing data)')
            [CompletionResult]::new('--clean', '--clean', [CompletionResultType]::ParameterName, 'Clean and reinitialize (remove all existing data)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Use verbose output (-vv very verbose/build.rs output)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Use verbose output (-vv very verbose/build.rs output)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'xyrev;dotfile' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Use verbose output (-vv for very verbose)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Use verbose output (-vv for very verbose)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
