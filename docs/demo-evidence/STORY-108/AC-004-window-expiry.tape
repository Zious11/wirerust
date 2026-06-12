Output docs/demo-evidence/STORY-108/AC-004-window-expiry.gif
Output docs/demo-evidence/STORY-108/AC-004-window-expiry.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-004: Window expiry resets counter — when now_ts.wrapping_sub(window_start_ts) > 60s,
# direct_operate_count=1, window_start_ts=now_ts, direct_operate_emitted=false.
# Second finding possible after reset.
# Traces to: BC-2.15.010 postcondition 4; STORY-108 AC-004.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_t1692_001_window_expiry_resets_counter -- --nocapture"
Enter
Sleep 5s
