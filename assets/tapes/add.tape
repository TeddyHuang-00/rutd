Output assets/gif/add.gif

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

Type "rutd-cli a -h" Enter@500ms

Sleep 1s

Type "# Add a simple task" Enter@500ms
Type "rutd-cli a 'Do laundry'" Enter@500ms

Sleep 1s

Type "# Fancy options to help organize tasks" Enter@500ms
Type "rutd-cli a"
Tab@500ms 2 Space@500ms
Tab@500ms 3 Space@500ms
Tab@500ms 2 Space@500ms
Tab@500ms 4 Space@500ms
Type "rutd" Space@500ms
Tab@500ms 5 Space@500ms
Tab@500ms 3 Space@500ms
Type "'Add demonstration'" Enter@500ms

Sleep 1s

Type "rutd-cli l" Enter@500ms

Sleep 1s