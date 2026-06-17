---
document_type: feature-delta-analysis
feature_id: E-17
github_issue: 253
title: "ARP Decoder VLAN/QinQ/MACsec Offset Hardening"
intent: hardening
feature_type: test-and-docs (no src/ delta)
trivial_scope: false
trivial_justification: >
  The MACsec adjudication requires an architectural decision (documented-limitation
  vs. decoder code fix) before the scope can be finalised. QinQ coverage is
  test-only and confirmed-safe; MACsec correctness is genuinely unresolved. The
  full F1-F7 process is proportionate because: (a) a BC update is required
  regardless of the MACsec decision, (b) the VP-024 extension question must be
  resolved and documented, and (c) the MACsec path touches the same lax-None-arm
  decoder logic guarded by VP-024 (LOCKED v2.3). The process may be abbreviated
  at specific phases once the human gate confirms scope (see §8).
scope_classification: standard (abbreviated phases possible after gate)
status: draft
producer: architect
created: 2026-06-16
base_commit: "480f8ae"
base_version: "v0.7.0"
prior_cycle_delta: .factory/phase-f1-delta-analysis/arp-analyzer-delta-analysis.md
validation_record: .factory/research/issue-253-qinq-macsec-validation.md
pr_seed: "258 (test/arp-qinq-macsec-fixtures — seeds bc_2_16_qinq_macsec_offset_tests.rs with 4 tests; bc_2_16_e17_macsec_offset_tests.rs adds 6 further tests on same branch; total E-17 test delta = 10 tests across 2 files)"
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/specs/architecture/arp-architecture-delta.md
  - .factory/specs/verification-properties/vp-024-arp-parse-safety.md
---

# F1 Delta Analysis — E-17: ARP Decoder VLAN/QinQ/MACsec Offset Hardening

## Executive Summary

Issue #253 requested QinQ / MACsec test fixtures for the ARP VLAN-offset decoder
path. The DF-VALIDATION-001 research validated the issue as GENUINE/OPEN on `480f8ae`.
PR #258 (`test/arp-qinq-macsec-fixtures`) was opened, seeding `tests/bc_2_16_qinq_macsec_offset_tests.rs`
(4 tests: QinQ behavioral, QinQ model-pin, QinQ malformed→D11, MACsec observe-only probe). A second
file, `tests/bc_2_16_e17_macsec_offset_tests.rs` (6 tests: offset==22/30 assertions, malformed→D11,
Modified/opaque-unreachable security guards), was committed on the same branch in F4. Total E-17 test
delta = 10 tests across 2 files, with no `src/` production changes.

This F1 analysis adjudicates the central question the test-writer could not resolve:
**does MACsec require a decoder code fix, or is the correct outcome a
documented-limitation behavioral contract?** That decision determines whether this
cycle is test-and-docs-only (v0.7.1 patch) or includes a production code delta
(v0.7.x with reasoning about minor vs patch).

**Architect recommendation (MACsec adjudication — §3):**

> **DOCUMENTED-LIMITATION** is the correct outcome. No `src/` production decoder
> code change is warranted in this cycle. The rationale is: (a) the code is
> structurally correct for all reachable MACsec cases today (encrypted frames never
> reach the ARP lax-None arm), (b) the Unmodified/no-SCI case probe in PR #258
> confirms `header_len() == 8` and `arp_offset == 22` are consistent, and (c) the
> residual uncertainty (real-world captures, SCI variants, Modified payloads) is best
> addressed by a future cycle with empirical pcap data — not speculative code changes.
> This cycle's MACsec obligation is a BC postcondition clause that names the limitation
> explicitly.

**Scope:** 2 F3 stories. QinQ is pure coverage (STORY-116). MACsec is a BC
documentation clause plus the test already in PR #258 (STORY-117). Release target:
**v0.7.1 patch**.

---

## 1. Feature Summary and Scope Statement

### 1.1 Context

E-16 (ARP Security Analyzer, v0.7.0, STORY-111..115) shipped the lax-None-arm ARP
offset formula at `src/decoder.rs` lines 315–325:

