Output docs/demo-evidence/STORY-109/AC-001-block-event-count.gif
Output docs/demo-evidence/STORY-109/AC-001-block-event-count.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-001: FC in {0x03, 0x04, 0x05} (NOT 0x06) timeout → block_event_count += 1 unconditionally.
# FC=0x06 (DIRECT_OPERATE_NR) is EXCLUDED — no response expected by design.
# Traces to: BC-2.15.014 postcondition 1; STORY-109 AC-001.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-109 && cargo build 2>/dev/null"
Enter
Sleep 10s
Show

Type "cargo test --test dnp3_correlation_tests test_block_event_count_increments_unconditionally -- --nocapture"
Enter
Sleep 5s
