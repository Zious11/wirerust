Output docs/demo-evidence/STORY-108/AC-012-threshold-semantics.gif
Output docs/demo-evidence/STORY-108/AC-012-threshold-semantics.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-012: Threshold check is strictly >, not >=.
# At count=10 (=threshold), no finding. At count=11 (>threshold), finding emitted.
# Default direct_operate_threshold=10; DETECTION_WINDOW_SECS=60.
# Traces to: BC-2.15.010 invariant (strict >); Canonical Test Vector §2; STORY-108 AC-012.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_BC_2_15_010_threshold_is_strictly_greater_not_gte -- --nocapture"
Enter
Sleep 5s
