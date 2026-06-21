Output .factory/demo-evidence/STORY-126/AC-002-block-skip-f07.gif
Output .factory/demo-evidence/STORY-126/AC-002-block-skip-f07.webm

Require bash

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-002 (F-07): Block-skip dispatch
# Fixture: SHB + IDB + NRB + ISB + SJE + EPB
# Demonstrates: non-packet blocks (NRB, ISB, SJE) are skipped silently;
# parsing continues and the EPB after them is successfully processed.
# The test_BC_2_01_015_dispatch_known_and_skip_unknown test verifies skipped_blocks=5.
# Traces to: BC-2.01.015 AC-006, AC-008, AC-010; STORY-126 F-07.

Hide
Type "python3 /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-126/mk_spb_fixtures.py"
Enter
Sleep 3s
Show

Type "# F-07 block-skip: NRB+ISB+SJE skipped; EPB after them parsed; no crash"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-126/target/release/wirerust summary /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-126/spb-skip.pcapng --no-color"
Enter
Sleep 4s
