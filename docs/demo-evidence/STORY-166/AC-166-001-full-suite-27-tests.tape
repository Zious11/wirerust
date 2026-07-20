# STORY-166 AC-166-001(d)-(e): backward compatibility + new tests.
# Shows: `python3 bin/test_validate_citations.py` running the FULL suite
# (27 tests, including new T23/T24/T25 anchor tests plus T26/T27) to
# "27 passed, 0 failed"; then `grep -c "def test_T" ...` independently
# counting 27 test functions in the source file.
# Traces: STORY-166 AC-166-001(d), (e).

Output docs/demo-evidence/STORY-166/AC-166-001-full-suite-27-tests.gif
Output docs/demo-evidence/STORY-166/AC-166-001-full-suite-27-tests.webm

Require bash
Require python3

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-166"
Enter
Wait
Type "clear"
Enter
Wait
Show

Type "# AC-166-001(d)-(e): full test_validate_citations.py suite, 27 tests"
Enter
Sleep 300ms
Type "python3 bin/test_validate_citations.py 2>&1 | tail -6"
Enter
Wait
Sleep 1s

Type "# independent count: 27 `def test_T...` functions in the source file"
Enter
Sleep 300ms
Type "grep -c 'def test_T' bin/test_validate_citations.py"
Enter
Wait
Sleep 2s
