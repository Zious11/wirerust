Output docs/demo-evidence/STORY-109/AC-003-block-events-300s-window.gif
Output docs/demo-evidence/STORY-109/AC-003-block-events-300s-window.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-003: Two block events at t=0 and t=150s (both within 300s) → block_event_count=2.
# Verifies single shared 300s window: no premature reset at 120s.
# Key Trace B precursor: both events count toward T0827 threshold.
# Traces to: BC-2.15.014 postcondition 2/invariant 7; STORY-109 AC-003.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_block_events_not_reset_at_120s -- --nocapture"
Enter
Sleep 5s
