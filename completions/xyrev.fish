# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_xyrev_global_optspecs
	string join \n v/verbose h/help V/version
end

function __fish_xyrev_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_xyrev_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_xyrev_using_subcommand
	set -l cmd (__fish_xyrev_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c xyrev -n "__fish_xyrev_needs_command" -s v -l verbose -d 'Use verbose output (-vv very verbose/build.rs output)'
complete -c xyrev -n "__fish_xyrev_needs_command" -s h -l help -d 'Print help'
complete -c xyrev -n "__fish_xyrev_needs_command" -s V -l version -d 'Print version'
complete -c xyrev -n "__fish_xyrev_needs_command" -f -a "init" -d 'Create a new configuration folder or reinitialize the existing one'
complete -c xyrev -n "__fish_xyrev_needs_command" -f -a "dotfile"
complete -c xyrev -n "__fish_xyrev_using_subcommand init" -s f -l force -d 'Force reinitialize configuration (overwrite existing)'
complete -c xyrev -n "__fish_xyrev_using_subcommand init" -s C -l clean -d 'Clean and reinitialize (remove all existing data)'
complete -c xyrev -n "__fish_xyrev_using_subcommand init" -s v -l verbose -d 'Use verbose output (-vv very verbose/build.rs output)'
complete -c xyrev -n "__fish_xyrev_using_subcommand init" -s h -l help -d 'Print help'
complete -c xyrev -n "__fish_xyrev_using_subcommand dotfile" -s v -l verbose -d 'Use verbose output (-vv for very verbose)'
complete -c xyrev -n "__fish_xyrev_using_subcommand dotfile" -s h -l help -d 'Print help'
