# STORY-132 AC-132-005: parse_cip_request_path extracts Class/Instance/Attribute segments
#
# AC-005: 2-byte segment walk with exact-match on 0x20 (Class), 0x24 (Instance), 0x30 (Attribute)
#         covers: empty path (vec![]), Class only, full Class+Instance+Attribute triple,
#         unrecognized segment type (skip; advance by 2), odd-length path (safe — no panic)
#
# ADR-010 Decision 8: NO bitmasking (& 0xE0) — exact match only to avoid 0x24 matching as 0x20.
# 16-bit extended segments (0x21, 0x25, 0x31) deferred to v0.12.0.
#
# Tests: cpf_cip::test_parse_cip_path_empty, test_parse_cip_path_class_only,
#        test_parse_cip_path_class_instance_attr, test_parse_cip_path_unrecognized_skip,
#        test_parse_cip_path_odd_length_safe
# Traces: BC-2.17.009 postconditions 1–4

Output AC-005-cip-request-path.gif
Output AC-005-cip-request-path.webm

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

Type "# AC-132-005: parse_cip_request_path Class/Instance/Attribute extraction (BC-2.17.009)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'cpf_cip::test_parse_cip_path' 2>&1 | grep -E 'cip_path|ok|FAILED|result'"
Enter
Wait
Sleep 2s
