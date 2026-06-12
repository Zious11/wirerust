Output docs/demo-evidence/STORY-109/AC-006-t0827-distinct-events.gif
Output docs/demo-evidence/STORY-109/AC-006-t0827-distinct-events.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-006: T0827 requires distinct events (restart + block). 2 restarts + 0 blocks = 2 < threshold.
# No T0827 emitted (combined must reach 3 from distinct event types).
# Traces to: BC-2.15.015 invariant 7; BC-2.15.015 EC-001; STORY-109 AC-006.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_t0827_requires_distinct_events -- --nocapture"
Enter
Sleep 5s
