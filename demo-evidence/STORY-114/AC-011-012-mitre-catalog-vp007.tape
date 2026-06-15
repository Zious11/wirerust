Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-114/AC-011-012-mitre-catalog-vp007.gif"
Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-114/AC-011-012-mitre-catalog-vp007.webm"

Set Theme "Dracula"
Set FontFamily "Menlo"
Set Width 1200
Set Height 750
Set Padding 20
Set FontSize 14

# AC-011: T0830 -> LateralMovement, T1557.002 -> CredentialAccess in catalog
# AC-012: vp007_catalog_drift_guard (SEEDED=25, EMITTED=17) + test_vp007_seeded_25_emitted_17

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-114"
Enter
Sleep 500ms
Show

Type "# AC-011/012: MITRE T0830+T1557.002 catalog (VP-007: SEEDED=25, EMITTED=17)"
Enter
Sleep 300ms

Type "cargo test vp007_catalog_drift_guard 2>&1 | grep -E 'vp007_catalog|test result.*passed'"
Enter
Sleep 8s

Type "cargo test --test bc_2_16_story114_arp_tests 2>&1 | grep -E 'test story_114|test result.*passed'"
Enter
Sleep 8s

Sleep 2s
