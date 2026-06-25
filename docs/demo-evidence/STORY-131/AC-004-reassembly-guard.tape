# STORY-131 AC-131-004: Missing TCP reassembly guard
#
# AC-004: --enip + --no-reassemble emits WARNING to stderr and disables ENIP analysis
# Expected stderr: "--enip requires TCP reassembly; ENIP analysis disabled"
# No ENIP analysis proceeds; mirrors --modbus/--dnp3 reassembly-guard pattern.
# Traces: BC-2.17.020 postcondition 2

Output AC-004-reassembly-guard.gif
Output AC-004-reassembly-guard.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-131-enip-dispatcher"
Enter
Wait
Show

Type "# AC-131-004: --enip without TCP reassembly emits warning (BC-2.17.020 postcondition 2)"
Enter
Sleep 300ms

Type "cargo run -- analyze tests/fixtures/slammer.pcap --enip --no-reassemble 2>&1 | grep -iE 'enip|reassembl'"
Enter
Wait
Sleep 2s
