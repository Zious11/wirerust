# STORY-137 AC-002: Carry buffer bounded below cap; is_non_enip latch unreachable (RULING-137-002)
#
# Demonstrates:
#   - test_carry_stays_bounded_below_cap: carry stays ≤23 bytes under spec algorithm
#   - test_carry_cap_does_not_fire_under_spec_algorithm: is_non_enip latch does NOT fire
#     under normal frame-walk (deferred v0.12.0 spec-defect-is_non_enip-dead-latch)
#   - test_subframe_accumulation_keeps_carry_bounded_no_latch: carry stays bounded (≤23 bytes)
#     across subframe accumulation — is_non_enip latch does NOT fire
#   - test_carry_overflow_third_malformed_fires_t0814_before_latch: T0814 fires BEFORE
#     is_non_enip latch on 3rd malformed (BC-2.17.018 EC-007 ordering constraint)
#
# Note: The carry-overflow is_non_enip latch (carry.len() > 600) is provably unreachable
# under the spec algorithm — BC-2.17.016 loop invariants bound carry below 600 bytes.
# RULING-137-002: deferred to v0.12.0 as spec-defect-is_non_enip-dead-latch.
#
# Traces to: BC-2.17.016 Invariant 1/4; BC-2.17.018 Precondition 6 / EC-007

Output AC-002-carry-bounded.gif
Output AC-002-carry-bounded.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-137-enip-frame-walk"
Enter
Wait
Show

Type "# STORY-137 AC-002: carry bounded; is_non_enip latch unreachable (RULING-137-002)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'frame_walk::test_carry_stays|frame_walk::test_carry_cap|frame_walk::test_subframe|frame_walk::test_carry_overflow' 2>&1 | grep -E 'frame_walk::(test_carry_stays|test_carry_cap|test_subframe|test_carry_overflow)|result'"
Enter
Wait
Sleep 2s
