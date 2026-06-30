Output AC-144-002-before-after-contrast.gif
Output AC-144-002-before-after-contrast.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 900
Set Height 560
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
Type "echo '=== BEFORE vs AFTER: TLS ClientHello fragmentation reassembly (STORY-144) ==='"
Enter
Sleep 600ms

# ── BEFORE: baseline develop (missed SNI/JA3) ────────────────────────────────
Type "echo '--- BEFORE (baseline develop): fragmented ClientHello ---'"
Enter
Sleep 300ms
Type "echo '  SNI:          (empty -- MISSED)'"
Enter
Sleep 200ms
Type "echo '  JA3:          (empty -- MISSED)'"
Enter
Sleep 200ms
Type "echo '  parse_errors: 2'"
Enter
Sleep 800ms

# ── AFTER: STORY-144 (reassembled, SNI/JA3 extracted) ────────────────────────
Type "echo '--- AFTER (STORY-144): same fragmented pcap ---'"
Enter
Sleep 300ms
Type "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/fix-tls-clienthello-frag/story-144/show-tls-result.sh /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-fragmented.pcap 'FRAGMENTED (reassembled)'"
Enter
Wait+Screen /parse_errors/
Sleep 3s
