Output docs/demo-evidence/STORY-107/AC-002-carry-frame-consumption.gif
Output docs/demo-evidence/STORY-107/AC-002-carry-frame-consumption.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-002: frame consumed from carry; remainder stays; frame_count++
# Traces to: BC-2.15.016 postconditions 3-4; STORY-107 AC-002

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-107 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_flow_state_tests test_carry_buffer_frame_consumption -- --nocapture"
Enter
Sleep 5s
