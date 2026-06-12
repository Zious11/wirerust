Output docs/demo-evidence/STORY-109/AC-014-wrapping-sub.gif
Output docs/demo-evidence/STORY-109/AC-014-wrapping-sub.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-014: Block-timeout check uses wrapping_sub — no panic on backward timestamps.
# request at ts=u32::MAX-5, trigger at ts=5: wrapping_sub(5, u32::MAX-5)=11 > 10 → timeout fires.
# overflow-checks=true release profile would panic without wrapping_sub.
# Traces to: BC-2.15.014 invariant 8; BC-2.15.016 invariant 8; STORY-109 AC-014.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_pending_request_timeout_wrapping_sub -- --nocapture"
Enter
Sleep 5s
