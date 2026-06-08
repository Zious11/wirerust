# STORY-055 Demo Evidence — SNI Classification Arms 1 and 2

**Story:** STORY-055 — SNI Classification Arms 1 and 2: Clean ASCII Baseline and C0/DEL Control-Byte Detection
**Wave:** 17
**Implementation strategy:** brownfield-formalization (zero production behaviour change — all tests formalise pre-existing behaviour of `contains_c0_or_del`, `extract_sni`, and the `handle_client_hello` match arms)
**Full suite result:** `cargo test --all-targets` — 0 failed across all test harnesses

---

## Coverage table

> BC column references the Behavioral Contract that each AC traces to.
> "what-it-proves" notes byte-class boundary coverage where relevant: 0x1F/0x20 boundary, 0x7E/0x7F boundary, NUL (0x00), and tab/CR/LF (all C0, `< 0x20`).

| AC | BC | Test function(s) | Result | What it proves |
|----|----|-----------------|--------|----------------|
| AC-001 | BC-2.07.013 pc1-3 | `test_BC_2_07_013_clean_ascii_no_finding_counted` | PASS | `extract_sni("example.com")` → `SniValue::Ascii`; `sni_counts["example.com"] == 1`; zero SNI findings. Also runs with `"test.local"` as second vector. |
| AC-001 | BC-2.07.013 pc1-3 | `test_parse_client_hello` | PASS | Canonical ClientHello round-trip with `"example.com"` SNI; `sni_counts` updated; no spurious findings. |
| AC-002 | BC-2.07.013 inv1 | `test_BC_2_07_013_arm1_only_arm_with_no_finding` | PASS | Arm 1 (`SniValue::Ascii`) is the ONLY arm that produces zero SNI findings; arm 2 (`SniValue::AsciiWithControl`, `\x01` embedded) produces exactly one. Confirms no stray `all_findings.push` in the `Ascii` match arm. |
| AC-002 | BC-2.07.013 inv1 | `test_ascii_sni_does_not_emit_non_utf8_finding` | PASS | Clean ASCII hostname does not trigger the non-UTF-8 finding path (arm 4 guard). |
| AC-002 | BC-2.07.013 inv1 | `test_printable_ascii_sni_emits_no_control_finding` | PASS | Fully printable ASCII SNI (no byte satisfying `b < 0x20 \|\| b == 0x7f`) emits no control-byte finding. |
| AC-003 | BC-2.07.014 pc1-4 | `test_BC_2_07_014_esc_emits_anomaly_inconclusive_low_t1027_c2s` | PASS | SNI `"foo\x1b[31m.example"` (ESC 0x1B): exactly one finding; `category=Anomaly`, `verdict=Inconclusive`, `confidence=Low`, `mitre_technique=Some("T1027")`, `direction=Some(ClientToServer)`; `evidence[0]` starts with `"hex: "` and contains `666f6f1b5b33316d2e6578616d706c65`; `sni_counts` keyed on raw hostname. |
| AC-003 | BC-2.07.014 pc1-4 | `test_ascii_sni_with_esc_emits_control_finding_and_counts_under_raw_key` | PASS | Legacy-named complement: ESC byte arm-2 finding fires; `sni_counts` raw-key semantics confirmed. |
| AC-004 | BC-2.07.014 inv4 | `test_BC_2_07_014_raw_bytes_preserved_not_debug_escaped` | PASS | `finding.summary` contains the raw ESC byte (0x1B) as a literal byte — **not** `\u{1b}` (Rust Debug-format). Confirms no `escape_for_terminal` at the `TlsAnalyzer` layer per ADR 0003 / INV-4. |
| AC-005 | BC-2.07.015 pc1-3 | `test_BC_2_07_015_multiple_c0_bytes_one_finding_full_hex_evidence` | PASS | SNI `"a\x01\x02\x03b"` (3 C0 bytes): exactly ONE finding; `evidence[0]` is `"hex: 610102036 2"` (hex of all bytes, not just control bytes). |
| AC-005 | BC-2.07.015 pc1-3 | `test_multiple_control_bytes_in_sni_produces_single_finding` | PASS | Legacy-named complement; `all_findings.len() == 1` for multi-control-byte hostname. |
| AC-006 | BC-2.07.015 inv1 | `test_BC_2_07_015_finding_count_o1_per_hostname_not_per_byte` | PASS | Arm 2 match arm calls `all_findings.push` exactly once regardless of control byte count (O(1) per hostname). Asserts `all_findings.len() == 1` for three-byte control vector. |
| AC-007 | BC-2.07.016 pc1-4 | `test_BC_2_07_016_boundary_0x1f_trips_0x20_does_not_0x7f_trips_0x7e_does_not` | PASS | **Boundary coverage:** byte 0x1F (US) → arm 2 (finding emitted); byte 0x20 (space) → arm 1 (no finding); byte 0x7F (DEL) → arm 2 (finding emitted); byte 0x7E (tilde `~`) → arm 1 (no finding). All four boundary assertions pass. |
| AC-007 | BC-2.07.016 pc3 | `test_ascii_control_boundary_bytes` | PASS | Legacy-named complement; 0x1F triggers, 0x20 does not — explicit boundary assertion. |
| AC-007 | BC-2.07.016 pc4 | `test_ascii_sni_with_del_emits_control_finding` | PASS | DEL byte (0x7F) triggers arm 2. |
| AC-008 | BC-2.07.016 inv1 | `test_BC_2_07_016_tab_cr_lf_are_c0_and_trip` | PASS | **C0 sub-range coverage:** tab (0x09), LF (0x0A), CR (0x0D) all satisfy `b < 0x20`; each triggers arm 2 independently. Predicate is exactly `b < 0x20 \|\| b == 0x7f`. |
| AC-008 | BC-2.07.016 inv1 | `test_ascii_sni_with_tab_emits_control_finding` | PASS | Tab (0x09) alone emits finding. |
| AC-008 | BC-2.07.016 inv1 | `test_ascii_sni_with_cr_and_lf_emits_control_finding` | PASS | CR (0x0D) and LF (0x0A) together emit finding. NUL (0x00) is also implicitly covered (0x00 < 0x20). |
| AC-009 | BC-2.07.018 pc1-3 | `test_BC_2_07_018_punycode_a_label_arm1_no_finding_counted` | PASS | `"xn--caf-dma.example"` (Punycode A-label for `café.example`): satisfies arm 1 conditions (`is_ascii() == true`, no C0/DEL); returns `SniValue::Ascii`; `sni_counts["xn--caf-dma.example"] == 1`; no finding emitted. |
| AC-009 | BC-2.07.018 pc1-3 | `test_punycode_a_label_does_not_emit_non_ascii_finding` | PASS | A-label does not trigger the non-ASCII UTF-8 arm (arm 3). |
| AC-009 | BC-2.07.018 pc1-3 | `test_punycode_a_label_emits_no_control_finding` | PASS | A-label does not trigger the C0/DEL arm (arm 2). |
| AC-010 | BC-2.07.018 inv1-2 | `test_BC_2_07_018_a_label_uses_same_arm1_as_plain_ascii` | PASS | A-labels are a subset of plain ASCII; no Punycode-specific code path exists in `extract_sni`. Confirms the invariant that arm 1 handles A-labels identically to any other pure-ASCII hostname. |

