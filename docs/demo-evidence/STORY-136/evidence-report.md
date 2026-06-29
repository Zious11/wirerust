# Demo Evidence Report — STORY-136

**Story:** ENIP Connection Lifecycle: ForwardOpen/ForwardClose Detection
**Story ID:** STORY-136
**Wave:** 60
**Product type:** Pure-core library (no CLI/UI surface — EtherNet/IP analyzer CIP connection-lifecycle detection only)
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test enip_analyzer_tests`)
**Recorded:** 2026-06-26
**Test result at recording time:** 10 passed / 0 failed / 0 ignored (mod connection_lifecycle)

---

## AC Coverage Map

| AC | Title | Test filter used | Artifact (GIF) | Artifact (WebM) | Tape |
|----|-------|-----------------|---------------|----------------|------|
| AC-136-001 | ForwardOpen (0x54) and LargeForwardOpen (0x5B) emit Anomaly/Possible/Low, mitre_techniques=[] | `connection_lifecycle::test_forward_open` | `AC-001-forward-open-finding.gif` | `AC-001-forward-open-finding.webm` | `AC-001-forward-open-finding.tape` |
| AC-136-002 | ForwardClose (0x4E) emits Anomaly/Possible/Low, mitre_techniques=[] | `connection_lifecycle::test_forward_close` | `AC-002-forward-close-finding.gif` | `AC-002-forward-close-finding.webm` | `AC-002-forward-close-finding.tape` |
| AC-136-003 | ForwardOpen/ForwardClose responses (0xD4, 0xCE) do not emit findings | `test_forward_open_response_no_finding`, `test_forward_close_response_no_finding` | `AC-003-responses-no-finding.gif` | `AC-003-responses-no-finding.webm` | `AC-003-responses-no-finding.tape` |
| AC-136-004 | `is_non_enip` suppresses all lifecycle findings | included in master suite (`AC-ALL`) | `AC-ALL-connection-lifecycle-10-green.gif` | `AC-ALL-connection-lifecycle-10-green.webm` | `AC-ALL-connection-lifecycle-10-green.tape` |
| AC-136-005 | `open_connection_count` / `close_connection_count` tracked in flow state | `connection_lifecycle::test_connection_counts_tracked` | `AC-005-connection-counts.gif` | `AC-005-connection-counts.webm` | `AC-005-connection-counts.tape` |

**Master green-run** covering all 10 tests (AC-136-001 through AC-136-005):

| Artifact | Description |
|----------|-------------|
| `AC-ALL-connection-lifecycle-10-green.gif` | Full `mod connection_lifecycle` — 10/10 green |
| `AC-ALL-connection-lifecycle-10-green.webm` | Full `mod connection_lifecycle` — 10/10 green |
| `AC-ALL-connection-lifecycle-10-green.tape` | VHS script for master suite |

---

## Recordings Detail

### AC-001-forward-open-finding

Demonstrates `EnipAnalyzer::process_pdu` CIP ForwardOpen (service=0x54) and LargeForwardOpen
(service=0x5B) → Anomaly/Possible/Low finding per occurrence with empty mitre_techniques (BC-2.17.015).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'connection_lifecycle::test_forward_open'`
- `test_forward_open_emits_finding`: CIP request service=0x54 + type_id=0x00B2 + !is_non_enip →
  exactly one Finding with category=Anomaly, verdict=Possible, confidence=Low,
  mitre_techniques=vec![],
  summary="CIP ForwardOpen connection establishment observed from src=...: connection lifecycle anomaly";
  evidence cites ADR-010 Decision 7
- `test_forward_open_no_mitre_technique`: asserts mitre_techniques is empty (ADR-010 Decision 7 —
  no dedicated ATT&CK for ICS v19.1 technique for CIP connection establishment anomaly)
- `test_forward_open_connected_item_no_finding`: CIP ForwardOpen via type_id=0x00B1 → no finding
  (F-P9-001 gate; 0x00B2 is the CIP protocol requirement for ForwardOpen)
- `test_large_forward_open_emits_finding`: CIP request service=0x5B (LargeForwardOpen) + 0x00B2 +
  !is_non_enip → one Anomaly/Possible/Low finding; treated identically to ForwardOpen per
  BC-2.17.015 Invariant 5
- All 4 tests pass green

**Tests in recording:**
- `test_forward_open_emits_finding`
- `test_forward_open_no_mitre_technique`
- `test_forward_open_connected_item_no_finding`
- `test_large_forward_open_emits_finding`

---

### AC-002-forward-close-finding

