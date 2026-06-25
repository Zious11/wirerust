# STORY-133 AC-133-005 / AC-133-006 / AC-133-007: SEEDED=28, EMITTED=20, T1693.001 not emitted,
#                                                    full mitre regression suite
#
# AC-005: SEEDED array has exactly 28 entries (25 baseline + T0858, T0816, T1693.001)
#          SEEDED_TECHNIQUE_ID_COUNT constant == 28
# AC-006: EMITTED_IDS has exactly 20 entries (17 baseline + T0858, T0816, T0846)
#          T1693.001 NOT in EMITTED_IDS (staged-only for v0.12.0)
#          T0846 IS in EMITTED_IDS (promoted from seeded-only; first emitter STORY-134)
# AC-007: All existing mitre consistency tests continue to pass (no regressions)
#
# Tests: mitre_seeding::test_seeded_count_is_28
#        mitre_seeding::test_emitted_count_is_20
#        mitre_seeding::test_t1693_001_not_emitted
#        mitre_seeding::test_t0846_in_emitted
#        All mitre_* tests (regression)
# Traces: VP-007 Steps 2, 3, 4, 6

Output AC-005-006-007-counts-regression.gif
Output AC-005-006-007-counts-regression.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-133-mitre-ics-seeding"
Enter
Wait
Show

Type "# AC-133-005/006: SEEDED=28, EMITTED=20, T1693.001 not emitted, T0846 emitted (VP-007 Steps 2-4)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'mitre_seeding::test_seeded\|mitre_seeding::test_emitted\|mitre_seeding::test_t1693\|mitre_seeding::test_t0846' 2>&1 | grep -E 'test_seeded|test_emitted|test_t1693|test_t0846|ok|FAILED|result'"
Enter
Wait
Sleep 500ms

Type "# AC-133-007: full mitre regression — cargo test mitre (VP-007 Step 6)"
Enter
Sleep 300ms

Type "cargo test mitre 2>&1 | grep -E 'test mitre|ok|FAILED|test result'"
Enter
Wait
Sleep 2s
