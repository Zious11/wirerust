Output docs/demo-evidence/STORY-110/AC-007-threshold-edge-values.gif
Output docs/demo-evidence/STORY-110/AC-007-threshold-edge-values.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 450
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-007: Threshold edge values.
# threshold=0: T1692.001 fires on first Control-class FC (count=1 > 0).
# threshold=u32::MAX (4294967295): T1692.001 never fires (counter cannot exceed u32::MAX).
# Traces to: BC-2.15.017 postconditions 3/4; STORY-110 AC-007.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build 2>/dev/null"
Enter
Sleep 15s
Show

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_threshold_0_fires_immediately -- --nocapture"
Enter
Sleep 5s

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_threshold_max_never_fires -- --nocapture"
Enter
Sleep 5s
