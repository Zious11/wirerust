Output AC-146-005-summarize-key-present.gif
Output AC-146-005-summarize-key-present.webm

Require cargo

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set WaitTimeout 60s

# AC-146-005: summarize() exposes "buffer_saturation_drops" in the detail map.
# Success path: wirerust --tls analyze on a real TLS 1.2 pcap shows the new
#               "buffer_saturation_drops: 0" key in ANALYZER: TLS section.
# Error path:   running without --tls omits the TLS analyzer section entirely,
#               demonstrating the key is exclusively from the TLS analyzer.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/story-146-tls-buffer-saturation-telemetry"
Enter
Sleep 300ms
Show

Type "echo '=== STORY-146: buffer_saturation_drops key in summarize() ==='"
Enter
Sleep 600ms

Type "echo '--- Success path: --tls flag shows buffer_saturation_drops ---'"
Enter
Sleep 300ms

Type "./target/release/wirerust --no-color analyze --tls tests/fixtures/tls12-aes256gcm.pcap"
Enter
Wait+Screen /buffer_saturation_drops/
Sleep 2s

Type "echo '--- Error path: without --tls, no TLS analyzer section ---'"
Enter
Sleep 300ms

Type "./target/release/wirerust --no-color analyze tests/fixtures/tls12-aes256gcm.pcap"
Enter
Wait+Screen /WIRERUST TRIAGE/
Sleep 2s
