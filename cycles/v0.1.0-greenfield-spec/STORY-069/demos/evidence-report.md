# STORY-069 Demo Evidence Report

**Story:** STORY-069 — Finding Struct, Verdict/Confidence Display, and Finding Display Format  
**Version:** 1.3  
**Generated:** 2026-05-21  
**Branch:** feature/story-069-finding-model  
**Worktree:** /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-069  
**Product type:** Pure-core Rust data-model (no CLI surface, no UI). Evidence is `cargo test` execution captured as VHS terminal recordings.

---

## Coverage Summary

| AC / EC | Test Function | Status | Recording |
|---------|--------------|--------|-----------|
| AC-001 | `test_finding_construction_with_all_fields` | PASS | AC-001-finding-struct.gif / .webm |
| AC-002 | `test_timestamp_always_none_in_all_emission_sites` | PASS | AC-002-003-emission-site-invariants.gif / .webm |
| AC-003a | `test_source_ip_set_at_reassembly_sites` | PASS | AC-002-003-emission-site-invariants.gif / .webm |
| AC-003b | `test_source_ip_none_at_http_tls_sites` | PASS | AC-002-003-emission-site-invariants.gif / .webm |
| AC-004 | `test_finding_display_format` | PASS | AC-004-005-finding-display.gif / .webm |
| AC-005 | `test_finding_display_preserves_raw_summary` | PASS | AC-004-005-finding-display.gif / .webm |
| AC-006 | `test_verdict_display_likely` | PASS | AC-006-011-verdict-confidence-display.gif / .webm |
| AC-007 | `test_verdict_display_unlikely` | PASS | AC-006-011-verdict-confidence-display.gif / .webm |
| AC-008 | `test_verdict_display_inconclusive` | PASS | AC-006-011-verdict-confidence-display.gif / .webm |
| AC-009 | `test_confidence_display_high` | PASS | AC-006-011-verdict-confidence-display.gif / .webm |
| AC-010 | `test_confidence_display_medium` | PASS | AC-006-011-verdict-confidence-display.gif / .webm |
| AC-011 | `test_confidence_display_low` | PASS | AC-006-011-verdict-confidence-display.gif / .webm |
| EC-001 | `test_bc_2_09_001_ec001_empty_evidence_is_valid` | PASS | EC-001-005-edge-cases.gif / .webm |
| EC-002 | `test_bc_2_09_002_ec002_empty_summary_display` | PASS | EC-001-005-edge-cases.gif / .webm |
| EC-003 | `test_bc_2_09_002_ec003_esc_byte_in_summary_preserved_in_display` | PASS | EC-001-005-edge-cases.gif / .webm |
| EC-004 | `test_bc_2_09_001_ec004_direction_some_server_to_client` | PASS | EC-001-005-edge-cases.gif / .webm |
| EC-005 | `test_bc_2_09_002_ec005_reconnaissance_category_display` | PASS | EC-001-005-edge-cases.gif / .webm |

**Full suite result (reporter_tests):** 50 passed; 0 failed; 0 ignored (captured in `full-suite-output.txt`).

---

## Tape-to-AC Mapping

### AC-001-finding-struct.tape / .gif / .webm
Demonstrates AC-001 (BC-2.09.001 postcondition 1): `Finding` constructed with all required and optional fields — `category`, `verdict`, `confidence`, `summary`, `evidence`, `mitre_technique`, `source_ip`, `timestamp`, `direction` — compiles and holds correct values.

- Test: `test_finding_construction_with_all_fields`
- Assertions cover: field presence, `timestamp.is_none()`, `source_ip == Some(192.168.1.1)`, `direction == Some(ClientToServer)`, evidence length == 2.

---

### AC-002-003-emission-site-invariants.tape / .gif / .webm
Demonstrates AC-002 and AC-003 (BC-2.09.001 invariants 1 and 2): grep-based in-process file scans verify that all 22 Finding emission sites set `timestamp: None`, reassembly sites set `source_ip: Some(...)` at exactly 5 positions (3 in `reassembly/mod.rs`, 2 in `reassembly/lifecycle.rs`), and HTTP/TLS sites set `source_ip: None` at all 16 positions.

- Tests: `test_timestamp_always_none_in_all_emission_sites`, `test_source_ip_set_at_reassembly_sites`, `test_source_ip_none_at_http_tls_sites`

---

### AC-004-005-finding-display.tape / .gif / .webm
Demonstrates AC-004 and AC-005 (BC-2.09.002 postconditions 1 and 5):
- AC-004: `format!("{finding}")` produces exactly `"[Anomaly] LIKELY (HIGH) — test"` for the canonical test vector.
- AC-005: `Finding::Display` preserves raw ESC byte (0x1B) in summary without escaping; raw bytes appear literally in Display output.

