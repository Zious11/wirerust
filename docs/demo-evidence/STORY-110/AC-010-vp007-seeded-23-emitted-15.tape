Output docs/demo-evidence/STORY-110/AC-010-vp007-seeded-23-emitted-15.gif
Output docs/demo-evidence/STORY-110/AC-010-vp007-seeded-23-emitted-15.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-010: VP-007 atomic-update invariants after full E-15 integration (STORY-106..110).
# Integration test: public API resolves all 23 seeded technique IDs with non-"Unknown" names.
# In-crate unit test: SEEDED_TECHNIQUE_IDS.len() == 23 AND SEEDED_TECHNIQUE_ID_COUNT == 23.
# Traces to: BC-2.15.021 VP-007 atomic-update; STORY-110 AC-010.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build 2>/dev/null"
Enter
Sleep 15s
Show

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_vp007_seeded_23_emitted_15 -- --nocapture"
Enter
Sleep 5s

Type "cargo test vp007_catalog_drift_guard -- --nocapture"
Enter
Sleep 5s
