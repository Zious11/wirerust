# STORY-133 AC-133-004: MitreTactic::IcsExecution variant — Display + tactic_id
#
# AC-004: MitreTactic::IcsExecution is a new enum variant in src/mitre.rs
#          .to_string() == "Execution (ICS)"
#          .tactic_id() == "TA0104"
#          Variant appears in ALL match arms (Display, tactic_id) — exhaustiveness enforced by rustc
#
# Tests: mitre_seeding::test_ics_execution_tactic_display
#        mitre_seeding::test_ics_execution_tactic_id
# Traces: VP-007 Step 5

Output AC-004-ics-execution-tactic.gif
Output AC-004-ics-execution-tactic.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-133-mitre-ics-seeding"
Enter
Wait
Show

Type "# AC-133-004: MitreTactic::IcsExecution — Display 'Execution (ICS)', tactic_id TA0104"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'mitre_seeding::test_ics_execution' 2>&1 | grep -E 'test_ics_execution|ok|FAILED|result'"
Enter
Wait
Sleep 2s
