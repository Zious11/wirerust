Output docs/demo-evidence/STORY-110/AC-004-take-dnp3-analyzer.gif
Output docs/demo-evidence/STORY-110/AC-004-take-dnp3-analyzer.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-004: Post-finalization, dispatcher.take_dnp3_analyzer() moves Dnp3Analyzer out.
# First call returns Some(analyzer); second call returns None (moved out).
# Mirrors take_modbus_analyzer(). Traces to: BC-2.15.021 invariant 5; STORY-110 AC-004.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build 2>/dev/null"
Enter
Sleep 15s
Show

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_take_dnp3_analyzer_moves_out -- --nocapture"
Enter
Sleep 5s
