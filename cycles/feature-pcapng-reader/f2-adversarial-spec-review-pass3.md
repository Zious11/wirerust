---
document_type: adversarial-spec-review
cycle: feature-pcapng-reader
phase: F2
pass: 3
date: 2026-06-19
spec_state: "ADR-009 rev 5, BC-2.01.009..018, error-taxonomy v2.8, VP-INDEX v2.4, HS-101..107"
verdict: NOT CLEAN
critical: 1
high: 5
medium: 7
low: 4
novelty: HIGH
novelty_class: "partial-fix-propagation + sibling-layer + dead-spec"
clean_pass_counter: 0
threshold_to_pass: "0 CRITICAL, 0 HIGH, <3 MEDIUM"
---

# Adversarial Spec Review — Pass 3

**Cycle:** feature-pcapng-reader
**Pass:** 3 (fresh context)
**Spec state at review:** ADR-009 rev 5, BC-2.01.009..018, error-taxonomy v2.8, VP-INDEX v2.4, HS-101..107
**Verdict:** NOT CLEAN
**Counts:** 1 CRITICAL / 5 HIGH / 7 MEDIUM / 4 LOW
**Novelty:** HIGH — partial-fix-propagation class, sibling-layer class, and dead-spec class (not anticipated by pass-1 or pass-2 novelty classes)

---

## CRITICAL Findings

### C-1 [CRITICAL]: BC-2.01.013 PC1 / AC-002 — Two-way min still present; three-way min required everywhere

**Affected locations:**
- BC-2.01.013 PC1 (line ~55): uses `min(original_len, snaplen)`
- BC-2.01.013 AC-002 (line ~75): uses `min(original_len, snaplen)`
- EC-001 / Invariant-2 / VP: already state three-way `min(original_len, snaplen, block_body_available)`

**Root cause:** The v1.2 changelog entry for BC-2.01.013 FALSELY claimed PC1 was fixed (finding I-4 from pass-2). The changelog claimed the fix landed, but the body of PC1 and AC-002 still use the two-way form. This is a partial-fix-propagation failure: the changelog was updated to reflect a fix that was not applied to the normative prose.

**Impact:** The two-way `min(original_len, snaplen)` form does not bound the slice by the actual block body bytes available. On a malformed SPB where `original_len` or `snaplen` exceeds the block body, the implementation will produce an out-of-bounds slice — a panic. This violates the no-panic AC (present in BC-2.01.013) and HS-107 Case B (malformed snaplen fixture authored in pass-2).

**Fix required:** Apply three-way `min(original_len, snaplen, block_body_available)` in PC1 and AC-002. The slice MUST be bounded by block body length unconditionally. Verify disk state matches changelog before marking this fixed.

---

## HIGH Findings

### H-1 [HIGH]: BC-2.01.010 / BC-2.01.009 — E-INP-008 SHB body-truncation fixture unconstructible

**Affected locations:** BC-2.01.010 PC5, BC-2.01.009 AC-004, error-taxonomy EC-005 (E-INP-008 for SHB body truncation)

**Root cause:** The pcap-file crate rejects any SHB where `block_total_length < 12` at framing level, and cannot be made to deliver a body shorter than `block_total_length - 12` bytes. Therefore:
- Any SHB body-truncation scenario classified as E-INP-008 via crate-level framing rejection is actually E-INP-010 (crate returned Err), not E-INP-008.
- A test fixture that forces the crate to return a semantically-correct SHB with a truncated body (body length < btl-12) is UNCONSTRUCTIBLE through the crate's public API.

**Impact:** BCs that specify E-INP-008 for SHB body-truncation describe test cases that cannot be written; the error code assignment is wrong.

**Fix required:** Narrow E-INP-008 for SHB to semantic failures only: invalid BOM (unrecognized byte-order magic), major version != 1, and similar semantic-parse errors that succeed at framing. All SHB framing truncation (body < btl-12) routes to E-INP-010 via crate Err. Update BC-2.01.010 PC5, AC-004, and EC-005 accordingly.

