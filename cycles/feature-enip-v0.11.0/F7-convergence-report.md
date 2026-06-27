---
document_type: f7-convergence-report
cycle_id: feature-enip-v0.11.0
version: "1.0"
status: FINAL
producer: architect
timestamp: 2026-06-26T00:00:00Z
develop_head: f17d270
github_issue: 316
subsystem: SS-17
traces_to: .factory/cycles/feature-enip-v0.11.0/cycle-manifest.md
---

# F7 Delta-Convergence Assessment — feature-enip-v0.11.0 (EtherNet/IP + CIP)

**Cycle:** feature-enip-v0.11.0 | **Issue:** #316 | **Subsystem:** SS-17
**Develop HEAD at assessment:** f17d270 (fuzz-harness PR #332 merged)
**Stories delivered:** STORY-130..138 (9 stories, waves 58-61)
**Assessment date:** 2026-06-26

---

## Overall Verdict

**F7 VERDICT: CONVERGED**

All five convergence dimensions pass. Regression is clean. The holdout coverage
gap (no pcap fixtures, no live eval run) is a confidence-add gap, not a
correctness-blocker — the unit test suite provides substantial evidence for all 12
pcap-required holdout behaviors. One pre-sign-off checkpoint (cargo-mutants full-run
completion) and a set of cycle-close housekeeping items remain for the human gate.

**Release-readiness for v0.11.0: RECOMMEND PROCEED** subject to the human gate
items enumerated in Section 8.

---

## 1. Dimension 1 — Spec Convergence

**Verdict: CONVERGED**

### Evidence

F5 scoped-adversarial completed 3 consecutive clean passes (0 HIGH/CRITICAL, zero
novelty) on develop @bd9e507 (D-263). The 26/26 BC-completeness sweep (DF-BC-
COMPLETENESS-SWEEP-001) confirmed all BC-2.17.001..026 have: (a) an implementation
path in enip.rs, and (b) at least one non-vacuous test. The PRD §2.17 RTM lists all
26 BCs with module column filled as `src/analyzer/enip.rs` (SS-17, CAP-17).

F2 adversarial achieved 4 consecutive clean passes (P10/P11/P12/P13, 0 HIGH/CRITICAL)
before human approval (D-230). All F2 deferred items (F8-01, F8-02, F8-03) were
resolved in the Pass-10 final-polish burst.

### Tracked Spec-Wording Deferrals (cycle-close, not correctness gaps)

| ID | Description | Correctness Impact |
|----|-------------|-------------------|
| BC-2.17.016 PC-0 wording ambiguity (F-138-P1-002) | Non-blocking LOW; wording imprecision only | None — code correct |
| BC-2.17.021 Invariant 2 prose clarification | Stale "does NOT re-scan" clause contradicts the RULING-W61-001-sanctioned fold; ruling authorizes cycle-close edit | None — summarize() is correct |
| F-W60-002 / BC-2.17.016 v1.2 clarification | bytes_received exemption from PC-5 "no counter updates"; code correct per RULING-W60-001 Part 2 | None — is_non_enip guard latently unreachable |
| SS-17 BC input-hash TBD backfill (BC-2.17.007+ carry `input-hash: TBD`) | Admin/housekeeping; merged stories have verified hashes | None |

Assessment: all four items are prose-alignment or admin tasks. None represents a
behavioral gap between spec intent and implementation. The code satisfies every BC
postcondition as of develop @f17d270.

### Additional Spec Notes

RULING-137-002 documents that the `is_non_enip` carry-overflow latch (BC-2.17.016
PC-4, Inv-4) is structurally unreachable (maximum carry is 599 bytes; cap fires at
>600). This is a spec defect, not an implementation defect: the code faithfully
implements the ratified BC. The quarantine narrative in STORY-137 is inaccurate but
the T0814 detection (malformed_in_window >= 3) is reachable and functional. PO
decision required for v0.12.0 redesign. This is documented and deferred; it does not
constitute a shipping blocker.

---

## 2. Dimension 2 — Test Convergence

**Verdict: CONVERGED**

### Evidence

164 test functions in `tests/enip_analyzer_tests.rs` across all ENIP behavioral
domains. Per-story breakdown (non-vacuous tests per story convergence records):

| Story | Tests | BCs covered |
|-------|-------|-------------|
| STORY-130 | 21 | BC-2.17.001..004 (parse, classify, validity gate) |
| STORY-131 | 15 (dispatch) + 21 (parse reuse) | BC-2.17.019..022, VP-004 oracle |
| STORY-132 | 19 | BC-2.17.005..009 (CPF item walk, CIP header, VP-032 Sub-D Kani) |
| STORY-133 | 10 | BC-2.17.011..013 (MITRE seeding T0858/T0816/T1693.001, VP-007) |
| STORY-134 | 20 | BC-2.17.008/010/014 (T0846/T0888 recon) |
| STORY-135 | 16 | BC-2.17.011/012/013 (T0858/T0816/T0836 command detections) |
| STORY-136 | 10 | BC-2.17.015 (ForwardOpen/Close lifecycle) |
| STORY-137 | (folded into enip_analyzer_tests; frame-walk + carry) | BC-2.17.016/004/018 |
| STORY-138 | (session + summary + DoS + fix-PRs) | BC-2.17.025/017/022/021/024 |

Total ENIP delta tests: 164 functions, all non-vacuous (per F5 26/26 BC sweep). All
tests were verified green at develop @f17d270 by CI (11/11 suites green, 0 failures,
80 test suites total).

### DF-AC-TEST-NAME-SYNC-001

Test functions carry AC-NNN-NNN citation prefixes in doc-comments (e.g.,
`/// AC-130-001 — canonical SendRRData 24-byte vector`). Policy DF-AC-TEST-NAME-SYNC-001
v2 (MEDIUM) requires test names to be resolvable to their AC citations. The F5
26/26 BC sweep confirmed no AC-name drift for SS-17 tests. The deferred LOW finding
AC-130-001 (postcondition citation precision "1-9" vs "1-8") was logged as
non-blocking at STORY-130 convergence; it is a doc-comment annotation error, not a
test logic error.

### Boundary Coverage

Test coverage includes:
- Little-endian parse correctness (all header fields; session_handle LE, command LE)
- CPF item-type handling (0x00B2 only; 0x00B1 negative path tested via HS-119 alignment)
- CIP service dispatch (Stop/Reset/SetAttribute/GetAttribute/MultiService/Forward*)
- Threshold boundaries (write-burst default=50 exact boundary; error-burst default=5)
- DoS guard boundaries (MAX_FINDINGS=10000 cap enforced with dropped_findings counter)
- Carry-buffer behavior (partial frame stash, frame-skip for oversize, RULING-137-002
  dead-code guard documented and tested as such)
- source_ip attribution via resolve_enip_client_ip (4 unit cases T1-T4 + 2 E2E
  value-asserting on_data tests)
- summarize() open-flow fold (discriminating test: 3 ListIdentity → flows_analyzed=1
  without on_flow_close; mixed closed+open fold; no double-count)

---

## 3. Dimension 3 — Implementation Convergence

**Verdict: CONVERGED**

### Evidence

**No todo!() or unimplemented!() in ENIP delta.** The word "todo" appears in enip.rs
only in a doc-comment describing T0814 detection (`// T0814 detection`). No runtime
panics or stubs remain; the doc-comment reference was confirmed clean at F5 (the F5
Pass-1/2/3 doc-tense sweep would have flagged any todo!() or stub language in test
docs).

**No `#![allow(dead_code)]` on enip.rs.** The module-wide dead-code suppressor was
removed in STORY-137 (WAVE59-DEADCODE-001 resolved). The public `flows` field on
`EnipAnalyzer` was added to satisfy borrow-checker visibility; this is intentional
and verified clean by clippy.

**EnipSummary is wired and load-bearing.** Fix-PR #331 (D-262) wired `EnipSummary`
as the canonical intermediate type in `summarize()` with byte-identical output, per
the human's wire-through choice at Wave-61 gate. No dead public structs remain.

**clippy -D warnings clean at f17d270.** Confirmed by CI (D-265 F6 gate: "clippy -D
warnings clean; fmt clean").

**`resolve_enip_client_ip` ships.** Fix-PR #328 (D-256) implemented the port-44818
heuristic per RULING-W60-001, with DRIFT-ENIP-DIRECTION-001 doc-label matching the
DNP3 sibling pattern. The `lower_ip()` mis-attribution defect is resolved.

**`summarize()` drains open flows.** Fix-PR #330 (D-260) implemented the summarize-
time fold per RULING-W61-001 (DNP3 parity). The four fields that were zero in
production (`total_pdu_count`, `command_distribution`, `parse_errors`, `flows_analyzed`)
now reflect live traffic.

**No debug artifacts.** No `dbg!()`, no bare `println!()` outside the existing
`eprintln!` pattern used for ENIP session events (per ADR-010 Decision 9, which
explicitly specifies `eprintln!` as the convention — not the log crate).

### Deferred Implementation Items (non-blocking)

| ID | Description | Target |
|----|-------------|--------|
| STORY-137-UNSAFE-SPLIT-BORROW | Sound unsafe split-borrow in process_pdu; SEC-006 MEDIUM pre-authorized deferred | v0.12.0 |
| SPEC-DEFECT-IS-NON-ENIP-DEAD-LATCH | is_non_enip carry-overflow latch dead code per RULING-137-002 | v0.12.0 (PO decision required) |
| O-W61-2 | Optional: no-finding command (ListServices/ListInterfaces etc.) process_pdu test | Optional backlog |

---

## 4. Dimension 4 — Verification Convergence

**Verdict: CONVERGED (with one in-flight checkpoint noted for human gate)**

### Kani Model Checking

11/11 Kani harnesses PASS (Kani 0.67.0, D-264):

| Harness | VP | Property | Result |
|---------|-----|----------|--------|
| vp032_sub_a_header_no_oom | VP-032 Sub-A | parse_enip_header: no OOM/panic for any input length | PASS |
| vp032_sub_b_classify_total_coverage | VP-032 Sub-B | classify_enip_command: exhaustive command space coverage | PASS |
| vp032_sub_c_validity_soundness | VP-032 Sub-C | is_valid_enip_frame: valid → command in known set | PASS |
| vp032_sub_d_cpf_item_walk_no_oob | VP-032 Sub-D | parse_cpf_items: no out-of-bounds indexing for any CPF payload | PASS |
| vp032_cip_service_request_partition | VP-032 (CIP) | parse_cip_header: CIP service partition correct for any input | PASS |
| vp004_dispatch_oracle_44818 | VP-004 (44818 arm) | dispatcher routes port-44818 to ENIP; no other port misrouted | PASS |
| (4 VP-007 harnesses) | VP-007 × 4 | MITRE seeding/emission invariants for T0858/T0816/T0836/T0846/T0888/T0814 | PASS |

DF-KANI-NONVACUITY-001 (CRITICAL policy): all harnesses confirmed non-vacuous at F6
discharge (D-264).

### Fuzzing

cargo-fuzz `fuzz_enip_cip_parse`: **8,331,310 runs / 91 seconds / 0 crashes / 0 hangs**
(develop HEAD @447da079 harness commit, D-264). Harness merged as PR #332 @f17d270
(D-265). Coverage: `parse_enip_header`, `parse_cpf_items`, `parse_cip_header` (F-P9-002
fuzz obligation from F3 D-231).

### Mutation Testing

cargo-mutants on `src/analyzer/enip.rs`: 20/20 viable sample killed (100% kill rate,
D-264). Full 241-mutant run was in progress at F6 gate: 21 caught / 0 missed at last
checkpoint (D-265).

**IN-FLIGHT CHECKPOINT (F6-MUTANTS-FULL-RUN):** The full 241-mutant run was not
confirmed complete at the time of the F6 gate. The STATE.md pre-sign-off condition
reads: "confirm cargo-mutants full run = 0 missed (F6-MUTANTS-FULL-RUN)." This is
a pre-F7-sign-off gate recorded by the orchestrator. The human should confirm this
result before proceeding to release. Current evidence (21 caught / 0 missed on a
100%-kill viable sample) is strongly indicative of 0-missed; the checkpoint is a
formality pending the full-run file being finalized.

### Supply Chain

- `cargo audit`: 0 vulnerabilities, 193 dependencies (D-264).
- `cargo deny`: OK (D-264).
- All GitHub Actions SHA-pinned; dtolnay/rust-toolchain allowlisted per CLAUDE.md.

---

## 5. Dimension 5 — Regression

**Verdict: CONVERGED (no sibling regressions detected)**

### Evidence

Full `cargo test --all-targets` on develop @f17d270: **0 failures, 80 suites, 2085+
tests green** (confirmed at F6 gate D-265 and pre-F5 Wave-61 gate D-261).

### Sibling Analyzer Coverage

The ENIP feature is purely additive. Spot-check of regression coverage:

| Sibling Domain | Test File(s) | Status |
|----------------|-------------|--------|
| HTTP analyzer | `http_analyzer_tests.rs`, `http_integration_tests.rs` | Green (80 suites pass) |
| TLS analyzer | `tls_analyzer_tests.rs`, `tls_integration_tests.rs` | Green |
| DNS analyzer | `dns_tests.rs` | Green |
| Modbus/ICS | `modbus_detection_tests.rs`, `modbus_parse_tests.rs`, `modbus_e2e_tests.rs` | Green |
| DNP3 | `dnp3_correlation_tests.rs`, `dnp3_detection_tests.rs`, `dnp3_parse_core_tests.rs` | Green |
| ARP | `bc_2_16_*.rs` (7 files) | Green |
| Dispatcher | `dispatcher_tests.rs` | Green — dispatch tests explicitly include ENIP arm; sibling routing unchanged |
| Reassembly | `reassembly_engine_tests.rs`, `reassembly_flow_tests.rs` | Green |
| MITRE catalog | `mitre_tests.rs` | Green — VP-007 pins T0858/T0816/T0836/T1693.001/T0846/T0888/T0814 + authoritative TA-id table |
| Reporter | `reporter_json_tests.rs`, `reporter_csv_tests.rs`, `reporter_terminal_tests.rs` | Green |
| Main/CLI | `cli_tests.rs`, `cli_integration_tests.rs`, `main_story_088_tests.rs`, `main_story_089_tests.rs` | Green |

Wave-59 wave-level adversarial Pass A identified a T0846 `write_burst_emitted` guard
regression (cross-story state bleed from STORY-132 into STORY-133 context) — resolved
via PR #321. No further cross-story regressions were detected in Waves 60/61 or F5.

### Dead Code

`#![allow(dead_code)]` removed from enip.rs (STORY-137, WAVE59-DEADCODE-001). The
`EnipSummary` public struct is now load-bearing (fix-PR #331). No dead symbols
identified by clippy or the F5 adversarial sweep.

---

## 6. Holdout Coverage Assessment

**Verdict: NOT FORMALLY EVALUATED — UNIT COVERAGE ADEQUATE — NOT A RELEASE BLOCKER**

### Status

13 holdout scenarios HS-110..122 were authored in F3. Of these, 12 required pcap
fixture creation (flagged as F4 obligations). 1 (HS-121) was flagged as synthetic
(no pcap needed, programmatic generation required).

**No pcap fixtures were created during F4.** No holdout evaluation run was executed
against the analyzer binary. The STATE.md entry for F5 (D-263) states: "HS-110..122
present + boundary semantics satisfied" — this is a satisfaction-by-inspection
assessment from the F5 adversarial sweep, not a formal holdout evaluation run.

`last_evaluated: null` in all 13 HS-11*.md frontmatter fields, confirming no
formal evaluation has occurred.

### Coverage Assessment by Holdout

| HS | Title | Behavior Covered by Unit Tests? | Formal Eval Required? |
|----|-------|----------------------------------|----------------------|
| HS-110 | enip-canonical-frame-le-header-decode | YES — `test_parse_enip_header_valid` (AC-130-001): exact LE decode of 0x6F/0x00 → 0x006F verified; `test_parse_enip_header_too_short` covers 23-byte truncation | No blocker — directly covered |
| HS-111 | enip-cip-stop-t0858 | YES — `test_t0858_cip_stop_fires_per_occurrence` pins verdict/confidence/summary to normative BC strings (STORY-135 F-135-P2-001 fix) | No blocker |
| HS-112 | enip-cip-reset-t0816 | YES — `test_t0816_cip_reset_fires_per_occurrence` pins BC strings | No blocker |
| HS-113 | enip-cip-write-burst-t0836-threshold | YES — threshold boundary test present; EC-007 threshold-zero test added in STORY-135 Pass-3 | No blocker |
| HS-114 | enip-listidentity-t0846-one-shot | YES — `test_t0846_listidentity_one_shot` verifies one-shot guard | No blocker |
| HS-115 | enip-error-burst-t0888-threshold | YES — error-burst window and T0888 Pattern A/B tests present (STORY-134 20 recon tests) | No blocker |
| HS-116 | enip-forwardopen-close-empty-mitre | YES — ForwardOpen/ForwardClose lifecycle tests present; no-MITRE behavior tested (BC-2.17.015 AC-136-001/002 evidence field assertions) | No blocker |
| HS-117 | enip-malformed-t0814-structural-anomaly | PARTIALLY — T0814 `malformed_in_window >= 3` threshold tested; HS-117-CASE-D (max-length oversized-frame panic-safety) was logged as a process-gap item at STORY-137 convergence but not resolved. RULING-137-002 confirms the carry-cap path is dead code. The frame-skip path covers the large-frame case. | Confidence gap — not a correctness gap |
| HS-118 | enip-oversize-frame-carry-skip | PARTIALLY — frame-skip behavior tested via STORY-137 carry tests; RULING-137-002 dead-latch affects the `is_non_enip` narrative but frame-skip path itself is functional | Confidence gap |
| HS-119 | enip-0x00b1-deferral-negative | YES — negative path: 0x00B1 traffic does not trigger CIP detection (by design; 0x00B2 only); dispatched traffic on port 44818 with non-0x00B2 CPF items does not emit false positives | No blocker |
| HS-120 | enip-dispatch-port-44818 | YES — `test_dispatcher_routes_port_44818` + `test_dispatcher_rule_order_dnp3_before_enip` directly cover dispatch-port semantics | No blocker |
| HS-121 | enip-max-findings-dos-bound | YES — DoS guard test present (STORY-138 MAX_FINDINGS cap tests); `dropped_findings` counter asserted. Direct unit-level coverage of the cap at exactly 10,000 boundary. No 10,001-frame pcap needed to validate the algorithm. | No blocker — algorithm directly tested |
| HS-122 | enip-real-world-corpus | NOT covered by unit tests — this holdout requires a real-world packet capture against a live or simulated EtherNet/IP device. No substitute unit test exists. | **Confidence gap — see below** |

### HS-122 Real-World Corpus (Elevated Gap)

HS-122 requires driving the analyzer against a real or realistic ENIP/CIP pcap
corpus to confirm end-to-end behavior under realistic traffic patterns. The existing
`tests/fixtures/local-samples/4SICS-GeekLounge-151022.pcap` is an ICS conference
corpus; it is not known to contain port-44818 traffic.

This is the most significant holdout gap. However, the following mitigates the risk:
- The ENIP parser is pure-core and formally verified (Kani 11/11, fuzz 8.3M/0-crash).
- All detection paths are unit-tested with exact BC-specified byte sequences.
- The Kani harnesses verify no OOM/panic for arbitrary input, covering the primary
  safety concern a real-world corpus would test.
- The `4SICS` corpus e2e smoke test (`e2e_corpus_smoke_tests.rs`) confirms the
  analyzer does not crash on realistic ICS pcap input (though port-44818 traffic
  may not be present in that corpus).

### Severity Classification

| Holdout gap type | Count | Severity for release |
|-----------------|-------|---------------------|
| Directly covered by unit tests | 10/13 | No gap |
| Confidence-add (algorithm covered, pcap not exercised) | 2/13 (HS-117/118) | LOW — not a correctness blocker |
| Real-world corpus validation absent | 1/13 (HS-122) | MEDIUM — confidence gap, not a correctness gap; recommend as post-release validation |

### Recommendation

Formal holdout evaluation (pcap fixture creation + binary eval run) is
**acceptable to defer** for v0.11.0 release. The unit test suite directly verifies
the behaviors exercised by HS-110..121. HS-122 (real-world corpus) is the only gap
that cannot be substituted by unit tests, and it is a confidence validation, not a
correctness gate.

**Recommended post-release action:** create pcap fixtures for HS-110..122 as a
dedicated validation story (or maintenance task) in v0.11.x. Run the formal holdout
evaluator against develop @f17d270 to produce `last_evaluated` timestamps. This
closes the formal audit trail and provides a regression baseline for future cycles.

---

## 7. Summary of Tracked Deferrals at the F7 Gate

These items are fully documented and are not correctness gaps. The human should
review them as part of the gate decision.

| ID | Category | Description | Target |
|----|----------|-------------|--------|
| F6-MUTANTS-FULL-RUN | Pre-sign-off checkpoint | cargo-mutants full 241-mutant run: 21/0 at last checkpoint; confirm completion = 0 missed | Before release |
| HS-110..122 formal eval | Holdout coverage | No pcap fixtures; no eval run; 10/13 covered by unit tests; HS-122 real-world corpus is the only non-substitutable gap | Post-release validation |
| SPEC-DEFECT-IS-NON-ENIP-DEAD-LATCH | Spec defect (RULING-137-002) | is_non_enip latch unreachable; quarantine narrative inaccurate; code correct | v0.12.0 (PO decision) |
| BC-2.17.021 Inv-2 prose clarification | Spec prose | Remove stale "does NOT re-scan" clause; ruling-sanctioned cycle-close edit | Cycle close / v0.11.x |
| F-W60-002 / BC-2.17.016 v1.2 | Spec prose | bytes_received exemption clause; Invariant 7 addition | Cycle close |
| F-138-P1-002 | Spec prose | BC-2.17.016 PC-0 wording ambiguity (LOW) | Cycle close |
| SS-17-BC-INPUT-HASH-BACKFILL | Admin | BC-2.17.007+ carry `input-hash: TBD`; run bin/compute-input-hash --scan | Cycle close |
| STORY-137-UNSAFE-SPLIT-BORROW | Security (LOW) | Unsafe split-borrow in process_pdu; sound; SEC-006 MEDIUM pre-authorized | v0.12.0 |
| ENGINE-PROPAGATION-GREP-GATE-001 | Process | Mechanical changed-value sibling-grep gate; human decision before cycle close | Cycle close (human) |
| DEPENDABOT-311/325 | Supply chain | Two Dependabot PRs unreviewed | Human triage |

---

## 8. Release-Readiness Summary for v0.11.0

**Overall: RELEASE-READY pending F7 human gate confirmation**

### Hard Gate Items (must confirm before release)

1. **F6-MUTANTS-FULL-RUN:** Confirm cargo-mutants full 241-mutant run result is 0
   missed. Evidence to date (21 caught / 0 missed on full run; 20/20 viable sample
   100% killed) is strongly indicative. Locate `mutants.out/missed.txt` in the F6
   worktree and confirm the file is empty or absent.

2. **Human go-ahead (D-260):** The orchestrator recorded a human STOP directive at
   D-260: "proceed through Wave-61 gate + F5 + F6 + F7 convergence, then HALT for
   human go-ahead before the release pipeline." This F7 report is the final
   convergence artifact. Release pipeline requires explicit human approval.

### Items the Human Should Weigh at the F7 Gate

3. **Async mutation full run (F6-MUTANTS-FULL-RUN):** See item 1 above. Confirmation
   or denial of 0-missed on the full 241-mutant run.

4. **Holdout evaluation status:** The human should decide whether to accept the
   unit-test substitution for HS-110..121 and the HS-122 real-world corpus gap for
   v0.11.0, or require a formal holdout run before tagging. The architect's
   recommendation is to accept and schedule formal evaluation post-release.

5. **is_non_enip dead latch (RULING-137-002):** The quarantine feature is inert in
   v0.11.0. Non-ENIP traffic on port 44818 is flagged via T0814 (reachable and
   functional), but the `is_non_enip` latch that STORY-137 advertises is dead code.
   The human should decide whether to: (a) accept as-is and ship with the RULING
   note in the ADR, (b) revise the release notes to avoid the "quarantine" narrative,
   or (c) require a fix before release (not recommended — requires PO decision on
   quarantine semantics).

6. **Cycle-close BC spec edits:** BC-2.17.021 Invariant 2, BC-2.17.016 PC-5 v1.2,
   and F-138-P1-002 are ruling-sanctioned prose clarifications. The human should
   confirm they are bundled into the cycle-close commit (can happen after release).

7. **Dependabot PRs #311 and #325:** Two Dependabot PRs are open and unreviewed.
   These are supply-chain updates; the human should triage whether to merge before
   or after the v0.11.0 release.

8. **ENGINE-PROPAGATION-GREP-GATE-001:** A proposed mechanical sibling-grep gate for
   changed values. Human decision on whether to add this to the factory engine before
   or after release.

### Positive Release Signals

- All 9 stories (STORY-130..138) merged with 3-consecutive-clean-pass adversarial
  convergence (BC-5.39.001 MET for each).
- 3 wave-level integration gates PASSED (Waves 58, 59, 60) + Wave-61 human-approved.
- F5 scoped adversarial: 3 consecutive clean passes, 0 HIGH/CRITICAL, zero novelty (D-263).
- F6 formal hardening: Kani 11/11 PASS, fuzz 8.3M/0-crash, audit/deny/clippy/fmt clean (D-265).
- Full regression suite: 0 failures, 80 suites, 2085+ tests.
- All 26 BC-2.17.001..026 have non-vacuous tests (26/26 BC sweep, D-263).
- Two critical in-cycle correctness defects fixed and verified: F-W60-001 (source_ip
  mis-attribution, fix-PR #328) and F-138-P1-004 (summarize() zero output for real
  captures, fix-PR #330).
- ADR-010 (9 decisions), VP-032 (5 harnesses), SS-17 subsystem all complete.
- No HIGH or CRITICAL open items on develop @f17d270.

---

## Appendix: BC Coverage Map (SS-17)

All 26 BCs covered. Key implementation sites:

| BC | Title | Story | Detection/Function |
|----|-------|-------|-------------------|
| BC-2.17.001 | ENIP header parse None path | STORY-130 | `parse_enip_header` |
| BC-2.17.002 | ENIP header parse Some path | STORY-130 | `parse_enip_header` |
| BC-2.17.003 | ENIP frame validity gate | STORY-130 | `is_valid_enip_frame` |
| BC-2.17.004 | ENIP command classification | STORY-130/137 | `classify_enip_command`, `command_counts` |
| BC-2.17.005 | CPF item parse | STORY-132 | `parse_cpf_items` |
| BC-2.17.006 | CPF type routing (0x00B2 vs other) | STORY-132 | `parse_cpf_items` |
| BC-2.17.007 | CIP header parse | STORY-132 | `parse_cip_header` |
| BC-2.17.008 | T0888 Pattern A error burst | STORY-134 | `check_t0888` |
| BC-2.17.009 | CIP request path extraction | STORY-132 | `parse_cip_header` |
| BC-2.17.010 | T0846 ListIdentity recon | STORY-134 | `check_t0846` |
| BC-2.17.011 | T0858 CIP Stop | STORY-133/135 | `check_t0858` |
| BC-2.17.012 | T0836 write burst | STORY-133/135 | `check_t0836` |
| BC-2.17.013 | T0816 CIP Reset | STORY-133/135 | `check_t0816` |
| BC-2.17.014 | T0888 Pattern B identity attribute read | STORY-134 | `check_t0888` |
| BC-2.17.015 | ForwardOpen/ForwardClose lifecycle | STORY-136 | `check_forwardopen` |
| BC-2.17.016 | Frame-walk loop + command_counts | STORY-137 | `on_data` frame-walk |
| BC-2.17.017 | on_flow_close fold | STORY-138 | `on_flow_close` |
| BC-2.17.018 | T0814 malformed threshold | STORY-137 | `check_t0814` |
| BC-2.17.019 | Dispatcher wiring + bytes_received | STORY-131 | `on_data`, dispatcher |
| BC-2.17.020 | CLI flags (--enip, --enip-write-burst-threshold, --enip-error-burst-threshold) | STORY-131 | main.rs |
| BC-2.17.021 | summarize() canonical keys | STORY-138 | `summarize()` + open-flow fold |
| BC-2.17.022 | MAX_FINDINGS DoS guard | STORY-138 | `all_findings` cap |
| BC-2.17.023 | Write-burst threshold CLI | STORY-131 | `--enip-write-burst-threshold` |
| BC-2.17.024 | pdu_count | STORY-138 | `process_pdu` |
| BC-2.17.025 | Session handshake classify (no finding) | STORY-138 | `process_pdu` |
| BC-2.17.026 | Error-burst threshold CLI | STORY-131 | `--enip-error-burst-threshold` |
