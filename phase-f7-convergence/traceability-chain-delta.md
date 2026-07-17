# Traceability Chain — Feature #100 Delta

Feature: pcap timestamp threading to `Finding.timestamp` (GitHub issue #100)
Develop HEAD at feature completion: `256a490`

---

## 4-Level Traceability Chains

### Chain 1 — BC-2.04.055 (`on_data` timestamp parameter)

```
BC-2.04.055
  └── VP-021 (timestamp-provenance-threading)
        └── tests/timestamp_threading_tests.rs
              ├── test_finding_timestamp_hot_path
              ├── test_finding_timestamp_close_flush
              ├── prop_finding_timestamp_matches_on_data_timestamp
              └── prop_cross_flow_timestamp_isolation
                    └── src/reassembly/handler.rs       (on_data trait signature)
                        src/dispatcher.rs               (dispatch call sites)
                        src/reassembly/mod.rs           (reassembly entry points)
                        src/reassembly/lifecycle.rs     (session lifecycle callbacks)
                              └── F5-ROUND3-CLEAN
                                    └── VP-021-LOCKED
                                          (test-sufficient: integration + proptest;
                                           mutation kill rate 100%)
```

### Chain 2 — BC-2.09.007 (Finding emission sites carry timestamp)

```
BC-2.09.007
  └── VP-021 (timestamp-provenance-threading)
        └── tests/timestamp_threading_tests.rs        (same suite as Chain 1)
            STORY-098 emission-site tests             (21 of 22 sites covered)
                  └── src/analyzer/http.rs            (HTTP analyzer — Some(...) sites)
                      src/analyzer/tls.rs             (TLS analyzer  — Some(...) sites;
                                                       16 Some-timestamp emission points)
                      src/reassembly/mod.rs           (5 Some-timestamp emission points)
                      src/reassembly/lifecycle.rs     (lifecycle-triggered emissions)
                      src/reassembly/mod.rs:673       (segment-limit path — None,
                                                       justified: no packet context)
                            └── F5-ROUND3-CLEAN
                                  └── VP-021-LOCKED
                                        (test-sufficient: integration + proptest;
                                         mutation kill rate 100%)
```

### Chain 3 — Story dependency chain

```
STORY-097  (BC-2.04.055; on_data timestamp param)
  └── STORY-098  (BC-2.09.007; 21/22 emission sites; depends_on: STORY-097)
        └── STORY-099  (VP-021 E2E + proptest; depends_on: STORY-098)
              └── Epic: E-12
                    └── Waves: 28 (STORY-097), 29 (STORY-098), 30 (STORY-099)
```

### Chain 4 — Cross-reference dependencies

```
BC-2.09.007
  └── depends_on: BC-2.09.006 (existing; version 1.4 — pcap timestamp capture)

BC-2.01.005
  └── existing; updated to version 1.6
      O-01 open obligation: RESOLVED (timestamp field nullable justification documented)
```

---

## Traceability-Chain Append Note

A search was conducted for a main traceability-chain file under
`.factory/cycles/**/convergence/traceability-chain.md`. No such file exists in the
repository (the `v0.1.0-greenfield-spec` cycle does not contain a `convergence/`
subdirectory, and no `traceability-chain.md` was found under any cycle path).
The append step is therefore skipped — the delta chain above is the sole traceability
artifact for Feature #100.

---

## Summary

| Level        | Artifact                                               | Status         |
|--------------|--------------------------------------------------------|----------------|
| BC           | BC-2.04.055, BC-2.09.007, BC-2.09.006 v1.4, BC-2.01.005 v1.6 | LOCKED / updated |
| VP           | VP-021 (timestamp-provenance-threading)                | LOCKED @ 256a490 |
| Tests        | timestamp_threading_tests.rs (hot-path, flush, 2× proptest) | ALL PASS |
| Source       | reassembly/{mod,lifecycle,handler}.rs, dispatcher.rs, analyzer/{http,tls}.rs | MERGED @ 256a490 |
| Adversarial  | F5 rounds 1-3; fix-PRs #200, #201                     | CONVERGED (novelty = 0) |
| Gate         | Phase F7 delta convergence                            | READY FOR MERGE |

---

# Traceability Chain — feature-iec104 Delta (IEC 60870-5-104 Passive Analyzer)

Feature: feature-iec104 (IEC 60870-5-104 passive analyzer, TCP 2404)
Develop HEAD at feature completion: `b36b884`
Spec versions: BC-INDEX v2.33 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56

---

## 4-Level Traceability Chain — IEC-104

### BC-2.19.001–028 → VP-044/045/046/047 + VP-004/007 → Tests → Source

```
BC-2.19.001–028 (IEC-104 passive analyzer BCs — session SM, ASDU parsing,
                  control-command detection, N(S)/N(R) tracking, carry buffers,
                  dispatcher integration, findings cap)
BC-2.05.012 (dispatcher — extends SS-05; E-22 IEC-104 rule 8)
BC-2.10.010 (MITRE EMITTED harness — extends SS-10)
BC-2.12.025 (IEC-104 entry in protocols catalog)
  └── VP-044 (iec104-kani: 89 checks / 5 facets — valid→Some, first-frame-guard,
               carry-bound, direction-Some, findings-cap)
        └── tests/story_167/ .. tests/story_174/   (per-story TDD suites)
            tests/fix_p4_001/                       (direction enrichment, 11 tests)
            tests/fix_f5_001/                       (source_ip+timestamp, 10 tests)
              └── src/analyzer/iec104.rs            (primary implementation)
                  src/dispatcher.rs                 (Rule 8 dispatch wiring)
                  src/mitre.rs                      (T0881 catalog entry)
                        └── F5-R5-NITPICK_ONLY (code frozen since R2 @ 9c5aa9a)
                              └── VP-044 SUCCESSFUL @ b36b884

  └── VP-045 (iec104-proptest: carry-buffer non-vacuous / interleaved-generator
               / state-comparison; VP-045 non-vacuity F-172-003 RESOLVED)
        └── tests/story_172/prop_carry_*
              └── src/analyzer/iec104.rs carry logic
                    └── VP-045 PASS (non-vacuous) @ b36b884

  └── VP-046 (iec104-proptest: N(S)/N(R) tracking / desync / sequence-window)
        └── tests/story_171/prop_ns_nr_*
              └── src/analyzer/iec104.rs N(S)/N(R) fields
                    └── VP-046 PASS @ b36b884

  └── VP-047 (iec104-fuzz: 2.64M execs / 5 min / 0 crashes)
        └── fuzz/fuzz_targets/iec104_*
              └── src/analyzer/iec104.rs
                    └── VP-047 PASS @ b36b884

  └── VP-004 (dispatcher-kani: 440/407/183 checks — Rule 8 wiring)
  └── VP-007 (mitre-kani: 122 checks / SEEDED=29 — T0881 entry)
        └── src/dispatcher.rs + src/mitre.rs
              └── VP-004 / VP-007 SUCCESSFUL @ b36b884
```

### Story Dependency Chain

```
STORY-167 (E-22 wave-76: IEC-104 frame-type discrimination + STARTDT/STOPDT SM)
  └── STORY-168 (wave-77: session SM fields + first-frame-guard)
        └── STORY-169 (wave-78: ASDU header extraction)
              └── STORY-170 (wave-79: control-command detection T1692.001/T0836/T0827)
                    └── STORY-171 (wave-80: N(S)/N(R) sequence tracking + desync)
                          └── STORY-172 (wave-81: carry buffers + overflow discard + EMIT-WITH-DEDUP)
                                └── STORY-173 (wave-82: dispatcher integration + T0881 + findings cap)
                                      └── STORY-174 (wave-83: VP-044 non-vacuity + VP-045 proptest
                                                     + green-doc-tense gate extension)
                                            └── FIX-P4-001 (direction enrichment — 10 emit sites)
                                                FIX-F5-001 (source_ip+timestamp — 10 emit sites)
                                                FIX-F5-002..004 (doc-accuracy sweeps)
```

### Cross-Reference Dependencies

```
E-22 extends dispatcher SS-05  (BC-2.05.012 depends on BC-2.05.*)
E-22 extends MITRE SS-10       (BC-2.10.010)
BC-2.19.006 v1.2               (is_valid_iec104_frame gate role — corrected pre-merge D-458)
BC-2.19.025 v1.3               (carry-bound invariants — re-anchored VP-045 D-462)
BC-2.19.026 v1.6               (EMIT-WITH-DEDUP for malformed-LEN — research-validated D-451)
BC-2.19.028 v1.0               (findings cap MAX_IEC104_FINDINGS=10_000 — CWE-400/770 D-456)
ADR-013 Decision 6             (first-frame-guard mandate — D-439)
SS-19 v1.9                     (IEC-104 subsystem spec final form)
```

### Convergence Ladder

```
F5-R5-NITPICK_ONLY (0 CRIT/HIGH/MED; 1 LOW non-blocking; code frozen R2)
  → F6-PASS (Kani/fuzz/mutation/audit/regression all green @ b36b884)
    → F7-CONVERGED (5/5 dims PASS; holdout 0.99; 2 MINOR doc-only deferred)
      → RELEASE-HELD (human direction 2026-07-17; v0.13.0 cut pending auth)
```

---

## Summary — feature-iec104

| Level         | Artifact                                                               | Status                        |
|---------------|------------------------------------------------------------------------|-------------------------------|
| BC            | BC-2.19.001–028 + BC-2.05.012 + BC-2.10.010 + BC-2.12.025 (32 total) | LOCKED @ BC-INDEX v2.33       |
| VP            | VP-044/045/046/047 + VP-004/007                                        | ALL PASS / SUCCESSFUL @ b36b884 |
| Tests         | story_167..174 + fix_p4_001 + fix_f5_001 (2627 total)                 | ALL PASS                      |
| Source        | src/analyzer/iec104.rs, src/dispatcher.rs, src/mitre.rs                | MERGED develop b36b884        |
| Adversarial   | F5 rounds 1–5; fix-PRs FIX-F5-001..004                                | CONVERGED (R5 NITPICK_ONLY)   |
| Verification  | F6 Kani/fuzz/mutation/audit                                            | PASS (D-469)                  |
| Gate          | Phase F7 delta convergence                                             | CONVERGED (D-470); RELEASE HELD |
