Output docs/demo-evidence/STORY-108/AC-005-t0814-restart.gif
Output docs/demo-evidence/STORY-108/AC-005-t0814-restart.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-005: T0814 per-occurrence for COLD_RESTART (0x0D) and WARM_RESTART (0x0E).
# FC 0x0F (INITIALIZE_DATA) does NOT trigger T0814 (Management-class).
# restart_event_count incremented unconditionally.
# Traces to: BC-2.15.011 postconditions 1/2; STORY-108 AC-005.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_t0814 -- --nocapture"
Enter
Sleep 5s
