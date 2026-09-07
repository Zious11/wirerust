# STORY-186 VP-050: proptest obligation (direction isolation, walk-first residual bound,
# resync one-byte-advance) over randomized inputs
#
# Demonstrates:
#   - proptest_vp050_walk_first_residual_bound: carry stays <= MAX_S7_ISO_ON_TCP_CARRY_BYTES
#     across randomized delivery patterns
#   - proptest_vp050_direction_isolation: carry_c2s only ever contains C2S-routed bytes,
#     carry_s2c only ever contains S2C-routed bytes, across 0..300-byte random payloads
#   - proptest_vp050_resync_one_byte_advance: resync never advances by more than 1 byte
#     per iteration, across randomized garbage-length inputs
#
# Method: proptest (randomized property-based testing, anchored in this story;
# the full walk-first equivalence property lands in STORY-194)

Output VP-050-proptests.gif
Output VP-050-proptests.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 460
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# VP-050: proptest harnesses -- direction isolation, walk-first bound, resync advance"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests vp050 2>&1 | grep -E 'story_186::vp050|test result:'"
Enter
Wait
Sleep 2s
