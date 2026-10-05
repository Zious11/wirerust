# STORY-188 VP-051 Kani harness: classify_job_ack_function parameter-slicing safety
# (supporting evidence for AC-188-008/009 totality and no-panic slicing)
# Test: cargo kani --tests --harness verify_classify_job_ack_function_param_slicing_safe

Output AC-VP051-kani-slicing-safe.gif
Output AC-VP051-kani-slicing-safe.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 560
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms
Set WaitTimeout 120s

Require cargo

Type "# STORY-188 VP-051: Kani proof, classify_job_ack_function param slicing is panic-free"
Enter
Sleep 300ms

Type "cargo kani --tests --harness verify_classify_job_ack_function_param_slicing_safe 2>&1 | grep -E 'VERIFICATION|Harness Summary|successfully verified|cover properties'"
Enter
Wait
Sleep 3s
