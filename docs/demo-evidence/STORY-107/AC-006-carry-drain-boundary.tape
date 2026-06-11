Output docs/demo-evidence/STORY-107/AC-006-carry-drain-boundary.gif
Output docs/demo-evidence/STORY-107/AC-006-carry-drain-boundary.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-006: VP-023 Sub-D carry-drain boundary (LENGTH=5 minimum frame, frame_len=10)
# Traces to: BC-2.15.016 invariant 1; VP-023 Sub-D; STORY-107 AC-006

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-107 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_flow_state_tests test_carry_drain_boundary_min_frame -- --nocapture"
Enter
Sleep 5s
