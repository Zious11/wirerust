# STORY-186 AC-186-004/005/006: carry bound + defense-in-depth overflow guard (BC-2.20.014)
#
# Demonstrates:
#   - test_BC_2_20_014_at_bound_residual_no_overflow: [SYNTHETIC] residual == 65,535 (the
#     u16::MAX ceiling), seeded via direct flow-state injection -- pins the guard's
#     strict '>' (not '>=') comparison at the literal boundary; UNREACHABLE via real
#     on_data traffic, since a residual that large is itself a complete, dispatchable
#     frame that the walk-first loop would extract, not stash to carry (AC-186-004(a))
#   - test_BC_2_20_014_live_near_bound_residual_reachable: [LIVE] the actual maximum
#     residual reachable via real on_data traffic -- 65,534 bytes -- delivered
#     progressively across multiple on_data calls, then completed by its final byte
#     (AC-186-004(b))
#   - test_BC_2_20_014_live_near_bound_residual_single_call: [LIVE] single-call variant
#     of the same 65,534-byte near-bound residual, delivered in one on_data call
#     (AC-186-004(b))
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
# Reclassification: BC-2.20.014 v1.2 (STORY-186 v1.2 consistency audit finding #2,
# 2026-09-24, FIX-STORY186-ATBOUND-RELABEL) -- the at-bound case (AC-186-004(a)) was
# previously mislabeled LIVE; it is actually SYNTHETIC direct-flow-state injection (same
# convention as AC-186-005/006), since a 65,535-byte residual is unrealizable via real
# on_data traffic under the walk-first design. The genuinely LIVE, on_data-reachable case
# is the 65,534-byte near-bound residual (AC-186-004(b)), covered by the two new tests
# above. AC-186-005/006 (over-bound) remain SYNTHETIC guard-mechanics tests exercised
# only via direct flow-state injection.
#
# Traces to: BC-2.20.014 invariant 1, postconditions 1/3/4, edge cases EC-001/EC-002/
# EC-004/EC-005/EC-006

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

Type "# AC-186-004(a) SYNTHETIC at-bound + (b) LIVE near-bound + AC-186-005/006 SYNTHETIC guard"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests BC_2_20_014 2>&1 | grep -E 'story_186::test|test result:'"
Enter
Wait
Sleep 2s
