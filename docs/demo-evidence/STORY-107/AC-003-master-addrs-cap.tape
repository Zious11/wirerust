Output docs/demo-evidence/STORY-107/AC-003-master-addrs-cap.gif
Output docs/demo-evidence/STORY-107/AC-003-master-addrs-cap.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-003: master_addrs_seen bounded at 64 — 65th silently ignored
# Traces to: BC-2.15.016 postconditions 5-6; STORY-107 AC-003

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-107 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_flow_state_tests test_master_addrs_cap_at_64 -- --nocapture"
Enter
Sleep 5s
