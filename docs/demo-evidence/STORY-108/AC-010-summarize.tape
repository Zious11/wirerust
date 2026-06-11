Output docs/demo-evidence/STORY-108/AC-010-summarize.gif
Output docs/demo-evidence/STORY-108/AC-010-summarize.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-010: summarize() emits function_code_distribution, control_operation_counts,
# total_frames, total_parse_errors, flows_analyzed.
# 5 DIRECT_OPERATE + 3 READ -> fn_code_counts={0x05:5, 0x01:3}, total_frames=8.
# Traces to: BC-2.15.020 postcondition 1; Canonical Test Vectors; STORY-108 AC-010.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_summarize_function_code_distribution -- --nocapture"
Enter
Sleep 5s
