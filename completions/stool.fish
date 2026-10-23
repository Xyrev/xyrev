# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_stool_global_optspecs
	string join \n v/verbose h/help V/version
end

function __fish_stool_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_stool_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_stool_using_subcommand
	set -l cmd (__fish_stool_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c stool -n "__fish_stool_needs_command" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_needs_command" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_needs_command" -s V -l version -d 'Print version'
complete -c stool -n "__fish_stool_needs_command" -f -a "init" -d 'Create a new configuration folder or reinitialize the existing one'
complete -c stool -n "__fish_stool_needs_command" -f -a "agent"
complete -c stool -n "__fish_stool_needs_command" -f -a "apple"
complete -c stool -n "__fish_stool_needs_command" -f -a "dotfile" -d 'Manage dotfiles from a Git repository'
complete -c stool -n "__fish_stool_needs_command" -f -a "completions" -d 'Generate shell completions from the current command definitions'
complete -c stool -n "__fish_stool_needs_command" -f -a "man" -d 'Render the manual, or write all command manuals to a directory'
complete -c stool -n "__fish_stool_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c stool -n "__fish_stool_using_subcommand init" -s f -l force -d 'Force reinitialize configuration (overwrite existing)'
complete -c stool -n "__fish_stool_using_subcommand init" -s C -l clean -d 'Clean and reinitialize (remove all existing data)'
complete -c stool -n "__fish_stool_using_subcommand init" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand init" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand agent; and not __fish_seen_subcommand_from init copilot help" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand agent; and not __fish_seen_subcommand_from init copilot help" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand agent; and not __fish_seen_subcommand_from init copilot help" -f -a "init"
complete -c stool -n "__fish_stool_using_subcommand agent; and not __fish_seen_subcommand_from init copilot help" -f -a "copilot"
complete -c stool -n "__fish_stool_using_subcommand agent; and not __fish_seen_subcommand_from init copilot help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from init" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from copilot" -s m -l method -d 'Install method (auto, brew, npm, winget, script)' -r
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from copilot" -l version -d 'Specific version to install (e.g., v0.0.369)' -r
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from copilot" -l prefix -d 'Custom installation prefix (for script method)' -r
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from copilot" -l prerelease -d 'Install prerelease version'
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from copilot" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from copilot" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from help" -f -a "init"
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from help" -f -a "copilot"
complete -c stool -n "__fish_stool_using_subcommand agent; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c stool -n "__fish_stool_using_subcommand apple" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand apple" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "init" -d 'Clone a Git URL and save configuration'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "diff" -d 'Preview content and permission changes without writing'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "status" -d 'List pending changes and conflicts'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "apply" -d 'Deploy the local source repository'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "update" -d 'Fast-forward the source repository and deploy changes'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "managed" -d 'List managed destination paths'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "add" -d 'Import existing destination files into source state'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "edit" -d 'Edit source files (encrypted files are decrypted only in a private temporary directory)'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "unmanaged" -d 'List destination files not managed by source'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "verify" -d 'Return nonzero when the destination differs from source'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "doctor" -d 'Diagnose repository, configuration, state and optional executables'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "archive" -d 'Export rendered target files as tar, tar.gz, or zip'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "execute-template" -d 'Evaluate a Jinja template using repository and local data'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "data" -d 'Print merged template data as JSON'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "gpg" -d 'Back up or restore GPG keys using encrypted dotfile storage'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "keygen" -d 'Generate an age identity and print its public recipient'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "cd" -d 'Start a shell in the source repository, or print its path'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and not __fish_seen_subcommand_from init diff status apply update managed add edit unmanaged verify doctor archive execute-template data gpg keygen cd help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from init" -l branch -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from init" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from init" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from init" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from init" -l apply -d 'Deploy after cloning'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from init" -l recurse-submodules
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from init" -l builtin -d 'Clone using the built-in Git backend'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from init" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from diff" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from diff" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from diff" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from diff" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from diff" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from status" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from status" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from status" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from apply" -l prompt -d 'Interactive conflict handling (auto only prompts on a terminal)' -r -f -a "auto\t''
always\t''
never\t''"
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from apply" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from apply" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from apply" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from apply" -l dry-run
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from apply" -l force -d 'Back up and overwrite conflicting destination entries'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from apply" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from apply" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from update" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from update" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from update" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from update" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from update" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from managed" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from managed" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from managed" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from managed" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from managed" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from add" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from add" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from add" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from add" -l template
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from add" -l encrypt
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from add" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from edit" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from edit" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from edit" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from edit" -l apply
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from edit" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from unmanaged" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from unmanaged" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from unmanaged" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from unmanaged" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from unmanaged" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from verify" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from verify" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from verify" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from verify" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from verify" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from doctor" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from doctor" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from doctor" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from doctor" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from doctor" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from archive" -l format -r -f -a "tar\t''
tgz\t''
zip\t''"
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from archive" -s o -l output -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from archive" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from archive" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from archive" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from archive" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from execute-template" -l file -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from execute-template" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from execute-template" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from execute-template" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from execute-template" -l init
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from execute-template" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from execute-template" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from data" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from data" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from data" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from data" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from data" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from gpg" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from gpg" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from gpg" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from gpg" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from gpg" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from gpg" -f -a "backup" -d 'Export local GPG private keys and ownertrust into an encrypted backup'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from gpg" -f -a "restore" -d 'Validate and restore an encrypted GPG key backup'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from gpg" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from keygen" -l output -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from keygen" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from keygen" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from keygen" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from keygen" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from keygen" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from cd" -l source -d 'Local source repository (saved by init)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from cd" -l destination -d 'Destination directory (defaults to the user\'s home)' -r -F
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from cd" -l data -d 'Override template data (key=value; JSON values are accepted)' -r
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from cd" -l print
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from cd" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from cd" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "init" -d 'Clone a Git URL and save configuration'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "diff" -d 'Preview content and permission changes without writing'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "status" -d 'List pending changes and conflicts'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "apply" -d 'Deploy the local source repository'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "update" -d 'Fast-forward the source repository and deploy changes'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "managed" -d 'List managed destination paths'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "add" -d 'Import existing destination files into source state'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit source files (encrypted files are decrypted only in a private temporary directory)'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "unmanaged" -d 'List destination files not managed by source'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "verify" -d 'Return nonzero when the destination differs from source'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "doctor" -d 'Diagnose repository, configuration, state and optional executables'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Export rendered target files as tar, tar.gz, or zip'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "execute-template" -d 'Evaluate a Jinja template using repository and local data'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "data" -d 'Print merged template data as JSON'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "gpg" -d 'Back up or restore GPG keys using encrypted dotfile storage'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "keygen" -d 'Generate an age identity and print its public recipient'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "cd" -d 'Start a shell in the source repository, or print its path'
complete -c stool -n "__fish_stool_using_subcommand dotfile; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c stool -n "__fish_stool_using_subcommand completions" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand completions" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand man" -l output -r -F
complete -c stool -n "__fish_stool_using_subcommand man" -s v -l verbose -d 'Show more debugging information'
complete -c stool -n "__fish_stool_using_subcommand man" -s h -l help -d 'Print help'
complete -c stool -n "__fish_stool_using_subcommand help; and not __fish_seen_subcommand_from init agent apple dotfile completions man help" -f -a "init" -d 'Create a new configuration folder or reinitialize the existing one'
complete -c stool -n "__fish_stool_using_subcommand help; and not __fish_seen_subcommand_from init agent apple dotfile completions man help" -f -a "agent"
complete -c stool -n "__fish_stool_using_subcommand help; and not __fish_seen_subcommand_from init agent apple dotfile completions man help" -f -a "apple"
complete -c stool -n "__fish_stool_using_subcommand help; and not __fish_seen_subcommand_from init agent apple dotfile completions man help" -f -a "dotfile" -d 'Manage dotfiles from a Git repository'
complete -c stool -n "__fish_stool_using_subcommand help; and not __fish_seen_subcommand_from init agent apple dotfile completions man help" -f -a "completions" -d 'Generate shell completions from the current command definitions'
complete -c stool -n "__fish_stool_using_subcommand help; and not __fish_seen_subcommand_from init agent apple dotfile completions man help" -f -a "man" -d 'Render the manual, or write all command manuals to a directory'
complete -c stool -n "__fish_stool_using_subcommand help; and not __fish_seen_subcommand_from init agent apple dotfile completions man help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from agent" -f -a "init"
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from agent" -f -a "copilot"
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "init" -d 'Clone a Git URL and save configuration'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "diff" -d 'Preview content and permission changes without writing'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "status" -d 'List pending changes and conflicts'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "apply" -d 'Deploy the local source repository'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "update" -d 'Fast-forward the source repository and deploy changes'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "managed" -d 'List managed destination paths'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "add" -d 'Import existing destination files into source state'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "edit" -d 'Edit source files (encrypted files are decrypted only in a private temporary directory)'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "unmanaged" -d 'List destination files not managed by source'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "verify" -d 'Return nonzero when the destination differs from source'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "doctor" -d 'Diagnose repository, configuration, state and optional executables'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "archive" -d 'Export rendered target files as tar, tar.gz, or zip'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "execute-template" -d 'Evaluate a Jinja template using repository and local data'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "data" -d 'Print merged template data as JSON'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "gpg" -d 'Back up or restore GPG keys using encrypted dotfile storage'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "keygen" -d 'Generate an age identity and print its public recipient'
complete -c stool -n "__fish_stool_using_subcommand help; and __fish_seen_subcommand_from dotfile" -f -a "cd" -d 'Start a shell in the source repository, or print its path'
