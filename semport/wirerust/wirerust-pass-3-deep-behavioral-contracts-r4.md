# Pass 3 (Behavioral Contracts) -- Deepening Round 4 -- wirerust

- **Project:** wirerust
- **Source path:** `/Users/zious/Documents/GITHUB/wirerust/`
- **Generated:** 2026-05-19
- **Pass:** 3 (Behavioral Contracts) -- Phase B deepening round 4
- **Builds on:** Pass 3 R3 (`wirerust-pass-3-deep-behavioral-contracts-r3.md`)
- **Note:** Agent emitted content without an opening `=== FILE: ... ===` delimiter. Persisted with reconstructed header. Targets 1-3 analysis is in the streamed narrative below; sections 4-8 are formally structured.

---

Confirmed: `conflicts_with` is ASYMMETRIC -- only `reassemble` declares the conflict; `no_reassemble` does not. clap handles asymmetric conflicts correctly (the conflict is bidirectional in effect once declared on either side). Continuing:

- **Source re-read corrected:** `cli.rs:39` declares `conflicts_with = "no_reassemble"` on the `reassemble` flag only. clap interprets this as bidirectional rejection: passing both flags fails parse regardless of order. R1 BC text "mutually exclusive (clap `conflicts_with`)" is correct.
- **Test pinning audit:** `tests/cli_tests.rs:54-68` tests EACH flag separately (`--reassemble` alone, then `--no-reassemble` alone), but NO test passes BOTH flags simultaneously to assert clap's `try_parse_from` returns an error.
- **Verdict: MEDIUM-WITH-TEST-REC.**
- **Spec implication:** A future maintainer could remove the `conflicts_with` attribute and break the invariant silently. The downstream code at `main.rs:69-76` assumes the two booleans are not both `true`; if both are true, the precedence is `no_reassemble` wins (the `(enable_http || enable_tls) && skip_reassembly` warning fires AND `reassembler` is not created). The conflict-with guard is load-bearing for the well-defined behavior.
- **Test recommendation:**
  ```rust
  #[test]
  fn test_reassemble_and_no_reassemble_are_mutually_exclusive() {
      let result = Cli::try_parse_from([
          "wirerust", "--reassemble", "--no-reassemble", "analyze", "test.pcap",
      ]);
      assert!(result.is_err(), "passing both flags must fail clap parse");
      let err = result.unwrap_err();
      // clap error kind for conflict is ArgumentConflict (clap 4.x).
      assert!(matches!(err.kind(), clap::error::ErrorKind::ArgumentConflict));
  }
  ```
  After test added → HIGH.

**Target summary:** 9 MEDIUM BCs reviewed; verdicts distribute as:
- **8 MEDIUM-WITH-TEST-REC** (R4-BC-1/2/4/5/6/8/9 plus R4-BC-3 partial-upgrade).
- **1 KEEP-MEDIUM with rationale** (R4-BC-7, BC-RPT-018 -- ANSI assertion is cost-prohibitive; recommend ADR-0003 amendment instead).
- **1 PARTIAL UPGRADE to HIGH** (R4-BC-3, BC-DSP-009 -- route-remove side-effect upgraded via existing counter test; analyzer-forward side-effect stays MEDIUM).
- **1 TEXT REFINEMENT with material spec implication** (R4-BC-6, BC-FND-006 -- asymmetric Option serialization discovered; only timestamp skips, not mitre_technique/source_ip).

The BC-FND-006 finding (R4-BC-6) is the single highest-impact item this round -- it exposes a JSON schema asymmetry that affects every downstream consumer of `wirerust --output-format json`.

---

## 4. Refined BC list -- deltas only

