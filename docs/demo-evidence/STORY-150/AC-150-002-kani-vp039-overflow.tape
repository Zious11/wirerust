# AC-150-002 (success path): Kani VP-039 verify_no_usize_overflow_on_advance
# Shows key RESULTS + SUMMARY from pre-run transcript (12/12 PASS, 2/2 covers SATISFIED).
# Full transcript: docs/demo-evidence/STORY-150/AC-150-002-kani-vp039-overflow-transcript.txt
Output docs/demo-evidence/STORY-150/AC-150-002-kani-vp039-overflow.gif
Output docs/demo-evidence/STORY-150/AC-150-002-kani-vp039-overflow.webm

Set Shell "bash"
Set Width 1200
Set Height 700
Set Framerate 15
Set TypingSpeed 60ms

Type "grep -E 'RESULTS:|Check [0-9]+:|Status:|SUMMARY:|failed|VERIFICATION:|verified|cover' docs/demo-evidence/STORY-150/AC-150-002-kani-vp039-overflow-transcript.txt | head -32"
Enter
Sleep 4s
