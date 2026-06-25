# STORY-132 AC-132-001 / AC-132-002: parse_cpf_items walks CPF item list; CpfItem struct shape
#
# AC-001: parse_cpf_items returns typed item list — item_count, type_id, data, bounds-safe partial
#         covers: single item, two items, empty (item_count=0), truncated (bounds violation)
# AC-002: CpfItem has type_id:u16 + data:Vec<u8> (2 fields only; length is transient parse local)
#         covers: type_id 0x00B2 (Unconnected Data), 0x00B1 (Connected Data), 0x0000 (null addr)
#
# Tests: cpf_cip::test_parse_cpf_items_single_item, test_parse_cpf_items_two_items,
#        test_parse_cpf_items_empty, test_parse_cpf_items_truncated, test_cpf_item_type_ids
# Traces: BC-2.17.005 postconditions 1–4

Output AC-001-002-cpf-walk.gif
Output AC-001-002-cpf-walk.webm

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

Type "# AC-132-001/002: parse_cpf_items walk + CpfItem struct (BC-2.17.005)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'cpf_cip::test_parse_cpf_items\|cpf_cip::test_cpf_item' 2>&1 | grep -E 'cpf_items|cpf_item|ok|FAILED|result'"
Enter
Wait
Sleep 2s