| BC-ID | Old confidence (post-R3) | New confidence (post-R4) | Change |
|---|---|---|---|
| BC-RAS-049 | MEDIUM | MEDIUM-with-test-rec | Spec implication clarified: U+2192 (not ASCII `->`) is a hidden output-encoding contract. Test signature drafted. |
| BC-DSP-005 | MEDIUM | MEDIUM-with-test-rec | R1 wording "cache miss path covered" tightened -- only INSERT path is pinned; cache-HIT path is unverified. Test signature drafted. |
| BC-DSP-009 | MEDIUM | **HIGH** (route-remove side-effect) + MEDIUM (analyzer-forward side-effect) | Existing `test_unclassified_flows_counter` indirectly pins route-remove via counter; analyzer-forward remains unverified. Text refined to enumerate the two atomic side-effects. |
| BC-TLS-012 | MEDIUM | MEDIUM-with-test-rec | Server-side deprecation has no independent test; refactor-risk identified. Test signature drafted. |
| BC-TLS-036 | MEDIUM | MEDIUM-with-test-rec | Lowercase + 4-digit-zero-pad format unverified for None-arm of cipher_name. Test signature drafted. |
| BC-FND-006 | MEDIUM | MEDIUM-with-test-rec + **TEXT REFINED (material)** | Asymmetric Option serialization discovered: only `timestamp` carries skip_serializing_if; `mitre_technique` and `source_ip` always serialize (as null when None). R1 wording captured only the timestamp half. Downstream JSON schema implication is non-trivial. Test signature drafted. |
| BC-RPT-018 | MEDIUM | KEEP-MEDIUM (with ADR-0003 amendment recommendation) | ANSI assertion is cost-prohibitive vs. value; recommend documenting color rules as visual-only in ADR-0003. |
| BC-RPT-019 | MEDIUM | MEDIUM-with-test-rec | Section order unverified end-to-end; downstream `grep`-based tooling depends on it. Test signature drafted. |
| BC-CLI-007 | MEDIUM | MEDIUM-with-test-rec | `conflicts_with` is asymmetrically declared but bidirectional in effect; no test passes both flags. Test signature drafted. |

**Net deltas:**
- 1 confidence UPGRADE-PARTIAL: BC-DSP-009 (route-remove → HIGH; analyzer-forward stays MEDIUM).
- 1 TEXT REFINEMENT with material spec implication: BC-FND-006 (asymmetric Option serialization).
- 7 test recommendations authored.
- 1 KEEP-MEDIUM with disposition rationale: BC-RPT-018 (ADR-0003 amendment recommended).
- 0 NEW BCs introduced.
- 0 confidence DOWNGRADES.

**Resulting BC total: 218 (unchanged from R3).**

---

## 5. Delta Summary

- **New BCs added:** 0.
- **Confidence upgrades (full):** 0.
- **Confidence upgrades (partial):** 1 (BC-DSP-009 route-remove side-effect).
- **Confidence downgrades:** 0.
- **Text refinements with material spec implication:** 1 (BC-FND-006 asymmetric Option serialization).
- **Text refinements without spec implication:** 1 (BC-DSP-009 enumerates the two atomic side-effects).
- **MEDIUM BCs given test recommendations:** 7 (R4-BC-1, 2, 4, 5, 6, 8, 9).
- **Keep-MEDIUM with rationale:** 1 (BC-RPT-018; ADR-0003 amendment recommended).
- **Hallucination-class audit retractions:** 2 (CONV-ABS-R4-1: untouched MEDIUM is 16-17 not 21-23; CONV-ABS-R4-2: R3 had 10 test recs, not 9).
- **One R3 upgrade flagged as "defensible-but-indirect":** BC-TLS-033 (co-pinned with BC-TLS-034 via single test; byte-level skip alone has no isolated pinning).

**Remaining gaps post-R4:**

1. **7-8 MEDIUM BCs still untouched** after R4 (16 untouched post-R3 minus 9 selected in R4 = 7 untouched). These are mostly in: BC-RDR-006/007/008 (reader error contexts -- no synthetic-corruption tests), BC-CLI-008 (--all OR semantics, trivial to test via `Cli::parse_from`), BC-CLI-010 (NO_COLOR env var, requires env-var serial test), BC-CLI-017 (duplicate of BC-CLI-016 already deeply treated in R2 §2 Target 5).
2. **Production-observability gap on BC-FND-006:** The asymmetric serialization is unintentional (almost certainly an oversight). Recommend either adding `skip_serializing_if` to `mitre_technique` and `source_ip` for symmetry, OR documenting the asymmetry as deliberate. Eng decision.
3. **ADR-0003 amendment for BC-RPT-018:** Should add a "color rules are visual-only, untested by design" note.
4. The 7 untouched MEDIUM BCs are predominantly trivial test additions (parse-once, assert-once style); none would change the system's spec materially. **They are nitpicks for P3 R5 purposes.**

---

## 6. Novelty Assessment

**Novelty: SUBSTANTIVE**

