# STORY-133 AC-133-001 / AC-133-002 / AC-133-003: technique_info arms for T0858, T0816, T1693.001
#
# AC-001: technique_info("T0858") → TechniqueInfo { id:"T0858", name:"Change Operating Mode",
#          tactic: MitreTactic::IcsExecution, description: (ODVA EtherNet/IP context) }
# AC-002: technique_info("T0816") → TechniqueInfo { id:"T0816", name:"Device Restart/Shutdown",
#          tactic: MitreTactic::IcsInhibitResponseFunction, ... }
# AC-003: technique_info("T1693.001") → TechniqueInfo { id:"T1693.001",
#          name:"Modify Firmware: System Firmware", tactic: IcsInhibitResponseFunction }
#          T1693.001 is SEEDED (in catalog) but NOT in EMITTED_IDS (staged-only, v0.12.0)
#
# Tests: mitre_seeding::test_technique_info_t0858
#        mitre_seeding::test_technique_info_t0816
#        mitre_seeding::test_technique_info_t1693_001
# Traces: VP-007 Step 1 (T0858/T0816/T1693.001 arms)

Output AC-001-002-003-technique-info.gif
Output AC-001-002-003-technique-info.webm

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

Type "# AC-133-001/002/003: technique_info arms — T0858, T0816, T1693.001 (VP-007 Step 1)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'mitre_seeding::test_technique_info' 2>&1 | grep -E 'test_technique_info|ok|FAILED|result'"
Enter
Wait
Sleep 2s
