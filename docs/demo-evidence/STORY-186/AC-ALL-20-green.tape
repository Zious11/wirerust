# STORY-186 full suite: all 20 tests green (17 unit/regression-guard tests + 3 VP-050
# proptests) across BC-2.20.013/014/015/016 and BC-2.21.003
#
# This is the top-level artifact showing the complete s7comm_analyzer_tests.rs suite
# passing in one run -- every AC-186-001..012 test is represented in this output.
#
# Test count updated 2026-09-24 (FIX-STORY186-ATBOUND-RELABEL): the at-bound test
# (AC-186-004(a)) was relabeled SYNTHETIC and two new LIVE near-bound tests
# (AC-186-004(b)) were added, bringing the suite from 18 to 20 tests.

Output AC-ALL-20-green.gif
Output AC-ALL-20-green.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 900
Set FontSize 13
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# STORY-186: full s7comm_analyzer_tests suite -- 20/20 green"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests 2>&1 | grep -E 'running [0-9]+ tests|story_186::|test result:'"
Enter
Wait
Sleep 3s
