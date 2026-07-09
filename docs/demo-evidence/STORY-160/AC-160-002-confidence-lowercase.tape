Output docs/demo-evidence/STORY-160/AC-160-002-confidence-lowercase.gif
Output docs/demo-evidence/STORY-160/AC-160-002-confidence-lowercase.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20

# AC-160-002: Confidence enum carries rename_all="lowercase"; all three variants
# serialize as lowercase JSON strings (BC-2.11.036).
# Tests: test_BC_2_11_036_confidence_high_serializes_lowercase
#        test_BC_2_11_036_confidence_all_variants_lowercase

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-160"
Enter
Sleep 1s
Show

Type "cargo test --test reporter_json_tests test_BC_2_11_036_confidence 2>&1"
Enter
Sleep 5s
