Output docs/demo-evidence/STORY-109/AC-008-broadcast-burst-retained.gif
Output docs/demo-evidence/STORY-109/AC-008-broadcast-burst-retained.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-008: 11 broadcast Control FCs → broadcast anomaly (Suspicious) PLUS burst finding (Execution).
# Both T1692.001 findings retained — no dedup by technique ID alone.
# Traces to: BC-2.15.018 invariant 4; BC-2.15.013 invariant 4; STORY-109 AC-008.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_broadcast_and_burst_both_retained -- --nocapture"
Enter
Sleep 5s
