# STORY-130 AC-004: is_valid_enip_frame biconditional gate (command-only)
#
# AC-004: returns true iff h.command in {0x0004..0x0075 known set};
#         command-only (h.length, h.status, h.options NOT inspected).
# Boundary: Cancel (0x0075) = true; 0x0076 = false; 0xFFFF = false.
# Traces: BC-2.17.003 postconditions 1-4; VP-032 Sub-C.

Output AC-004-validity-gate.gif
Output AC-004-validity-gate.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-130-enip-pure-core-parse"
Enter
Wait
Show

Type "# AC-004: is_valid_enip_frame biconditional gate (BC-2.17.003)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests parse_header::test_is_valid 2>&1 | tail -16"
Enter
Wait
Sleep 2s
