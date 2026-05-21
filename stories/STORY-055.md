---
document_type: story
story_id: "STORY-055"
epic_id: "E-5"
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.013.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.014.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.015.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.016.md
  - .factory/specs/behavioral-contracts/ss-07/BC-2.07.018.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-052]
blocks: [STORY-056, STORY-057]
behavioral_contracts:
  - BC-2.07.013
  - BC-2.07.014
  - BC-2.07.015
  - BC-2.07.016
  - BC-2.07.018
verification_properties: [VP-005]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 17
target_module: src/analyzer/tls.rs
subsystems: [SS-07]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield
---

> **Execute:** `/vsdd-factory:deliver-story STORY-055`

# STORY-055: SNI Classification Arms 1 and 2 — Clean ASCII Baseline and C0/DEL Control-Byte Detection

## Narrative
- **As a** forensic analyst
- **I want** the TLS analyzer to correctly classify SNI hostnames that are clean ASCII (no finding) or ASCII containing C0/DEL control bytes (one T1027 finding) — with precise boundary semantics at 0x1F/0x20 — including Punycode A-labels being treated as clean ASCII
- **So that** standard well-formed TLS connections are silently passed through while SNI obfuscation via control bytes is reliably flagged with one finding per hostname regardless of how many control bytes are present

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.07.013 | Clean ASCII SNI Produces No Finding |
| BC-2.07.014 | SNI Containing C0/DEL Byte Emits Anomaly/Inconclusive/Low Finding Mapped to T1027 |
| BC-2.07.015 | Multiple Control Bytes in One SNI Produce Exactly ONE Finding |
| BC-2.07.016 | C0 Boundary: 0x1F Trips Finding; 0x20 (Space) Does NOT |
| BC-2.07.018 | Punycode A-label is Pure ASCII; Emits No SNI Finding |

## Acceptance Criteria

### AC-001 (traces to BC-2.07.013 postcondition 1-3)
When `extract_sni` receives SNI bytes that are valid UTF-8, pass `is_ascii()`, and contain no byte satisfying `b < 0x20 || b == 0x7f` (contains_c0_or_del), it returns `SniValue::Ascii(hostname)`. The hostname is inserted/incremented in `sni_counts` under the raw string key. No finding is pushed to `all_findings`.
- **Test:** `test_parse_client_hello` (SNI "example.com")

### AC-002 (traces to BC-2.07.013 invariant 1)
Arm 1 is the ONLY arm that produces no finding. The `SniValue::Ascii(_)` match arm in `handle_client_hello` contains no `all_findings.push` call.
- **Test:** `test_ascii_sni_does_not_emit_non_utf8_finding`; `test_printable_ascii_sni_emits_no_control_finding`

### AC-003 (traces to BC-2.07.014 postcondition 1-4)
When `extract_sni` receives SNI bytes that are valid UTF-8, pass `is_ascii()`, but contain at least one byte where `b < 0x20 || b == 0x7f`, it returns `SniValue::AsciiWithControl { hostname, hex }` where `hostname` is the raw ASCII string and `hex` is the lossless lowercase hex encoding of the raw bytes. One `Finding` is pushed with: `category = Anomaly`, `verdict = Inconclusive`, `confidence = Low`, `mitre_technique = Some("T1027")`, `direction = Some(Direction::ClientToServer)`, `evidence = vec![hex representation of the control bytes]`.
- **Test:** `test_ascii_sni_with_esc_emits_control_finding_and_counts_under_raw_key` (ESC byte 0x1B)

### AC-004 (traces to BC-2.07.014 invariant 4)
Raw bytes are preserved in the finding summary and evidence at the `TlsAnalyzer` layer. No `escape_for_terminal` is called at this layer. Display-layer escaping is deferred to the terminal reporter (ADR 0003 / INV-4).
- **Test:** Unit test asserting `finding.summary` contains raw hostname with embedded control byte (not Debug-escaped)

