Output docs/demo-evidence/STORY-108/AC-011-summarize-zero-flows.gif
Output docs/demo-evidence/STORY-108/AC-011-summarize-zero-flows.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-011: When no DNP3 flows analyzed, summarize() present with zero counts (not absent).
# flows_analyzed=0, total_frames=0, parse_errors=0,
# function_code_distribution={} (empty object present).
# Traces to: BC-2.15.020 postcondition 2 and invariant 4; STORY-108 AC-011.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/fix-pc-014-evidence && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_summarize_zero_flows -- --nocapture"
Enter
Sleep 5s
