Output docs/demo-evidence/STORY-107/AC-004-frame-count-increments.gif
Output docs/demo-evidence/STORY-107/AC-004-frame-count-increments.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-004: frame_count increments by 1 per complete frame consumed
# Traces to: BC-2.15.016 postcondition 7; STORY-107 AC-004

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-107 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_flow_state_tests test_frame_count_increments -- --nocapture"
Enter
Sleep 5s