### H-2 [HIGH]: BC-2.01.011 — IDB body-truncation fixture unconstructible; constructible window undefined

**Affected locations:** BC-2.01.011 PC5, error-taxonomy EC for IDB body truncation

**Root cause:** Same unconstructible-fixture class as H-1, applied to IDB. The crate rejects IDB framing truncation. The constructible E-INP-008 window for IDB is body length 0–7 bytes (i.e., `12 <= block_total_length < 20`), which corresponds to a minimal IDB where the required fixed fields are incomplete. BC-2.01.011 does not state this window explicitly and references "crate returned a short body" which is not achievable.

**Fix required:** State explicitly that the constructible E-INP-008 window for IDB is `12 <= btl < 20` (body 0–7 bytes); remove language saying "crate returned a short body" for IDB body-truncation.

### H-3 [HIGH]: E-INP-001 orphaned — BC-2.01.016 linktype-whitelist not in error-taxonomy BC-ref; E-INP-001 not in BC-2.01.017

**Affected locations:**
- error-taxonomy E-INP-001 entry: BC-ref does not include BC-2.01.016
- BC-2.01.017 context-strings and error-code table: does not enumerate E-INP-001 (range listed as E-INP-008..013)

**Root cause:** E-INP-001 (linktype-not-in-whitelist) predates the pcapng BCs and was not swept when BC-2.01.016 was authored to own linktype-whitelist enforcement. BC-2.01.016 fires E-INP-001 but the error-taxonomy entry does not back-reference it, and the cross-cutting parent BC-2.01.017 does not enumerate it.

**Fix required:** Add BC-2.01.016 to E-INP-001 BC-ref in error-taxonomy. Add E-INP-001 to BC-2.01.017 error-code table and context-strings section.

### H-4 [HIGH]: BC-2.01.013 EC-007 / Case-B SPB snaplen/padding self-contradiction (same root as C-1)

**Affected locations:** BC-2.01.013 EC-007, Case-B description

**Root cause:** EC-007 and Case-B describe SPB snaplen enforcement and padding calculation using the two-way min form in a manner that self-contradicts the three-way min stated in Invariant-2. The pass-2 remediation updated Invariant-2 but did not propagate the three-way min to EC-007 and Case-B.

**Fix required:** Propagate three-way min to EC-007 and Case-B. This is the same underlying fix as C-1 but in additional locations within BC-2.01.013.

### H-5 [HIGH]: Multi-section interface-table reset is dead spec

**Affected locations:**
- BC-2.01.011 Invariant 2: mandates per-SHB interface-table reset
- BC-2.01.018 Invariant 4, EC-005: mandate per-SHB reset; EC-005 says multi-section "succeeds per section"
- ADR-009 Decision 7: rejects the 2nd SHB before any reset occurs

**Root cause:** Decision 7 (multi-section reject) causes the implementation to return E-INP-012 when a second SHB is encountered. The reject fires before any interface-table reset. Therefore:
- BC-2.01.011 Invariant 2 ("reset interface table on each SHB") describes behavior that never executes in the implementation.
- BC-2.01.018 Invariant 4 and EC-005 ("succeeds per section") contradicts the reject.

These invariants are dead spec — they describe a code path that the reject decision makes unreachable.

**Fix required:** Delete or explicitly defer per-section-reset invariants from BC-2.01.011 Invariant 2 and BC-2.01.018 Invariant 4. Correct BC-2.01.018 EC-005: multi-section results in E-INP-012 (reject), not "succeeds per section."

---

## MEDIUM Findings

### M-1 [MEDIUM]: BC-2.01.013 traceability cites wrong HS-107 path

**Affected location:** BC-2.01.013 traceability section

**Finding:** The path cited for HS-107 is `.factory/specs/holdout-scenarios/` which does not exist. The real location is `.factory/holdout-scenarios/`.

**Fix required:** Correct the path to `.factory/holdout-scenarios/HS-107.md`.

