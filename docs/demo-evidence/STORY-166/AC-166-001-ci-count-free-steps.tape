# STORY-166 AC-166-001(g): W75 NIT-1 resolved -- hardcoded test counts
# `(22 tests)` / `(10 tests)` removed from the bin-selftest CI step names
# (ci.yml previously silently staled these as suites grew). Shows the grep
# for the old hardcoded-count pattern returning EMPTY (negative/error-path
# check that the stale pattern is gone), then the actual count-free step
# names that replaced them.
# Traces: STORY-166 AC-166-001(g).

Output docs/demo-evidence/STORY-166/AC-166-001-ci-count-free-steps.gif
Output docs/demo-evidence/STORY-166/AC-166-001-ci-count-free-steps.webm

Require bash

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

Type "# AC-166-001(g): old hardcoded-count pattern is GONE (empty grep, exit 1)"
Enter
Sleep 300ms
Type `grep -n '(22 tests)\|(10 tests)' .github/workflows/ci.yml; echo "exit=$?"`
Enter
Wait
Sleep 1s

Type "# the count-free bin-selftest step names that replaced them"
Enter
Sleep 300ms
Type "sed -n '472,481p' .github/workflows/ci.yml"
Enter
Wait
Sleep 2s
