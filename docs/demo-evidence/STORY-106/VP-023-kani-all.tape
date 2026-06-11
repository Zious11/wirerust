# VHS Demo — STORY-106 VP-023 Kani Formal Verification (All 4 harnesses)
#
# Demonstrates all 4 Kani harnesses reporting VERIFICATION:- SUCCESSFUL.
# Sub-A: verify_parse_dnp3_dl_header_safety
# Sub-B: verify_classify_dnp3_fc_total
# Sub-C: verify_is_valid_dnp3_frame_gate
# Sub-D: verify_compute_dnp3_frame_len
#
# Usage: vhs VP-023-kani-all.tape  (run from worktree root)
# Output: VP-023-kani-all.gif + VP-023-kani-all.webm

Output docs/demo-evidence/STORY-106/VP-023-kani-all.gif
Output docs/demo-evidence/STORY-106/VP-023-kani-all.webm

Require cargo

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms

# === Hidden Setup ===
Hide
Sleep 300ms
Show

# === VP-023 Sub-A: parse safety, None/Some, LE decode ===
Type "cargo kani --harness verify_parse_dnp3_dl_header_safety 2>&1 | grep -E 'Checking harness|VERIFICATION'"
Enter
Sleep 90s

# === VP-023 Sub-B: FC totality + set membership ===
Type "cargo kani --harness verify_classify_dnp3_fc_total 2>&1 | grep -E 'Checking harness|VERIFICATION'"
Enter
Sleep 90s

# === VP-023 Sub-C: validity gate biconditional ===
Type "cargo kani --harness verify_is_valid_dnp3_frame_gate 2>&1 | grep -E 'Checking harness|VERIFICATION'"
Enter
Sleep 90s

# === VP-023 Sub-D: frame_len arithmetic, [10,292], no overflow ===
Type "cargo kani --harness verify_compute_dnp3_frame_len 2>&1 | grep -E 'Checking harness|VERIFICATION'"
Enter
Sleep 90s
Sleep 2s
