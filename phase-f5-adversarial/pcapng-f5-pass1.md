---
document_type: adversarial-review
pass: 1
phase: F5
cycle: feature-pcapng-reader
date: 2026-06-21
reviewer_role: adversary (fresh-context scoped adversarial sweep)
verdict: NOT CLEAN
novelty: MEDIUM-HIGH
bc_completeness_sweep: "11/11 — 0 blockers"
disposition: ALL RESOLVED (PR #287 merged, develop=97c66b0)
adjudication_docs:
  - phase-f5-adversarial/F-F5P1-001-vp027-adjudication.md
  - phase-f5-adversarial/F-F5P1-003-O2-adjudication.md
---

# F5 Pass 1 — pcapng Holistic Adversarial Review

**Cycle:** feature-pcapng-reader
**Scope:** INTEGRATED pcapng delta — cross-story integration defects the per-story passes could not see.
Sources reviewed: `src/reader.rs`, `src/main.rs`, `.factory/specs/` (BC-2.01.009..018, VP-INDEX, ADR-009).

---

## Verdict: NOT CLEAN

| Severity | Count |
|----------|-------|
| HIGH | 1 |
| MEDIUM | 2 |
| LOW | 2 |
| **Total** | **5** |

Novelty: **MEDIUM-HIGH** (F-F5P1-001 is a formal-verification anti-pattern with security implications
not previously surfaced; F-F5P1-002/F-F5P1-003 are implementation-correctness gaps).

---

## Findings

### F-F5P1-001 (HIGH) — VP-027 Kani Harness Tautological: Never Called Real EPB Decode Path

**Category:** test-flaw / process-gap
**BC refs:** BC-2.01.012 PC9, AC-001; VP-027 (Kani)
**ADR refs:** ADR-009 Decision 2

**Summary:** The `vp027_epb_parse_safety` harness in `tests/kani_proofs.rs` asserts its own
`if`-guard conditions (tautologies), never invokes `decode_epb_body` (which did not yet exist as an
extractable function — the EPB body was inlined in the `EPB_BLOCK_TYPE` match arm of `read_pcapng_crate`),
and in one case asserts `"E-INP-009" != "E-INP-010"` (distinct string literals; true by construction,
proves nothing about code behavior). All 5 tautological cases + 1 vacuous assertion were confirmed.

**False-green class:** `cargo kani --harness vp027_epb_parse_safety` reports VERIFICATION SUCCESSFUL
while proving zero of VP-027's properties. VP-027 is the sole formal proof obligation for
SEC-004 (guard-before-allocate) and SEC-005 (no-panic) over the attacker-controlled EPB path
(captured_len, interface_id). If carried into F6 in this form, VP-027 would be locked as
"verified" while proving nothing — at which point the full VP withdrawal process is required.

**Status:** UPHELD by formal-verifier adjudication (F-F5P1-001-vp027-adjudication.md).
**Decision:** Option A — extract `pub fn decode_epb_body(...)` from the EPB arm, rewrite harness to
call the real function and assert E-INP-008/009/010 over symbolic (BMC-bounded) input.
**Resolved:** PR #287 merged (develop=97c66b0). Harness produces `VERIFICATION SUCCESSFUL`
with 687 checks; non-vacuity confirmed via deliberate-flip negative test.

---

### F-F5P1-002 (MEDIUM) — `read_magic` Doc-Comment in Stub/Todo Tense on Implemented Code

**Category:** impl-flaw / doc
**Policy:** DF-GREEN-DOC-TENSE (v1)
**BC refs:** BC-2.01.009 PC3 (magic-byte probe)

**Summary:** The `read_magic` function in `src/reader.rs` carried doc-comment language in
aspirational/stub tense ("TODO", "placeholder") on fully implemented, shipped, production
code. This violates the DF-GREEN-DOC-TENSE policy: all comments on production code must
describe what the code does, not what it was intended to do. The discrepancy creates an
auditor false-impression that the function is incomplete.

**Status:** UPHELD.
**Resolved:** Doc-comment corrected to present-tense declarative prose in PR #287
(develop=97c66b0). No normative behavioral change.

---

### F-F5P1-003 (MEDIUM) — `format_zero_packet_notice` Re-reads File: Redundant I/O + TOCTOU Mislabel

**Category:** impl-flaw / edge
**BC refs:** BC-2.01.009 PC6 (zero-packet notice format, "pcap|pcapng" discriminant)
**ADR refs:** ADR-009 Decision 19

**Summary:** `format_zero_packet_notice` (src/main.rs) called `read_magic(path)` to re-open the file
a second time solely to discriminate "pcap file" vs "pcapng file" in the notice wording. The format
discriminant was already known at the branch point inside `PcapSource::from_pcap_reader`
(`reader.rs:603-614`). Two concrete defects:

(a) **Redundant I/O:** Every zero-packet file caused two `open(2)` calls — avoidable.
(b) **TOCTOU mislabel:** If the file disappears between opens, `read_magic` returns `None`;
    the code defaults to "pcapng file", yielding a spec-incorrect notice for a classic-pcap file
    that was deleted between reads (BC-2.01.009 PC6 EC-009 mandates "pcap file" for classic-pcap).

**Status:** UPHELD. Adjudicated in F-F5P1-003-O2-adjudication.md.
**Decision:** Option A — add `is_pcapng: bool` to `PcapSource`; populate in both branch return
sites in `from_pcap_reader`; remove `read_magic` call from `format_zero_packet_notice`.
**Resolved:** PR #287 merged (develop=97c66b0). `PcapSource.is_pcapng` field added.

---

### O-1 (LOW) — Weak `contains("1")` / `contains("2")` Digit Assertions in Tests

**Category:** observation / test-quality
**BC refs:** BC-2.01.009 (zero-packet notice format)

**Summary:** Several test assertions used overly broad `contains("1")` or `contains("2")` substring
matches on notice strings where a more specific pattern was available. While these assertions
do not produce false-greens (the production code was correct), the weak assertions lower the
defect-detection sensitivity of the test suite — a future regression could alter numeric output
without triggering these checks.

**Status:** Observation accepted as LOW.
**Resolved:** Test assertions strengthened to use more specific patterns in PR #287.

---

### O-2 (LOW) — SPB vs EPB Guard-Ordering Asymmetry

**Category:** observation / intent adjudication
**BC refs:** BC-2.01.012 PC9, BC-2.01.013 (SPB parse)
**ADR refs:** ADR-009 Decision 22

**Summary:** BC-2.01.012 PC9 mandates a five-step EPB evaluation order (body.len() → read interface_id
→ empty-table E-INP-009 → OOB E-INP-010 → captured_len). The SPB arm checks empty-table (E-INP-009)
BEFORE body.len() (E-INP-008). For the single constructible overlap case (btl=12, body=0, empty table),
EPB-aligned ordering would fire E-INP-008; SPB fires E-INP-009. Both correctly reject the block.

**Status:** Adjudicated DO NOT ALIGN in F-F5P1-003-O2-adjudication.md.
**Rationale:** BC-2.01.013 does not define an evaluation order between these guards; SPB has no
`interface_id` field (always uses interface 0), so checking empty-table early is semantically valid
and documented. BC-2.01.013 minor bump (v1.9→v1.10) adds the accepted-behavior note.
**Resolved:** BC-2.01.013 v1.10 published (O-2 accepted behavior documented). No impl change.

---

## BC Completeness Sweep (11 BCs)

| BC | Sweep Status |
|----|-------------|
| BC-2.01.009 | CLEAN |
| BC-2.01.010 | CLEAN |
| BC-2.01.011 | CLEAN |
| BC-2.01.012 | CLEAN (VP-027 fix landed PR #287) |
| BC-2.01.013 | CLEAN (O-2 documented BC-2.01.013 v1.10) |
| BC-2.01.014 | CLEAN |
| BC-2.01.015 | CLEAN |
| BC-2.01.016 | CLEAN |
| BC-2.01.017 | CLEAN |
| BC-2.01.018 | CLEAN |
| BC-2.12.011 | CLEAN |

**Blockers from completeness sweep: 0.**

---

## Resolution Summary

| Finding | Severity | Disposition |
|---------|----------|-------------|
| F-F5P1-001 | HIGH | RESOLVED — PR #287 merged (97c66b0); genuine non-vacuous VP-027 proof; 687 Kani checks |
| F-F5P1-002 | MED | RESOLVED — PR #287; `read_magic` doc-comment corrected to green tense |
| F-F5P1-003 | MED | RESOLVED — PR #287; `PcapSource.is_pcapng` field eliminates TOCTOU + redundant open |
| O-1 | LOW | RESOLVED — PR #287; test assertions strengthened |
| O-2 | LOW | ADJUDICATED — DO NOT ALIGN; BC-2.01.013 v1.10 accepted-behavior note added |

**All findings resolved.** develop=97c66b0 (post PR #287 merge).

**Follow-up drift items (non-blocking, DO NOT block F6):**
- SEC-001 (F5P1) [MED]: no automated equivalence enforcement between `decode_epb_body` and
  `decode_epb_body_discriminant` twin — add a `#[cfg(test)]` parity smoke test.
  Target: follow-up maintenance/hardening story (placeholder STORY-F5-001).
- SEC-002 (F5P1) [LOW]: replace `wrapping_sub` in PC6b padding with plain subtraction + comment
  for auditor clarity. Target: same follow-up story.

**NEXT ACTION:** Run F5 Pass 2 (fresh-context adversary) toward 3 consecutive clean passes.
Then PAUSE for human review before F6.
