Output docs/demo-evidence/STORY-160/AC-160-008-changelog-entry.gif
Output docs/demo-evidence/STORY-160/AC-160-008-changelog-entry.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20

# AC-160-008: CHANGELOG.md contains an Unreleased/v0.12.0 BREAKING CHANGE entry
# covering: (1) enum casing mapping table, (2) schema_version field, (3) terminal/CSV
# unchanged, (4) cargo-semver-checks scope note, (5) Direction enum carve-out.

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-160"
Enter
Sleep 1s
Show

Type "head -62 CHANGELOG.md"
Enter
Sleep 4s
