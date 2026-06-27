Output AC-139-001-direction-split.gif
Output AC-139-001-direction-split.webm

Require cargo

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20

# AC-139-001: carry_c2s/carry_s2c direction split + on_data Direction param
# Success path: three EC-X1 tests pass — cross-direction isolation, independent buffers,
#               direction-based source IP.
# Error path: misspelled test name → 0 tests run (demonstrates test filter precision).

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/enip-direction-clock"
Enter
Sleep 2s
Show

Type "cargo test --test enip_analyzer_tests direction_and_clock::test_ec_x1_cross_direction_no_splice 2>&1"
Enter
Sleep 8s

Type "cargo test --test enip_analyzer_tests direction_and_clock::test_carry_c2s_and_carry_s2c_are_independent 2>&1"
Enter
Sleep 8s

Type "cargo test --test enip_analyzer_tests direction_and_clock::test_direction_based_source_ip 2>&1"
Enter
Sleep 8s

Type "# Error path: misspelled test name → 0 tests run"
Enter
Sleep 1s

Type "cargo test --test enip_analyzer_tests direction_and_clock::test_ec_x1_cross_direction_spliced 2>&1"
Enter
Sleep 8s

Sleep 2s
