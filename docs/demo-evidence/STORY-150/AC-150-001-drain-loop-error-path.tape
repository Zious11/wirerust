# AC-150-001 (error path): structural tests WERE red at commit 10551ad
# Shows the red gate commit where test_BC_150_001_parse_hs_call_not_duplicated
# and test_BC_150_001_msg_bytes_extraction_not_duplicated FAILED (2 occurrences).
# Full red gate log: .factory/cycles/wave-71/STORY-150/implementation/red-gate-log.md
Output docs/demo-evidence/STORY-150/AC-150-001-drain-loop-error-path.gif
Output docs/demo-evidence/STORY-150/AC-150-001-drain-loop-error-path.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set Framerate 15
Set TypingSpeed 60ms

Type "git log --oneline -10"
Enter
Sleep 2s

Type "git show 10551ad --stat"
Enter
Sleep 3s
