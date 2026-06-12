Output docs/demo-evidence/STORY-110/AC-002-content-first-precedence.gif
Output docs/demo-evidence/STORY-110/AC-002-content-first-precedence.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 450
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-002: TLS ClientHello on port 20000 returns DispatchTarget::Tls (Rule 1, not Rule 6).
# HTTP GET on port 20000 returns DispatchTarget::Http (Rule 2, not Rule 6).
# Content-first precedence (INV-2) preserved when port 20000 matches Rule 6.
# Traces to: BC-2.15.021 postconditions 5/6; STORY-110 AC-002.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-110 && cargo build 2>/dev/null"
Enter
Sleep 15s
Show

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_tls_on_port_20000_routes_to_tls -- --nocapture"
Enter
Sleep 5s

Type "cargo test --test bc_2_15_110_dnp3_dispatcher_tests test_http_on_port_20000_routes_to_http -- --nocapture"
Enter
Sleep 5s
