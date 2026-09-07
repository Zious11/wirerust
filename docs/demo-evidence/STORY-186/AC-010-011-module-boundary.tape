# STORY-186 AC-186-010/011: frozen SS-20/SS-21 module boundary (BC-2.20.016)
#
# Demonstrates:
#   - test_BC_2_20_016_iso_on_tcp_has_no_stream_analyzer_impl: static regression-guard
#     confirms src/analyzer/iso_on_tcp.rs contains zero `impl StreamAnalyzer` blocks and
#     zero DispatchTarget::IsoOnTcp-shaped references (AC-186-010)
#   - test_BC_2_20_016_no_iso_on_tcp_flow_state_type_exists: static regression-guard
#     confirms no `IsoOnTcpFlowState` type exists anywhere in the tree -- carry buffers
#     live exclusively on S7commFlowState (SS-21) (AC-186-011)
#
# Traces to: BC-2.20.016 postconditions 1 and 3

Output AC-010-011-module-boundary.gif
Output AC-010-011-module-boundary.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 460
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# AC-186-010/011: iso_on_tcp.rs frozen module boundary regression guards (BC-2.20.016)"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests BC_2_20_016 2>&1 | grep -E 'story_186::test|test result:'"
Enter
Wait
Sleep 2s
