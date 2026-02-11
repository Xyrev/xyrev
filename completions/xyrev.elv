
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
            cand -v 'Use verbose output (-vv very verbose/build.rs output)'
            cand --verbose 'Use verbose output (-vv very verbose/build.rs output)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand init 'Create a new configuration folder or reinitialize the existing one'
            cand dotfile 'dotfile'
        }
        &'xyrev;init'= {
            cand -f 'Force reinitialize configuration (overwrite existing)'
            cand --force 'Force reinitialize configuration (overwrite existing)'
            cand -C 'Clean and reinitialize (remove all existing data)'
            cand --clean 'Clean and reinitialize (remove all existing data)'
            cand -v 'Use verbose output (-vv very verbose/build.rs output)'
            cand --verbose 'Use verbose output (-vv very verbose/build.rs output)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'xyrev;dotfile'= {
            cand -v 'Use verbose output (-vv for very verbose)'
            cand --verbose 'Use verbose output (-vv for very verbose)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
    ]
    $completions[$command]
}
