Output AC-001-tls-frag-evasion-baseline.gif
Output AC-001-tls-frag-evasion-baseline.webm

Require wirerust

Set FontFamily "Menlo"
Set FontSize 14
Set Width 900
Set Height 420
Set Theme "Dracula"
Set Padding 20
Set WaitTimeout 30s

# Setup: navigate to the repo
Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust"
Enter
Sleep 300ms
Show

# ── Title banner ───────────────────────────────────────────────────────────────
Type "echo '=== BEFORE fix: TLS ClientHello fragmentation evasion (develop) ==='"
Enter
Sleep 600ms

# ── CONTROL: single TLS record, ClientHello with SNI ─────────────────────────
Type "echo '--- CONTROL: ClientHello in ONE TLS record ---'"
Enter
Sleep 300ms
Type ".factory/demo-evidence/fix-tls-clienthello-frag/show-tls-result.sh .factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-control.pcap 'CONTROL (single TLS record)'"
Enter
Wait+Screen /parse_errors/
Sleep 1500ms

# ── EVASION: fragmented ClientHello across two TLS records ───────────────────
Type "echo '--- EVASION: same ClientHello fragmented across 2 TLS records ---'"
Enter
Sleep 300ms
Type ".factory/demo-evidence/fix-tls-clienthello-frag/show-tls-result.sh .factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-fragmented.pcap 'EVASION (2 TLS records)'"
Enter
Wait+Screen /MISSED/
Sleep 3s
