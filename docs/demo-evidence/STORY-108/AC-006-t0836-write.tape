Output docs/demo-evidence/STORY-108/AC-006-t0836-write.gif
Output docs/demo-evidence/STORY-108/AC-006-t0836-write.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-006: WRITE FC (0x02) emits T0836 per-occurrence, Confidence::Medium.
# T0836 only -- NOT also T1692.001 (ADR-007 Decision 5 separation).
# 20 WRITEs -> 20 T0836 findings, 0 T1692.001 findings.
# Traces to: BC-2.15.012 postcondition 1; STORY-108 AC-006.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_t0836 -- --nocapture"
Enter
Sleep 5s
