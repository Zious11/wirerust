# AC-150-005 (success path): all CI gates pass
# Shows cargo test --all-targets, cargo clippy, and cargo fmt --check all green.
Output docs/demo-evidence/STORY-150/AC-150-005-full-gates.gif
Output docs/demo-evidence/STORY-150/AC-150-005-full-gates.webm

Set Shell "bash"
Set Width 1200
Set Height 700
Set Framerate 15
Set TypingSpeed 60ms

Type "cargo test --all-targets 2>&1 | grep -E '^test result|FAILED|^error' | head -20"
Enter
Sleep 15s

Type "cargo clippy --all-targets -- -D warnings 2>&1 | tail -3"
Enter
Sleep 8s

Type "cargo fmt --check 2>&1 && echo fmt:PASS"
Enter
Sleep 5s
