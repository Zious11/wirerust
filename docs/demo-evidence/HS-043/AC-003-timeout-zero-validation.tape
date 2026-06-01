Output docs/demo-evidence/HS-043/AC-003-timeout-zero-validation.gif
Output docs/demo-evidence/HS-043/AC-003-timeout-zero-validation.webm

Require wirerust

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Padding 20
Set Theme "Dracula"
Set Framerate 24

# AC-003: Error path — --flow-timeout 0 is rejected by clap range validation.
# Minimum is 1; 0 produces "0 is not in 1.." error message.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/hs043-flow-expiry"
Enter
Type "clear"
Enter
Show

Type "# Validation: --flow-timeout 0 is rejected (minimum is 1)"
Enter
Sleep 500ms

Type "wirerust analyze tests/fixtures/flow-expiry.pcap --all --flow-timeout 0"
Enter
Wait+Screen@10s /not in 1/
Sleep 2s
