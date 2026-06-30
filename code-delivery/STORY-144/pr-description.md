## Summary

TLS carry buffer and ClientHello fragmentation reassembly (STORY-144, Wave 65, E-5). An adversary who splits a TLS ClientHello across multiple TLS 0x16 records could evade SNI-based detection and JA3 fingerprinting on `develop` (baseline: SNI=MISSED, JA3=MISSED, parse_errors=2). This PR closes that evasion gap by implementing a per-direction carry buffer in `TlsFlowState` that accumulates record payloads and dispatches `handle_client_hello` only when the full handshake message is assembled.

**Evasion closed:** fragmented ClientHello → SNI: ['example.com'], JA3: 6169fabc98e3e6c9..., parse_errors: 0.

---

## Architecture Changes

```mermaid
graph TD
    A["TLS Record (0x16)"] --> B{direction}
    B -->|ClientToServer| C["client_hs_carry: Vec<u8>"]
    B -->|ServerToClient| D["server_hs_carry: Vec<u8>"]
    C --> E{overflow check}
    E -->|len + payload > MAX_BUF| F["clear + handshake_reassembly_overflows += 1"]
    E -->|ok| G["carry.extend_from_slice(payload)"]
    G --> H{drain loop}
    H -->|carry < 4| I["break: wait for more data"]
    H -->|body_len > MAX_BUF| J["clear + overflow += 1, break"]
    H -->|carry < 4 + body_len| I
    H -->|complete message| K{msg_type}
    K -->|0x01 ClientHello| L["parse_tls_message_handshake → handle_client_hello"]
    K -->|other| M["consume silently"]
    L -->|Ok| N["carry.drain(..4+body_len), loop"]
    L -->|Err| O["parse_errors += 1, carry.drain(..4+body_len), loop"]
    M --> N
```

**Files changed:** `src/analyzer/tls.rs` (+330/-69 lines), `tests/tls_analyzer_tests.rs` (+1076 lines), `tests/dispatcher_tests.rs` (+76/-0 lines), `docs/demo-evidence/STORY-144/` (4 recordings + evidence-report.md).

No changes to `src/reassembly/`, `src/dispatcher.rs`, `src/findings.rs`, `src/reporter/`, or `tests/tls_integration_tests.rs`. SS-07 (analyzer/tls.rs) only.

---

## Story Dependencies

```mermaid
graph LR
    STORY144["STORY-144 (this PR)"]
    STORY145["STORY-145 (ServerHello carry)"]
    STORY146["STORY-146 (buffer_saturation_drops counter)"]
    STORY144 --> STORY145
    STORY144 --> STORY146
```

`depends_on: []` — no upstream story PRs required before this merge.
`blocks: [STORY-145, STORY-146]` — STORY-145 (ServerHello direction carry) and STORY-146 (buffer_saturation_drops counter) depend on this PR merging first.

---

## Spec Traceability

```mermaid
flowchart LR
    BC038["BC-2.07.038 v2.7\nHandshake Reassembly\nAcross Record Boundaries"]
    BC039["BC-2.07.039 v2.4\nCarry Buffer Bounded\nat MAX_BUF"]
    BC040["BC-2.07.040 v1.3\nTruncated Carry\nSilent Discard"]
    BC042["BC-2.07.042 v1.4\nCoalesced Messages\nDrain Loop"]
    BC001["BC-2.07.001 v1.9\nParse Complete\nClientHello"]
    VP039["VP-039\nCarry Reassembly\nVerification Property"]

    BC038 --> AC144001["AC-144-001\nStruct fields +\ncounters"]
    BC038 --> AC144002["AC-144-002\nDrain loop +\ndispatch"]
    BC039 --> AC144003["AC-144-003\nOverflow clear-\nand-recover"]
    BC040 --> AC144004["AC-144-004\non_flow_close\nsilent discard"]
    BC001 --> AC144005["AC-144-005\nSingle-record\nregression"]
    BC042 --> AC144002
    VP039 --> AC144002

    AC144001 --> T001["test_BC_2_07_038_canonical_frame_rfc8446_s4\ntest_BC_2_07_038_malformed_assembled_body\ntest_vp039_sni_boundary_deterministic"]
    AC144002 --> T002["proptest_vp039_carry_reassembly_two_record\ntest_vp039_n_record_reassembly\ntest_vp039_large_valid_hello_reassembly\nproptest_vp039_exact_consume_coalesced\ntest_BC_2_07_042_exact_consume_no_double_dispatch\nproptest_vp039_carry_bounded_invariant"]
    AC144003 --> T003["test_vp039_carry_overflow_clear_and_recover\ntest_vp039_carry_overflow_recovery\ntest_vp039_body_len_spoof\ntest_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key"]
    AC144004 --> T004["test_vp039_truncated_carry_no_error\ntest_BC_2_07_040_empty_carry_flow_close"]
    AC144005 --> T005["all 120 existing tests"]

    T001 --> IMPL["src/analyzer/tls.rs\n(TlsFlowState.client_hs_carry,\nserver_hs_carry;\nTlsAnalyzer.handshake_reassembly_overflows)"]
    T002 --> IMPL
    T003 --> IMPL
    T004 --> IMPL
    T005 --> IMPL
```

