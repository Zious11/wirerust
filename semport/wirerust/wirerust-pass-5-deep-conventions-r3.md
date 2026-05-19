# Pass 5 (Convention Catalog) -- Deepening Round 3 -- wirerust

- Project: wirerust
- Source path: /Users/zious/Documents/GITHUB/wirerust/
- Generated: 2026-05-19
- Pass: 5 (Conventions) -- Phase B deepening, round 3 (convergence-anchor)
- Builds on: P5 R2 (wirerust-pass-5-deep-conventions.md), P5 R1 (wirerust-pass-5-conventions.md), P2 R3, P3 R3/R4
- Scope: Re-verification of R2's metric corrections, deferred `#[allow]` audit, spot-checks on R2's two highest-risk new IDs (CNV-ERR-011, CNV-PAT-002).

---

## 1. Re-verification matrix

| # | R2 claim under audit | Method | Observed | Verdict |
|---|---|---|---|---|
| 1 | R2 §4: "85 + 6 new IDs = 91 conventions" | Enumerate the new-ID list in R2 §3 and §4 | R2 §3 introduces exactly 5 new IDs: `CNV-ERR-011`, `CNV-PAT-001`, `CNV-PAT-002`, `CNV-FMT-009`, `CNV-CLI-001`. `CNV-FMT-008` is explicitly **subsumed** (named-then-merged into CNV-FMT-007; not a net add). R2's own §4 bullet text reads "5 new IDs ... CNV-FMT-008 subsumed" yet the sum line says "+6". | **RETRACT CONV-ABS-P5-R2-1.** Actual net additions: **5**. Corrected catalogue total: **85 + 5 = 90 conventions** (not 91). |
| 2 | R2 §4: post-R2 by-category breakdown sums to 91 (NAM=12 + MOD=6 + PUB=7 + ERR=11 + LOG=6 + TST=11 + FMT=9 + DEP=8 + GIT=8 + DOC=10 + CLI=1 + PAT=2) | Sum the per-category totals | 12+6+7+11+6+11+9+8+8+10+1+2 = **91**. But R2's FMT count says "9 (+2)" while only 1 new FMT ID (CNV-FMT-009) was added (CNV-FMT-008 subsumed). The "+2" should be "+1", making FMT=8. | **RETRACT CONV-ABS-P5-R2-1 (companion).** Corrected: FMT=8, total = **90**. |
| 3 | R1 + R2: "zero `#[allow]` clusters in src/" (R2 §5 deferred re-verify) | `find src -name '*.rs' -exec awk '/#\[allow\(/'` | Zero matches across all 20 src files. | **VERIFIED.** R1's claim stands; R2's deferral is now closed. No retraction. |
| 4 | R2 CNV-ERR-011: "Zero `impl Drop` in src/" | `find src -name '*.rs' -exec awk '/impl Drop/'` | Zero matches across all 20 src files. | **VERIFIED.** CNV-ERR-011 stands as catalogued. No retraction. |
| 5 | R2 CNV-PAT-002: "every silent-drop site already follows this pattern" (universality = "all") | Inspect `pub struct TlsAnalyzer` (tls.rs:271-281) for a truncation/drop counter analogous to HttpAnalyzer's `poisoned_bytes_skipped` (http.rs:862) | TlsAnalyzer fields: `flows`, `sni_counts`, `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts`, `handshakes_seen`, `parse_errors`, `all_findings`. **No `dropped_tls_bytes`, no `truncated_records`, no truncation counter of any kind.** The only error-adjacent counter is `parse_errors`, which is not a silent-drop byte count. HttpAnalyzer has `poisoned_bytes_skipped: u64` (http.rs:862, incremented at lines 442 and 454). | **RETRACT CONV-ABS-P5-R2-2.** CNV-PAT-002's R2 framing "every silent-drop site already follows this pattern" is **OVER-EXTRAPOLATED from one site (HTTP) to all sites (TLS, DNS, reassembly)**. TLS has at least one silent-drop site (handshake-record truncation, per P2 R3 §2 Target 4) with **no** instrumented counter. Correct framing: CNV-PAT-002 is an **aspirational** convention drawn from HTTP + reassembly precedent; **TLS does not yet conform.** Universality demotes from "all" to "**most**". |

---

## 2. Retractions emitted by R3

**CONV-ABS-P5-R2-1** (arithmetic) -- R2's "+6 new IDs / 91 total" is off-by-one. Subsumption (`CNV-FMT-008`) was counted as a net add. Corrected: **5 new IDs, 90 total**.

**CONV-ABS-P5-R2-2** (over-extrapolation) -- R2's CNV-PAT-002 universality "all" requires retraction to "most". TLS silent-drop sites are uninstrumented. CNV-PAT-002 should be re-issued as:

> When the engine silently drops a forensic event due to a cap or quota, the drop event SHOULD be instrumented as a `u64` counter on the owning struct's stats type, named with a `_dropped` / `_skipped` / `_evicted` suffix. **HTTP and reassembly conform; TLS does not yet conform (no truncation counter on `TlsAnalyzer`).** Closing the gap is engineering work, not analysis work.

This also creates a follow-on engineering item: **add `TlsAnalyzer::truncated_records: u64`** (or analogous) to bring TLS into CNV-PAT-002 compliance.

