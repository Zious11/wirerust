---
document_type: adversarial-spec-review
cycle: feature-pcapng-reader
phase: F2
pass: 5
date: 2026-06-20
spec_state: "ADR-009 rev 7, BC-2.01.009..018, error-taxonomy v3.2, VP-INDEX v2.6, HS-101..108"
verdict: NOT CLEAN
critical: 1
high: 4
medium: 5
low: 3
novelty: HIGH
novelty_class: "partial-fix sibling miss (padding-overrun still on E-INP-010 after body-too-short reclassified); Decision 17 precedence mis-derived in EC-006/EC-008; OPB silent data loss; SPB snaplen asymmetry vs EPB"
clean_pass_counter: 0
threshold_to_pass: "0 CRITICAL, 0 HIGH, <3 MEDIUM"
trajectory: "P1:23 / P2:24 / P3:17 / P4:13 / P5:13 — PLATEAU (persistent 1C+4-5H last 3 passes)"
---

# Adversarial Spec Review — Pass 5

**Cycle:** feature-pcapng-reader
**Pass:** 5 (fresh context)
**Spec state at review:** ADR-009 rev 7, BC-2.01.009..018, error-taxonomy v3.2, VP-INDEX v2.6, HS-101..108
**Verdict:** NOT CLEAN
**Counts:** 1 CRITICAL / 4 HIGH / 5 MEDIUM / 3 LOW
**Novelty:** HIGH — partial-fix sibling miss (padding-overrun left on E-INP-010 after body-too-short reclassified to E-INP-008); Decision 17 precedence mis-derived in BC-2.01.018 EC-006/EC-008; OPB-only silent data-loss (SOUL #4 incomplete fix); SPB snaplen-clamp asymmetry vs EPB; HS-107 VV mis-description + stale deferral notes.
**Trajectory plateau:** P1:23 / P2:24 / P3:17 / P4:13 / P5:13 — plateau at 13 for two consecutive passes with persistent 1C+4-5H pattern.

---

## CRITICAL Findings

### C-1 [CRITICAL]: EPB padding-aware overrun + bound-by-body checks routed to E-INP-010; should be E-INP-008 per Decision 20

**Affected locations:**
- BC-2.01.012 PC6 / AC-002 / EC-010
- error-taxonomy E-INP-010 item (c) (scope list still includes "EPB captured_len + padding overrun" as an E-INP-010 trigger)
- HS-104 Case E
- VP-027 (if it cites E-INP-010 for the captured_len+padding path)

**Root cause:** Pass-4 (D-150) fixed BC-2.01.012 to add the padding-aware bound (`EPB_FIXED + captured_len + pad(captured_len) <= body.len()`) and an unconditional `body.len()` guard. It also reclassified EPB/SPB `body-too-short` failures to E-INP-008 per Decision 20. However, the re-audit (D-151) focused on BC-2.01.011 and error-taxonomy items (d)/(e) (the body-too-short items for EPB<20/SPB<4). It did not sweep the *sibling defect*: the padding-aware overrun check itself (when the body slice overruns due to a non-mult-of-4 `captured_len`) is a wirerust body-decode failure and should route to E-INP-008 under Decision 20's "aligned-framed-body-short → E-INP-008" tier. Yet BC-2.01.012 EC-010 and error-taxonomy item (c) still classify this path as E-INP-010.

**Decision 20 (ADR-009 rev 7) three-tier rule:**
- Tier 1: crate framing fail → E-INP-010
- Tier 2: crate-framed, wirerust body-decode finds body too short → E-INP-008
- Tier 3: semantic fail → E-INP-008

The captured_len+padding overrun fires *after* crate-framing succeeds (the block_total_length is well-formed); wirerust's own guard rejects the body layout. This is Tier 2 → E-INP-008, not Tier 1.

**This is a partial-fix sibling miss identical in structure to P4-C-1 (SPB fix not propagated to EPB).** The D-151 re-audit closed E-INP-010 items (d)/(e) but left item (c) (the padding-overrun trigger) on E-INP-010.

**Fix:**
1. Reclassify BC-2.01.012 EC-010 to E-INP-008 for the captured_len+padding overrun path.
2. Remove padding-overrun from error-taxonomy E-INP-010 item (c); add it to E-INP-008 scope.
3. Update HS-104 Case E error-code assertion to E-INP-008.
4. Verify VP-027 does not cite E-INP-010 for this path.

---

## HIGH Findings

### H-1 [HIGH]: BC-2.01.018 EC-006 and EC-008 contradict Decision 17 IDB-parse precedence

**Affected locations:**
- BC-2.01.018 EC-006 (ETHERNET IDB then IEEE802_11 IDB)
- BC-2.01.018 EC-008 (non-whitelisted first IDB, then ETHERNET second IDB)

**Root cause:** Decision 17 (ADR-009 rev 7) defines IDB-parse precedence as:
1. E-INP-013 (interleaved-IDB position check) — first
2. E-INP-001 (linktype whitelist) — second
3. E-INP-011 (multi-IDB linktype conflict) — third

**EC-006 analysis:** The scenario has two sequential IDBs (ETHERNET then IEEE802_11), both whitelisted. The first IDB is parsed successfully. The second IDB arrives: per precedence step 2, it hits the whitelist check (PASS — IEEE802_11 is whitelisted); per step 3, it hits the conflict check (FAIL — conflicts with ETHERNET first IDB) → E-INP-011. EC-006 correctly states E-INP-011, but the narrative implies the check order is inverted: it says "ETHERNET accepted → IEEE802_11 linktype conflict → E-INP-011" without mentioning the whitelist step at all. While the *outcome* is correct for this specific pair (both whitelisted, so step 2 passes silently), the omission of the whitelist step makes EC-006's prose inconsistent with Decision 17's explicit three-step derivation. An implementer reading EC-006 alone could implement conflict-before-whitelist ordering.

**EC-008 analysis:** The scenario has a non-whitelisted first IDB (linktype SOME_EXOTIC), then a whitelisted ETHERNET second IDB. Per Decision 17 precedence, when SOME_EXOTIC is parsed as the *first* IDB it hits the whitelist check (step 2 at first-IDB-parse time) → E-INP-001 immediately. The file fails at the first IDB. EC-008 claims the outcome is E-INP-011 (conflict), which requires the second IDB to be reached. This is wrong: the first IDB's whitelist rejection (E-INP-001) fires before the second IDB is ever parsed. EC-008 error code and narrative are incorrect per Decision 17.

**Fix:**
1. EC-006: add explicit Decision 17 step derivation (step 2 whitelist-PASS → step 3 conflict-FAIL → E-INP-011); the outcome stays E-INP-011 but the reasoning must be stated.
2. EC-008: reclassify to E-INP-001 (whitelist rejects first IDB before second IDB is read); rewrite narrative to show step-2 fires at first-IDB-parse. Remove E-INP-011 claim entirely.

---

### H-2 [HIGH]: OPB-only file → silent packet-data loss; zero-packet notice does not distinguish packet-bearing skips from non-packet skips (SOUL #4 incomplete)

**Affected locations:**
- BC-2.01.009 (zero-packet one-shot notice rule)
- BC-2.01.015 (block-dispatch skip-arm for obsolete-Packet-Block)
- HS-108 (zero-packet cases — does not include an OPB-only variant)

**Root cause:** Pass-3 (D-147 M-3) broadened the zero-packet notice to fire on "valid file, zero packets regardless of skip count." Pass-4 (D-150 H-4) authored HS-108 covering IDB-only and IDB+skipped-blocks cases. However, neither fix distinguishes between:
- A skip because the block type carries no packet data (NRB, ISB, DSB, SystemdJournal) — user sees zero packets but loses nothing
- A skip because the block is an obsolete Packet Block (OPB/type 0x2) — the OPB *does carry packet data* that wirerust intentionally does not ingest per ADR-009 Decision 8

The current spec and notice text say "valid file, zero packets — notice emitted." An OPB-only file (containing packet data exclusively in OPBs) will produce this notice, but the user cannot tell from the notice that packet data was present and intentionally skipped. This is a SOUL #4 (silent data loss) violation: the user has a file with packets, wirerust returns Ok(zero packets) with a generic zero-packet notice, and the user has no signal that obsolete-Packet-Block data was not ingested.

**Fix:**
1. BC-2.01.015 skip-arm for `Block::ObsoletePacket`: add a dedicated notice/warning stating "obsolete Packet Block(s) detected — packet data in OPBs is not ingested per Decision 8 (ADR-009)."
2. BC-2.01.009: distinguish the OPB data-loss notice from the generic zero-packet notice. The OPB-data-loss notice must be emitted regardless of whether other packet blocks exist.
3. Add an HS-108 Case D (or new HS-109): OPB-only pcapng → Ok(zero packets) + obsolete-block-data notice.

---

### H-3 [HIGH]: SPB silent truncation when block_body_available > snaplen — three-way min discards on-disk bytes governed by advisory snaplen (asymmetry with EPB)

**Affected locations:**
- BC-2.01.013 PC1 / AC-002 (three-way min formula)
- ADR-009 Decision 9 (snaplen enforcement policy)
- VP-031 (SPB captured-len proptest)
- error-taxonomy (no specific finding, but SPB/EPB snaplen parity gap)

**Root cause:** The D-147 C-1 fix established: `captured_len = min(original_len, snaplen, block_body_available)`. However, the snaplen field in an IDB is advisory: it records what the capture program set as a buffer limit, not a hard wire constraint. EPB's `captured_length` field is already the post-capture on-disk byte count; EPB parsing does not involve snaplen at all (EPB reads `captured_length` directly). SPB has no explicit `captured_length` field; the spec computes it from `original_length` and `snaplen`. But clamping to `snaplen` can silently drop on-disk bytes: if the capture tool wrote `original_len=200` bytes of packet data into the SPB body but set `snaplen=100`, the three-way min returns 100 and wirerust reads only half the on-disk packet data.

Decision 9 (ADR-009 rev 7) states: "Neither EPB nor SPB enforces snaplen at the read layer." However BC-2.01.013 PC1 / AC-002 include snaplen as the second operand of the three-way min — which does enforce snaplen for SPB. This is a direct BC↔ADR contradiction, and creates an EPB/SPB asymmetry: EPB ignores snaplen, SPB is clamped by it.

**Fix:**
1. Drop `snaplen` from the SPB captured_len formula: `captured_len = min(original_len, block_body_available)` — matching EPB's approach and Decision 9.
2. Update BC-2.01.013 PC1 and AC-002 to remove the snaplen operand.
3. Update ADR-009 Decision 9 to explicitly state "SPB captured_len = min(original_len, block_body_available); snaplen is not applied at read time for either EPB or SPB."
4. Update VP-031 proptest to use the two-way min.
5. Update HS-107 Case B to remove snaplen from the example computation (the case with `snaplen=100` now resolves as `min(200,100)=100` only because `block_body_available=100`, not because of snaplen).

---

### H-4 [HIGH]: BC-2.01.013 VV table mis-describes HS-107; 4× stale "deferred to a separate burst" notes in BC-2.01.013

**Affected locations:**
- BC-2.01.013 Verification Vectors (VV) table row for HS-107
- BC-2.01.013 (body text) — 4 occurrences of "HS-107 btl=12 holdout deferred to a separate burst"

**Root cause (VV table):** The VV table in BC-2.01.013 describes HS-107 as "real-world no-false-positives scenario." HS-107 is the SPB framing/snaplen/no-IDB holdout — it is a synthetic boundary-condition scenario (truncation, padding, no-IDB error cases), not a real-world no-false-positives scenario. This description is factually wrong and will mislead a holdout evaluator reading the BC.

**Root cause (stale deferral notes):** BC-2.01.013 body contains 4 notes of the form "HS-107 btl=12 holdout deferred to a separate burst." HS-107 was authored in D-144 (Case B added, expanded in D-147/D-148/D-150). Case F (body-too-short) now exists. The deferral notes are stale and create reader confusion about whether HS-107 is AUTHORED or still DEFERRED.

**Fix:**
1. Correct the VV table description for HS-107 to accurately describe its content (SPB framing/truncation/snaplen/no-IDB boundary cases; synthetic; authored as of D-144).
2. Remove all 4 stale "deferred to a separate burst" notes from BC-2.01.013 body (or replace with a stable reference: "see HS-107 for SPB framing holdout — AUTHORED").

---

## MEDIUM Findings

### M-1 [MEDIUM]: BC-2.01.009 Precondition 3 (>=4 bytes) contradicts EC-003 graceful Err — should be postcondition, not precondition

**Affected locations:**
- BC-2.01.009 Precondition 3 ("input stream MUST have >= 4 bytes readable for block-type dispatch")

**Root cause:** BC-2.01.009 lists as Precondition 3 that the input stream has at least 4 bytes available for the block-type dispatch peek. However, EC-003 specifies that a truncated/empty stream is handled gracefully with a return of `Err(E-INP-010)`. If the 4-byte availability were a true precondition (contract obligation of the caller), then EC-003 could never fire — it would be the caller's fault, not wirerust's responsibility to handle. In fact, wirerust is expected to handle truncated inputs from any untrusted source. The 4-byte check is an internal implementation invariant (wirerust verifies it and returns Err) — not an obligation of the caller. Framing it as a Precondition inverts the trust model and may lead an implementer to add an unsafe unwrap instead of a graceful Err.

**Fix:** Remove Precondition 3 from BC-2.01.009. The 4-byte check is correctly represented by EC-003 (the Err path) and does not need to be a caller precondition. If desired, add an Implementation Note stating that the implementation checks for 4-byte availability internally before dispatching.

---

### M-2 [MEDIUM]: ADR Decision 9 says neither EPB nor SPB enforces snaplen; BC-2.01.013+VP-031 mandate snaplen in SPB slice — ADR↔BC contradiction (overlaps H-3)

This finding is the ADR-level statement of H-3's BC↔ADR contradiction. Documenting separately for tracker completeness.

**Affected locations:**
- ADR-009 Decision 9
- BC-2.01.013 PC1 / AC-002
- VP-031

**Fix:** Aligned via H-3 fix (drop snaplen from SPB captured_len formula). Update Decision 9 wording explicitly: "SPB captured_len = min(original_len, block_body_available); snaplen advisory, not enforced at read layer." VP-031 proptest must use two-way min for all generated test cases.

---

### M-3 [MEDIUM]: BC-2.01.014 PC4 µs fast-path `ts_sec = ticks/1_000_000 as u32` lacks .min(u32::MAX) saturation — diverges from general formula at large ts_high

**Affected locations:**
- BC-2.01.014 PC4 (microsecond fast-path timestamp formula)
- VP-025 (Kani totality proof — must cover large ts_high in fast path)

**Root cause:** BC-2.01.014 specifies a fast path for the common µs case: `ts_sec = (ts_high * 1_000_000 + ts_low) / 1_000_000` (approximately `ts_high + ts_low/1_000_000` before integer truncation). The fast path computes `ticks / 1_000_000 as u32`, which wraps on overflow — unlike the general formula which saturates via checked arithmetic. For very large `ts_high` values (where the u64 ticks computation produces a value > u32::MAX), the general formula saturates to u32::MAX while the fast path wraps to a small value. This introduces a divergence that the Kani VP-025 proof cannot detect unless it explicitly covers the large-ts_high case in the fast-path branch.

**Fix:**
1. Add `.min(u32::MAX as u64) as u32` saturation to the fast-path `ts_sec` computation in BC-2.01.014 PC4 (or drop the fast path for ts_sec, using the general saturating form for ts_sec in all paths).
2. Add a large-ts_high canonical test vector in BC-2.01.014 that exercises the fast path at the u32 saturation boundary.
3. Ensure VP-025 Kani harness covers the fast-path branch with large ts_high inputs.

---

### M-4 [MEDIUM]: `from_pcap_reader` BufReader wrap-site unspecified (peek + move-into-PcapReader::new)

**Affected locations:**
- BC-2.01.009 (from_pcap_reader API entry; no AC pinning internal BufReader wrapping)

**Root cause:** BC-2.01.009 specifies that `from_pcap_reader` wraps the caller-supplied `Read` impl in a `BufReader` internally. However, the spec does not pin: (a) at what point the wrap occurs relative to the 4-byte peek (the peek must use the buffered reader, not the raw reader), and (b) the ownership transfer into `PcapReader::new`. If an implementer wraps the reader at a different point (e.g., after the peek) or uses the raw reader for the peek and the buffered reader for subsequent reads, the cursor may be in an inconsistent state.

**Fix:** Add an AC to BC-2.01.009 pinning the internal BufReader wrap as the first operation in `from_pcap_reader` before any peek. Add a regression test asserting correct behavior with an unbuffered `Read` impl (e.g., a cursor that returns data byte-by-byte).

---

### M-5 [MEDIUM]: Zero-packet notice emitted from reader (no filename available) but Decision 19 format requires `<filename>:`; layering violation; format mismatch with BC-2.01.009; classic-pcap empty-file asymmetry

**Affected locations:**
- BC-2.01.009 (zero-packet notice spec — says "wirerust:" prefix)
- ADR-009 Decision 19 (zero-packet notice gating and format)
- BC-2.12.011 / main.rs (directory mode — filename known at dispatch layer)

**Root cause:**
1. **Format mismatch:** BC-2.01.009 specifies the notice prefix as `"wirerust:"` (the program name). Decision 19 (ADR-009 rev 7) requires the format `"notice: <filename>: ..."` (Decision 19 §format). These two specifications contradict each other.
2. **Layering violation:** The notice is emitted from the reader layer (inside `from_pcap_reader` or equivalent), which does not have access to the filename (only a `Read` impl is passed). Decision 19's `<filename>` format requires filename context only available at the `main.rs` dispatch layer. The current spec implicitly places emission in the wrong layer.
3. **Classic-pcap asymmetry:** Classic-pcap empty files (zero packets) currently return `Ok(zero results)` silently. pcapng zero-packet files would emit a notice. This asymmetry may surprise users opening two zero-packet files of different formats and seeing different output.

**Fix:**
1. Decide the notice emission layer: emit from `main.rs` (has filename), not from the reader. The reader returns a typed result (e.g., a `ZeroPackets` variant or a flag in the return value) that `main.rs` interprets and formats.
2. Reconcile the format: pick one of Decision 19's `"notice: <filename>: ..."` or BC-2.01.009's `"wirerust: ..."` and apply consistently across the spec.
3. Document the classic-pcap zero-packet behavior in BC-2.01.009 or BC-2.01.002 and decide if parity is required.

---

## LOW Findings

### L-1 [LOW]: VP-INDEX count propagation (VP-031) unverified this pass

VP-INDEX v2.6 total count (31 VPs) was not independently verified against the on-disk VP rows in this pass. The count was carried from D-147/D-150. Recommend a count sweep before F3 entry.

**Status:** OPEN — informational; does not block remediation.

---

### L-2 [LOW]: BC-2.01.012 / HS-104 dual btl-32 vs body.len() framing

BC-2.01.012 uses both `block_total_length - 32` (a btl-relative form) and `body.len()` (a body-relative form) in adjacent guards. The relationship between `btl - 32` and `body.len()` is not stated; if the crate sets `body.len() = block_total_length - 12` (overhead), then `btl - 32 = body.len() - 20`, which is well-defined. However the BC does not make this equivalence explicit, leaving the dual-framing confusing for an implementer.

**Fix:** Add a brief derivation note stating `body.len() = block_total_length - 12 (EPB overhead = type(4)+btl(4)+iface_id(4)+timestamp_hi(4)+timestamp_lo(4)+captured_len(4)+original_len(4) = 28 total fixed; body.len() = btl - 28 + 16 [?])` — or just pick one framing and use it consistently.

**Status:** OPEN — low priority; cosmetic clarity.

---

### L-3 [LOW]: error-taxonomy next_free changelog trail cosmetic

error-taxonomy v3.2 next_free field correctly shows E-INP-014. The changelog trail for v3.0/v3.1/v3.2 transitions is complete. No normative issue; purely cosmetic trailing whitespace in 2 lines.

**Status:** OPEN — cosmetic; does not block F3.

---

## Process-Gap Observations

### [process-gap] "Deferred to a separate burst" idiom recurs with no burst tracker

BC-2.01.010, BC-2.01.013 (4× occurrences), and possibly others contain "deferred to a separate burst" notes introduced in pass-2 or pass-3 remediation. These notes have no corresponding tracked-deferral entry (no deferral ID, no burst reference, no tracker row). Some have since been resolved (HS-107 Case F now EXISTS) but the notes remain stale (H-4 above). Others may remain open with no visibility.

**Recommendation:** Introduce a `TRACKED-DEFERRAL-NNN` idiom: any deferred item must have a row in the remediation tracker (or a named decision-log entry) before the burst containing the deferral note is committed. The note should reference the tracker ID: "deferred — see TRACKED-DEFERRAL-003."

---

### [process-gap] STORY-128 existence unconfirmed

BC-2.01.018 EC-009 and BC-2.12.011 trace to STORY-128 (per-file isolation, main.rs). No confirmation that STORY-128 has been authored in `.factory/stories/`. If STORY-128 does not exist, these BCs have an unresolvable story reference that will block F3 story decomposition and traceability.

**Recommendation:** Verify STORY-128 exists on-disk before F3 entry. If absent, author it from the BC-2.01.018 EC-009 scope.

---

### [process-gap] arp-baseline-16pkt.cap SHB/IDB params unverified

BC-2.01.012 canonical-vector claim cites `arp-baseline-16pkt.cap` as a valid little-endian pcapng with standard IDB params (if_tsresol default). It is unverified this pass whether `arp-baseline-16pkt.cap` is in fact little-endian and what its `if_tsresol` byte is. If the file is actually a classic-pcap (not pcapng), any BC citing it as a pcapng canonical vector is wrong.

**Recommendation:** Run `file arp-baseline-16pkt.cap` + check magic bytes before F3 to confirm it is pcapng LE.

---

## Convergence Assessment

**Trajectory:** P1:23 / P2:24 / P3:17 / P4:13 / P5:13

This is a **plateau**: two consecutive passes at 13 findings with a persistent 1C+4H core. The plateau pattern indicates the remediation approach is not eliminating the critical/high class systematically. Root causes of the persistent plateau:

1. **Sibling-sweep failures** (C-1 in P4, C-1 in P5): each pass finds the EPB/SPB/SHB sibling that the previous pass's fix did not propagate to.
2. **Derived-from-principle errors** (H-1 in P5 — Decision 17 derivation): Decision 17 is stated correctly in ADR-009 but EC-level examples in the BCs are derived incorrectly from it.
3. **Scope-completeness gaps** (H-2 in P5 — OPB data loss; M-5 — notice layering): these are new-class findings each pass.

**Recommendation for remediation round-5:** Before dispatching pass-6, sweep all EC/Case examples in BC-2.01.009..018 and HS-101..108 against each relevant ADR Decision (especially Decision 17, Decision 19, Decision 20) to derive the expected outcome independently and confirm it matches the BC text. This addresses root causes 1 and 2 above. Root cause 3 requires a completeness-mindset review of all SOUL #4 (silent failure) paths.

**Clean-pass counter: 0/3. Remediation round-5 required before pass-6.**