### M-2 [MEDIUM]: HS-107 bound to VP-028 (fuzz); SPB has no Kani/proptest VP for byte-exact framing arithmetic

**Affected locations:** HS-107 VP binding, VP-INDEX

**Finding:** HS-107 is bound to VP-028 (cargo-fuzz corpus target). However, HS-107 asserts byte-exact framing arithmetic (captured-len = min of three values, padding alignment) that fuzz testing cannot express as a property assertion. SPB has no Kani proof or proptest VP that verifies the arithmetic invariants.

**Fix required:** Add an SPB captured-len proptest or unit VP that verifies the three-way min and padding arithmetic, OR explicitly document that HS-107 is holdout-only (no VP can express this assertion) and note this in VP-INDEX.

### M-3 [MEDIUM]: Zero-packet one-shot notice fires only when skipped_blocks > 0; silent on valid IDB-only / SHB-only files

**Affected location:** BC-2.01.011 one-shot notice postcondition (added in pass-2 fix I-3)

**Finding:** The one-shot notice AC was written to fire when `skipped_blocks > 0` (i.e., the file had OPB-only blocks). A valid pcapng file consisting of only SHB + IDB (no packet blocks at all) or SHB-only also yields zero packets but `skipped_blocks == 0`. These files silently return empty without any observable notice, violating SOUL #4 (no silent empty returns for valid inputs).

**Fix required:** Broaden the zero-packet one-shot notice to "valid file yielded zero packets" regardless of skip count. The notice MUST fire whenever a structurally valid pcapng file produces zero output packets, not only when blocks were skipped.

### M-4 [MEDIUM]: BC-2.01.014 Invariant 2 over-claims classic-pcap parity for ts_high > 0

**Affected location:** BC-2.01.014 Invariant 2

**Finding:** Invariant 2 claims pcapng timestamp parity with classic-pcap handling for all inputs. Classic-pcap stores raw u32 seconds in the ts_sec field. pcapng with ts_high > 0 undergoes saturation arithmetic (BC-2.01.014 v1.2 saturating multiply). The two behaviors differ for large timestamps.

**Fix required:** Scope the parity claim to ts_high == 0 only. For ts_high > 0, explicitly state that pcapng applies saturation which has no classic-pcap equivalent.

### M-5 [MEDIUM]: No BC owns the happy-path (valid single-section N-packet in-order + payload-fidelity)

**Affected location:** BC set BC-2.01.009..018

**Finding:** No BC specifies the nominal happy path: valid single-section pcapng with N EPBs, all in-order, producing N output packets with correct payload fidelity. The arp-baseline-16pkt.cap fixture appears only as a test-vector line in STORY notes, not as a normative postcondition anchored to a BC.

**Fix required:** Add a postcondition to BC-2.01.009 (or a new BC-2.01.009a) specifying: "given a valid single-section pcapng file with N EPBs, the reader emits exactly N RawPacket structs with payload equal to the captured bytes, in declaration order; anchor: arp-baseline-16pkt.cap (16 packets)."

### M-6 [MEDIUM]: Block OPTIONS TLV walking unspecified — IDB if_tsresol option parse has no bounds-check / no-panic spec

**Affected locations:** BC-2.01.011 (IDB parse), error-taxonomy

**Finding:** IDB `if_tsresol` is an option carried in the IDB options TLV section (code=2, length=2, padded value). The raw parsing path must walk the options TLV to find `if_tsresol` before the fixed-field parse is complete. No BC specifies:
- How to walk options TLV (code:2 + len:2 + padded value, repeated)
- What to do on a malformed option length (e.g., option length > remaining body)
- A no-panic / no-over-read guarantee for this walk

This is a real over-read attack surface: a crafted IDB with `option_length > remaining_body` can produce a slice panic.

**Fix required:** Add to BC-2.01.011: an IDB options-walk postcondition specifying the TLV iteration algorithm, a malformed-option-length case routed to E-INP-008 (body parse failure), and a no-panic / no-over-read AC for the TLV walk.

### M-7 [MEDIUM]: E-INP-001 / E-INP-011 / E-INP-013 precedence undefined at IDB-parse time

