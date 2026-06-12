Output docs/demo-evidence/STORY-108/AC-008-max-findings-cap.gif
Output docs/demo-evidence/STORY-108/AC-008-max-findings-cap.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-008: MAX_FINDINGS cap preserves first finding in multi-finding sequence.
# Pre-fill to MAX_FINDINGS-1; COLD_RESTART pushes T0814 (last slot consumed).
# Second COLD_RESTART: cap hit, T0814 dropped, restart_event_count still=2.
# Traces to: BC-2.15.013 postconditions 4/5; BC-2.15.022; STORY-108 AC-008.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_max_findings_cap_preserves_first_finding -- --nocapture"
Enter
Sleep 5s
