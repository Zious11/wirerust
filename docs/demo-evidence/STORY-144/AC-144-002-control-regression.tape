Output AC-144-002-control-regression.gif
Output AC-144-002-control-regression.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 900
Set Height 380
Set Theme "Dracula"
Set Padding 20
Set WaitTimeout 30s

# Setup: navigate to the worktree
Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/story-144-tls-carry-reassembly"
Enter
Sleep 200ms
Show

# ── Title banner ───────────────────────────────────────────────────────────────
Type "echo '=== STORY-144: single-record ClientHello regression check ==='"
Enter
Sleep 600ms

# ── CONTROL: single TLS record, must still work ───────────────────────────────
Type "echo '--- Control regression: ClientHello in ONE TLS record ---'"
Enter
Sleep 300ms
Type "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/fix-tls-clienthello-frag/story-144/show-tls-result.sh /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-control.pcap 'CONTROL (single TLS record) -- no regression'"
Enter
Wait+Screen /parse_errors/
Sleep 3s
