# Wave-71 Integration Demo: DEMO-002 — Fragmented ClientHello Equivalence
# Evidences: STORY-150 key behavior-preservation claim
# Both fragmented and control pcaps yield identical JA3 (6169fabc...) and SNI.
#
# RECORDING NOTE: VHS preferred but failed due to output scrolling past viewport.
# Use DEMO-002-frag-equivalence.txt transcript as primary evidence artifact.

Output DEMO-002-frag-equivalence.gif
Output DEMO-002-frag-equivalence.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set Padding 20
Set WaitTimeout 30s

# Setup: navigate to repo root
Hide
Type "cd <repo>"
Enter
Sleep 500ms
Show

Type "# Wave-71 Demo 2 — fragmented ClientHello: JA3 and SNI match control"
Enter
Sleep 400ms

Type "./target/release/wirerust analyze .factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-fragmented.pcap --tls --json 2>/dev/null | jq '{ja3: .analyzers[1].detail.ja3_hashes, sni: .analyzers[1].detail.top_snis}'"
Enter
Wait+Screen /example.com/
Sleep 800ms

Type "./target/release/wirerust analyze .factory/demo-evidence/fix-tls-clienthello-frag/tls-clienthello-control.pcap --tls --json 2>/dev/null | jq '{ja3: .analyzers[1].detail.ja3_hashes, sni: .analyzers[1].detail.top_snis}'"
Enter
Wait+Screen /6169fabc/
Sleep 2s
