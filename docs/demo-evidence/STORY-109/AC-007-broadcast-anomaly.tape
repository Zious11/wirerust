Output docs/demo-evidence/STORY-109/AC-007-broadcast-anomaly.gif
Output docs/demo-evidence/STORY-109/AC-007-broadcast-anomaly.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-007: DEST in broadcast range (0xFFFD/0xFFFE/0xFFFF) + Control-class FC → T1692.001 anomaly.
# Suspicious/Possible/Medium. READ to broadcast: NO anomaly.
# direct_operate_count also incremented (feeds burst threshold).
# Traces to: BC-2.15.018 postcondition 1; BC-2.15.018 EC-004; STORY-109 AC-007.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests 'test_broadcast_control_anomaly_fires_for_dest_ffff\|test_broadcast_read_no_anomaly' -- --nocapture"
Enter
Sleep 6s