- Tests: `test_finding_display_format`, `test_finding_display_preserves_raw_summary`

---

### AC-006-011-verdict-confidence-display.tape / .gif / .webm
Demonstrates AC-006 through AC-011 (BC-2.09.003 postconditions 1–3, BC-2.09.004 postconditions 1–3):
- AC-006: `Verdict::Likely` → `"LIKELY"`
- AC-007: `Verdict::Unlikely` → `"UNLIKELY"`
- AC-008: `Verdict::Inconclusive` → `"INCONCLUSIVE"`
- AC-009: `Confidence::High` → `"HIGH"`
- AC-010: `Confidence::Medium` → `"MEDIUM"`
- AC-011: `Confidence::Low` → `"LOW"`

- Tests: `test_verdict_display_likely`, `test_verdict_display_unlikely`, `test_verdict_display_inconclusive`, `test_confidence_display_high`, `test_confidence_display_medium`, `test_confidence_display_low`

---

### EC-001-005-edge-cases.tape / .gif / .webm
Demonstrates edge cases EC-001 through EC-005:
- EC-001: `evidence = vec![]` — valid Finding; JSON reporter emits empty evidence array without panicking.
- EC-002: `summary = ""` — Display renders `"[Anomaly] LIKELY (HIGH) — "` (em-dash, space, empty).
- EC-003: `summary` containing ESC byte (0x1B) — ESC byte appears literally in Display output (no escaping at data-model layer; ADR 0003 assigns escaping responsibility to reporters).
- EC-004: `direction = Some(ServerToClient)` — field holds value; Display does not include direction token.
- EC-005: `category = Reconnaissance` — Display renders `"[Reconnaissance] INCONCLUSIVE (LOW) — scan"`.

- Tests: `test_bc_2_09_001_ec001_empty_evidence_is_valid`, `test_bc_2_09_002_ec002_empty_summary_display`, `test_bc_2_09_002_ec003_esc_byte_in_summary_preserved_in_display`, `test_bc_2_09_001_ec004_direction_some_server_to_client`, `test_bc_2_09_002_ec005_reconnaissance_category_display`

---

## Artifact Inventory

| File | Description |
|------|-------------|
| `AC-001-finding-struct.tape` | VHS tape source — AC-001 |
| `AC-001-finding-struct.gif` | VHS recording — AC-001 (GIF) |
| `AC-001-finding-struct.webm` | VHS recording — AC-001 (WebM) |
| `AC-002-003-emission-site-invariants.tape` | VHS tape source — AC-002, AC-003a, AC-003b |
| `AC-002-003-emission-site-invariants.gif` | VHS recording — AC-002, AC-003a, AC-003b (GIF) |
| `AC-002-003-emission-site-invariants.webm` | VHS recording — AC-002, AC-003a, AC-003b (WebM) |
| `AC-004-005-finding-display.tape` | VHS tape source — AC-004, AC-005 |
| `AC-004-005-finding-display.gif` | VHS recording — AC-004, AC-005 (GIF) |
| `AC-004-005-finding-display.webm` | VHS recording — AC-004, AC-005 (WebM) |
| `AC-006-011-verdict-confidence-display.tape` | VHS tape source — AC-006..AC-011 |
| `AC-006-011-verdict-confidence-display.gif` | VHS recording — AC-006..AC-011 (GIF) |
| `AC-006-011-verdict-confidence-display.webm` | VHS recording — AC-006..AC-011 (WebM) |
| `EC-001-005-edge-cases.tape` | VHS tape source — EC-001..EC-005 |
| `EC-001-005-edge-cases.gif` | VHS recording — EC-001..EC-005 (GIF) |
| `EC-001-005-edge-cases.webm` | VHS recording — EC-001..EC-005 (WebM) |
| `full-suite-output.txt` | Verbatim `cargo test --test reporter_tests` output (50 passed) |

---

## Notes

- Implementation strategy for STORY-069 is `brownfield-formalization`: the `Finding` struct, `Verdict`, `Confidence`, and `ThreatCategory` types with their Display impls were already present in `src/findings.rs`. Tests confirm existing code satisfies all BCs.
- No construction-site escaping was found (`grep -rn 'escape_for_terminal' src/ | grep -v reporter` returns empty); the single call site is inside `TerminalReporter` as required by BC-2.09.002 invariant 2.
- The `old AC-001-005-finding-struct-display.tape` file is an unused draft; the canonical tapes are the per-group files listed above.
