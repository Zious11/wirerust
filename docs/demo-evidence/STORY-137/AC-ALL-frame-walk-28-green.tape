# STORY-137 AC-ALL: Full mod frame_walk suite — 28/28 green
#
# Master green-run covering all STORY-137 acceptance criteria:
#   AC-137-001: carry buffer partial/multi/three-segment assembly
#   AC-137-002: carry bounded; is_non_enip latch unreachable (RULING-137-002)
#   AC-137-003: byte-walk resync + oversized-frame-skip; both continue (EC-010/EC-012)
#   AC-137-004: T0814 windowed threshold=3, one-shot guard, re-fire after 300s
#   AC-137-005: valid frames no malformed count; invalid frames increment both counters
#   AC-137-006: command_counts single canonical site incl. Unknown (0xFF00) (BC-2.17.016 PC-0)

Output AC-ALL-frame-walk-28-green.gif
Output AC-ALL-frame-walk-28-green.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-137-enip-frame-walk"
Enter
Wait
Show

Type "# STORY-137: mod frame_walk full suite (28 tests) — BC-2.17.016/004/018"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests frame_walk 2>&1 | grep -E 'test frame_walk::|result'"
Enter
Wait
Sleep 2s
