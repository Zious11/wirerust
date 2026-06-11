Output docs/demo-evidence/STORY-108/AC-003-one-shot-guard.gif
Output docs/demo-evidence/STORY-108/AC-003-one-shot-guard.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-003: One-shot guard — after direct_operate_emitted=true, additional Control FCs
# in the same window do NOT push more T1692.001 findings.
# 16 total Control FCs -> exactly 1 finding.
# Traces to: BC-2.15.010 postcondition 3 guard; Canonical Test Vector §3; STORY-108 AC-003.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_t1692_001_one_shot_guard -- --nocapture"
Enter
Sleep 5s
