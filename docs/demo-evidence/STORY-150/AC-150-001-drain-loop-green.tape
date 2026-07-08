# AC-150-001 (success path): drain-loop DRY tests 10/10 green
# process_handshake_carry unified; exactly 1 parse_tls_message_handshake
# call site and at most 1 msg_bytes extraction — both structural tests pass.
Output docs/demo-evidence/STORY-150/AC-150-001-drain-loop-green.gif
Output docs/demo-evidence/STORY-150/AC-150-001-drain-loop-green.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set Framerate 15
Set TypingSpeed 60ms

Type "cargo test --test bc_150_drain_loop_dry_tests 2>&1"
Enter
Sleep 8s
