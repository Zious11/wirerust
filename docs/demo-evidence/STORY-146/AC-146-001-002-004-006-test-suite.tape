Output AC-146-001-002-004-006-test-suite.gif
Output AC-146-001-002-004-006-test-suite.webm

Require cargo

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set WaitTimeout 120s

# AC-146-001 (fill_buf_for_testing seam), AC-146-002 (counter increments on tail-drop),
# AC-146-004 (accessor buffer_saturation_drop_count), AC-146-006 (both directions,
# same aggregate counter) — all covered by the story_146 test module.
#
# The saturation path requires the buffer to hit MAX_BUF=65536 which is only
# reachable via fill_buf_for_testing; the 8-test suite exercises:
#   partial-drop, full-drop, no-drop boundary, exact-fit boundary,
#   both-directions, counter persistence across flow close, and summarize value equality.
#
# Success path: all 8 story_146 tests pass.
# Error path:   misspelled module name -> 0 tests run (demonstrates test filter precision).

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/story-146-tls-buffer-saturation-telemetry"
Enter
Sleep 300ms
Show

Type "echo '=== STORY-146: buffer_saturation_drops counter AC-001/002/004/006 ==='"
Enter
Sleep 600ms

Type "echo '--- Success path: all 8 story_146 tests pass ---'"
Enter
Sleep 300ms

Type "cargo test --test tls_analyzer_tests story_146 2>&1"
Enter
Wait+Screen /test result/
Sleep 2s

Type "echo '--- Error path: misspelled module -> 0 tests run ---'"
Enter
Sleep 300ms

Type "cargo test --test tls_analyzer_tests story_146_typo 2>&1"
Enter
Wait+Screen /test result/
Sleep 2s
