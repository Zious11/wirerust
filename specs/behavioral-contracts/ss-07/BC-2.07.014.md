---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: mitre_technique: Some(\"T1027\") → mitre_techniques: vec![\"T1027\"] in Postconditions (ARP-F2 P14 B6) — 2026-06-13"
  - "v1.4: PG-ARP-F2-007 ss-07 full re-anchor — SniValue enum 200→201; extract_sni fn 246→247; match block 251-265→252-266; AsciiWithControl emission 426→437 — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.014: SNI Containing C0/DEL Byte Emits Anomaly/Inconclusive/Low Finding Mapped to T1027

## Description

When an SNI hostname byte sequence is valid UTF-8, passes `is_ascii()`, and contains at least
one C0 control byte (0x00-0x1F) or DEL (0x7F), `extract_sni` classifies it as
`SniValue::AsciiWithControl { hostname, hex }` (arm 2 of INV-5) -- a struct variant with two
fields: `hostname: String` (the raw ASCII string) and `hex: String` (lossless lowercase hex of
the raw bytes, per ADR 0003). A single `Anomaly/Inconclusive/Low` finding is emitted for the
hostname with MITRE technique T1027 (Obfuscated Files or Information).

## Preconditions

1. A TLS ClientHello is being parsed by TlsAnalyzer.
2. The ClientHello contains an SNI extension with a ServerNameList.
3. The first ServerName entry's hostname bytes are inspectable.
4. `str::from_utf8(bytes) == Ok(s)` -- bytes are valid UTF-8.
5. `s.is_ascii() == true` -- all code points are U+0000-U+007F.
6. `contains_c0_or_del(s) == true` -- at least one byte in [0x00-0x1F, 0x7F].

## Postconditions

1. `extract_sni` returns `SniValue::AsciiWithControl { hostname, hex }` where `hostname` is the raw ASCII string and `hex` is the lossless lowercase hex encoding of the raw bytes (ADR 0003 forensic evidence).
2. One Finding is emitted with:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Low
   - mitre_techniques: vec!["T1027"]
   - summary: format string containing the raw hostname (with RFC 6066 reference)
   - evidence: vec![hex representation of the control bytes]
   - direction: Some(Direction::ClientToServer)
3. The SNI is counted in `sni_counts` under the raw hostname key.
4. handshakes_seen is incremented.

## Invariants

1. Exactly ONE finding per AsciiWithControl SNI hostname (not one per control byte).
   BC-2.07.015 covers the multiple-control-bytes case.
2. Space (0x20) does NOT trigger arm 2; 0x1F is the last C0 byte, 0x20 is the boundary.
   BC-2.07.016 covers the boundary conditions.
3. This arm fires ONLY for valid UTF-8 AND is_ascii() -- bytes that are valid UTF-8 but
   non-ASCII (e.g., 0x80-0xFF in UTF-8 multi-byte sequences) go to arm 3 (NonAsciiUtf8).
4. Raw bytes are preserved in finding summary and evidence (ADR 0003 / INV-4). No
   escape_for_terminal is called at TlsAnalyzer level.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SNI = "evil\x00.com" (NUL byte, C0 start) | AsciiWithControl; finding with T1027 |
| EC-002 | SNI = "evil\x1f.com" (0x1F, C0 end) | AsciiWithControl; finding with T1027 |
| EC-003 | SNI = "evil\x7f.com" (DEL) | AsciiWithControl; finding with T1027 |
| EC-004 | SNI = "evil\x20.com" (space, NOT C0) | Arm 1 fires (Ascii): NO finding |
| EC-005 | SNI = "cafe\x01.com" (NUL after a-f) | AsciiWithControl; arm 2 fires |
| EC-006 | SNI with C0 when sni_counts at MAX_MAP_ENTRIES | Finding still fires (finding is decoupled from count insertion, per BC-2.07.028) |
| EC-007 | Tab (0x09), CR (0x0D), LF (0x0A) | All are C0 bytes; AsciiWithControl fires |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ClientHello with SNI "evil\x1b.com" (ESC byte) | Finding(Anomaly/Inconclusive/Low, T1027, direction=ClientToServer) | happy-path |
| ClientHello with SNI "example.com" | No SNI finding | happy-path |
| ClientHello with SNI "evil\x00\x01.com" (multiple C0) | Exactly ONE finding (not two) | edge-case |
| ClientHello with SNI "caf\x01\xe9" (C0 + non-ASCII) | Arm 3 fires (NonAsciiUtf8), not arm 2 | edge-case (see BC-2.07.037) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-005 | AsciiWithControl SNI produces exactly one Anomaly/Inconclusive/Low T1027 finding | unit: multiple test variants (ESC, BEL, DEL, tab, CR, LF) |
| VP-005 | Space (0x20) does not trigger finding | unit: test_ascii_control_boundary_bytes |
| VP-005 | Raw bytes preserved in finding summary | unit: assert finding.summary contains raw hostname |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- SNI C0/DEL detection is a core TLS anomaly finding in the SNI 4-way classification |
| L2 Domain Invariants | INV-5 (SNI 4-way classification ordered match), INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:201-266, C-13) |
| Stories | STORY-055 |
| Origin BC | BC-TLS-014 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.013 -- related to (arm 1: clean ASCII is the non-finding path)
- BC-2.07.015 -- composes with (multiple C0 bytes still produce one finding)
- BC-2.07.016 -- composes with (boundary: 0x1F vs 0x20)
- BC-2.07.037 -- supersedes for mixed non-ASCII+C0 case (arm 3 wins over arm 2)

## Architecture Anchors

- `src/analyzer/tls.rs:201` -- `enum SniValue` definition
- `src/analyzer/tls.rs:247` -- `fn extract_sni(extensions: &[TlsExtension]) -> Option<SniValue>`
- `src/analyzer/tls.rs:252-266` -- extract_sni match block: arm ordering (Ascii, AsciiWithControl, NonAsciiUtf8, NonUtf8)
- `src/analyzer/tls.rs:437` -- AsciiWithControl finding emission site in handle_client_hello
- `tests/tls_analyzer_tests.rs` -- multiple test_ascii_sni_with_* functions

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:247` (fn extract_sni), `:252-266` (match block), `:437` (AsciiWithControl emission) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: from_utf8 + is_ascii + contains_c0_or_del checks
- **assertion**: test_ascii_sni_with_esc_emits_control_finding_and_counts_under_raw_key, and 7 other SNI tests

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, sni_counts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
