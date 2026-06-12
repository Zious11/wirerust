Output docs/demo-evidence/STORY-108/AC-007-co-emission-ordering.gif
Output docs/demo-evidence/STORY-108/AC-007-co-emission-ordering.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-007: Restart findings append in observation order (inter-call ordering).
# COLD_RESTART first, WARM_RESTART second -> all_findings[0]=T0814, all_findings[1]=T0814.
# T0827 is NOT present (STORY-109 deferred).
# Traces to: BC-2.15.013 postconditions 2/3; STORY-108 AC-007.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_restart_findings_append_in_observation_order -- --nocapture"
Enter
Sleep 5s
