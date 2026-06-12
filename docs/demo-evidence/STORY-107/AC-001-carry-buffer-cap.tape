Output docs/demo-evidence/STORY-107/AC-001-carry-buffer-cap.gif
Output docs/demo-evidence/STORY-107/AC-001-carry-buffer-cap.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-001: carry buffer capped at 292 — carry.len() NEVER exceeds MAX_DNP3_FRAME_LEN
# Traces to: BC-2.15.016 postconditions 1-2; STORY-107 AC-001

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-107 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_flow_state_tests test_carry_buffer_cap_at_292 -- --nocapture"
Enter
Sleep 5s
