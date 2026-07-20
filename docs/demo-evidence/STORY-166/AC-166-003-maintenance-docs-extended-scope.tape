# STORY-166 AC-166-003: the two amended maintenance docs' new sections.
# .factory/maintenance/ lives on the `factory-artifacts` branch, a
# SEPARATE git worktree mounted as a sibling of .worktrees/ at the parent
# repo root (NOT inside this story worktree, NOT on this feature branch).
# Reached here via a relative traversal (../../.factory/maintenance) from
# the worktree cwd -- this avoids embedding any absolute host path in the
# recording while still reading the real, live files on the
# factory-artifacts tree. See evidence-report.md for the branch-split note.
# Traces: STORY-166 AC-166-003(a), (b).

Output docs/demo-evidence/STORY-166/AC-166-003-maintenance-docs-extended-scope.gif
Output docs/demo-evidence/STORY-166/AC-166-003-maintenance-docs-extended-scope.webm

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

Type "# (a) demo-evidence-scrub-gate.md: new .factory/demo-evidence/ Extended Scope section"
Enter
Sleep 300ms
Type "grep -n -A8 'Extended Scope' ../../.factory/maintenance/demo-evidence-scrub-gate.md"
Enter
Wait
Sleep 1500ms

Type "# (b) delivery-doc-currency-protocol.md: Step 3 note on .factory/demo-evidence/"
Enter
Sleep 300ms
Type "grep -n -B1 -A3 'also subject to the path-scrub gate' ../../.factory/maintenance/delivery-doc-currency-protocol.md"
Enter
Wait
Sleep 2s
