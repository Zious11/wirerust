# STORY-132 AC-132-007 + full suite: classify_cip_service 15 variants + response-bit + Unknown
#
# AC-007: CipServiceClass enum has 15 variants (13 named + Response + Unknown).
#         Response-bit invariant: service & 0x80 != 0 → Response (checked FIRST).
#         Unknown: high-bit clear, not in named set (e.g., 0x7F).
#         Function is pure — terminates for all 256 u8 inputs.
#         VP-032 Sub-D Kani harnesses (vp032_cip_service_classification_totality,
#         vp032_cip_service_request_partition) require `cargo kani` — not in demo scope.
#
# Tests: cpf_cip::test_classify_cip_service_named_codes (13 named codes verified),
#        cpf_cip::test_classify_cip_service_response_bit (0x80-bit priority),
#        cpf_cip::test_classify_cip_service_unknown (0x7F → Unknown)
# Traces: BC-2.17.007 postconditions 1–6, invariants 1–3; VP-032 Sub-D (Kani — deferred)
#
# Full suite: 19/19 cpf_cip tests green, covering AC-001..005 + AC-007.

Output AC-007-classify-and-full-suite.gif
Output AC-007-classify-and-full-suite.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 900
Set FontSize 13
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-132-cpf-cip-parse"
Enter
Wait
Show

Type "# AC-132-007: classify_cip_service — 15 variants, response-bit, Unknown (BC-2.17.007)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'cpf_cip::test_classify_cip_service' 2>&1 | grep -E 'classify|ok|FAILED|result'"
Enter
Wait
Sleep 500ms

Type "# Full cpf_cip suite: 19/19 green (AC-001..005 + AC-007)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests cpf_cip 2>&1"
Enter
Wait
Sleep 2s