Demonstrates `EnipAnalyzer::process_pdu` CIP ForwardClose (service=0x4E) → Anomaly/Possible/Low
finding with empty mitre_techniques (BC-2.17.015 Postconditions 4–5).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests 'connection_lifecycle::test_forward_close'`
- `test_forward_close_emits_finding`: CIP request service=0x4E + type_id=0x00B2 + !is_non_enip →
  exactly one Finding with category=Anomaly, verdict=Possible, confidence=Low,
  mitre_techniques=vec![],
  summary="CIP ForwardClose connection teardown observed from src=...: connection lifecycle closed"
- `test_forward_close_no_mitre_technique`: asserts mitre_techniques is empty (ADR-010 Decision 7)
- All 2 tests pass green

**Tests in recording:**
- `test_forward_close_emits_finding`
- `test_forward_close_no_mitre_technique`

---

### AC-003-responses-no-finding

Demonstrates that CIP ForwardOpen response (0xD4) and ForwardClose response (0xCE) do NOT
produce findings — request-only detection guaranteed by `classify_cip_service` response-bit
masking (BC-2.17.007 Invariant 1).

**What the recording shows:**
- Runs `test_forward_open_response_no_finding`: service=0xD4 (high bit set) →
  `classify_cip_service` returns `CipServiceClass::Response`, never ForwardOpen → no finding
- Runs `test_forward_close_response_no_finding`: service=0xCE (high bit set) →
  `classify_cip_service` returns `CipServiceClass::Response`, never ForwardClose → no finding
- Both tests pass green; confirms no hand-rolled `& 0x80 == 0` predicate at call site

**Tests in recording:**
- `test_forward_open_response_no_finding`
- `test_forward_close_response_no_finding`

---

### AC-005-connection-counts

Demonstrates `EnipFlowState.open_connection_count` and `close_connection_count` tracking
across multiple ForwardOpen and ForwardClose requests (BC-2.17.015 Invariant 3).

**What the recording shows:**
- Runs `test_connection_counts_tracked`: multiple ForwardOpen + LargeForwardOpen requests increment
  `open_connection_count`; multiple ForwardClose requests increment `close_connection_count`;
  counts remain accurate regardless of MAX_FINDINGS cap (feeds STORY-138 session summary)
- Test passes green

**Tests in recording:**
- `test_connection_counts_tracked`

---

### AC-ALL-connection-lifecycle-10-green

Master green-run for the full `mod connection_lifecycle` suite — 10 tests covering all
STORY-136 acceptance criteria including `is_non_enip` suppression (AC-136-004) and
connection count tracking (AC-136-005).

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests connection_lifecycle`
- All 10 tests in `mod connection_lifecycle` execute and pass
- Test result line: `test result: ok. 10 passed; 0 failed; 0 ignored`

**Tests in recording (all 10):**
- `test_forward_open_emits_finding`
- `test_forward_open_no_mitre_technique`
- `test_forward_open_connected_item_no_finding`
- `test_large_forward_open_emits_finding`
- `test_forward_close_emits_finding`
- `test_forward_close_no_mitre_technique`
- `test_forward_open_response_no_finding`
- `test_forward_close_response_no_finding`
- `test_non_enip_suppresses_connection_lifecycle`
- `test_connection_counts_tracked`

---

## Full connection_lifecycle Test Suite Summary

All 10 tests in `mod connection_lifecycle` pass at recording time:

```
test connection_lifecycle::test_forward_close_response_no_finding ... ok
test connection_lifecycle::test_forward_close_no_mitre_technique ... ok
test connection_lifecycle::test_forward_close_emits_finding ... ok
test connection_lifecycle::test_forward_open_response_no_finding ... ok
test connection_lifecycle::test_forward_open_no_mitre_technique ... ok
test connection_lifecycle::test_non_enip_suppresses_connection_lifecycle ... ok
test connection_lifecycle::test_forward_open_connected_item_no_finding ... ok
test connection_lifecycle::test_large_forward_open_emits_finding ... ok
test connection_lifecycle::test_forward_open_emits_finding ... ok
test connection_lifecycle::test_connection_counts_tracked ... ok

test result: ok. 10 passed; 0 failed; 0 ignored
```

---

## Deferred / Not Applicable

None. All 5 ACs have recorded demo coverage.

- AC-136-001 (ForwardOpen/LargeForwardOpen): dedicated `AC-001-forward-open-finding` recording
  (4 tests: emits finding, no MITRE technique, 0x00B1 gate, LargeForwardOpen) + master suite.
- AC-136-002 (ForwardClose): dedicated `AC-002-forward-close-finding` recording (2 tests:
  emits finding, no MITRE technique) + master suite.
- AC-136-003 (response suppression): dedicated `AC-003-responses-no-finding` recording (2 tests:
  0xD4 and 0xCE response bytes → no finding via classify_cip_service response-bit invariant).
- AC-136-004 (`is_non_enip` suppression): covered in master suite via
  `test_non_enip_suppresses_connection_lifecycle`.
- AC-136-005 (connection count tracking): dedicated `AC-005-connection-counts` recording
  (`test_connection_counts_tracked`) + master suite.

## Approach Note

**Test-run recordings used (not pcap-driven CLI output).** This is the established pattern for
STORY-13x demos: the ENIP analyzer is a pure-core library with no standalone CLI entry point
for injecting hand-crafted CIP frames. The acceptance criteria are fully expressed as unit tests
in `tests/enip_analyzer_tests.rs`. Recordings show `cargo test` output filtered to relevant
test names and the final `test result:` line, identical to STORY-134 and STORY-135 precedent.
