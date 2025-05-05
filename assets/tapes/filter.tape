Output assets/gif/filter.gif

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

Type "# Filter by scope" Enter@500ms
Type "rutd-cli list "
Tab@500ms 7 Space@500ms
Tab@500ms 3 Enter@500ms 2

Sleep 1s

Type "# Fuzzy match description" Enter@500ms
Type "rutd-cli list "
Tab@500ms 4 Space@500ms
Type "demo" Enter@500ms

Sleep 1s
