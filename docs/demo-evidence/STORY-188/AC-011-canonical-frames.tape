# STORY-188 AC-188-011: canonical public-reference byte vectors (story_188::canonical)
# Library story: demonstration vehicle is the story_188 test harness
# (tests/s7comm_analyzer_tests.rs, mod story_188); no CLI surface until STORY-193.
# Test: cargo test --test s7comm_analyzer_tests story_188::canonical

Output AC-011-canonical-frames.gif
Output AC-011-canonical-frames.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 640
Set FontSize 13
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# STORY-188 AC-188-011: canonical public-reference byte vectors (story_188::canonical)"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests story_188::canonical 2>&1 | grep -E 'running [0-9]+ tests|story_188::|test result:'"
Enter
Wait
Sleep 3s
