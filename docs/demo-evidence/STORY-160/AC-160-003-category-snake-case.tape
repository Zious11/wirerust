Output docs/demo-evidence/STORY-160/AC-160-003-category-snake-case.gif
Output docs/demo-evidence/STORY-160/AC-160-003-category-snake-case.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20

# AC-160-003: ThreatCategory enum carries rename_all="snake_case"; all ten variants
# serialize as snake_case JSON strings including lateral_movement and c2 (BC-2.11.036).
# Tests: test_BC_2_11_036_threat_category_lateral_movement_snake_case
#        test_BC_2_11_036_threat_category_c2_snake_case
#        test_BC_2_11_036_threat_category_all_variants_snake_case

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-160"
Enter
Sleep 1s
Show

Type "cargo test --test reporter_json_tests test_BC_2_11_036_threat_category 2>&1"
Enter
Sleep 5s
