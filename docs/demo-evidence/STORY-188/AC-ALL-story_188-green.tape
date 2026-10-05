# STORY-188 full story_188 module: 34/34 green
# Library story: demonstration vehicle is the story_188 test harness
# (tests/s7comm_analyzer_tests.rs, mod story_188); no CLI surface until STORY-193.
# Test: cargo test --test s7comm_analyzer_tests story_188::

Output AC-ALL-story_188-green.gif
Output AC-ALL-story_188-green.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 1080
Set FontSize 11
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# STORY-188 full story_188 module: 34/34 green"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests story_188:: 2>&1 | grep -E 'running [0-9]+ tests|story_188::|test result:'"
Enter
Wait
Sleep 3s
