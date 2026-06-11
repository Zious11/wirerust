Output docs/demo-evidence/STORY-108/AC-009-counters-when-capped.gif
Output docs/demo-evidence/STORY-108/AC-009-counters-when-capped.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-009: When all_findings >= MAX_FINDINGS, counters still updated.
# direct_operate_count, restart_event_count, fc_counts, fn_code_counts, frame_count
# all increment even when findings are suppressed.
# Traces to: BC-2.15.022 postconditions 1/3; STORY-108 AC-009.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_max_findings_counters_updated_when_capped -- --nocapture"
Enter
Sleep 5s