### AC-005 (traces to BC-2.07.015 postcondition 1-3)
When an `SniValue::AsciiWithControl` hostname contains multiple C0/DEL bytes, exactly ONE finding is pushed to `all_findings` (not one per control byte). The finding's `evidence` contains one entry: `"hex: {hex}"` where `hex` is the lowercase hex of ALL raw hostname bytes (not just the control bytes).
- **Test:** `test_multiple_control_bytes_in_sni_produces_single_finding`

### AC-006 (traces to BC-2.07.015 invariant 1)
Finding count is O(1) per SNI hostname. The `AsciiWithControl` match arm calls `all_findings.push` exactly once regardless of control byte count.
- **Test:** `test_multiple_control_bytes_in_sni_produces_single_finding` (assert `all_findings.len() == 1`)

### AC-007 (traces to BC-2.07.016 postcondition 1-4)
`contains_c0_or_del(s)` uses the predicate `s.bytes().any(|b| b < 0x20 || b == 0x7f)`. Byte 0x1F (US, Unit Separator) triggers arm 2. Byte 0x20 (space) does NOT trigger arm 2 (it lands in arm 1). Byte 0x7F (DEL) triggers arm 2.
- **Test:** `test_ascii_control_boundary_bytes` (0x1F triggers; 0x20 does not); `test_ascii_sni_with_del` (0x7F triggers)

### AC-008 (traces to BC-2.07.016 invariant 1)
The predicate is exactly `b < 0x20 || b == 0x7f` — a bitwise disjunction. Tab (0x09), LF (0x0A), CR (0x0D) are all C0 bytes (`< 0x20`) and all trigger arm 2.
- **Test:** Unit tests for tab, LF, CR bytes; each must emit a finding

### AC-009 (traces to BC-2.07.018 postcondition 1-3)
A Punycode A-label SNI (e.g., `"xn--caf-dma.example"`) satisfies arm 1 conditions: valid UTF-8, `is_ascii() == true`, no C0/DEL bytes. `extract_sni` returns `SniValue::Ascii(hostname)`. No finding is emitted. The A-label is counted in `sni_counts` under its raw string key.
- **Test:** `test_punycode_a_label_does_not_emit_non_ascii_finding`; `test_punycode_a_label_emits_no_control_finding`

