Output demos/story-145/AC-003-cross-flow-isolation.webm

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

# AC-145-003: Cross-flow isolation
#
# test_BC_2_07_041_cross_flow_isolation: two concurrent flows analyzed by one
# TlsAnalyzer instance.
#   Flow A: complete single-record ClientHello (SNI=a.example) + complete S2C ServerHello
#   Flow B: fragmented 2-record ClientHello (SNI=b.example) + fragmented 2-record S2C ServerHello
#
# Asserts: both flows have server_hello_seen==true; sni_counts has exactly 2
# entries (no cross-flow bleed); ja3s_counts >= 1; parse_errors == 0.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/story-145-tls-serverhello-symmetry"
Enter
Show

Type "# AC-145-003: two concurrent flows with fragmented ServerHellos -- no cross-flow bleed"
Enter
Sleep 1s
Type "cargo test --test tls_analyzer_tests story_145::test_BC_2_07_041_cross_flow_isolation"
Enter
Sleep 5s
