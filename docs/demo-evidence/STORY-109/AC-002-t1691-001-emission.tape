Output docs/demo-evidence/STORY-109/AC-002-t1691-001-emission.gif
Output docs/demo-evidence/STORY-109/AC-002-t1691-001-emission.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-002: 3 DIRECT_OPERATE requests each without response within 10s →
# T1691.001 emitted at 3rd block event; verdict Possible, confidence Low.
# Exact summary: "DNP3 inferred blocked command: 3 requests without response within 10s (dest=0x0003)"
# Traces to: BC-2.15.014 postcondition 3; Canonical Test Vector §1; STORY-109 AC-002.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_t1691_001_emitted_at_threshold_3_of_300s -- --nocapture"
Enter
Sleep 5s
