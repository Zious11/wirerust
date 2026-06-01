Output AC-002-memcap-zero-rejected.gif
Output AC-002-memcap-zero-rejected.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 900
Set Height 300
Set Theme "Dracula"
Set Padding 20

Hide
Type "BINARY=/Users/zious/Documents/GITHUB/wirerust/.worktrees/FIX-P5-002/target/debug/wirerust"
Enter
Type "PCAP=/Users/zious/Documents/GITHUB/wirerust/.worktrees/FIX-P5-002/tests/fixtures/http.pcap"
Enter
Show

Type `$BINARY analyze --reassembly-memcap 0 "$PCAP" 2>&1; echo "exit: $?"`
Sleep 200ms
Enter
Sleep 2s
