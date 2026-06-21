Output .factory/demo-evidence/STORY-126/AC-003-spb-before-idb-e-inp-009.gif
Output .factory/demo-evidence/STORY-126/AC-003-spb-before-idb-e-inp-009.webm

Require bash

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-003 (BC-2.01.017): Error surface -- SPB before any IDB => E-INP-009
# Fixture: SHB + SPB (no IDB)
# Demonstrates: graceful E-INP-009 error with exact context string;
# no panic, no silent failure; full anyhow error chain shown.
# Traces to: BC-2.01.017 PC1; BC-2.01.013 AC-001; STORY-126 error surface.

Hide
Type "python3 /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-126/mk_spb_fixtures.py"
Enter
Sleep 3s
Show

Type "# BC-2.01.017: SPB before any IDB -> E-INP-009 with full context, no panic"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-126/target/release/wirerust summary /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-126/spb-error.pcapng --no-color"
Enter
Sleep 4s