---

## Boundary byte coverage summary

| Byte | Value | Arm triggered | Notes |
|------|-------|---------------|-------|
| NUL  | 0x00  | Arm 2 | `< 0x20` — covered by `test_BC_2_07_016_ec003_nul_byte_is_c0_start_trips_arm2` |
| Tab  | 0x09  | Arm 2 | `< 0x20` — `test_ascii_sni_with_tab_emits_control_finding` |
| LF   | 0x0A  | Arm 2 | `< 0x20` — `test_ascii_sni_with_cr_and_lf_emits_control_finding` |
| CR   | 0x0D  | Arm 2 | `< 0x20` — `test_ascii_sni_with_cr_and_lf_emits_control_finding` |
| US   | 0x1F  | Arm 2 | `< 0x20` — last C0; `test_BC_2_07_016_boundary_...` |
| Space| 0x20  | Arm 1 | NOT C0; `test_BC_2_07_016_boundary_...`; also `test_BC_2_07_016_ec004_space_only_sni_is_arm1` |
| Tilde| 0x7E  | Arm 1 | NOT DEL; `test_BC_2_07_016_boundary_...` |
| DEL  | 0x7F  | Arm 2 | `== 0x7f`; `test_BC_2_07_016_boundary_...`, `test_ascii_sni_with_del_emits_control_finding` |

---

## Full-suite confirmation

```
cargo test --all-targets (worktree .worktrees/story-055)
  test result: ok. 75 passed; 0 failed — tls_analyzer_tests
  All other harnesses: ok (zero failures across all targets)
```

All 10 ACs have at least one PASS-verified test. No production behaviour was changed; every test formalises an invariant that was already satisfied by the implementation delivered by STORY-052 and the arm-1/arm-2 implementation in this story.
