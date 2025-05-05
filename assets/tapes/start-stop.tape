Output assets/gif/start-stop.gif

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

Type "rutd-cli list" Enter@500ms

Sleep 1s

Type "# Start working on a task" Enter@500ms
Type "rutd-cli start "
Tab@500ms 4 Enter@500ms 2
Type "# Doing some work..." Enter@500ms

Sleep 1s

Type "# Stop working" Enter@500ms
Type "rutd-cli stop" Enter@500ms

Sleep 1s

Type "rutd-cli l" Enter@500ms

Sleep 1s