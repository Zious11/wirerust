# STORY-134 AC-134-002 / AC-134-003: CIP error-window accumulation and 10s window reset
#
# AC-002: CIP error responses accumulate per-status in error_counts_in_window
#   - Scope gate: only type_id == 0x00B2 (Unconnected Data Item); skip 0x00B1
#   - Precondition: cip_item_data.len() >= 4
#   - general_status (byte 2) != 0x00 → entry incremented; error_window_active seeded
#   - general_status == 0x00 → no increment
#   - Window expires after wrapping_sub > 10s → reset counts, roll window forward
#
# AC-003 (ts=0 regression fix):
#   - error_window_active bool is the unseeded sentinel (NOT ts == 0)
#   - A CIP error at timestamp=0 must seed window: error_window_active=true, ts=0
#   - Subsequent error at ts=5 (delta=5): window NOT expired; count accumulates
#   - Error at ts=11 (delta=11 from ts=0): window expires; counts reset, new window seeded
#
# Tests:
#   recon::test_error_accumulation_increments_per_status
#   recon::test_error_accumulation_ignores_success
#   recon::test_error_window_resets_after_10s
#   recon::test_error_window_resets_after_10s_from_ts_zero
#   recon::test_error_accumulation_skips_connected_item
#   recon::test_error_accumulation_requires_4_bytes
# Traces: BC-2.17.008 postconditions 1-5; BC-2.17.008 invariants 1-4

Output AC-002-003-cip-error-window.gif
Output AC-002-003-cip-error-window.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-134-enip-recon"
Enter
Wait
Show

Type "# AC-134-002/003: CIP error accumulation + 10s window reset + ts=0 regression (BC-2.17.008)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'recon::test_error' 2>&1 | grep -E 'test_error|test_aggregate|ok|FAILED|result'"
Enter
Wait
Sleep 2s
