Output demos/story-145/AC-001-002-direction-carry-drain.webm

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

# AC-145-001 / AC-145-002
# Direction-parameterized carry drain + C2S/S2C carry isolation
#
# proptest_vp039_direction_isolation: delivers fragmented ClientHello (C2S) and
# fragmented ServerHello (S2C) in interleaved order; each direction drains its
# own carry buffer independently; both client_hello_seen and server_hello_seen
# end true; parse_errors == 0; both carry buffers drain to zero.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/story-145-tls-serverhello-symmetry"
Enter
Show

Type "# AC-145-001/002: fragmented ServerHello S2C carry drain + direction isolation"
Enter
Sleep 1s
Type "cargo test --test tls_analyzer_tests story_145::proptest_vp039_direction_isolation"
Enter
Sleep 5s
