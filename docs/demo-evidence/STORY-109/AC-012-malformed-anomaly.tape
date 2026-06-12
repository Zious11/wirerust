Output docs/demo-evidence/STORY-109/AC-012-malformed-anomaly.gif
Output docs/demo-evidence/STORY-109/AC-012-malformed-anomaly.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-012: 3 malformed frames in 300s window → T0814 Possible/Low at 3rd.
# parse_errors=3 (lifetime), malformed_in_window=3 (windowed). Category: Anomaly.
# Summary contains "possible Crain-Sistrunk crash-probe" and flow fragment.
# Traces to: BC-2.15.024 postconditions 2/3; STORY-109 AC-012.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_malformed_anomaly_at_threshold_3_of_300s -- --nocapture"
Enter
Sleep 5s