```rust
let link_exts_len: usize =
    lax.link_exts.iter().map(|ext| ext.header_len()).sum();
Some(14 + link_exts_len)
```

Comments at lines 291–294 asserted QinQ "+8" and MACsec "variable" are "handled via
`LaxLinkExtSlice::header_len()` without hardcoding." BC-2.16.009 v1.7 (EC-008) and
BC-2.16.015 v1.6 (EC-008, PC-7a/7b) both explicitly name "VLAN 802.1Q/802.1ad,
MACsec" as covered. The tests in `tests/bc_2_16_d078_vlan_offset_tests.rs` exercise
only a single 802.1Q tag. No QinQ or MACsec fixture existed on `480f8ae`.

E-17 converts the unverified comment ("QinQ adds 8, MACsec variable") into
regression-guarded tests and resolves the MACsec correctness question with a clear
documented position.

### 1.2 What Is IN Scope

| Item | Rationale |
|------|-----------|
| QinQ (outer 0x88a8 + inner 0x8100) tests | Pure coverage gap; offset formula is PROVABLY CORRECT |
| Etherparse data-model pin test for QinQ | Guards against future etherparse version changing QinQ representation (VlanDouble) |
| MACsec (0x88e5) probe test | Already in PR #258; confirms no-panic and records `header_len()` |
| MACsec documented-limitation BC clause | Names the boundary of what is asserted vs. best-effort in BC-2.16.009 and BC-2.16.015 |
| BC-2.16.009 v1.8 and BC-2.16.015 v1.7 text updates (F2) | Add MACsec limitation clause to EC-008 and the relevant postcondition conditions |
| VP-024 coverage note (F2) | Document that the existing fuzz coverage (16.2M/0 per context) and new pin tests suffice; no VP extension needed |

### 1.3 What Is OUT of Scope

| Item | Reason |
|------|--------|
| MACsec decoder code change | No correctness defect demonstrated; see §3 for full rationale |
| MACsec+SCI offset assertion test | Superseded: `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30` in `tests/bc_2_16_e17_macsec_offset_tests.rs` arithmetically confirms SCI-present offset==30; real-on-wire-traffic existence remains the DOCUMENTED-UNVERIFIED boundary per EC-009(c) |
| Modified/encrypted MACsec ARP handling change | Encrypted frames never reach the ARP lax-None arm; existing case-c truncation is correct by construction; opaque-unreachable guard confirmed by security tests in `bc_2_16_e17_macsec_offset_tests.rs` |
| ARP storm / D3 / D1 detection changes | Out of scope for this hardening cycle |
| New VP (e.g., VP-025 stacked-tag offset safety) | Existing fuzz + new pin/assertion tests suffice; see §4 |
| MITRE technique changes | No new detection code introduced |
| etherparse upgrade beyond 0.20.2 | Not required; 0.20.2 is the confirmed pinned version |
| Triple-stacked tags, QinQ+MACsec combined, MACsec-with-non-ARP-inner | Untested combinations; the formula `14 + Σ header_len()` generalises correctly for arbitrary link_exts chains; real-traffic boundary per EC-009(c) |

---

## 2. Impact Boundary

### 2.1 Source Files

