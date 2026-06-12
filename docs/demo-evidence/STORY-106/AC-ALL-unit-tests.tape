# VHS Demo — STORY-106 DNP3 Parse Core Unit Tests (AC-001..AC-009)
#
# Demonstrates all 36 unit tests passing across AC-001 through AC-009.
# This is a PURE-CORE LIBRARY story — evidence is cargo test output.
#
# Usage: vhs AC-ALL-unit-tests.tape  (run from worktree root)
# Output: AC-ALL-unit-tests.gif + AC-ALL-unit-tests.webm

Output docs/demo-evidence/STORY-106/AC-ALL-unit-tests.gif
Output docs/demo-evidence/STORY-106/AC-ALL-unit-tests.webm

Require cargo

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontSize 16
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms

# === Hidden Setup ===
Hide
Sleep 300ms
Show

# === AC-001..AC-009: All DNP3 Parse Core Tests ===
Type "cargo test --test dnp3_parse_core_tests"
Enter
Sleep 15s
Sleep 2s
