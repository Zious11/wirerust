Output docs/demo-evidence/STORY-110/E2E-cli-dnp3-dispatcher.gif
Output docs/demo-evidence/STORY-110/E2E-cli-dnp3-dispatcher.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# END-TO-END CLI DEMO: DNP3 dispatcher integration + --dnp3-direct-operate-threshold flag.
# Builds the release binary, shows the new --dnp3-direct-operate-threshold flag in
# wirerust analyze --help (with default 10), then runs the full 26-test dispatcher suite.
# This is the headline STORY-110 delivery: Rule 6 wired, CLI flag live, 26/26 PASS.
# Traces to: BC-2.15.017, BC-2.15.021; STORY-110 end-to-end CLI evidence.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build --release 2>&1 | tail -1"
Enter
Sleep 30s
Show

Type "./target/release/wirerust analyze --help 2>&1 | grep -A3 dnp3-direct-operate"
Enter
Sleep 3s

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests -- --nocapture 2>&1 | tail -5"
Enter
Sleep 15s
