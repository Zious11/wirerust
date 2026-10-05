# STORY-188 fixture pcap end-to-end FC classification (supporting AC-188-001..008)
# Library story: demonstration vehicle is the story_188 test harness
# (tests/s7comm_analyzer_tests.rs, mod story_188); no CLI surface until STORY-193.
# Test: cargo test --test s7comm_analyzer_tests story_188::test_BC_2_21_010_fc_classification_fixture

Output AC-FIXTURE-e2e-fc-classification-pcap.gif
Output AC-FIXTURE-e2e-fc-classification-pcap.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 520
Set FontSize 13
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# STORY-188 fixture pcap end-to-end FC classification (supporting AC-188-001..008)"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests story_188::test_BC_2_21_010_fc_classification_fixture 2>&1 | grep -E 'running [0-9]+ tests|story_188::|test result:'"
Enter
Wait
Sleep 3s