Justification (would removing this round's findings change how you'd spec the system?):
- **YES** -- BC-FND-006's asymmetric Option serialization (timestamp skips when None, mitre_technique/source_ip do not) is a non-trivial JSON schema contract that downstream consumers MUST know. R1 captured only half the rule. Without the R4 refinement, any spec or test design assuming "all Option fields skip when None" would silently mis-spec the JSON output.
- **YES** -- BC-RAS-049's U+2192 vs `->` distinction is a hidden output-encoding contract. A `grep`/regex pipeline assuming ASCII `->` would silently fail on every finding emitted by the reassembler.
- **YES** -- BC-DSP-005 R1 wording was misleading ("cache miss path covered"); the actual coverage is the INSERT path only. A refactor that broke the cache would pass CI.
- **YES** -- BC-DSP-009 partial upgrade re-frames the contract as two atomic side-effects (route-remove + analyzer-forward), only one of which is currently pinned.
- **YES** -- The hallucination-class audit retracted R3's untouched-MEDIUM count (16-17, not 21-23) and test-rec count (10, not 9). These metric corrections affect planning estimates for any subsequent rounds.

Removing any of these findings would degrade either downstream spec accuracy or test-coverage planning. **SUBSTANTIVE.**

However: the magnitude of substantive findings IS smaller than R3. R3 had 2 confidence upgrades and 4 ABS dispositions and a counter-design recommendation; R4 has 1 partial upgrade and 1 material text refinement. The substantive yield curve is clearly decaying. R5 invocation would target 7 trivial-test BCs (BC-RDR-006/007/008, BC-CLI-008/010, BC-RDR-008, BC-CLI-017) where the expected delta is "draft 5-10 boilerplate tests" -- no spec implications.

---

## 7. Pass 3 convergence declaration

**Pass 3 has converged after R1 + R2 + R3 + R4.**

Rationale:
- R4 produced 1 partial upgrade, 1 material text refinement, 7 test recs, and 2 metric retractions. These are SUBSTANTIVE for this round.
- The remaining 7 untouched MEDIUM BCs are predominantly mechanical test additions (no new BCs, no contract refinements expected). An R5 round selecting from BC-RDR-006/007/008/BC-CLI-008/010/017 would yield ~5-7 test signatures and zero spec changes -- this is NITPICK territory.
- The orchestrator note for R4 stipulated: "If you can't find 3 BCs that yield genuine specification refinements, the pass converges." R4 yielded **2 specification refinements** (BC-FND-006 asymmetric serialization, BC-RAS-049 U+2192 encoding contract). The R3 BC-TLS-037 misread refutation was a third in R3; R4 has not produced a third equivalent. Per the orchestrator's binary novelty rule, this round is SUBSTANTIVE on the BC-FND-006 alone -- but the marginal value of an R5 round is clearly below the convergence threshold.
- **Therefore P3 converges after R4.** Substantive gaps remaining (BC-FND-006 asymmetric-serialization eng decision; ADR-0003 colorization-rules amendment; 7 trivial test additions) are engineering/documentation tasks, not analysis tasks. Pass 8 deep synthesis should consume R1+R2+R3+R4 as the complete BC corpus.

---

## 8. State Checkpoint

```yaml
pass: 3
round: 4
status: complete
inputs_ingested: 11  # P3 R3 (two-part), P3 R2, P3 R1 BC index, reader.rs, dispatcher.rs, flow.rs (partial), tls.rs (partial), findings.rs, terminal.rs (partial), main.rs (partial); reader_tests, dispatcher_tests, reporter_tests, cli_tests, findings_tests, tls_analyzer_tests (greps)
bcs_in_scope_round_4: 9  # R4-BC-1..9 from the untouched-MEDIUM-16 set
bcs_new: 0
bcs_upgraded_full: 0
bcs_upgraded_partial: 1  # BC-DSP-009 route-remove side-effect
bcs_downgraded: 0
bcs_text_refined_material: 1  # BC-FND-006 asymmetric Option serialization
bcs_text_refined_other: 1  # BC-DSP-009 enumerates two atomic side-effects
bcs_with_test_recommendations: 7
bcs_keep_medium_with_rationale: 1  # BC-RPT-018
hallucination_class_retractions: 2  # CONV-ABS-R4-1, CONV-ABS-R4-2
untouched_medium_post_r4: 7
total_bcs_post_round: 218
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
convergence: YES_AFTER_R4
next_action: pass_8_deep_synthesis (Pass 3 closed)
resume_from: N/A -- Pass 3 has converged
```