---

## 3. Corrected R2 §4 totals (post-R3)

- R1 catalogued (corrected by R2): **85**
- R2 net new IDs (corrected by R3): **+5** (CNV-CLI-001, CNV-ERR-011, CNV-PAT-001, CNV-PAT-002, CNV-FMT-009; CNV-FMT-008 subsumed, NOT counted)
- **Catalogue total: 90 conventions** (not 91)

By category (post-R3 correction):
NAM=12, MOD=6, PUB=7, ERR=11 (+1 from R2), LOG=6, TST=11, FMT=8 (+1 from R2, was incorrectly "+2"), DEP=8, GIT=8, DOC=10, CLI=1 (NEW), PAT=2 (NEW) = **90**.

By universality (post-R3 correction): all=68 (was 69; CNV-PAT-002 demoted), most=17 (was 16; CNV-PAT-002 promoted-to-most), some=5, none=0. Sum = 90.

---

## 4. Engineering follow-up flagged

A single new TODO not previously called out by R1 or R2:

- **TlsAnalyzer truncation counter.** Add `truncated_records: u64` (or equivalent name) to `TlsAnalyzer` struct (tls.rs:271-281) and increment at each silent record-discard site identified by P2 R3 §2 Target 4. This brings TLS into CNV-PAT-002 conformance and matches the HTTP precedent (`poisoned_bytes_skipped`). Estimated PR cost: ~10 LOC + 1 test.

No other engineering follow-ups discovered.

---

## 5. Delta Summary

- New items added: **0 new convention IDs** (R3 is verification-only).
- Existing items refined: **2 retractions on R2 headline numbers** (catalogue total 91 -> 90; CNV-PAT-002 universality "all" -> "most"). **2 confirmations** (`#[allow]` zero; `impl Drop` zero). **1 engineering follow-up surfaced** (TlsAnalyzer truncation counter).
- Remaining gaps: None analysis-side. The two R2 open engineering decisions (CNV-PAT-001 missing-by-intent canonicalization; CNV-FMT-009 Option-serialize symmetry) stand; R3 adds one more (TLS truncation counter).

---

## 6. Novelty Assessment

**Novelty: SUBSTANTIVE** (narrowly, on the strict-binary test)

Justification -- would removing R3 change how the system is specced?
- YES on retraction #1: downstream docs would cite "91 conventions" with an off-by-one error from a counted-but-subsumed ID. The headline number propagates into Pass 8 deep synthesis.
- YES on retraction #2: CNV-PAT-002's "universality = all" would be quoted as a settled convention; in fact it is partially adopted, and TLS is the non-conformant site. This is a model-level distinction (aspirational vs. observed), not a wording refinement.
- The two confirmations (`#[allow]`, `impl Drop`) are NITPICK in isolation, but they retire R2's deferred audit -- which had to be retired before P5 could be declared converged.

The retractions are arithmetic/scope corrections to R2's headline claims. They are not new conventions, not refinements of cell-level data; they are model-level corrections that downstream consumers (Pass 8, create-brief, semport-analyze) would otherwise carry forward incorrectly. **Strict binary: SUBSTANTIVE.**

The marginal yield is now visibly exhausted. A hypothetical R4 would re-audit R3's own arithmetic -- definitionally NITPICK territory.

---

## 7. Pass 5 final convergence declaration

**Pass 5 has converged after R3.**

Rationale:
- R1 (broad) catalogued 85 conventions; correctly.
- R2 (deepening) added 5 new IDs (not 6), set direction on 4 in-transit conventions, produced 4 concrete refactor/codification action plans, and deferred 1 audit.
- R3 (convergence-anchor) closed the deferred audit (zero `#[allow]` confirmed), verified `impl Drop` zero, corrected R2's "+6 / 91" arithmetic to "+5 / 90", and demoted CNV-PAT-002 universality from "all" to "most" with a surfaced engineering follow-up (TLS truncation counter).

R3's findings are confined to: (a) two retraction-class corrections of R2 headline numbers, (b) two confirmations retiring deferred audits, (c) one engineering follow-up. No new conventions discovered. No new patterns discovered. No new categories. **Per the binary novelty rule, P5 converges at R3.**

A hypothetical R4 would only re-audit R3's arithmetic -- NITPICK-class. Pass 8 deep synthesis should consume R1 + R2 + R3 as the complete convention corpus, **with R3's corrections applied** (catalogue total 90, not 91; CNV-PAT-002 universality "most", not "all").

---

## 8. State Checkpoint

```yaml
pass: 5
round: 3
status: complete
sub_pass: deep_conventions_r3
targets_addressed: 4  # 1 arithmetic re-derive + 1 #[allow] audit + 2 spot-checks
new_convention_ids: 0
retractions_emitted: 2  # CONV-ABS-P5-R2-1 (arithmetic), CONV-ABS-P5-R2-2 (CNV-PAT-002 over-extrapolation)
confirmations: 2  # #[allow] zero, impl Drop zero
engineering_followups_surfaced: 1  # TlsAnalyzer truncation counter
conventions_total_post_r3: 90
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
convergence: YES_AFTER_R3
next_action: pass_8_deep_synthesis
resume_from: null
```

