Output AC-144-003-overflow-clear-recover.gif
Output AC-144-003-overflow-clear-recover.webm

Set FontFamily "Menlo"
Set FontSize 13
Set Width 900
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set WaitTimeout 120s

# Setup: navigate to the worktree
Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/story-144-tls-carry-reassembly"
Enter
Sleep 300ms
Show

# ── Title banner ───────────────────────────────────────────────────────────────
Type "echo '=== STORY-144 AC-144-003: carry overflow -- clear and recover ==='"
Enter
Sleep 600ms

# ── Run the AC-144-003 overflow tests ─────────────────────────────────────────
Type "echo '--- Red-Gate tests: carry overflow clears state + counter in summarize() ---'"
Enter
Sleep 300ms
Type "cargo test 'test_vp039_carry_overflow_clear_and_recover' 'test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key' -- --nocapture 2>&1 | grep -E 'test story_144|test result'"
Enter
Wait+Screen /test result/
Sleep 3s