### AC-010 (traces to BC-2.07.018 invariant 1-2)
A-labels by definition are pure ASCII and trivially satisfy arm 1. Only raw U-labels (non-ASCII internationalized hostnames that bypassed Punycode encoding) would trigger arm 3. The A-label path is a special case of BC-2.07.013 arm 1 — there is no Punycode-specific code path.
- **Test:** `test_punycode_a_label_does_not_emit_non_ascii_finding` (confirm no separate code path exists)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `contains_c0_or_del` | src/analyzer/tls.rs:231-238 | pure-core |
| `extract_sni` (arm 1 and arm 2) | src/analyzer/tls.rs:246-265 | pure-core (returns SniValue) |
| Arm 1 match in `handle_client_hello` | src/analyzer/tls.rs:424-425 | effectful-shell (mutates sni_counts; no findings push) |
| Arm 2 match in `handle_client_hello` | src/analyzer/tls.rs:424-447 | effectful-shell (mutates sni_counts, all_findings) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | SNI = "example.com" (clean ASCII) | Arm 1; no finding; `sni_counts["example.com"]++` |
| EC-002 | SNI = "evil\x00.com" (NUL byte) | Arm 2; one T1027 finding |
| EC-003 | SNI = "evil\x1f.com" (0x1F, last C0) | Arm 2; one T1027 finding |
| EC-004 | SNI = "evil\x20.com" (space, NOT C0) | Arm 1; no finding |
| EC-005 | SNI = "evil\x7f.com" (DEL) | Arm 2; one T1027 finding |
| EC-006 | SNI = "evil\x7e.com" (0x7E tilde, NOT DEL) | Arm 1; no finding |
| EC-007 | SNI = "xn--caf-dma.example" (Punycode A-label) | Arm 1; no finding |
| EC-008 | SNI = "a\x01\x02\x03b" (3 control bytes) | ONE finding; evidence = hex of full hostname |
| EC-009 | SNI = " " (space only) | Arm 1 (0x20 is not C0); no finding |
| EC-010 | Same clean ASCII SNI seen twice | `sni_counts["example.com"] == 2` |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/analyzer/tls.rs (contains_c0_or_del) | pure-core | No I/O, no state; pure boolean predicate over a string |
| src/analyzer/tls.rs (extract_sni arm 1+2) | pure-core | Returns SniValue variant; no state mutation inside extract_sni |
| src/analyzer/tls.rs (arm 1+2 in handle_client_hello) | effectful-shell | Mutates sni_counts and/or all_findings |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| Referenced code (tls.rs lines 200-265, 424-447) | ~4,000 |
| Test files (tls_analyzer_tests.rs SNI arm 1/2 tests) | ~5,000 |
| BC files (5 BCs) | ~5,500 |
| Tool outputs overhead | ~2,000 |
| **Total** | **~20,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~10%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 through AC-010 (test-writer)
2. [ ] Verify Red Gate: all AC tests fail before implementation
3. [ ] Implement `contains_c0_or_del(s: &str) -> bool` predicate with `b < 0x20 || b == 0x7f` per BC-2.07.016
4. [ ] Implement `SniValue` enum with variants: `Ascii(String)`, `AsciiWithControl { hostname: String, hex: String }`, `NonAsciiUtf8 { hostname: String, hex: String }`, `NonUtf8 { lossy: String, hex: String }`
5. [ ] Implement `extract_sni` arm 1: `Ok(s) if s.is_ascii() && !contains_c0_or_del(s) => SniValue::Ascii(s.to_string())`
6. [ ] Implement `extract_sni` arm 2: `Ok(s) if s.is_ascii() => SniValue::AsciiWithControl { hostname: s.to_string(), hex: bytes_to_hex(hostname) }`
7. [ ] Implement arm 1 handling in `handle_client_hello`: insert `sni_counts`; no finding
8. [ ] Implement arm 2 handling in `handle_client_hello`: insert `sni_counts`; push ONE Anomaly/Inconclusive/Low T1027 finding with raw hostname in summary and hex evidence
9. [ ] Write boundary tests: 0x1F triggers, 0x20 does not, 0x7F triggers, 0x7E does not
10. [ ] Write Punycode A-label tests: `"xn--caf-dma.example"` lands in arm 1
11. [ ] Run all tests; verify all pass
12. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-052 | `extract_sni` is called from `handle_client_hello` after JA3 computation | SNI classification is per-hostname (not per-byte); findings are O(1) per hostname | `sni_counts` key is the raw hostname string (no transformation for arms 1 and 2) |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `extract_sni` match arms are evaluated top-down: arm 1 before arm 2 | BC-2.07.016 — `is_ascii()` check gates arm 2 | Code review: confirm arm ordering in match block |
| No `escape_for_terminal` at the TlsAnalyzer layer — raw bytes in findings | ADR 0003 / INV-4 | Code review: no escape call in arm 2 finding push |
| Arm 2 finding: exactly one push per hostname, not per control byte | BC-2.07.015 invariant 1 | Unit test: AC-005, AC-006 |
| `evidence` contains one entry with hex of ALL hostname bytes (not just control bytes) | BC-2.07.015 postcondition 2 | Unit test: assert `evidence[0] == "hex: {full_hostname_hex}"` |
| C1 control characters (0x80-0x9F) are NOT checked by `contains_c0_or_del` | BC-2.07.016 invariant 2 | Code review: confirm predicate only checks `< 0x20 || == 0x7f` |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| tls-parser | 0.12 | `TlsExtension::SNI` for SNI extension extraction |
| Rust std | 2024 edition (stable) | `str::is_ascii`, `str::from_utf8`, `str::bytes`, iterator `.any()` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/analyzer/tls.rs | modify | `contains_c0_or_del` (231-238), `SniValue` enum (200), `extract_sni` arms 1+2 (246-265), arm handling in `handle_client_hello` (424-447) |
| tests/tls_analyzer_tests.rs | modify | SNI arm 1/2 tests: clean ASCII, C0 control, DEL, boundary bytes, multiple C0, Punycode A-labels |
