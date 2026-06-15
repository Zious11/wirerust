Output AC-009-sliceerror-len-contracts.gif
Output AC-009-sliceerror-len-contracts.webm

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-111"
Enter
Sleep 1s
Show

Type "# AC-009: SliceError::Len contract tests still green after etherparse 0.20 migration"
Enter
Sleep 0.3s
Type "# VP-008 fuzz harness return type updated to Result<DecodedFrame>"
Enter
Sleep 0.3s
Type "cargo test test_decode_snaplen_truncated_ipv6 test_decode_structurally_corrupt test_VP_008 2>&1 | grep -E '(test |result)'"
Enter
Sleep 10s
