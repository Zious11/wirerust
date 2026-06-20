---
document_type: adversarial-spec-review
cycle: feature-pcapng-reader
phase: F2
pass: 4
date: 2026-06-19
spec_state: "ADR-009 rev 6, BC-2.01.009..018, error-taxonomy v3.0, VP-INDEX v2.5, HS-101..107"
verdict: NOT CLEAN
critical: 1
high: 4
medium: 5
low: 3
novelty: HIGH
novelty_class: "EPB/SPB sibling-propagation gap + false-unconstructibility over-correction + VP satisfiability failure"
clean_pass_counter: 0
threshold_to_pass: "0 CRITICAL, 0 HIGH, <3 MEDIUM"
---

# Adversarial Spec Review — Pass 4

**Cycle:** feature-pcapng-reader
**Pass:** 4 (fresh context)
**Spec state at review:** ADR-009 rev 6, BC-2.01.009..018, error-taxonomy v3.0, VP-INDEX v2.5, HS-101..107
**Verdict:** NOT CLEAN
**Counts:** 1 CRITICAL / 4 HIGH / 5 MEDIUM / 3 LOW
**Novelty:** HIGH — EPB/SPB sibling-propagation gap (SPB fix not propagated to EPB); false-unconstructibility over-correction (pass-3 SHB E-INP-008 fix applied on false premise); VP-030 satisfiability failure due to linktype-whitelist short-circuit; zero-packet holdout gap (SOUL #4)

---

## CRITICAL Findings

### C-1 [CRITICAL]: BC-2.01.012 EPB captured_len guard — SPB over-read fix not propagated to EPB sibling

**Affected locations:**
- BC-2.01.012 captured_len guard (EPB)
- HS-104 (EPB holdout — only tests captured_len multiple-of-4; does not test non-mult-of-4 boundary)

**Root cause:** Pass-3 fixed the SPB captured_len guard in BC-2.01.013 (three-way min propagated; D-147 C-1). However BC-2.01.012, the EPB sibling, was not swept. BC-2.01.012 carries an analogous `captured_len <= block_total_length - 32` guard, but this guard has two defects that the SPB fix revealed and did not propagate:

1. The formula `block_total_length - 32` ignores the 4-byte padding term. The EPB body layout is: 20 bytes fixed fields + captured_len bytes + pad(captured_len) bytes + options (where pad rounds up to 4-byte alignment). The guard must include the padding term: `EPB_FIXED + captured_len + pad(captured_len) <= body.len()`. Without this, a captured_len that is not a multiple of 4 can pass the current guard while the padded slice overruns body.
2. The guard does not include the unconditional "bound by body.len() first" clause that D-147 established for the SPB. The slice extraction must first be bounded by `body.len()`, unconditionally, before any other computation — otherwise a crafted EPB with a malformed `block_total_length` can bypass the subtraction guard.

**Impact:** On a malformed EPB where captured_len is not a multiple of 4 and is near-but-below the arithmetic limit, the padded slice index overruns the block body → out-of-bounds slice panic. This is the SPB-class over-read defect re-instantiated in the EPB path. HS-104 only tests captured_len values that are multiples of 4, so the existing holdout cannot catch it.

**Fix required:**
1. BC-2.01.012: Update captured_len guard to `EPB_FIXED + captured_len + pad(captured_len) <= body.len()`; add "bound slice by body.len() first, unconditionally" clause matching BC-2.01.013 PC1 contract.
2. HS-104: Add a Case testing captured_len NOT a multiple of 4 (e.g., captured_len=5 in a body of minimum size), verifying no panic and correct slice boundary.

---

## HIGH Findings

### H-1 [HIGH]: body-decode-truncation error-code routing inconsistent across block types; pass-3 SHB E-INP-008 narrowing based on false "unconstructible" premise

**Affected locations:**
- BC-2.01.010 PC5 / AC-004 / EC-005 (SHB — E-INP-008 narrowed to semantic-only in D-147)
- BC-2.01.011 IDB (E-INP-008 for 12<=btl<20 body-too-short window)
- BC-2.01.012 EPB (routes body-too-short to E-INP-010)
- BC-2.01.013 SPB (routes body-too-short to E-INP-010)
- error-taxonomy v3.0 E-INP-008 scope note

**Root cause:** Pass-3 narrowed SHB E-INP-008 to "semantic failures only (invalid BOM / major!=1)" on the stated premise that "SHB framing truncation is unconstructible — crate rejects btl<12." That premise is false. A pcapng SHB with `block_total_length = 16` (minimum-aligned; 8-byte type+length header + 8 bytes body) gives a body of 4 bytes, which is less than the 16-byte SHB minimum body (byte-order magic 4 + major 2 + minor 2 + section_length 8 = 16). This IS constructible at the framing level (btl=16 is valid framing; the crate does not reject it) but truncated at the body level. A valid framing can deliver a semantically short body. Therefore the claim "crate rejects framing truncation → SHB body-truncation always routes to E-INP-010" is wrong.

Additionally, IDB uses a 12<=btl<20 constructible window for E-INP-008 (body 0-7 bytes) while SPB and EPB route their body-too-short cases to E-INP-010. There is no uniform rule.

**Impact:** A constructible SHB with btl=16 produces a body of 4 bytes (SHB body minimum = 16 bytes). If the implementation follows the pass-3-narrowed spec, it routes this case to E-INP-010 (crate Err) even though the crate may succeed at framing. The error code is wrong, and the test fixture is constructible — meaning there is a gap in the spec's stated test coverage that the adversary cannot verify.

**Fix required:** Establish a uniform body-decode-truncation rule. One coherent option: E-INP-008 fires when the crate delivers a block body that is shorter than the block type's minimum required body size (regardless of whether the frame itself was well-formed). This makes E-INP-008 constructible for SHB (btl=16 → body=4 < 16-byte minimum), consistent with IDB, and removes the false-unconstructibility claim from error-taxonomy v3.0.

---

### H-2 [HIGH]: BC-2.01.009 probe `consume(4)` call breaks invariant and every downstream branch

**Affected locations:**
- BC-2.01.009 (stream-level dispatch, BufRead probe section)

**Root cause:** BC-2.01.009 mentions a probe step that calls `consume(4)` to advance past the block-type bytes after peeking them. However, the dispatch invariant requires byte 0 to remain un-consumed so that the block parser (IDB, EPB, SPB, SHB) can re-read the full block including the type field. Both downstream parser branches (pcap-file crate path and manual-parse path) require the stream cursor to be positioned at byte 0 of the block, not byte 4. Calling `consume(4)` before dispatching would cause every block parse to see the wrong bytes at position 0-3, breaking framing for every block type in the file.

**Impact:** If implemented as specified, the `consume(4)` call corrupts the stream cursor for the first block and every subsequent block. No valid pcapng file would parse correctly.

**Fix required:** Remove all references to `consume(4)` from BC-2.01.009. The probe step must be peek-only via `fill_buf` (no cursor advance). The block type is read again by the block parser from position 0. Clarify that `fill_buf` is side-effect-free on cursor position.

---

### H-3 [HIGH]: VP-030 (multi-IDB agreement) specified over arbitrary u16 sequences but is unsatisfiable given linktype-whitelist short-circuit

**Affected locations:**
- VP-030 in VP-INDEX v2.5
- BC-2.01.016 (linktype whitelist, E-INP-001)
- BC-2.01.018 (multi-IDB agreement check, E-INP-011)
- ADR-009 Decision 17 (IDB-parse precedence: E-INP-013 → E-INP-001 → E-INP-011)

**Root cause:** VP-030 as written specifies a verification property over arbitrary pairs of `u16` linktype values to test multi-IDB agreement (E-INP-011). However, any non-whitelisted linktype value short-circuits at E-INP-001 (step 2 in Decision 17) before reaching the E-INP-011 conflict check (step 3). Therefore, a proptest that generates arbitrary u16 pairs will almost always hit E-INP-001, never reaching E-INP-011, making VP-030 as written unsatisfiable as a verification of multi-IDB agreement.

**Impact:** The VP cannot be fulfilled with arbitrary u16 inputs. A test harness following the spec will conclude that multi-IDB agreement works simply because E-INP-001 fires first, leaving the actual conflict-detection logic (E-INP-011) uncovered.

**Fix required:** Scope VP-030 to whitelisted DataLink values only (the test must generate pairs drawn from the whitelist set). Pin the comparison unit in VP-030 to `DataLink` (typed enum), not `u16`, so that equality comparison is well-defined at the type level. Add a separate coverage requirement for the E-INP-011 conflict path with two distinct whitelisted linktypes.

---

### H-4 [HIGH]: No holdout scenario for zero-packet one-shot notice (SOUL #4 property); valid-vs-error disambiguation absent

**Affected locations:**
- BC-2.01.009 (dispatch level — zero-packet success path)
- BC-2.01.011 (zero-packet one-shot notice, M-3 fix from pass-3)
- HS-101..107 (no HS covers the zero-packet success case with notice)

**Root cause:** Pass-3 M-3 broadened the zero-packet one-shot notice to fire on "valid file, zero packets regardless of skip count" (D-147). However, there is no holdout scenario that exercises this property. HS-107 covers SPB-only malformed scenarios; none of HS-101..107 covers a valid IDB-only or SHB-only pcapng file with zero packet blocks that should produce a success notice. Additionally, BC-2.01.009 does not state a disambiguation rule distinguishing the zero-packet success path (valid file; no EPB/SPB/OPB blocks) from the EPB-before-IDB error path (E-INP-010/009); both can result in zero packets returned, but one is `Ok(iterator)` and the other is `Err(E-INP-NNN)`.

**Impact:** The SOUL #4 observable-notice property is uncovered by any holdout scenario. A regression that silently drops the zero-packet notice cannot be caught by the existing holdout suite. The ambiguity between zero-packet success and zero-packet error also risks mis-implementation.

**Fix required:** Author HS-108 covering the zero-packet success case (valid IDB-only pcapng file with no packet blocks → `Ok(empty iterator)` + one-shot notice). Add a disambiguation rule to BC-2.01.009 distinguishing the zero-packet success path (SHB + IDB + no packet blocks = `Ok` + notice) from the EPB-before-IDB error path (`Err(E-INP-NNN)`).

---

## MEDIUM Findings

### M-1 [MEDIUM]: "crate enforces body-minimum" over-claim in BC-2.01.012 AC-003; may apply to BC-2.01.011 and BC-2.01.013 as well

**Affected locations:**
- BC-2.01.012 AC-003 (EPB body-minimum enforcement)
- BC-2.01.011 and BC-2.01.013 (potential same over-claim)

**Description:** BC-2.01.012 AC-003 (and similar language in sibling BCs) asserts that "the crate enforces the body-minimum" or that enforcement is delegated to the pcap-file crate. On the raw-block path (ADR-009 rev 4 architectural pivot), it is wirerust — not the crate — that performs the body-length check before slice extraction. The crate delivers raw bytes; the guard `EPB_FIXED + captured_len + pad(captured_len) <= body.len()` is wirerust's check, not the crate's. Attributing enforcement to the crate is an over-claim that could cause the implementer to omit the guard, believing it is already handled.

**Fix required:** Correct AC-003 in BC-2.01.012 (and sweep BC-2.01.011, BC-2.01.013) to attribute body-minimum enforcement to wirerust's pre-slice guard, not the crate. Keep the crate reference for framing-level truncation (crate returns Err for block_total_length < type-minimum), which remains accurate.

---

### M-2 [MEDIUM]: if_tsoffset (IDB option code 10) extracted in BC-2.01.011 PC6 but never applied; silent timestamp offset wrongness

**Affected locations:**
- BC-2.01.011 PC6 (if_tsoffset extraction)
- BC-2.01.014 (timestamp computation — no if_tsoffset term)

**Description:** BC-2.01.011 PC6 specifies that the IDB option `if_tsoffset` (code 10) is extracted from the options TLV walk added in pass-3 M-6. However, BC-2.01.014 (timestamp reconstruction) contains no term for `if_tsoffset`. The pcapng spec defines if_tsoffset as an additive correction to the timestamp (offset in seconds). If extracted but never applied, timestamps will be silently wrong by the offset amount for any file that includes this option.

**Fix required:** Either (a) declare if_tsoffset out of scope in BC-2.01.011 (do not extract; document as unsupported with a rationale note in ADR-009), or (b) add if_tsoffset to the BC-2.01.014 timestamp formula and require it in the options-walk postcondition. Do not leave an extracted value silently unused.

---

### M-3 [MEDIUM]: BC-2.01.012 PC8 over-promises: one ARP fixture covers EC-008 and EC-009 boundary fidelity

**Affected locations:**
- BC-2.01.012 PC8 (payload-fidelity postcondition citing arp-baseline-16pkt.cap)

**Description:** BC-2.01.012 PC8 uses the arp-baseline-16pkt.cap fixture to assert payload-fidelity postconditions including EC-008 (captured_len < original_len truncation) and EC-009 (captured_len == original_len full payload). The ARP baseline fixture uses full-capture packets (no truncation). It does not exercise the truncation path (EC-008). A single fixture cannot cover both boundary cases unless the fixture contains both truncated and non-truncated packets. The over-claim leaves EC-008 boundary fidelity without a concrete test vector.

**Fix required:** Scope PC8's claim to match what arp-baseline-16pkt.cap actually tests (full-capture, EC-009 only). Move EC-008 boundary cases to HS-104 where a purpose-built fixture with captured_len < original_len can be crafted.

---

### M-4 [MEDIUM]: BC-2.01.009 PC6 / BC-2.01.015 PC9 cite "ADR Decision 17" for zero-packet notice; Decision 17 is IDB-parse precedence — incorrect anchor

**Affected locations:**
- BC-2.01.009 PC6 (zero-packet notice reference)
- BC-2.01.015 PC9 (if present, same class of reference)
- ADR-009 Decision 17 (IDB-parse precedence: E-INP-013 → E-INP-001 → E-INP-011)

**Description:** BC-2.01.009 PC6 and related BCs cite "ADR Decision 17" as the rationale for the zero-packet one-shot notice behavior. Decision 17 records the IDB-parse error-code precedence order (E-INP-013 → E-INP-001 → E-INP-011). It has nothing to do with zero-packet notice. The zero-packet notice was broadened in pass-3 M-3 (D-147) but no numbered Decision was added to ADR-009 for it. The citation is a mis-anchor.

**Fix required:** Either add a numbered ADR Decision for the zero-packet one-shot notice behavior (correct anchor) or remove the Decision reference and cite the BC directly (BC-2.01.011). Update BC-2.01.009 PC6 and any other BC citing Decision 17 in the context of zero-packet notice.

---

### M-5 [MEDIUM]: Block sequence numbering convention inconsistent (E-INP-012 counts SHB in "#seq"; E-INP-010 and E-INP-013 count "after SHB")

**Affected locations:**
- error-taxonomy v3.0 E-INP-012 (second SHB → E-INP-012; error message uses "#seq within file" counting SHB as block 1)
- error-taxonomy v3.0 E-INP-010 (body-too-short; error message counts "block N after SHB")
- error-taxonomy v3.0 E-INP-013 (interleaved IDB; error message counts "block N after SHB")

**Description:** Error message templates in the taxonomy use two different counting conventions for the `#seq` block position field: E-INP-012 counts the second SHB itself (e.g., "block 5 in file" where SHB is block 1), while E-INP-010 and E-INP-013 count blocks after the SHB (e.g., "block 3 after SHB"). These conventions produce different numeric values for the same physical block, which will confuse users comparing error messages.

**Fix required:** Pin one counting convention across all E-INP-NNN error message templates. Recommended: "block N after SHB" (0-indexed or 1-indexed, stated explicitly) to exclude the SHB itself from the count for packet-context errors. Update E-INP-012 to match or explicitly document that E-INP-012 uses "within file" counting (with SHB as block 1) and that the difference is intentional. Whichever convention is chosen, it must be stated once in the error-taxonomy preamble and consistently applied.

---

## LOW Findings

### L-1 [LOW]: BC-2.01.016 numeric DLT codes in error message need source verification

**Affected locations:**
- BC-2.01.016 PC2 / EC-002 (numeric DLT codes cited in E-INP-001 error message)

**Description:** BC-2.01.016 cites specific numeric DLT codes (e.g., DLT_NULL=0, DLT_EN10MB=1) in the E-INP-001 error message template. These values should be verified against the `pcap-file` crate's `DataLink` enum discriminants and the official IANA/tcpdump.org linktype registry. If the crate uses a different numeric space than the registry, or if the crate's `DataLink::from(u16)` conversion differs, the codes in the error message will be wrong.

**Fix required:** Verify numeric DLT codes against the pcap-file 2.0.0 `DataLink` enum source and the official linktype registry. Correct any discrepancies in BC-2.01.016 and document the source of authority.

---

### L-2 [LOW]: BC-2.01.011 EC-003 unescaped pipe character `0x80 | 0x0A` in Markdown table

**Affected locations:**
- BC-2.01.011 EC-003 (IDB error case for malformed option length)

**Description:** BC-2.01.011 EC-003 contains the expression `0x80 | 0x0A` (bitwise OR to illustrate option code construction) as a bare string in a Markdown table cell. The pipe character `|` is a Markdown table delimiter and will split the cell into two columns when rendered, corrupting the table layout.

**Fix required:** Escape the pipe as `\|` or wrap the expression in a code span (`` `0x80 | 0x0A` ``) so the Markdown table renders correctly.

---

### L-3 [LOW — process-gap]: error-taxonomy `input-hash: N/A` — pcapng error contract outside drift guard

**Affected locations:**
- error-taxonomy frontmatter `input-hash` field

**Description:** The error-taxonomy file carries `input-hash: N/A` (as noted in D-141 O-2 and lessons.md item 3). This means the error taxonomy is not covered by the drift guard (`bin/compute-input-hash`). The error taxonomy is an input file for story files that specify E-INP-NNN error codes. If the taxonomy changes between now and F3 story generation, the story input-hashes will silently diverge from reality, and `bin/compute-input-hash --scan` will not detect the drift.

**Fix required (process-gap):** Before F3 story decomposition begins, run `bin/compute-input-hash --write` to populate the error-taxonomy `input-hash` with a computed 7-char hash. This closes the gap identified in D-141 O-2. No spec change required; this is a tooling-operation obligation for the state-manager or PO pre-F3 checklist.

---

## Summary

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 1 | C-1 |
| HIGH | 4 | H-1, H-2, H-3, H-4 |
| MEDIUM | 5 | M-1, M-2, M-3, M-4, M-5 |
| LOW | 3 | L-1, L-2, L-3 |
| **Total** | **13** | |

**Clean-pass counter: 0/3. Remediation round-4 required before pass-5.**

**Key patterns this pass:**
- **EPB/SPB sibling-propagation gap (C-1):** The SPB over-read fix (D-147) was not propagated to its EPB sibling (BC-2.01.012). This is the same class of error as pass-3 C-1 (changelog without disk-verification), but at the sibling-BC level. Rule: after every leaf-BC fix, sweep all sibling block-type BCs for the same defect class.
- **False-unconstructibility over-correction (H-1):** Pass-3 narrowed SHB E-INP-008 to semantic-only based on a "crate rejects btl<12" premise that is false for SHBs with btl=16 (valid framing, semantically short body). Removing a valid error path on a false premise is a new defect class introduced by the remediation itself.
- **VP satisfiability (H-3):** VP-030 is written over arbitrary u16 inputs but the linktype whitelist short-circuits before the conflict-detection code under test. The VP cannot be satisfied as written.
- **SOUL #4 holdout gap (H-4):** The zero-packet one-shot notice property added in pass-3 has no holdout coverage, and the success/error disambiguation is absent from BC-2.01.009.