**Affected locations:** BC-2.01.016 (linktype whitelist), BC-2.01.011 (interleaved-IDB handling), error-taxonomy

**Finding:** At IDB-parse time, three error conditions may be applicable simultaneously:
- E-INP-013: interleaved-IDB (IDB encountered after first packet block)
- E-INP-001: linktype not in whitelist
- E-INP-011: interface-ID conflict (duplicate IDB with different parameters)

No BC or ADR decision defines the evaluation order when multiple conditions apply. The first condition checked determines which error code the caller sees, which affects testability and holdout scenario authoring.

**Fix required:** Define evaluation order for these three conditions at IDB-parse time. Recommended order: E-INP-013 position-check first (structural); then E-INP-001 whitelist check; then E-INP-011 conflict check. Encode this in BC-2.01.016 or a new ordering postcondition.

---

## LOW / OBSERVATION Findings

### O-1 [LOW]: HS-104 cites BC-2.01.012 PC3/PC4 but EPB interface_id cases are PC5

**Affected location:** HS-104 BC cross-reference

**Finding:** HS-104 scenario description references BC-2.01.012 PC3 and PC4 for the EPB interface_id-out-of-range case. The correct postcondition is PC5 (interface_id OOB → E-INP-009 routing was corrected in pass-1 remediation to PC5). The HS-104 citation is stale.

**Fix required:** Update HS-104 to cite BC-2.01.012 PC5.

### O-2 [LOW]: HS-107 Case A and Case D contain stale pre-correction byte lines

**Affected location:** HS-107 Cases A and D

**Finding:** Cases A and D include byte-sequence lines that predate the BOM and overhead corrections from D-143/D-144. These stale byte values are now inconsistent with ADR-009 rev 5 and BC-2.01.010 v1.7 on-disk values.

**Fix required:** Update HS-107 Cases A and D to match current on-disk canonical byte values from ADR-009 rev 5 and BC-2.01.010 v1.7.

### O-3 [LOW / process-gap]: Stale "taxonomy updated in separate burst" notes for codes that already landed; no validator that forward-referenced codes exist

**Affected locations:** Multiple BCs referencing error codes added in D-142/D-143 bursts

**Finding:** Several BCs contain prose notes of the form "E-INP-013 to be added in a separate taxonomy burst." E-INP-013 has since landed in error-taxonomy v2.8, but the forward-reference notes were not swept. Additionally, there is no automated check that every error code referenced in a BC exists in the error-taxonomy.

**Fix required (process):** Remove stale forward-reference notes from BCs after the referenced code lands. Propose policy: after each taxonomy burst, sweep all BCs for forward-reference notes citing codes that now exist.

### O-4 [informational]: VP-INDEX arithmetic GREEN

**Finding:** VP-INDEX v2.4 arithmetic was verified: VP-025..030 assignments, all BC cross-references, and all story anchors are internally consistent. No action required.

---

## Process-Gap Summary

**C-1 is a changelog-lie class failure.** The BC-2.01.013 v1.2 changelog asserted that PC1 was fixed (three-way min applied). Disk verification shows this was not applied to the normative PC1 and AC-002 text. This is the same class of defect that caused C-4 in pass-2 (stale error-code table). The pattern: a changelog is updated to reflect an intended fix, but the fix is not fully propagated to the BC body.

**Process implication:** Changelog claims MUST be disk-verified before a pass is declared complete. The adversary reading only the changelog would incorrectly conclude PC1 is clean. Verification requires reading the on-disk normative text, not the changelog entry.

---

## Remediation Required Before Pass-4

All CRITICAL and HIGH findings (C-1, H-1, H-2, H-3, H-4, H-5) MUST be addressed before pass-4 dispatch. MEDIUM findings (M-1..M-7) must also be addressed; they individually fall below the HIGH threshold but collectively (7) exceed the <3 MEDIUM clean-pass threshold.

**Clean-pass counter: 0/3. Remediation round-3 required.**
