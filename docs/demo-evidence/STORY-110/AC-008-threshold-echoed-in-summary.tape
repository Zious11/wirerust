Output docs/demo-evidence/STORY-110/AC-008-threshold-echoed-in-summary.gif
Output docs/demo-evidence/STORY-110/AC-008-threshold-echoed-in-summary.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-008: The flag value is echoed in the T1692.001 finding summary: "(threshold {N})".
# When omitted, the default (10) appears in the summary.
# Traces to: BC-2.15.017 postcondition 5; STORY-110 AC-008.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build 2>/dev/null"
Enter
Sleep 15s
Show

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_threshold_echoed_in_t1692_summary -- --nocapture"
Enter
Sleep 5s