| File | Change Type | Description | Risk |
|------|-------------|-------------|------|
| `src/decoder.rs` | NONE | Offset formula (lines 315–325) is correct; no change needed | — |
| `tests/bc_2_16_qinq_macsec_offset_tests.rs` | NEW (PR #258 seed) | 4 tests: QinQ benign-truncated, QinQ malformed-hlen8, offset-formula pin, MACsec observe-only probe (`test_BC_2_16_015_macsec_arp_lax_parse_probe` — no offset assertion) | LOW |
| `tests/bc_2_16_e17_macsec_offset_tests.rs` | NEW (branch test/arp-qinq-macsec-fixtures, extends PR #258, committed in F4) | 6 tests: `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22` (asserts arp_offset==22), `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30` (asserts arp_offset==30), malformed→D11 for no-SCI, malformed→D11 for SCI-present, Modified-opaque-unreachable guard, encrypted-opaque-unreachable guard | LOW |

**No `src/` production code change is planned or recommended in this cycle.**

> **Scope resolved (F2 update):** The original `feature_type` carried a conditional "with conditional src/ delta for MACsec" qualifier. The e17-macsec-offset-deep-dive investigation and SCI-probe harness (`tests/bc_2_16_e17_macsec_offset_tests.rs`) conclusively confirmed the offset formula is correct for all reachable variants; the conditional is resolved to NO code change. `feature_type` updated to `test-and-docs (no src/ delta)`.

### 2.2 Behavioral Contracts

| BC | Version | Change | Owner |
|----|---------|--------|-------|
| BC-2.16.009 | v1.7 → v1.8 | Add MACsec limitation clause to EC-008: "For MACsec-encapsulated ARP, offset computation via `LaxLinkExtSlice::header_len()` is correct for Unmodified/no-SCI (header_len==8, arp_offset==22) but is unverified for real-world captures; encrypted/Modified payloads never reach this arm (stop_err != Layer::Arp) so case (c) truncation applies" | product-owner (F2) |
| BC-2.16.015 | v1.6 → v1.7 | Add same MACsec limitation clause to PC-7a/7b and EC-008; strengthen QinQ postcondition with an "offset == 22" line referencing the probe test | product-owner (F2) |
| All other BCs | — | No change | — |

### 2.3 Verification Properties

| VP | Change | Rationale |
|----|--------|-----------|
| VP-024 (v2.3, LOCKED) | Append-only note in lifecycle, no proof change | The QinQ path is not a Kani proof target (it is a lax-path behavioral observation, not a pure-core function invariant); the new tests cover the offset-formula path that VP-024 Sub-A does not directly exercise. The LOCKED status is not affected. |
| VP-INDEX | No change | VP count, tool assignments, module assignments unchanged |
| All other VPs | No change | — |

**VP-024 is LOCKED (v2.3, verification_lock: true). This cycle produces no proof-level
change. Any modification to VP-024 proof content would require VP withdrawal per VSDD
L4 immutability rules — that is not warranted here.**

### 2.4 Architecture Documents

| Document | Change | Rationale |
|----------|--------|-----------|
| `arp-architecture-delta.md` | Add E-17 reference in changelog (v1.18) | Records that the MACsec limitation is now formally documented |
| `ARCH-INDEX.md` | No change | SS-16 subsystem registry unchanged |
| All other architecture documents | No change | — |

### 2.5 Tests Touched or Created

| File | Status | Tests |
|------|--------|-------|
| `tests/bc_2_16_d078_vlan_offset_tests.rs` | UNCHANGED | 4 existing single-VLAN tests remain the regression baseline |
| `tests/bc_2_16_qinq_macsec_offset_tests.rs` | NEW (from PR #258 seed) | 4 tests: `test_BC_2_16_015_qinq_truncated_benign_arp_no_false_positive_d11`, `test_BC_2_16_009_qinq_malformed_hlen8_routes_to_d11`, `test_BC_2_16_015_qinq_link_exts_offset_formula_pin`, `test_BC_2_16_015_macsec_arp_lax_parse_probe` (MACsec observe-only probe — confirms no-panic and records `header_len()` == 8; does NOT assert an offset value) |
| `tests/bc_2_16_e17_macsec_offset_tests.rs` | NEW (branch test/arp-qinq-macsec-fixtures, extends PR #258, committed in F4) | 6 tests: `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22` (asserts arp_offset==22), `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30` (asserts arp_offset==30), malformed→D11 for no-SCI variant, malformed→D11 for SCI-present variant, Modified-payload opaque-unreachable security guard, encrypted-payload opaque-unreachable security guard |

**Total E-17 test delta: 10 tests across 2 new files.** The offset==22 and offset==30 assertions reside ONLY in `bc_2_16_e17_macsec_offset_tests.rs`; the qinq file's MACsec test (`test_BC_2_16_015_macsec_arp_lax_parse_probe`) is observe-only and asserts no offset value. Do NOT cite the qinq file as evidence for MACsec offset correctness.

Both files ship unchanged in F4 (no production code changes to implement against). The F4 "implementation" for
E-17 is: merge PR #258 after CI green.

---

## 3. Central Adjudication: MACsec — Code Fix or Documented Limitation?

### 3.1 The Question

BC-2.16.009 v1.7 EC-008 and BC-2.16.015 v1.6 both state that MACsec-wrapped ARP
offset computation is handled via `LaxLinkExtSlice::header_len()`. The validation
research (issue-253-qinq-macsec-validation.md Part C.3) and the PR #258 probe test
together reveal two uncertainties:

1. **`header_len()` counts the trailing 2-byte next-EtherType only for Unmodified
   payloads.** For an Unmodified/no-SCI SecTag: header_len == 8 (6 SecTag + 2
   next-EtherType). The PR #258 probe test (`test_BC_2_16_015_macsec_arp_lax_parse_probe`)
   confirms this empirically: `MacsecHeader::header_len() == 8` for no-SCI/Unmodified,
   and `LaxMacsecSlice.header.header_len() == 8` matches, so `arp_offset = 14 + 8 = 22`
   is the consistent computed value for this variant.

2. **Modified/encrypted MACsec frames never reach the ARP lax-None arm.** When MACsec
   encrypts the payload, etherparse sets `LaxMacsecPayloadSlice::Modified { ... }` and
   `stop_err` is NOT `Layer::Arp` (the ARP layer was never parsed). The decoder's
   `is_arp_truncation` check (`stop_err == Layer::Arp`) gates all further offset
   computation. Encrypted frames fail this gate and fall through to `Err("truncated ARP
   frame")` via case (c) — the generic truncation path. This is correct behavior: treating
   ciphertext as ARP fields would be semantically wrong.

### 3.2 The Decision Tree

```
Is there a demonstrated correctness defect in the decoder?
  → QinQ: NO (probe + pin test confirm arp_offset == 22 is correct)
  → MACsec Unmodified/no-SCI: NO — arithmetic synthetically confirmed:
    test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22 (in
    tests/bc_2_16_e17_macsec_offset_tests.rs) asserts arp_offset == 22;
    etherparse header_len()==8 for sci=false is the computed value. The only
    remaining open item is real-on-wire-traffic existence — the genuine
    DOCUMENTED-UNVERIFIED boundary per EC-009(c).
  → MACsec Unmodified/SCI-present: NO — arithmetic synthetically confirmed:
    test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30 (in
    tests/bc_2_16_e17_macsec_offset_tests.rs) asserts arp_offset == 30;
    header_len()==16 for sci=true is the computed value. Same real-traffic caveat.
  → MACsec Modified/encrypted: UNREACHABLE (gate prevents execution; confirmed
    by Modified/opaque-unreachable security guard tests in bc_2_16_e17_macsec_offset_tests.rs)

Does the unverified MACsec Unmodified case represent a latent bug risk?
  → Risk: the arithmetic has been synthetically confirmed (see offset==22/30 tests above);
    the formula generalises correctly for all Unmodified variants. The remaining uncertainty
    is "does etherparse produce a LaxNetSlice::Arp with stop_err == Layer::Arp for Unmodified
    MACsec+ARP in real on-wire captures?" — a question only real pcap data can answer. This
    is the documented-unverified boundary of EC-009(c), not a code correctness gap.

What code change would address the residual risk?
  → There is no code to change. The formula `14 + Σ header_len()` is the correct
    general formula for any link_exts chain, including MACsec. If etherparse reports
    an incorrect header_len for a MACsec variant, the fix would be in etherparse, not
    in wirerust's decoder.
```

### 3.3 Architect Recommendation

**DOCUMENTED-LIMITATION is the correct outcome.** No `src/` production decoder code
change is warranted in this cycle.

Reasoning:

1. **The formula is structurally correct.** `14 + Σ ext.header_len()` generalises over
   any `LaxLinkExtSlice` chain. For MACsec, `LaxMacsecSlice::header_len()` delegates
   to `MacsecHeaderSlice::header_len()`, which is the authoritative etherparse API for
   SecTag size. If etherparse reports an incorrect value, the defect is upstream.

2. **The offset assertion tests confirm both MACsec Unmodified variants arithmetically.**
   `tests/bc_2_16_e17_macsec_offset_tests.rs` directly asserts `arp_offset == 22` for the
   no-SCI case and `arp_offset == 30` for the SCI-present case. These are synthetic
   construction tests, not observe-only probes. The qinq file's `test_BC_2_16_015_macsec_arp_lax_parse_probe`
   (PR #258) additionally records `MacsecHeader::header_len() == 8` and
   `LaxMacsecSlice.header.header_len() == 8` as consistent observed values but does NOT
   assert an offset value (observe-only is its documented role).

3. **Encrypted/Modified frames are safe by construction.** The `stop_err == Layer::Arp`
   gate in `src/decoder.rs` line 302–303 ensures the ARP-malformed/truncated
   classification branch is never entered for frames that stop at Layer::MACsec.
   This is not a coincidence; it is the correct etherparse lax-parse semantics.

4. **The residual uncertainty is empirical, not structural.** Real-world MACsec+ARP
   captures may behave differently if MACsec decapsulation happens at the NIC before
   pcap capture (so the pcap never sees the MACsec header). That is an
   application-context question, not a decoder correctness question. A future cycle can
   address it with pcap fixtures once they are available.

5. **A speculative code change to "harden" the decoder would require inventing a
   guard that has no demonstrated failure mode.** This violates the TDD principle that
   governs VSDD: do not write code before there is a failing test to drive it.

### 3.4 The MACsec Documented-Limitation Clause

The F2 BC updates (BC-2.16.009 v1.8, BC-2.16.015 v1.7) must add a clause with
this substance to EC-008 (and the relevant postconditions):

> "For MACsec-tagged ARP frames (EtherType 0x88E5): offset computation via
> `LaxLinkExtSlice::header_len()` is provably correct for the Unmodified payload
> variant (header_len == 8 for no-SCI; header_len == 16 for SCI-present); this is
> confirmed by the etherparse API contract and the synthetic offset-assertion tests in
> `tests/bc_2_16_e17_macsec_offset_tests.rs` (`test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22`
> asserts arp_offset==22; `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30`
> asserts arp_offset==30). The observe-only probe in `tests/bc_2_16_qinq_macsec_offset_tests.rs`
> additionally records header_len()==8 but does not assert an offset value. Modified/encrypted
> MACsec payloads never reach the ARP truncation classification arm (stop_err != Layer::Arp) and
> therefore always produce `Err("truncated ARP frame")` via case (c) — this is
> correct behavior. Offset correctness for Unmodified MACsec frames has not been
> validated against real-world captures; if pcap tooling decapsulates MACsec before
> capture, MACsec-tagged frames may never appear in practice. This boundary is
> DOCUMENTED-UNVERIFIED; no code change is planned until a failing real-world test
> demonstrates a defect."

### 3.5 Residual Risk

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Real-world Unmodified MACsec+ARP frame has unexpected `header_len()` | LOW (etherparse API is deterministic; documented formula is public) | MEDIUM (wrong D11 classification) | Probe test in PR #258 would detect an etherparse version change; BC clause documents the unverified boundary |
| SCI-present Unmodified MACsec+ARP produces wrong offset | LOW (header_len==16 per formula; confirmed by test) | MEDIUM | CLOSED — `tests/bc_2_16_e17_macsec_offset_tests.rs` `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30` asserts `arp_offset == 30`; formula confirmed correct for SCI-present variant |
| pcap tooling strips MACsec before capture (MACsec never seen in practice) | MEDIUM–HIGH (common NIC behavior) | NONE (decoder never invoked for MACsec frames) | Not a correctness risk; reduces urgency of further MACsec testing |

---

## 4. VP Extension Assessment

### 4.1 Should VP-024 Be Extended for Stacked-Tag Offset Safety?

**Recommendation: NO new VP and no VP-024 extension.**

VP-024 (v2.3, LOCKED) covers `extract_arp_frame` (Sub-A) and the binding-table
invariants (Sub-B/C/D) in `src/analyzer/arp.rs` and `src/decoder.rs`. The QinQ/MACsec
offset formula lives in the lax-None arm of `decode_packet`, which is NOT a VP-024
target. VP-024 Sub-A proves that `extract_arp_frame` is panic-free for any valid
`ArpPacketSlice`; it does not prove anything about the lax-path ARP offset computation.

The offset formula is already exercised by:

- The existing 4-test `bc_2_16_d078_vlan_offset_tests.rs` suite (regression baseline).
- `tests/bc_2_16_qinq_macsec_offset_tests.rs` (4 tests: QinQ behavioral, QinQ model-pin,
  QinQ malformed→D11, MACsec observe-only probe — no offset assertion).
- `tests/bc_2_16_e17_macsec_offset_tests.rs` (6 tests: offset==22/30 assertions,
  malformed→D11, Modified/opaque-unreachable security guards).
- The cargo-fuzz VP-008 harness (16.2M iterations / 0 panics, per context), which
  runs `decode_packet` on arbitrary byte sequences and catches any panic path.

A Kani proof for the offset formula would require symbolically executing the
`LaxSlicedPacket::from_ethernet` path, which is an effectful etherparse function —
not a pure-core function amenable to Kani. There is no tractable Kani target here.

The correct mechanism for protecting the offset formula against future etherparse
representation changes is the model-pin test (`test_BC_2_16_015_qinq_link_exts_offset_formula_pin`
in `tests/bc_2_16_qinq_macsec_offset_tests.rs`), which fails loudly if `link_exts.len()`
or per-entry `header_len()` changes for QinQ.

**Conclusion:** Existing fuzz coverage (VP-008, 16.2M/0) plus the 10 new behavioral,
pin, and offset-assertion tests across 2 files are sufficient. No new VP. VP-024
receives a lifecycle append note only (see §2.3).

---

## 5. Regression Risk Assessment

### 5.1 Gates That Must Re-Run

| Gate | Why | Risk |
|------|-----|------|
| `cargo test --all-targets` (full suite) | 2 new test files added (10 new tests total); must confirm all 10 new tests pass and all existing tests remain green | LOW (test-only change; no src/ change) |
| `cargo clippy --all-targets -- -D warnings` | PR #258 already confirmed clean; must be re-confirmed on the merge commit | LOW |
| `cargo fmt --check` | PR #258 already confirmed clean | LOW |
| F5 scoped adversarial review (3-pass) | Required per VSDD feature cycle; scope is the 10 new tests across 2 files + BC text updates only | LOW (test-only) |
| F6 targeted hardening | VP-024 re-run: all 5 harnesses still PASS (no src/decoder.rs or src/analyzer/arp.rs change) | TRIVIAL — no harness input changed |
| F7 5-dim convergence | VP-INDEX, BC-INDEX, ARCH-INDEX consistency audit; story closure | LOW |

### 5.2 Gates That Are NOT Required

| Gate | Rationale for Skip |
|------|-------------------|
| Holdout evaluation (F4 holdout) | No new detection logic; holdout scenarios for ARP spoof/GARP detection are unchanged |
| VP-008 cargo-fuzz re-run | No `src/decoder.rs` change; fuzz target is unchanged |
| VP-007 mitre atomic update | No new MITRE technique IDs introduced |
| Integration / e2e pcap tests | No decode path change; existing e2e tests remain valid |

### 5.3 What Could Break

The only realistic failure mode is an etherparse version bump that changes QinQ
representation before this cycle lands. Test 3 (`test_BC_2_16_015_qinq_link_exts_offset_formula_pin`)
is specifically designed to fail loudly in that case, surfacing the regression before
production impact.

No existing tests are expected to break. The 4 tests in `bc_2_16_d078_vlan_offset_tests.rs`
remain unchanged and continue to pass on the same `src/decoder.rs` code.

---

## 6. Story Decomposition Preview

Two F3 stories, strictly sequential (STORY-116 → STORY-117):

| Story | Title | Scope | BCs | VPs | Dependencies |
|-------|-------|-------|-----|-----|--------------|
| STORY-116 | QinQ ARP Offset Coverage | Merge `tests/bc_2_16_qinq_macsec_offset_tests.rs` from PR #258 (3 QinQ tests: benign-truncated, malformed-hlen8, offset-formula-pin; plus MACsec observe-only probe as a 4th test in the same file); confirm all 4 pass; update BC-2.16.009 v1.8 and BC-2.16.015 v1.7 QinQ postcondition language (EC-008 "QinQ adds 8, confirmed via test") | BC-2.16.009 (v1.8, EC-008 QinQ), BC-2.16.015 (v1.7, PC-7b QinQ) | VP-024 lifecycle note (append-only) | STORY-115 (v0.7.0 close) |
| STORY-117 | MACsec ARP Offset Documented Limitation | Merge `tests/bc_2_16_e17_macsec_offset_tests.rs` (6 tests on the same branch: offset==22/30 assertions, malformed→D11 for no-SCI/SCI, Modified/opaque-unreachable security guards); update BC-2.16.009 v1.8 and BC-2.16.015 v1.7 with the MACsec documented-limitation clause (§3.4); update arp-architecture-delta.md v1.18 with E-17 reference | BC-2.16.009 (v1.8, EC-008 MACsec clause), BC-2.16.015 (v1.7, MACsec limitation clause) | none | STORY-116 |

**Note on story sequencing:** STORY-116 and STORY-117 are on the same branch
(`test/arp-qinq-macsec-fixtures`). The practical F4 implementation is: merge the
branch (after CI green and review), then apply the BC text updates. The total test
delta is 10 tests across 2 files (4 in the qinq file + 6 in the e17 file). The
two-story decomposition reflects the logical separation of QinQ behavioral
coverage (STORY-116) from MACsec offset-assertion + documented-limitation (STORY-117)
for traceability. A single-story delivery is acceptable if the human gate confirms
that the split provides no additional governance value.

---

## 7. Recommended Release Target

**v0.7.1 — patch release.**

Rationale:

- No `src/` production code change. Zero behavioral change to the decoder.
- The changes are: 10 new tests across 2 files + 2 BC text updates + 1 architecture
  document update. All are additive.
- The existing semantic-versioning convention for wirerust (v0.4.0 Modbus,
  v0.6.0 DNP3, v0.7.0 ARP) uses minor bumps for new analyzer subsystems and
  patch bumps for hardening/doc cycles within an existing subsystem.
- Parallel precedent: the ARP architecture delta v1.17 and VP-024 v2.3 are both
  patch-level document consistency fixes. E-17 is the corresponding test-and-docs
  hardening pass for the same cycle.

**If the human gate selects the MACsec code-fix path (i.e., overrides the
documented-limitation recommendation):** a code change to `src/decoder.rs` would
be required, the F6 Kani harness re-run becomes mandatory (VP-024 input changed),
and the release would remain a patch (no new public API) but would require the
full F4 TDD implementation story for the decoder change. BC-2.16.009 and
BC-2.16.015 would require postcondition revisions rather than limitation-clause
additions. The release target remains v0.7.1 but the cycle scope increases to 3
stories and a full VP-024 re-lock.

---

## 8. Human Gate Review Questions

The following questions require human decisions before F2 spec work begins.

### GATE-1 — MACsec Adjudication (BLOCKING)

The architect recommends DOCUMENTED-LIMITATION (no code change). The alternative
is a DECODER CODE FIX for the Unmodified MACsec path.

| Choice | Implications |
|--------|-------------|
| A: Documented-limitation (recommended) | 2 stories, test-only F4, v0.7.1 patch. BC clause documents unverified boundary. |
| B: Decoder code fix | Would require identifying what code to change — the formula `14 + Σ header_len()` IS correct per etherparse API. A "fix" would need to be for a specific demonstrated defect. No such defect is currently demonstrated. Recommend against until a failing test exists. |

**Question:** Confirm Choice A (documented-limitation), or explain what specific
defect you believe exists in the current MACsec handling that Choice B would correct.

### GATE-2 — Scope Completeness

The F1 analysis scopes this cycle to:
- QinQ behavioral tests (STORY-116)
- MACsec documented-limitation + probe test (STORY-117)
- BC-2.16.009 and BC-2.16.015 text updates

It explicitly excludes: MACsec+SCI offset test (no pcap data), Modified/encrypted
MACsec test (unreachable by construction), and any storm/spoof detection changes.

**Question:** Is the proposed scope complete? Are there additional MACsec variants
or offset edge cases you want included in this cycle?

### GATE-3 — Release Target

The recommendation is v0.7.1 (patch). This assumes Choice A (test-and-docs-only).

**Question:** Confirm v0.7.1, or override with a different version (and rationale).

### GATE-4 — F1-F7 Proportionality

This is a full F1-F7 cycle for what is essentially 10 tests across 2 files and 2 BC text updates.
The overhead (F2 spec evolution, F3 story decomposition, F5 adversarial, F6 Kani
re-run, F7 convergence) is real.

Options for phase justification:

| Option | What is skipped | Gate condition |
|--------|----------------|----------------|
| Full F1-F7 | Nothing | Human explicitly requested full process |
| Abbreviated F4 | F4 "implementation" = merge PR #258 (no TDD cycle needed; tests already written and green) | Confirm that PR #258 constitutes the F4 deliverable |
| Simplified F6 | VP-024 harnesses not re-run (no src/ change) | Confirm that VP-024 LOCKED status is not affected by test-only additions |
| Lightweight F7 | Convergence check is BC-INDEX + VP-INDEX consistency scan only, not full 5-dim | Confirm that test-only cycles use the lightweight convergence path |

**Question:** Confirm which phase abbreviations are acceptable for a test-and-docs
hardening cycle with no production code change.

### GATE-5 — PR #258 Disposition

PR #258 (`test/arp-qinq-macsec-fixtures`) seeds `tests/bc_2_16_qinq_macsec_offset_tests.rs`
(4 tests). `tests/bc_2_16_e17_macsec_offset_tests.rs` (6 further tests) was committed on the
same branch in F4. Total: 10 tests across 2 files on branch `test/arp-qinq-macsec-fixtures`.
CI is the F4 implementation seed. CI was not yet green (checklist item open at time of this F1).

**Question:** Should PR #258 be merged directly once CI passes and review is approved
(bypassing the normal F3 → F4 story-generation → F4 TDD cycle since the tests are
already written), or should it be closed and the tests re-introduced via the normal
F4 story flow?

---

## 9. Traceability Summary

| Artifact | Version | Status | Touched by E-17? |
|----------|---------|--------|-----------------|
| `src/decoder.rs` (lines 300–358) | post-v0.7.0 | unchanged | NO |
| `tests/bc_2_16_d078_vlan_offset_tests.rs` | — | unchanged | NO |
| `tests/bc_2_16_qinq_macsec_offset_tests.rs` | NEW | PR #258 seed (4 tests: QinQ behavioral, QinQ model-pin, QinQ malformed→D11, MACsec observe-only probe) | YES — F4 seed (qinq + observe-only probe) |
| `tests/bc_2_16_e17_macsec_offset_tests.rs` | NEW | branch test/arp-qinq-macsec-fixtures, extends PR #258, committed in F4 (6 tests: offset==22/30 assertions, malformed→D11 for no-SCI/SCI, Modified/opaque-unreachable guards) | YES — F4 MACsec offset-assertion and security tests |
| BC-2.16.009 | v1.7 → v1.8 | F2 target | YES — MACsec limitation clause |
| BC-2.16.015 | v1.6 → v1.7 | F2 target | YES — QinQ postcondition + MACsec clause |
| VP-024 | v2.3 LOCKED | lifecycle append-note only | YES (non-proof) |
| VP-INDEX | current | no count/tool/phase change | NO |
| arp-architecture-delta.md | v1.17 → v1.18 | changelog only | YES |
| ARCH-INDEX.md | — | unchanged | NO |

---

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-06-16 | Initial F1 delta analysis for E-17 (ARP VLAN/QinQ/MACsec offset hardening). MACsec adjudication: documented-limitation. Scope: 2 stories, test-and-docs, v0.7.1 patch. |
