Output assets/gif/clone-sync.gif

Require rutd-cli
Require fish

Set Shell xonsh
Set FontSize 18
Set Width 1200
Set Height 800
Set Theme catppuccin-mocha

Env RIGHT_PROMPT ""

# Load auto completion for shell and skip this part
Hide
Type@0 "xontrib load fish_completer" Enter
Type@0 "clear" Enter
Sleep 1s
Show

Type "# Clone from your repo to resume your work" Enter@500ms
Type "rutd-cli clone git@github.com:TeddyHuang-00/rutd-tasks-example.git" Enter@500ms

Sleep 2s

Type "rutd-cli l" Enter@500ms

Sleep 1s

Type "# Sync your changes so you don't lose them" Enter@500ms
Type "rutd-cli sync" Enter@500ms

Sleep 5s