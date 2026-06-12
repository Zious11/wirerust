Output docs/demo-evidence/STORY-110/AC-009-rule-ordering-no-stolen-flows.gif
Output docs/demo-evidence/STORY-110/AC-009-rule-ordering-no-stolen-flows.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-009: Rule 5 (port 502, Modbus) fires before Rule 6 (port 20000, DNP3).
# Flow with both port 502 and 20000 routes to DispatchTarget::Modbus (Rule 5 wins).
# DispatchTarget::None is now Rule 7. No existing Modbus flows are stolen by Rule 6.
# Traces to: BC-2.15.021 invariants 1/2; STORY-110 AC-009.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build 2>/dev/null"
Enter
Sleep 15s
Show

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_port_502_and_20000_routes_to_modbus -- --nocapture"
Enter
Sleep 5s
