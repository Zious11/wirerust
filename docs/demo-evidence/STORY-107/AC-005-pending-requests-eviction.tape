Output docs/demo-evidence/STORY-107/AC-005-pending-requests-eviction.gif
Output docs/demo-evidence/STORY-107/AC-005-pending-requests-eviction.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-005: pending_requests bounded at 256 with oldest-eviction on overflow
# Traces to: BC-2.15.016 postconditions 8-10; STORY-107 AC-005

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-107 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_flow_state_tests test_pending_requests_eviction_at_256 -- --nocapture"
Enter
Sleep 5s
