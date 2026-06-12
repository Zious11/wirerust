Output docs/demo-evidence/STORY-109/AC-005-six-field-window-reset.gif
Output docs/demo-evidence/STORY-109/AC-005-six-field-window-reset.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-005: At 300s window expiry, ALL SIX windowed fields reset to defaults.
# parse_errors (lifetime) is NOT reset. Six fields: restart_event_count, block_event_count,
# block_finding_emitted_this_window, loss_of_control_emitted, malformed_in_window,
# malformed_anomaly_emitted.
# Traces to: BC-2.15.015 postcondition 3; BC-2.15.024 Invariant 1; STORY-109 AC-005.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_correlation_window_expiry_resets_six_fields -- --nocapture"
Enter
Sleep 5s
