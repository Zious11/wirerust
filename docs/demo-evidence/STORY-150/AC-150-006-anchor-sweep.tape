# AC-150-006 (success path): anchor sweep — zero numeric line anchors in BC files
# BC-2.07.004.md and BC-2.07.028.md have zero tls.rs:[digit] anchors (TD-031
# symbol anchors only). STORY-054.md shows corrected line-range refs.
Output docs/demo-evidence/STORY-150/AC-150-006-anchor-sweep.gif
Output docs/demo-evidence/STORY-150/AC-150-006-anchor-sweep.webm

Set Shell "bash"
Set Width 1200
Set Height 700
Set Framerate 15
Set TypingSpeed 60ms

Type "grep -c 'tls\\.rs:[0-9]' ../../.factory/specs/behavioral-contracts/ss-07/BC-2.07.004.md 2>/dev/null || echo 0"
Enter
Sleep 1s

Type "grep -c 'tls\\.rs:[0-9]' ../../.factory/specs/behavioral-contracts/ss-07/BC-2.07.028.md 2>/dev/null || echo 0"
Enter
Sleep 1s

Type "grep 'tls\\.rs::' ../../.factory/specs/behavioral-contracts/ss-07/BC-2.07.004.md | head -3"
Enter
Sleep 2s

Type "grep 'tls\\.rs:6' ../../.factory/stories/STORY-054.md | head -4"
Enter
Sleep 2s
