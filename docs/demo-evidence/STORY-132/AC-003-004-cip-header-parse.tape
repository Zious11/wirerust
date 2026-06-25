# STORY-132 AC-132-003 / AC-132-004: parse_cip_header from 0x00B2 items; 0x00B1 items skipped
#
# AC-003: parse_cip_header decodes service:u8 + request_path:Vec<u8> from 0x00B2 item data
#         covers: request (service code, path bytes), response (0x80-bit set), too-short (None),
#         truncated path (None when item_data.len() < 2 + path_byte_count)
# AC-004: CIP-layer parse is 0x00B2-only; 0x00B1 Connected Data Items are stored but NOT parsed
#         F-P9-001: sequence-count prefix on 0x00B1 would corrupt CIP service/path parse
#
# Tests: cpf_cip::test_parse_cip_header_request, test_parse_cip_header_response,
#        test_parse_cip_header_too_short, test_parse_cip_header_truncated_path,
#        test_cip_parse_skips_0x00b1_items, test_cip_parse_processes_0x00b2_items
# Traces: BC-2.17.006 postconditions 1–7, invariant 3 (F-P9-001)

Output AC-003-004-cip-header-parse.gif
Output AC-003-004-cip-header-parse.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-132-cpf-cip-parse"
Enter
Wait
Show

Type "# AC-132-003/004: parse_cip_header (0x00B2 only, F-P9-001 gate) (BC-2.17.006)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'cpf_cip::test_parse_cip_header\|cpf_cip::test_cip_parse' 2>&1 | grep -E 'cip_header|cip_parse|ok|FAILED|result'"
Enter
Wait
Sleep 2s
