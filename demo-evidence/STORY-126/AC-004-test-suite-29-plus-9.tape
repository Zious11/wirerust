Output .factory/demo-evidence/STORY-126/AC-004-test-suite-29-plus-9.gif
Output .factory/demo-evidence/STORY-126/AC-004-test-suite-29-plus-9.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 13
Set FontFamily "Menlo"
Set Padding 20

# AC-004: Test evidence -- 29 SPB unit tests + 9 VP-029/031 proptests all pass
# Demonstrates: full BC-2.01.013/015/017 and VP-029/031 test suite green.
# Traces to: STORY-126 overall; all ACs in bc_2_01_story126_spb_tests.rs
# and proptest_reader.rs.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-126"
Enter
Sleep 1s
Show

Type "# 29 unit tests: BC-2.01.013 / 015 / 017 (SPB parse, block-skip, error surface)"
Enter
Sleep 300ms

Type "cargo test --test bc_2_01_story126_spb_tests 2>&1 | tail -5"
Enter
Sleep 15s

Type "# 9 VP-029/031 property tests (1000 cases each)"
Enter
Sleep 300ms

Type "cargo test --test proptest_reader 2>&1 | tail -5"
Enter
Sleep 15s
