Output docs/demo-evidence/STORY-108/AC-001-control-fc-counter.gif
Output docs/demo-evidence/STORY-108/AC-001-control-fc-counter.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-001: Every Control-class FC (0x03..0x06) on FIR=1 increments direct_operate_count.
# On first Control-class FC in new window, window_start_ts = now_ts.
# Traces to: BC-2.15.010 postconditions 1/2; STORY-108 AC-001.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_direct_operate_count_increments_on_control_fc -- --nocapture"
Enter
Sleep 5s