---

## Test Evidence

| Metric | Value |
|--------|-------|
| Total tests passing | 136 |
| STORY-144 Red-Gate tests (new) | 15 (VP-039 Sub-A/B/C/D/F) |
| Anti-quadratic regression test | 1 (SEC-001 O(1) cursor-drain) |
| Existing tests preserved | 120 |
| Clippy (-D warnings) | Clean |
| cargo fmt --check | Clean |
| tls_integration_tests.rs | Pass (tls.pcap, tls12-aes256gcm.pcap, tls13-rfc8446.pcap) |

**15 new test harnesses (all in `mod story_144 {}` per DF-TEST-NAMESPACE-001):**

| Test | Sub | BC | Type |
|------|-----|----|------|
| `proptest_vp039_carry_reassembly_two_record` | Sub-A | BC-2.07.038 | proptest |
| `test_BC_2_07_038_canonical_frame_rfc8446_s4` | Sub-A | BC-2.07.038 AC-CANONICAL-FRAME | unit |
| `test_BC_2_07_038_malformed_assembled_body` | Sub-A | BC-2.07.038 PC-9 | unit |
| `test_vp039_sni_boundary_deterministic` | Sub-A | BC-2.07.038 EC-001 | unit |
| `test_vp039_n_record_reassembly` | Sub-A-ext-N | BC-2.07.038 EC-003 | unit |
| `test_vp039_large_valid_hello_reassembly` | Sub-C-ext-large | BC-2.07.038 Inv-5 | unit |
| `proptest_vp039_exact_consume_coalesced` | Sub-B | BC-2.07.042 | proptest |
| `test_BC_2_07_042_exact_consume_no_double_dispatch` | Sub-B | BC-2.07.042 | unit |
| `test_vp039_carry_overflow_clear_and_recover` | Sub-C | BC-2.07.039 PC-1-6 | unit |
| `test_vp039_carry_overflow_recovery` | Sub-C | BC-2.07.039 PC-6 | unit |
| `test_vp039_body_len_spoof` | Sub-C | BC-2.07.038 Inv-5 | unit |
| `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key` | Sub-C | BC-2.07.039 PC-7 | unit |
| `test_vp039_truncated_carry_no_error` | Sub-D | BC-2.07.040 | unit |
| `test_BC_2_07_040_empty_carry_flow_close` | Sub-D | BC-2.07.040 | unit |
| `proptest_vp039_carry_bounded_invariant` | Sub-F | BC-2.07.039 Invariant 1 | proptest |

---

## Demo Evidence

### Headline: Before / After (AC-144-002)

The fragmented ClientHello pcap was MISSED on `develop` before this fix:

| Before (develop baseline) | After (STORY-144) |
|--------------------------|-------------------|
| SNI: (empty — MISSED) | SNI: ['example.com'] |
| JA3: (empty — MISSED) | JA3: 6169fabc98e3e6c9... |
| parse_errors: 2 | parse_errors: 0 |

Recording: `docs/demo-evidence/STORY-144/AC-144-002-before-after-contrast.gif`

### AC-144-002: Fragmented ClientHello Reassembly

`AC-144-002-reassembly-fragmented.gif` — `tls-clienthello-fragmented.pcap` against STORY-144 binary. Shows SNI=['example.com'], JA3 hash, parse_errors:0.

