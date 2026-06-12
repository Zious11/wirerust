Output docs/demo-evidence/STORY-109/AC-010-disable-unsolicited.gif
Output docs/demo-evidence/STORY-109/AC-010-disable-unsolicited.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-010: FC=0x15 (DISABLE_UNSOLICITED) → T0814, verdict Likely, confidence Medium, per-occurrence.
# Raw FC check (not via classify_dnp3_fc). Summary: "DNP3 DISABLE_UNSOLICITED observed: ..."
# Traces to: BC-2.15.023 postcondition 1; STORY-109 AC-010.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_disable_unsolicited_emits_t0814_likely_medium -- --nocapture"
Enter
Sleep 5s
