# STORY-131 AC-131-001 / AC-131-002: StreamDispatcher routing + take_enip_analyzer
#
# AC-001: StreamDispatcher routes port-44818 TCP to EnipAnalyzer (Rule 7 after Rule 6 DNP3)
#         bytes_received > 0 after feeding port-44818 flow; non-44818 flows unchanged
#         EC-007 guard: None enip_analyzer with port-44818 is no-op (no panic)
# AC-002: take_enip_analyzer() transfers ownership; second call returns None
#
# No ENIP pcap fixture exists yet (deferred to F4 HS-110..122).
# Dispatch unit tests provide routing/ownership evidence per story instructions.
# Traces: BC-2.17.019 postconditions 1-4

Output AC-001-002-dispatch-tests.gif
Output AC-001-002-dispatch-tests.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-131-enip-dispatcher"
Enter
Wait
Show

Type "# AC-131-001/002: dispatcher routing + ownership transfer (BC-2.17.019)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests dispatch 2>&1 | grep -E 'test dispatch|ok|FAILED|result'"
Enter
Wait
Sleep 2s