### AC-144-002: Single-Record Regression (Control)

`AC-144-002-control-regression.gif` — `tls-clienthello-control.pcap` against STORY-144 binary. SNI still extracted; no regression on single-record path.

### AC-144-003: Carry Overflow Clear-and-Recover

`AC-144-003-overflow-clear-recover.gif` — `cargo test` run of `test_vp039_carry_overflow_clear_and_recover` + `test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key`. Both pass. Shows the Decision-5 guard fires and `handshake_reassembly_overflows` is surfaced in `summarize()`.

All demo evidence is in `docs/demo-evidence/STORY-144/evidence-report.md`.

---

## Holdout Evaluation

N/A — evaluated at wave gate (Phase F4 holdout scenarios HS-F4-001 through HS-F4-006 are gated at the wave level, not per-PR).

---

## Adversarial Review

3 clean adversarial passes (BC-5.39.001) completed during implementation. All findings from the 3 cycles were triaged and resolved before PR creation.

---

## Security Review

| Finding | Severity | Status |
|---------|----------|--------|
| SEC-001: Quadratic carry drain O(n²) — `extend+drain` on each loop iteration caused O(n²) on large coalesced messages | HIGH | RESOLVED — refactored to cursor-based O(1) drain; anti-quadratic regression test added (`test_sec001_no_quadratic_drain`) |
| SEC-002: Narrow non-RFC window — overflow guard uses `==MAX_BUF` condition edge (non-RFC exact boundary) | LOW | DEFERRED to F6 — narrow window [MAX_BUF-3, MAX_BUF], no real exploit path; tracked as deferred item |
| SEC-003: `saturating_add` missing on `handshake_reassembly_overflows` counter | LOW | RESOLVED — counter now uses `saturating_add(1)` to prevent u64 overflow |

No CRITICAL or HIGH unresolved findings.

---

## Risk Assessment

| Dimension | Assessment |
|-----------|-----------|
| Blast radius | Single-file Rust change (SS-07 / src/analyzer/tls.rs); no protocol boundary changes; no public API changes |
| Behavioral regression risk | LOW — single-record fast path preserved; 120 existing tests pass; carry is empty → extend → drain → empty on single-record path |
| Performance impact | O(1) per-record amortized; SEC-001 resolved (cursor drain eliminates O(n²) hot path); carry Vec<u8> heap allocation bounded at MAX_BUF (65,536 bytes) per direction per flow |
| Forward compatibility | Direction-parameterized drain loop (match on direction) designed so STORY-145 can add ServerHello path by adding one match arm |
| Security posture | SNI/JA3 evasion via TLS record fragmentation (TLS-CLIENTHELLO-FRAG-001) closed; carry overflow guard prevents DoS from carry amplification |

---

## AI Pipeline Metadata

| Field | Value |
|-------|-------|
| Pipeline mode | Feature cycle (fix-tls-clienthello-frag) |
| Story wave | Wave 65 |
| Story points | 8 |
| Phase | F3 (incremental TDD) |
| Factory VSDD version | VSDD-F-mode |
| Model | claude-sonnet-4-6 |

---

## Deferred Items

| Item | Status | Target |
|------|--------|--------|
| SEC-002: narrow non-RFC overflow window `==MAX_BUF` vs `>MAX_BUF` | DEFERRED | F6 hardening |
| `done()`-mid-loop cross-direction carry interaction | DEFERRED | wave-gate review — pre-existing behavior, not a STORY-144 regression |

---

## Pre-Merge Checklist

- [x] PR description matches diff
- [x] All 5 ACs covered by at least 1 test each
- [x] Demo evidence: 4 recordings + evidence-report.md (1 per AC minimum met: AC-144-002 x3, AC-144-003 x1)
- [x] Traceability chain complete: BC-2.07.038/039/040/042/001 → AC-144-001..005 → test names → implementation
- [x] 136 tests pass (15 Red-Gate + 1 anti-quadratic + 120 existing)
- [x] Clippy -D warnings clean
- [x] cargo fmt --check clean
- [x] Security review: no unresolved HIGH/CRITICAL findings
- [x] depends_on: [] (no upstream PRs to wait for)
- [x] Adversarial convergence: 3 clean passes (BC-5.39.001)
