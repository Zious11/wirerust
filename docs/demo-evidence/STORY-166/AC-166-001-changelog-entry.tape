# STORY-166 AC-166-001(f): CHANGELOG obligation (AC-158-001, PG-W71-CHANGELOG).
# The AC-166-001 develop PR modifies bin/validate-citations and
# bin/test_validate_citations.py (both under bin/), so it MUST carry an
# [Unreleased] CHANGELOG entry. This shows the entry actually present.
# Traces: STORY-166 AC-166-001(f).

Output docs/demo-evidence/STORY-166/AC-166-001-changelog-entry.gif
Output docs/demo-evidence/STORY-166/AC-166-001-changelog-entry.webm

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

Type "# AC-166-001(f): [Unreleased] CHANGELOG entry is present"
Enter
Sleep 300ms
Type "grep -n -A3 -F '[Unreleased]' CHANGELOG.md | head"
Enter
Wait
Sleep 2s
