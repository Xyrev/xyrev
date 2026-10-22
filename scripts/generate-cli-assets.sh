#!/bin/sh
set -eu
# Run after building xyrev when changing the clap command definitions.
xyrev_binary=${1:-target/debug/xyrev}
"$xyrev_binary" completions bash > completions/xyrev.bash
"$xyrev_binary" completions zsh > completions/_xyrev
"$xyrev_binary" completions fish > completions/xyrev.fish
"$xyrev_binary" completions powershell > completions/_xyrev.ps1
"$xyrev_binary" completions elvish > completions/xyrev.elv
"$xyrev_binary" man --output man-page
