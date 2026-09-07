# STORY-186 AC-186-007/008/009: 1-byte resync, never 2 (BC-2.20.015)
#
# Demonstrates:
#   - test_BC_2_20_015_resync_advances_exactly_one_byte: bytes
#     [0x01, 0x03, 0x00, 0x00, 0x07] -- a spurious 0x01 immediately followed by a valid
#     length=7 frame at offset 1; a 2-byte advance would skip it entirely (AC-186-007)
#   - test_BC_2_20_015_single_resync_implementation_shared: the same 1-byte-advance
#     sub-routine is invoked for both bad-version-byte and post-overflow conditions --
#     exactly one resync implementation, not two (AC-186-008)
#   - test_BC_2_20_015_resync_terminates_no_valid_anchor: 200 bytes of non-0x03 garbage
#     with no valid frame anywhere -- resync advances to the end without an infinite
#     loop (AC-186-009)
#
# Traces to: BC-2.20.015 postcondition 1, invariants 1-3

Output AC-007-009-resync.gif
Output AC-007-009-resync.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# AC-186-007/008/009: resync advances exactly 1 byte, never 2 (BC-2.20.015)"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests BC_2_20_015 2>&1 | grep -E 'story_186::test|test result:'"
Enter
Wait
Sleep 2s
