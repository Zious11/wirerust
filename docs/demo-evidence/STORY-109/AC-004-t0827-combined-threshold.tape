Output docs/demo-evidence/STORY-109/AC-004-t0827-combined-threshold.gif
Output docs/demo-evidence/STORY-109/AC-004-t0827-combined-threshold.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-004: Trace B — 2 block events + 1 restart = combined 3 → T0827 emitted.
# T0827: category Impact, tactic IcsImpact (NEW variant), verdict Likely, confidence Medium.
# Block-crossing regression tests: also covers F-P9-001 (Trace C-rev and Trace D).
# Traces to: BC-2.15.015 postconditions 1/2; Canonical Trace B; STORY-109 AC-004.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests 'test_t0827_emitted_at_combined_threshold\|test_t0827_emitted_when_block_crosses_threshold_after_restarts\|test_t0827_emitted_when_second_block_crosses_threshold_after_one_restart' -- --nocapture"
Enter
Sleep 8s
