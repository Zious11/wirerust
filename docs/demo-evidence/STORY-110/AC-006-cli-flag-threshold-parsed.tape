Output docs/demo-evidence/STORY-110/AC-006-cli-flag-threshold-parsed.gif
Output docs/demo-evidence/STORY-110/AC-006-cli-flag-threshold-parsed.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-006: wirerust analyze --dnp3-direct-operate-threshold N parses N as u32 and sets
# Dnp3Analyzer.direct_operate_threshold = N. Omitted flag applies default 10.
# Shows unit test passing + --help confirming the flag exists with default 10.
# Traces to: BC-2.15.017 postconditions 1/2; STORY-110 AC-006.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build 2>/dev/null"
Enter
Sleep 15s
Show

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_cli_flag_dnp3_direct_operate_threshold_parsed -- --nocapture"
Enter
Sleep 5s

Type "./target/debug/wirerust analyze --help 2>&1 | grep -A2 dnp3-direct-operate"
Enter
Sleep 3s
