Output AC-144-002-reassembly-fragmented.gif
Output AC-144-002-reassembly-fragmented.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 900
Set Height 380
Set Theme "Dracula"
Set Padding 20
Set WaitTimeout 30s

# Setup: navigate to the worktree and put the STORY-144 binary on PATH
Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/story-144-tls-carry-reassembly"
Enter
Sleep 200ms
Show

# ── Title banner ───────────────────────────────────────────────────────────────
Type "echo '=== AFTER fix (STORY-144): fragmented ClientHello is now REASSEMBLED ==='"
Enter
Sleep 600ms

# ── EVASION case now DETECTED ─────────────────────────────────────────────────
Type "echo '--- AC-144-002: ClientHello split across 2 TLS records ---'"
Enter
Sleep 300ms
Type "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/fix-tls-clienthello-frag/story-144/show-tls-result.sh /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-fragmented.pcap 'FRAGMENTED (2 TLS records) -- STORY-144'"
Enter
Wait+Screen /parse_errors/
Sleep 3s
