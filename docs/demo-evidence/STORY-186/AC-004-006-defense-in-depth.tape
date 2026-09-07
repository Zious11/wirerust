# STORY-186 AC-186-004/005/006: carry bound + defense-in-depth overflow guard (BC-2.20.014)
#
# Demonstrates:
#   - test_BC_2_20_014_at_bound_residual_no_overflow: residual == 65,535 (the u16::MAX
#     ceiling) is LIVE, reachable via real on_data traffic -- no overflow fires, strict
#     '>' not '>=' (AC-186-004, the only reachable-via-on_data case in this group)
#   - test_BC_2_20_014_overflow_clear_resync_one_t0814_per_direction: [SYNTHETIC] guard
#     mechanics via direct flow-state injection -- clear-not-truncate, resync, exactly
#     one T0814 (AC-186-005)
#   - test_BC_2_20_014_repeated_overflow_dedup_same_direction: [SYNTHETIC] second
#     injected overflow in the same direction does not re-emit (AC-186-005)
#   - test_BC_2_20_014_overflow_dedup_independent_per_direction: [SYNTHETIC] c2s and
#     s2c dedup flags are independent (AC-186-006)
#   - test_BC_2_20_014_overflow_unreachable_via_on_data: positive on_data-path proof --
#     a real 200,000-byte non-anchored garbage flood emits NO T0814 and carry stays
#     bounded <= 65,534 (AC-186-005 positive assertion, VP-050 reachability property)
#
# Reclassification: BC-2.20.014 v1.1 (human ruling 2026-09-07, Option B) -- the
# residual.len() > 65,535 guard is a structural defense-in-depth safety net, provably
# unreachable via the real on_data data path under BC-2.20.013 walk-first + BC-2.20.015
# 1-byte-resync (TPKT length is u16-capped, so carry is bounded <= 65,534 by
# construction). AC-186-004 (at-bound) remains live; AC-186-005/006 (over-bound) are
# exercised only via direct flow-state injection, named SYNTHETIC below.
#
# Traces to: BC-2.20.014 invariant 1, postconditions 1/3/4, edge cases EC-001/EC-004/EC-005

Output AC-004-006-defense-in-depth.gif
Output AC-004-006-defense-in-depth.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 560
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# AC-186-004 (LIVE at-bound) + AC-186-005/006 (SYNTHETIC guard mechanics) (BC-2.20.014)"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests BC_2_20_014 2>&1 | grep -E 'story_186::test|test result:'"
Enter
Wait
Sleep 2s
