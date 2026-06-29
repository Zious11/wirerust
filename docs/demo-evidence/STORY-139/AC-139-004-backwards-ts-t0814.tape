Output AC-139-004-backwards-ts-t0814.gif
Output AC-139-004-backwards-ts-t0814.webm

Require cargo

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20

# AC-139-004: T0814 malformed frame threshold — backwards-ts no-reset (saturating_sub + strict >300)
# Success path: test passes, confirming backward timestamps don't reset the malformed-frame window.
# Error path: misspelled suffix (_reset vs _no_reset) → 0 tests run.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/enip-direction-clock"
Enter
Sleep 2s
Show

Type "cargo test --test enip_analyzer_tests test_ec_x2_backwards_ts_t0814_no_reset 2>&1"
Enter
Sleep 8s

Type "# Error path: wrong suffix → 0 tests run"
Enter
Sleep 1s

Type "cargo test --test enip_analyzer_tests test_ec_x2_backwards_ts_t0814_reset 2>&1"
Enter
Sleep 8s

Sleep 2s
