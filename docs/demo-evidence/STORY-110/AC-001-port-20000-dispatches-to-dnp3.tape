Output docs/demo-evidence/STORY-110/AC-001-port-20000-dispatches-to-dnp3.gif
Output docs/demo-evidence/STORY-110/AC-001-port-20000-dispatches-to-dnp3.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-001: StreamDispatcher::classify(ports={20000}, data=[]) returns DispatchTarget::Dnp3 (Rule 6).
# Rules 1-5 do not match (no TLS/HTTP content, no TLS/HTTP/Modbus port).
# Traces to: BC-2.15.021 postcondition 1; STORY-110 AC-001.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build 2>/dev/null"
Enter
Sleep 15s
Show

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_port_20000_dispatches_to_dnp3 -- --nocapture"
Enter
Sleep 5s
