# Wave-71 Integration Demo: DEMO-001 — Combined Analyzer Run (TLS + ARP)
# Evidences: STORY-150 (TLS drain-loop) + STORY-156 (ARP BC-2.16.016)
# Note: output piped through grep to keep within VHS viewport.
#
# RECORDING NOTE: VHS preferred but this tape failed due to wirerust's unicode
# output (box-drawing chars) scrolling past VHS viewport before Wait+Screen match.
# Use DEMO-001-combined-analyzer.txt transcript as primary evidence artifact.
# This tape is retained as a runnable reference for future re-recording attempts.

Output DEMO-001-combined-analyzer.gif
Output DEMO-001-combined-analyzer.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set WaitTimeout 60s

# Setup: navigate to repo root
Hide
Type "cd <repo>"
Enter
Sleep 500ms
Show

Type "# Wave-71 Demo 1 — TLS path (STORY-150 drain-loop refactor)"
Enter
Sleep 400ms

Type "./target/release/wirerust analyze tests/fixtures/tls13-rfc8446.pcap --tls --json 2>/dev/null | grep -E 'ja3_hashes|tls_versions|top_snis'"
Enter
Wait+Screen /ja3_hashes/
Sleep 800ms

Type "# ARP path (STORY-156 BC-2.16.016 coverage)"
Enter
Sleep 300ms

Type "./target/release/wirerust analyze tests/fixtures/local-samples/arp-storm.pcap --arp --arp-storm-rate 5 --no-color | grep -E 'D3:|storm_findings|frames_analyzed'"
Enter
Wait+Screen /storm_findings/
Sleep 2s
