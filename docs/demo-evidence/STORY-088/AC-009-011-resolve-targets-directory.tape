Output AC-009-011-resolve-targets-directory.gif
Output AC-009-011-resolve-targets-directory.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-088"
Enter
Sleep 300ms
Type "DEMODIR=$(mktemp -d)"
Enter
Sleep 300ms
Type "cp tests/fixtures/http-ooo.pcap $DEMODIR/a.pcap"
Enter
Sleep 200ms
Type "cp tests/fixtures/dns-remoteshell.pcap $DEMODIR/b.pcap"
Enter
Sleep 200ms
Type "cp tests/fixtures/tls.pcap $DEMODIR/c.pcapng"
Enter
Sleep 200ms
Type "cp tests/fixtures/http.pcap $DEMODIR/d.PCAP"
Enter
Sleep 200ms
Type "mkdir $DEMODIR/subdir && cp tests/fixtures/dns-remoteshell.pcap $DEMODIR/subdir/nested.pcap"
Enter
Sleep 300ms
Show

Type "# AC-009/010/011: dir has .pcapng, .PCAP, subdir; only a.pcap + b.pcap processed"
Enter
Sleep 500ms

Type "ls $DEMODIR"
Enter
Sleep 1000ms

Type "./target/debug/wirerust analyze $DEMODIR --dns --no-reassemble 2>&1 | grep -E 'Packets:|ANALYZER:|dns_queries'"
Enter
Sleep 3s

Hide
Type "rm -rf $DEMODIR"
Enter
Sleep 300ms
