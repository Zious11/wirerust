Output docs/demo-evidence/STORY-108/AC-002-t1692-emission.gif
Output docs/demo-evidence/STORY-108/AC-002-t1692-emission.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20

# AC-002: T1692.001 emitted at 11th Control FC (threshold=10; check is >, not >=).
# No finding at count=10; exactly ONE finding at count=11.
# Traces to: BC-2.15.010 postcondition 3; Canonical Test Vector §1; STORY-108 AC-002.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-108 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_detection_tests test_t1692_001_emitted_at_threshold_plus_one -- --nocapture"
Enter
Sleep 5s
