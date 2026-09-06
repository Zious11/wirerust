---
document_type: verification-delta
feature: feature-s7comm
phase: f2
sub_phase: integrate-step-1-of-3
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
status: draft
consumed_by: spec-steward (F2 INTEGRATE step 2 — VP-INDEX registration)
---

# feature-s7comm F2 INTEGRATE — Verification Property Delta

This document stages the verification-property authoring obligation named by ADR-014
Decision 9 ("VP numbering is explicitly deferred to product-owner at F2 BC/VP
authoring... product-owner should expect to allocate in the VP-048 range"). It follows
the **feature-iec104 F2 precedent**: new Feature-Mode VPs are authored as rich,
self-contained `VP-INDEX.md`-ready table rows (Title/Module/Tool/Phase/Status/Verified
BCs, with a long-form embedded description in the Title cell, mirroring VP-044 through
VP-047's style) rather than as separate `vp-NNN-<slug>.md` files — no
`.factory/specs/verification-properties/vp-044-*.md` through `vp-047-*.md` files exist
on disk or in git history for the IEC-104 precedent, confirming this is the
established Feature-Mode convention, not an omission.

**Scope discipline (per this burst's explicit constraints):** this document contains
proof *obligations* only — properties to be verified and their planned proof method —
no Kani harness code, no proptest strategy code (that is F6 formal-hardening's job).
It does **not** edit `VP-INDEX.md` itself, does **not** bump `VP-INDEX.md`'s version,
and does **not** register index rows — those are spec-steward's F2 INTEGRATE step 2
actions, consuming this document as input. It also does not touch any `input-hash:`
field — that is state-manager's F2 INTEGRATE step 3 action.

**Next free VP number confirmed against VP-INDEX.md:** the highest registered ID is
VP-047 (IEC-104 Parser No-Panic Fuzz). This document allocates new IDs starting at
**VP-048**, consistent with ADR-014 Decision 9's anticipated range.

---

## Part 1 — New Verification Properties (VP-048 through VP-055)

Eight new VPs are allocated (ADR-014 Decision 9 estimated 4-6; this INTEGRATE pass
allocates 8 because the BC set's proof-obligation surface, once fully enumerated
across SS-20's two Kani-P0 header parsers, SS-21's Kani-P0 header-bounds function, four
separate proptest-totality obligations, and the combined-chain fuzz harness ADR-014
Decision 9 also names, does not compress into fewer than 8 without conflating
functions with materially different proof methods or module boundaries — consistent
with the "at least these obligations" framing of the INTEGRATE brief).

### VP-048 — TPKT Header Parse Safety and Four-Way Totality

| Field | Value |
|---|---|
| VP-ID | **VP-048** |
| Title | TPKT Header Parse Safety and Four-Way Totality: `parse_tpkt_header(data: &[u8]) -> Option<TpktHeader>` never panics or reads out-of-bounds for any input; the four outcomes — `data.len() < 4` reject (BC-2.20.001), `data[0] != 0x03` version reject (BC-2.20.002), decoded `length < 4` reject (BC-2.20.003), and the accept path `Some(TpktHeader{version:3, length})` for `length ∈ [4, 65535]` (BC-2.20.004) — are exhaustive and mutually exclusive over all possible `data` inputs; for any `Some(h)` returned, `h.length` decoding cannot overflow `u16`. Pure-core free fn — NOT an impl method (Kani amenability), mirroring `parse_apci_header`/`parse_mbap_header`/`parse_enip_header` precedent (VP-044/VP-022/VP-032). ADR-014 Decision 1 (frozen `TpktHeader` struct), Decision 9 (Kani P0 target, "the smallest, most tractable pure functions in the new surface"). |
| Module | `analyzer/iso_on_tcp.rs` |
| Tool | Kani |
| Phase | P0 |
| Status | draft |
| Verified BCs | BC-2.20.001, BC-2.20.002, BC-2.20.003, BC-2.20.004 |

### VP-049 — COTP Header Parse Safety, TPDU-Type Exhaustiveness, and Protocol-ID Extraction Totality

| Field | Value |
|---|---|
| VP-ID | **VP-049** |
| Title | COTP Header Parse Safety, TPDU-Type Exhaustiveness, and Protocol-ID Extraction Totality: `parse_cotp_header(tpkt_payload: &[u8]) -> Option<CotpHeader>` never panics or reads out-of-bounds for any input, including the LI-truncation bounds check (`tpkt_payload.len() < 1 + tpkt_payload[0] as usize`, BC-2.20.006) where the LI value (max 255 as a `u8`) is used only as a length-comparison bound, never a direct unchecked index. The TPDU-type classification over `tpkt_payload[1]`'s high nibble is exhaustive and non-overlapping across all 16 possible nibble values: CR (`0xE`, BC-2.20.007), CC (`0xD`, BC-2.20.008), DT-with-payload (`0xF` + `len > payload_offset`, BC-2.20.009), DT-empty-payload (`0xF` + `len == payload_offset`, BC-2.20.010), and the 13-remaining-nibble-values reject arm (BC-2.20.011) — every one of the 256 possible `tpkt_payload[1]` byte values maps to exactly one of these five outcomes. Separately, the protocol-ID byte extraction (BC-2.20.009/012) is a total, uninterpreted identity mapping over all 256 `u8` values — `parse_cotp_header` performs zero comparison against `0x32`/`0x72`/any other value (the frozen SS-20→SS-21 module-boundary guarantee). Pure-core free fn, Kani P0 (ADR-014 Decision 9). |
| Module | `analyzer/iso_on_tcp.rs` |
| Tool | Kani |
| Phase | P0 |
| Status | draft |
| Verified BCs | BC-2.20.005, BC-2.20.006, BC-2.20.007, BC-2.20.008, BC-2.20.009, BC-2.20.010, BC-2.20.011, BC-2.20.012 |

### VP-050 — TPKT/COTP Carry-Buffer Residual-Bound Reassembly, Overflow Isolation, and 1-Byte Resync

| Field | Value |
|---|---|
| VP-ID | **VP-050** |
| Title | TPKT/COTP Carry-Buffer Residual-Bound Reassembly, Overflow Isolation, and 1-Byte Resync: (a) walk-first-residual-bound semantics (BC-2.20.013) — the frame-walk loop over `carry[direction] ++ incoming_data` extracts every complete TPKT frame before any byte-count bound is applied to the leftover residual; no aggregate carry-plus-delivery pre-check exists anywhere in the implementation (anti-evasion property, mirrors VP-045/IEC-104 and the DNP3/ENIP carry-isolation precedent, VP-035/VP-033/VP-037); (b) directional isolation — `carry_c2s` and `carry_s2c` are never mixed, and interleaved c2s/s2c delivery produces the same frame_count as the sum of independent same-direction runs; (c) the residual carry is bounded at `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535` (`u16::MAX`, derived from the TPKT length field's own ceiling, BC-2.20.014); on overflow, the offending direction's carry is CLEARED (never truncated/partially retained) and exactly one T0814 finding is emitted per direction via a dedicated dedup flag distinct from the malformed-length dedup flag; (d) the resync walk (BC-2.20.015) advances the cursor by EXACTLY 1 byte per iteration on a bad-version-byte or post-overflow condition — never 2 — guaranteeing termination (strictly increasing cursor over a finite byte sequence) and that no real `0x03` frame start is ever skipped. ADR-014 Decision 8 (WALK-FIRST-RESIDUAL-BOUND, reusing the ADR-007/ADR-013 anti-Ptacek/Newsham-evasion fix). |
| Module | `analyzer/s7comm.rs` (carry buffers live on `S7commFlowState` per ADR-014 Decision 1, not on `iso_on_tcp.rs`) |
| Tool | proptest |
| Phase | P1 |
| Status | draft |
| Verified BCs | BC-2.20.013, BC-2.20.014, BC-2.20.015 |

### VP-051 — S7comm Header Bounds-Before-Slice Safety

| Field | Value |
|---|---|
| VP-ID | **VP-051** |
| Title | S7comm Header Bounds-Before-Slice Safety: `parse_s7comm_header(data: &[u8]) -> Option<S7commHeader>` never panics or reads out-of-bounds for `data.len() < 10` (BC-2.21.004, the classic-S7comm common-header minimum-length guard: Protocol ID + ROSCTR + Reserved(2) + PDU Reference(2) + Parameter Length(2) + Data Length(2) = 10 bytes for Job/AckData/Userdata ROSCTR values); and, separately, the caller-side (`S7commAnalyzer`) bounds obligation (BC-2.21.009) that `data.len() >= header_len + param_length as usize + data_length as usize` MUST be verified BEFORE any parameter-block or data-block slice is attempted — no out-of-bounds slice is ever reachable regardless of adversarial `param_length`/`data_length` values, and `header_len (10 or 12) + param_length (max 65,535) + data_length (max 65,535)` cannot overflow `usize` on any platform wirerust targets (32-bit or 64-bit). This is the ADR-014 Decision 9-designated Kani P0 candidate for SS-21 ("smallest, most tractable pure functions... arithmetic safety"), mirroring `parse_apci_header`'s ASDU minimum-length guard (VP-044) applied to S7comm's two-length-field header. |
| Module | `analyzer/s7comm.rs` |
| Tool | Kani |
| Phase | P0 |
| Status | draft |
| Verified BCs | BC-2.21.004, BC-2.21.009 |

### VP-052 — S7comm Function-Code and Userdata-Group Classification Totality (Including the Load-Bearing 0x03/0x04/0x07 Group Correction)

| Field | Value |
|---|---|
| VP-ID | **VP-052** |
| Title | S7comm Function-Code and Userdata-Group Classification Totality: (a) the Job/Ack_Data function-code match over `data[header_len]` is exhaustive and non-overlapping across all 256 `u8` values — every FC byte maps to exactly one of: named FCs (`0xF0` Setup Communication, `0x04`/`0x05` Read/Write Var, `0x1A`-`0x1C` download triad, `0x1D`-`0x1F` upload triad, `0x28` PLC Control, `0x29` PLC Stop), `Unrecognized(fc)` for any other non-empty-parameter-block byte (BC-2.21.017), or `NoParameterBlock` when `param_length == 0` — the two "no positive classification" outcomes are never conflated; (b) the Userdata function-group match over the low nibble of `data[header_len + 4]` is exhaustive across all 16 possible nibble values: group `0x03` = Block functions (subfunctions `0x01`/`0x02`/`0x03` named, BC-2.21.019 — the LOAD-BEARING correction that group `0x03`, not `0x07`, is Block functions, reversing a common documentation error), group `0x04` = CPU functions, group `0x07` = Time functions (BC-2.21.022, the matched-pair negative-space counterpart confirming `0x07` is NOT Block functions), and `OtherGroup(group, subfn)` for the remaining 13 nibble values (BC-2.21.023, with no invented/unverified group-ID names). This VP's non-vacuity requirement: a harness that merely asserts "every byte classifies to something" is insufficient — the harness MUST specifically assert group `0x03` classifies as `BlockFunctions` and group `0x07` classifies as `TimeFunctions` (not vice versa), since a transposed implementation would otherwise pass a naive totality check while being semantically backwards and silently breaking the T0888 (BC-2.21.038) emission call-site's correctness. |
| Module | `analyzer/s7comm.rs` |
| Tool | proptest |
| Phase | P1 |
| Status | draft |
| Verified BCs | BC-2.21.017, BC-2.21.019, BC-2.21.022, BC-2.21.023 |

### VP-053 — `protocol_id` Four-Way Dispatch Totality and Unclassified Never-Force-Fit

| Field | Value |
|---|---|
| VP-ID | **VP-053** |
| Title | `protocol_id` Four-Way Dispatch Totality and Unclassified Never-Force-Fit: `S7commAnalyzer::on_data`'s branch on `CotpHeader::protocol_id` (BC-2.21.002) is exhaustive and mutually exclusive over every possible `parse_cotp_header` return value: `None` → unclassified-gap (BC-2.21.028); `Some` with `tpdu_type ∈ {ConnectRequest, ConnectConfirm}` → session-tracking only, no classification; `Some` with `tpdu_type: DataTransfer, protocol_id: Some(0x32)` → classic S7comm dissection; `Some(0x72)` → S7comm-plus framing-only path; `Some(other)` for `other ∉ {0x32, 0x72}`, or `protocol_id: None` on an empty-payload DT → unclassified-gap (BC-2.21.027). This is the **load-bearing correctness property ADR-014 names explicitly**: for ALL `other ∉ {0x32, 0x72}` (256 minus 2 = 254 possible byte values, covering IEC 61850 MMS, ICCP/TASE.2, and any unrecognized value), the resulting flow is NEVER attributed to S7comm (`classified_protocol` is set to `Some(S7Protocol::Unclassified)`, a distinct variant from both `Classic` and `Plus`, and the flow's traffic never counts toward S7comm's `Support::Supported` coverage in any report). Also verifies: no proximity-based fallback exists (e.g. `Some(0x73)` is `Unclassified`, never treated as a "probably S7comm-plus" guess); first-classification-wins is applied uniformly across all three outcomes (Classic/Plus/Unclassified) and is sticky for the flow's lifetime. |
| Module | `analyzer/s7comm.rs` |
| Tool | proptest |
| Phase | P0 |
| Status | draft |
| Verified BCs | BC-2.21.002, BC-2.21.027, BC-2.21.028 |

### VP-054 — Program-Download / Upload Structural Disjointness

| Field | Value |
|---|---|
| VP-ID | **VP-054** |
| Title | Program-Download / Upload Structural Disjointness: the Download triad (`0x1A` RequestDownload, `0x1B` DownloadBlock, `0x1C` DownloadEnded — BC-2.21.013) and the Upload triad (`0x1D` StartUpload, `0x1E` Upload, `0x1F` EndUpload — BC-2.21.014) map to eight FC-value-adjacent (`0x1A`-`0x1F`) but semantically and directionally disjoint `S7ClassicFunction` variant sets, with NO shared match arm — the classification match treats `0x1A..=0x1C` and `0x1D..=0x1F` as two independent sub-ranges, never a single collapsed `0x1A..=0x1F` range. For all 6 FC values in `{0x1A,...,0x1F}`, exactly one of the six named variants results, and no Download-triad value is ever classified as, aliased to, or conflated with any Upload-triad value or vice versa. This property is load-bearing for B2's downstream MITRE correctness (T0843/T0889 must never fire from an Upload-classified sequence, since Upload is PLC→station backup traffic, not program deployment) even though B2's sequence-correlation state machine itself is out of this VP's scope (this VP covers only the per-frame FC classification B1 delivers, not the B2 session-correlation layer). |
| Module | `analyzer/s7comm.rs` |
| Tool | proptest |
| Phase | P1 |
| Status | draft |
| Verified BCs | BC-2.21.013, BC-2.21.014 |

### VP-055 — S7comm/ISO-on-TCP Combined Parse-Chain No-Panic Fuzz

| Field | Value |
|---|---|
| VP-ID | **VP-055** |
| Title | S7comm/ISO-on-TCP Combined Parse-Chain No-Panic Fuzz (`fuzz_s7comm_parser`): `S7commAnalyzer::on_data` never panics, unwinds, or reads/writes out-of-bounds on arbitrary byte sequences delivered across arbitrarily many `on_data` calls, exercising the FULL TPKT→COTP→S7comm parse chain (`parse_tpkt_header` → `parse_cotp_header` → `parse_s7comm_header` → FC/Userdata-group classification) as one integrated harness rather than per-function unit proofs; directional carry buffers (`carry_c2s`/`carry_s2c`) remain bounded at `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535` after any input sequence; the frame-walk loop terminates for every input (each iteration either extracts a complete frame, advancing the cursor by at least 4 bytes, or stashes a residual to carry and returns — no infinite loop) — mirrors VP-028 (pcapng) and VP-047 (IEC-104)'s combined no-panic fuzz harness pattern and ADR-014 Decision 9's explicit tool-selection rationale ("cargo-fuzz P1 for the combined TPKT→COTP→S7comm parse chain's no-panic property under arbitrary byte input (mirrors VP-047)"). SOURCE_BC NOTE: per the VP-028/VP-047 module-wide fuzz-VP convention, `source_bc` lists a representative subset of the SS-20/SS-21 parse-chain entry points and boundary BCs rather than every BC in the feature — all SS-20/SS-21 parse paths are covered by `fuzz_s7comm_parser`. |
| Module | `analyzer/iso_on_tcp.rs` + `analyzer/s7comm.rs` (combined) |
| Tool | cargo-fuzz |
| Phase | P1 |
| Status | draft |
| Verified BCs | BC-2.20.001, BC-2.20.005, BC-2.20.013, BC-2.20.014, BC-2.21.004, BC-2.21.009 |

---

## Part 2 — Existing VPs Requiring Amendment (VP-004, VP-007, VP-041)

None of these are new VP allocations. Per ADR-014 Decision 9: "VP-004 and VP-007 are
pre-existing obligations being extended, not new VPs." VP-041's amendment is likewise
an extension of feature-protocol-coverage's existing scope, not a new allocation.
**No VP file or VP-INDEX row is edited by this document** — the exact amendment text
below is handed to spec-steward for registration in F2 INTEGRATE step 2.

### VP-004 amendment — Content-First Dispatch Precedence (add `DispatchTarget::S7comm` Rule 9 arm)

**Current VP-INDEX row:** `VP-004 | Content-First Dispatch Precedence | dispatcher.rs |
Kani | P0 | verified | BC-2.05.001, BC-2.05.002, BC-2.05.003, BC-2.05.004, BC-2.05.005,
BC-2.05.006, BC-2.05.012`

**Amendment required:** extend the Kani `classify_oracle` proof (in
`#[cfg(kani)] mod kani_proofs`) and the `verify_content_first_precedence_exhaustive`
harness to cover the new port-102 `DispatchTarget::S7comm` variant (Rule 9, after
Rule 8/IEC-104; the prior "no match" arm renumbers to Rule 10), per ADR-014 Decision 2's
**six-step atomic obligation** (executed together in one commit, mirroring ADR-013
Decision 9 / ADR-010 Decision 1):

1. Add `DispatchTarget::S7comm` variant to the `DispatchTarget` enum.
2. Add the port-102 arm to `classify()` (Rule 9, after Rule 8 IEC-104).
3. Add the corresponding `DispatchTarget::S7comm` arm to `classify_oracle` in
   `#[cfg(kani)] mod kani_proofs`, mirroring production `classify()` syntactically.
4. Extend the early-exit guard to include `self.s7comm.is_none()`.
5. Add `S7comm` match arms to `on_data` and `on_flow_close`.
6. Re-run `verify_content_first_precedence_exhaustive` and confirm VERIFICATION
   SUCCESSFUL.

**Failure to update `classify_oracle` atomically invalidates the whole VP-004 proof**
(ADR-014 Decision 2) — this is a single, indivisible amendment, not a partial one.

**Proposed amended row (for spec-steward to apply verbatim, source_bc list pending a
new SS-05 dispatcher-level BC — see Outstanding Gap note below):**
`VP-004 | Content-First Dispatch Precedence (extended: port-102 Rule 9,
DispatchTarget::S7comm) | dispatcher.rs | Kani | P0 | draft (pending re-verification) |
BC-2.05.001, BC-2.05.002, BC-2.05.003, BC-2.05.004, BC-2.05.005, BC-2.05.006,
BC-2.05.012, BC-2.20.016, [SS-05 Rule-9 BC — TBD, see Outstanding Gap]`

**Outstanding gap (flagged, not resolved by this document):** BC-2.20.016 Invariant/VP
Anchors notes explicitly that "the cross-subsystem dispatcher BC that part B / the
INTEGRATE sub-burst will author for `DispatchTarget::S7comm` Rule 9, mirroring
BC-2.05.012's IEC-104 precedent" has not yet been authored. Authoring a new SS-05
`BC-2.05.0NN` (Rule 9, port-102 dispatch) is **out of this INTEGRATE step's explicit
task scope** (VP authoring + prose refresh + flagged-item reconciliation only) and is
called out here for the orchestrator/spec-steward to schedule as a follow-up
product-owner action before VP-004's amended row can cite a complete source_bc set.

### VP-007 amendment — MITRE Technique ID Format and Catalog Completeness (seed count 29→32, EMITTED_IDS extension)

**Current VP-INDEX row:** `VP-007 | MITRE Technique ID Format and Catalog Completeness
| mitre.rs | Kani | P0 | verified | BC-2.10.005, BC-2.10.006, BC-2.10.007, BC-2.10.008,
BC-2.10.010`

**Amendment required**, per ADR-014 Decision 5's **six-part atomic obligation**
(executed together in one commit, mirroring ADR-013 Decision 10 / ADR-010's VP-007
decision):

1. Add `"T0843"`, `"T0889"`, `"T0821"` to `SEEDED_TECHNIQUE_IDS` (29 → 32 entries).
2. Bump `SEEDED_TECHNIQUE_ID_COUNT` to 32.
3. Add `technique_info("T0843")`, `technique_info("T0889")`, `technique_info("T0821")`
   arms — the first two returning two NEW `MitreTactic` variants
   (`MitreTactic::IcsLateralMovement`/`TA0109`, `MitreTactic::IcsPersistence`/`TA0110` —
   neither existing variant covers these tactics per ADR-014 Decision 5's live-page
   verification); the third returns the EXISTING `MitreTactic::IcsExecution`.
4. Add `"T0843"`, `"T0889"`, `"T0821"`, and the 8 reused IDs' (T0835, T0836, T0858,
   T0816, T0888, T0846, T0814, T1692.001) S7comm emission call-sites to `EMITTED_IDS`
   (reused IDs already present from Modbus/ENIP need only the new call-site, not a new
   `EMITTED_IDS` entry) — **except** T0816, whose S7comm call-site is RESOLVED to ZERO
   this cycle per BC-2.21.037's finalized INTEGRATE disposition (T0816 remains
   seeded-and-emitted via ENIP only; no S7comm call-site is added).
5. Verify `SEEDED_TECHNIQUE_IDS.len() == SEEDED_TECHNIQUE_ID_COUNT` (VP-007 drift
   guard) — this is the "MITRE drift-guard extension" the INTEGRATE brief names.
6. Verify `technique_info` resolves all SEEDED IDs (VP-007 catalog completeness
   harness), including the two new `MitreTactic` variants' `Display`/`tactic_id()`
   impls and `all_tactics_in_report_order()` membership.

**Proposed amended row:**
`VP-007 | MITRE Technique ID Format and Catalog Completeness (extended: T0843/T0889/
T0821 seeding, SEEDED_TECHNIQUE_ID_COUNT 29→32, IcsLateralMovement/IcsPersistence
MitreTactic variants) | mitre.rs | Kani | P0 | draft (pending re-verification) |
BC-2.10.005, BC-2.10.006, BC-2.10.007, BC-2.10.008, BC-2.10.010, BC-2.21.030,
BC-2.21.031, BC-2.21.032, BC-2.21.041`

BC-2.21.041 is included in the amended source_bc list because it is the negative-space
constraint (T0851/T0873/T0873.001 excluded, T0813 deferred, version pin retained) that
the VP-007 drift guard's "no ID emitted that was never seeded" structural property
covers by construction (BC-2.21.041's own Verification Properties table already states
this explicitly).

### VP-041 amendment — Protocol Coverage Catalog Set-Difference Correctness (Support-enum re-derivation)

**Current VP-INDEX row:** `VP-041 | Protocol Coverage Catalog Set-Difference
Correctness — oracle-cross-check + partition invariant. ... | src/protocols.rs |
proptest | P1 | draft | BC-2.18.003, BC-2.18.004`

**Amendment required:** this amendment is already fully specified in BC-2.18.003 and
BC-2.18.004's own bodies (v1.6/v1.4, feature-s7comm F2 part A) — this section
summarizes it for spec-steward's registration, it does not add new proof content
beyond what those two BC files already commit to:

1. **`proptest_vp041_oracle_cross_check` — re-derive the oracle.** The independently-
   computed oracle predicate changes from `oracle_supported = (any canonical_port in
   SUPPORTED_PORTS) OR entry.name == "ARP"` to `entry.support == Support::Supported`
   (BC-2.18.003 Postcondition 1/Invariant 1). The oracle continues to NOT call
   `supported_protocols()`/`unsupported_protocols()` — non-vacuity is preserved, only
   the independent-computation basis changes from a port-list check to a direct field
   read (still independent of the functions under test, since the oracle reads the
   `support` field directly rather than calling the functions that also read it via a
   `.filter()`).
2. **`proptest_vp041_partition_invariant` — exercise with a `DetectionOnly` entry
   present.** The partition/disjointness harness must be run with S7comm-plus (the
   first `Support::DetectionOnly` entry in the catalog) present in `KNOWN_PROTOCOLS`,
   to concretely exercise the case that regresses if `unsupported_protocols()` is ever
   implemented as `== Support::KnownUnsupported` instead of `!= Support::Supported`
   (BC-2.18.003 Invariant 3 / BC-2.18.004 Invariant 3 — the canonical regression this
   amendment's non-vacuity depends on catching).
3. **New unit-test-level regression guards** (BC-2.18.003 VP table): 
   `test_BC_2_18_003_detection_only_retained_in_unsupported` and
   `test_BC_2_18_003_bacnet_unsupported`, complementing the two proptest harnesses.

**Proposed amended row:**
`VP-041 | Protocol Coverage Catalog Set-Difference Correctness — oracle-cross-check
(re-derived over the Support enum field, ADR-014 Decision 3) + partition invariant
(exercised with a Support::DetectionOnly entry present, S7comm-plus). ... | 
src/protocols.rs | proptest | P1 | draft | BC-2.18.003, BC-2.18.004, BC-2.18.005,
BC-2.18.006`

---

## Part 3 — Final Tally (for spec-steward registration, F2 INTEGRATE step 2)

| Metric | Count | Detail |
|---|---|---|
| New BCs (this feature, cumulative across F2 parts A/B1/B2) | 59 | SS-20: BC-2.20.001-016 (16); SS-21: BC-2.21.001-041 (41); SS-18: BC-2.18.005-006 new (2); SS-18: BC-2.18.003-004 amended in place (not counted as new — pre-existing IDs) |
| New VPs allocated | 8 | VP-048, VP-049, VP-050, VP-051, VP-052, VP-053, VP-054, VP-055 |
| Existing VPs requiring amendment | 3 | VP-004 (dispatch, extended), VP-007 (MITRE catalog, extended), VP-041 (catalog partition, re-derived) |
| New CAPs | 2 | CAP-20 (ISO-on-TCP Framing), CAP-21 (S7comm Analysis) |
| New `MitreTactic` enum variants (via VP-007 amendment) | 2 | `IcsLateralMovement` (TA0109), `IcsPersistence` (TA0110) |
| New seeded MITRE technique IDs | 3 | T0843, T0889, T0821 (`SEEDED_TECHNIQUE_ID_COUNT` 29→32) |
| Flagged items reconciled this burst | 4 | (1) BC-2.21.037 T0816 finalized as gated, zero-call-site-this-cycle; (2) BC-2.21.040 T1692.001 DNP3-style gating confirmed intentional per-protocol divergence from Modbus; (3) BC-2.21.033/039 T0846 Setup-Communication-sweep scope (not SYN-sweep) finalized, plus a stale BC-2.21.038→BC-2.21.039 cross-reference bug fixed in 4 places; (4) BC-2.21.032 Finding.confidence per-finding limitation recorded as pre-existing, not fixed this cycle |
| Stale-prose files refreshed | 2 | `.factory/specs/architecture/ss-18-protocol-coverage-catalog.md` (v1.5→1.6); `.factory/specs/domain/capabilities/cap-18-protocol-coverage-catalog.md` (Key caveats section + BC range) |

**Next steps (not performed by this document):**
- spec-steward (F2 INTEGRATE step 2): register VP-048 through VP-055 in `VP-INDEX.md`;
  apply the VP-004/VP-007/VP-041 amended rows above; bump BC-INDEX/VP-INDEX/PRD/
  ARCH-INDEX versions as needed; register the new CAP-20/CAP-21 entries if not already
  indexed elsewhere.
- state-manager (F2 INTEGRATE step 3): rebaseline `input-hash:` fields on all touched
  files (the two prose files, BC-2.21.033/037/040, BC-2.18.003's frontmatter is
  unchanged by this burst — no edits were made to BC-2.18.003/004/005/006 files
  themselves, only referenced).
- A future product-owner action (not this burst): author the missing SS-05
  dispatcher-level BC for `DispatchTarget::S7comm` Rule 9 (flagged under the VP-004
  amendment above) so VP-004's amended source_bc list is complete.
