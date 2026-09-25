# STORY-187 full module: all 63 story_187:: tests green, covering AC-187-001
# through AC-187-013 (S7commFlowState completion, four-way protocol_id dispatch
# skeleton, and parse_s7comm_header pure-core parser)
#
# This is the top-level artifact showing the complete story_187 test module
# passing in one run -- every AC-187-001..013 test is represented in this output.
# (story_187 is one module inside tests/s7comm_analyzer_tests.rs, which also
# hosts STORY-186's story_186 module -- 83 tests total in the file; this
# recording filters to story_187's 63.)
#
# Traces to: BC-2.21.001, BC-2.21.002, BC-2.21.004, BC-2.21.005, BC-2.21.006,
# BC-2.21.007, BC-2.21.008, BC-2.21.009
# Test: cargo test --test s7comm_analyzer_tests story_187::

Output AC-ALL-story_187-green.gif
Output AC-ALL-story_187-green.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 1080
Set FontSize 11
Set Padding 24
Set Shell "bash"
Set TypingSpeed 30ms

Require cargo

Type "# STORY-187: full story_187 test module -- 63/63 green"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests story_187:: 2>&1 | grep -E 'running [0-9]+ tests|story_187::|test result:'"
Enter
Wait
Sleep 3s
