Output docs/demo-evidence/STORY-109/AC-011-enable-unsolicited.gif
Output docs/demo-evidence/STORY-109/AC-011-enable-unsolicited.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-011: FC=0x14 (ENABLE_UNSOLICITED) → T0814, verdict Possible, confidence Low, per-occurrence.
# Raw FC check. Evidence: "FC=0x14 dest={dest} src={src}" (dest before src, no FC name parens).
# Traces to: BC-2.15.023 postcondition 1; STORY-109 AC-011.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_enable_unsolicited_emits_t0814_possible_low -- --nocapture"
Enter
Sleep 5s
