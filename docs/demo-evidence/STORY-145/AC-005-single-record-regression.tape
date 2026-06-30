Output demos/story-145/AC-005-single-record-regression.webm

Set FontFamily "Menlo"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set PlaybackSpeed 1.0

# AC-145-005: Single-record ServerHello regression check
#
# Verifies that adding S2C carry-buffer drain logic does NOT break analysis
# of a normal (non-fragmented) single-record ServerHello.
#
# Part 1: unit test -- test_parse_server_hello sends a complete single-record
#   ServerHello in S2C direction and asserts server_hello_seen == true.
#
# Part 2: CLI smoke -- wirerust analyze on a real TLS 1.2 pcap still
#   produces a triage report with no errors.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/story-145-tls-serverhello-symmetry"
Enter
Show

Type "# AC-145-005: single-record ServerHello regression -- unit test"
Enter
Sleep 1s
Type "cargo test --test tls_analyzer_tests test_parse_server_hello"
Enter
Sleep 5s

Type "# AC-145-005: CLI smoke -- real TLS 1.2 pcap analyzed without error"
Enter
Sleep 1s
Type "./target/release/wirerust analyze tests/fixtures/tls12-aes256gcm.pcap"
Enter
Sleep 3s
