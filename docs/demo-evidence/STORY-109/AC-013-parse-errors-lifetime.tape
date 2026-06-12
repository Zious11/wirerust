Output docs/demo-evidence/STORY-109/AC-013-parse-errors-lifetime.gif
Output docs/demo-evidence/STORY-109/AC-013-parse-errors-lifetime.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-013: parse_errors is a lifetime monotonic counter — NEVER reset at window expiry.
# After 300s expiry: parse_errors stays at 3; malformed_in_window resets to 0.
# Two-counter model per BC-2.15.024 v1.1 (F-C-2 adversarial fix).
# Traces to: BC-2.15.024 invariant 1; BC-2.15.015 postcondition 3; STORY-109 AC-013.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_parse_errors_not_reset_at_window_expiry -- --nocapture"
Enter
Sleep 5s
