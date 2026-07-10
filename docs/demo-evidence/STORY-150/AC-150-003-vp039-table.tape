# AC-150-003 (success path): VP-039 line-correspondence table
# Shows the 8-row table mapping model steps to refactored process_handshake_carry
# production lines, and confirms single parse_tls_message_handshake call site.
Output docs/demo-evidence/STORY-150/AC-150-003-vp039-table.gif
Output docs/demo-evidence/STORY-150/AC-150-003-vp039-table.webm

Set Shell "bash"
Set Width 1200
Set Height 700
Set Framerate 15
Set TypingSpeed 60ms

Type "sed -n '1807,1816p' src/analyzer/tls.rs"
Enter
Sleep 2s

Type "grep -n 'parse_tls_message_handshake\\|let msg_bytes\\|consumed +=' src/analyzer/tls.rs | grep -v assertion"
Enter
Sleep 3s
