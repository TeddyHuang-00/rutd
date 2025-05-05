Output assets/gif/autocomplete.gif

Require rutd-cli
Require fish

Set Shell xonsh
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme catppuccin-mocha

Env RIGHT_PROMPT ""

# Load auto completion for shell and skip this part
Hide
Type@0 "xontrib load fish_completer" Enter
Type@0 "clear" Enter
Sleep 1s
Show

Type "# Advanced dynamic completion" Enter@500ms
Type "rutd-cli l -o " Sleep 500ms
Tab@500ms 3 Enter@500ms
Tab@500ms 4 Enter@500ms
Tab@500ms 2 Enter@500ms
Tab@500ms 5 Enter@500ms 2

Sleep 1s
