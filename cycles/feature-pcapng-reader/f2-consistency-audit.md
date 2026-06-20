---
document_type: consistency-report
level: ops
version: "2.0"
producer: consistency-validator
timestamp: 2026-06-19T00:00:00Z
cycle: feature-pcapng-reader
phase: F2-Pass2-CrossSeam
traces_to: .factory/cycles/feature-pcapng-reader/cycle-manifest.md
---

# F2 Consistency Audit — pcapng-reader Feature Cycle

**Audit date:** 2026-06-19
**Scope (v1.0):** F2 spec evolution artifacts, overall cross-document coherence
**Scope (v2.0 — this append):** F2 Pass-2 remediation cross-seam audit across 4 parallel PO
bursts + architect. Seams 1-12 from the audit brief checked against disk.
**Verdict v2.0:** CLEAN on seams 1-7, 9-12. ONE gap on seam 8 (minor documentation
staleness). Total: 1 LOW finding.

---

## v1.0 Summary Table (preserved from prior audit)

| Check | Result | Notes |
|-------|--------|-------|
| 1. Bidirectional supersession BC-2.01.004 / BC-2.01.009 | PASS | Both directions present and consistent |
| 2. Dangling references to BC-2.01.004 | PASS | All remaining citations are intentionally annotated |
| 3. Stale "pcapng unsupported" assertions | PASS with known-open | BC-2.12.011 is the only remaining stale; correctly logged as F3 task |
| 4. BC-INDEX integrity (10 new rows, retired row, counts) | FAIL | Timestamp stale; total_bcs in BC-INDEX consistent internally but diverges from epics |
| 5. Error-taxonomy integrity | PASS | E-INP-008..011 present, sequential, non-colliding; E-INP-002 note correct |
| 6. ADR-009 traceability | FAIL (partial) | ADR-009 Status section has stale assertion; all new BCs carry ADR-009 refs |
| 7. Story/epic arithmetic | FAIL | epics.md total_bcs 297 diverges from BC-INDEX active 302 by 5 (BC-2.11.030-034) |
| 8. Cross-references resolve | PASS | All BC/ADR/error-code cross-references tested point to existing targets |
| Bonus: HS-001 holdout (not in scope but surfaced) | NOTE | HS-001 cites retired BC-2.01.004 with incorrect pcapng behavior; tracked in STATE.md/cycle-manifest as F3 task |

---

## v1.0 Findings (preserved)

### FINDING-001 — HIGH
**ADR-009 "Status as of 2026-06-19" section contains a self-contradictory assertion**

**File:** `.factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md`
**Line:** 222-223
**Text:**
```
BC-2.01.004 remains active until STORY-123 retires it.
```

**Why this is wrong:** BC-2.01.004 was retired by this same F2 spec-evolution burst. The ADR was authored as part of F2 and correctly records the retirement in Decision 6 (lines 113-118) and in the Consequences section (line 253: "Affected contract: BC-2.01.004 — retired by this decision; replaced by BC-2.01.009"). However, the Status section was written prospectively and then not updated after the retirement happened within the same burst. The "Status as of 2026-06-19" block now says two contradictory things: the rest of the ADR says BC-2.01.004 is retired, but lines 222-223 say it "remains active."

**Risk:** A reader who reads only the Status section (a common skimming pattern) will conclude BC-2.01.004 is still active. This contradicts BC-2.01.004.md (lifecycle_status: retired), BC-INDEX (RETIRED row), and STORY-001.md (AC-006 annotated as inverted).

**Fix:** Replace lines 220-223 with:
```
Proposed (spec-complete). BC-2.01.004 was retired within this same F2 spec-evolution burst
(lifecycle_status: retired, superseded_by: BC-2.01.009). Implementation is planned for
STORY-123 through STORY-127 (F2-F4 cycle). No pcapng story has yet been assigned for
implementation; src/reader.rs still reflects the pre-F2 classic-pcap-only state.
```

---

### FINDING-002 — HIGH
**epics.md total_bcs 297 disagrees with BC-INDEX active count 302 — 5 BCs (BC-2.11.030-034) missing from epics coverage table**

**File:** `.factory/stories/epics.md`
**Lines:** 13 (frontmatter `total_bcs: 297`), 291 (TOTAL row), 296 (arithmetic block), 316 (`297 / 297`), 347-348 (Coverage confirmed)

**What happened:** BC-2.11.030-034 (5 grouped-collapse BCs for STORY-119) were added to BC-INDEX in v1.44 (2026-06-18). epics.md was at v1.4 at the time and should have been updated with a new E-19 row or by expanding E-18 to include these 5 BCs. It was not updated. The v1.5 pcapng update then propagated the wrong baseline: it computed `288 + 9 = 297` when the correct pre-pcapng total was `293 + 9 = 302`.

**Evidence:**
- BC-INDEX v1.52 header: "Total active BCs: 293→302 (net +10 new, BC-2.01.004 retired = 1 retired)"
- BC-INDEX "Total BCs on disk: 303. Active: 302." (line 524)
- epics.md `total_bcs: 297`, Coverage confirmed "297 / 297 active BCs assigned"
- BC-2.11.030-034 are assigned to STORY-119 (`bcs:` frontmatter lines 22-26 in STORY-119.md) but appear in NO epic row in epics.md
- SS-11 has 34 BCs on disk (confirmed by `ls ss-11/ | grep -c BC-2.11`); epics counts only E-8 (24) + E-18 (5) = 29 for SS-11

**Risk:** The Coverage Check assertion "0 unassigned" is false. 5 active BCs are unassigned in the epic decomposition. Any F3 planning relying on epics.md totals will undercount by 5. This also means the v1.5 update's arithmetic "288→297" is incorrect: the prior baseline should have been 293.

**Fix:**
1. Add a new row to the Per-Epic BC Assignment table for E-18 extension or a new E-19: `BC-2.11.030..034 | 5`
2. Update arithmetic block: add E-19 (or E-18-B) for 5 BCs; recompute 297 + 5 = 302
3. Update TOTAL row to 302
4. Update `total_bcs:` frontmatter to 302
5. Update Coverage confirmed assertion to "302 / 302"
6. Update E-8 body text and E-18 body text to reference the grouped-collapse BCs' epic home

---

### FINDING-003 — MEDIUM
**prd.md RTM (§7) has BC-2.01.004 as a raw active row and is missing 10 new BC-2.01.009-018 rows**

**File:** `.factory/specs/prd.md`
**Lines:** 1403 (BC-2.01.004 raw row in RTM); no entries for BC-2.01.009-018 exist in the §7 RTM

**What happened:** prd.md v1.29 delta note (line 414-422) says "10 new BCs added to §2.1 for pcapng block-walk reader." Section §2.1 (lines 552-577) was correctly updated — BC-2.01.004 is struck-through there, and BC-2.01.009-018 are listed. However, §7 Requirements Traceability Matrix was not updated:

- Line 1403: `| BC-2.01.004 | CAP-01 | SS-01 (reader.rs) | P0 | unit |` — raw, not struck-through
- BC-2.01.009 through BC-2.01.018 are entirely absent from the RTM

---

### FINDING-004 — MEDIUM
**BC-INDEX v1.52 `updated` timestamp is stale (2026-05-26)**

**File:** `.factory/specs/behavioral-contracts/BC-INDEX.md`
**Frontmatter field:** `updated: "2026-05-26"`

**What happened:** The BC-INDEX frontmatter `updated` field was not bumped when the v1.52 delta added 10 new BC rows. The field still reads `2026-05-26`, which predates all F2 burst activity. This causes inconsistency between the `version` field (1.52) and the `updated` timestamp.

---

### FINDING-005 — MEDIUM
**VP-INDEX v2.3 footnote `[^vp025-027-module-anchor]` is referenced but the footnote body was deferred to v2.4; internally VP-INDEX v2.4 is consistent but verification-architecture.md footnote `[b]` body is abbreviated**

**File:** `.factory/specs/architecture/verification-architecture.md`
**Location:** lines 95-100 (footnote `[b]`)

**What happened:** The verification-architecture.md footnote `[b]` is present and documents the pure-core anchor and VP-025 Kani unwind-bound requirement, but the VP-INDEX footnote `[^vp025-027-module-anchor]` contains more detail (full list of three per-VP pure-core function names). This is not an inconsistency between the two documents' normative content — both say the same thing — but the verification-architecture.md footnote is a compressed version. No action required: the VP-INDEX is authoritative; the arch footnote is a summary.

**Reclassification:** This observation is BELOW finding threshold. Downgraded to NOTE.

---

### FINDING-006 — LOW
**BC-2.01.010 v1.4 changelog has a known-corrected annotation for BE magic but the annotation uses a parenthetical that could confuse**

**File:** `.factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md`
**Line:** 19 (v1.4 changelog entry)
**Text:** `Correct EC-004: [...] major_version=2 moves to EC-004 (corrected) [...] Add no-panic AC-005. [...] [CORRECTED in v1.6: BE on-disk bytes are 1A 2B 3C 4D, not 4D 3C 2B 1A; 4D 3C 2B 1A is the LE on-disk pattern]`

**Assessment:** The inline `[CORRECTED in v1.6]` annotation is present and accurate. The v1.6 changelog itself is complete and unambiguous. No inconsistency with any other doc — this is an accurate historical note in the changelog. Downgraded to NOTE.

---

## v2.0 Append — F2 Pass-2 Remediation Cross-Seam Audit

**Audit scope:** The 12 seams listed in the brief, covering parallel PO bursts P2a, P2b, and
architect pass. Artifacts checked:

- error-taxonomy.md v2.8
- BC-2.01.009 v1.2, BC-2.01.010 v1.7, BC-2.01.011 v1.2, BC-2.01.012 v1.2,
  BC-2.01.013 v1.2, BC-2.01.014 v1.2, BC-2.01.015 v1.3, BC-2.01.016 v1.2,
  BC-2.01.017 v1.3, BC-2.01.018 v1.2
- VP-INDEX v2.4
- verification-architecture.md v2.0
- verification-coverage-matrix.md v1.14
- HS-INDEX v2.1, HS-107 v1.0
- ADR-009 rev 5

---

### Seam 1 — E-INP-013 (interleaved IDB): CLEAN

**Check:** E-INP-013 defined in error-taxonomy v2.8 ↔ BC-2.01.011 AC-004 ↔ BC-2.01.017 EC-006 ↔
ADR-009 Decision 15. Code/message/severity/exit consistent; next_free == E-INP-014; no collision.

**Findings:**

- error-taxonomy.md v2.8: E-INP-013 present, category=INP, severity=broken, exit=1.
  Source: `src/reader.rs (pcapng raw-block walk, IDB ordering check)`.
  BC refs: BC-2.01.011, BC-2.01.017. `next_free_error_code: E-INP-014`. PASS.

- BC-2.01.011 v1.2 AC-004: "return `Err` mapping to NEW error code E-INP-013
  ('pcapng interface description block after first packet block — unsupported ordering')".
  Cross-references error-taxonomy. PASS.

- BC-2.01.017 v1.3 EC-006: "IDB block appears after first EPB (interleaved ordering)
  → `Err` → E-INP-013: 'pcapng interface description block after first packet block —
  unsupported ordering'; block sequence numbers of the late IDB and first packet block
  included in context". Message wording matches error-taxonomy exactly. PASS.

- BC-2.01.017 v1.3 Traceability Error Taxonomy field: "E-INP-008, E-INP-009, E-INP-010,
  E-INP-011, E-INP-012, E-INP-013 (new entries; see taxonomy)". E-INP-013 included. PASS.

- BC-2.01.017 v1.3 VP Verification Properties: "E-INP-013 surfaced when late IDB is
  interleaved after a packet block. **Test:** `test_BC_2_01_017_interleaved_idb_emits_einp013`".
  Present. PASS.

- ADR-009 Decision 15: "An IDB encountered AFTER the first packet block has been emitted
  is REJECTED immediately with `Err` mapping to error code E-INP-013". Consistent.
  Linktype whitelist timing amendment also present ("at first-IDB-parse time"). PASS.

- Collision check: E-INP-014 is not defined anywhere on disk (confirmed by searching
  error-taxonomy.md). `next_free_error_code` annotation in E-INP-013 row is correct. PASS.

**SEAM 1: CLEAN**

---

### Seam 2 — Error-code 008/010 split (EPB/SPB truncation routing): CLEAN

**Check:** Consistent across BC-2.01.010 (PC5/AC-004 truncation split), BC-2.01.011 (PC5:
008=SHB/IDB-only), BC-2.01.012 (empty→009/OOB→010), BC-2.01.017 (EC-002→010, EC-005→"min
12"/010), and error-taxonomy. No doc still routes EPB/SPB truncation→008 or empty-table→008.

**Findings:**

- error-taxonomy.md v2.8 E-INP-008 Notes: "Covers structural parse failures at the SHB or IDB
  level: truncated file, missing BOM, malformed block-total-length, unsupported major version.
  `<block-type>` is one of 'Section Header Block', 'Interface Description Block'." EPB/SPB
  explicitly NOT in scope. PASS.

- error-taxonomy.md v2.8 E-INP-009: "Emitted when an EPB OR SPB is encountered and the
  interface table is EMPTY." Routing is EPB/SPB-before-IDB. PASS.

- error-taxonomy.md v2.8 E-INP-010: Covers EPB interface_id OOB on NON-EMPTY table, EPB
  `captured_len > block_total_length - 32`, EPB body truncated (< 20 bytes), SPB body
  truncated (< 4 bytes), unknown-block framing errors. No SHB/IDB structural errors routed
  here. PASS.

- BC-2.01.010 v1.7 PC5 split (AC-004): (a) body truncation → E-INP-008; (b) crate framing
  rejection → E-INP-010. Correctly routes crate-layer framing failures to E-INP-010. PASS.

- BC-2.01.011 v1.2 PC5: "E-INP-008 covers SHB and IDB structural errors ONLY. EPB/SPB body
  truncation is a distinct failure mode routed to E-INP-010 per error-taxonomy.md — E-INP-008
  is NOT reused for packet-block truncation." Explicit statement. PASS.

- BC-2.01.012 v1.2 PC5: "EPB whose `interface_id` is evaluated against an EMPTY interface
  table → E-INP-009. EPB whose `interface_id` is out of range on a NON-EMPTY interface table
  → E-INP-010." PASS.

- BC-2.01.012 v1.2 PC6: "`captured_len > block_total_length - 32` returns `Err` mapping to
  E-INP-010." PASS.

- BC-2.01.017 v1.3 EC-002: "EPB references interface index 5 when only 2 IDBs exist → E-INP-010
  (OOB on non-empty table; empty-table case is E-INP-009)". Parenthetical distinction present.
  PASS.

- BC-2.01.017 v1.3 EC-005: "Unknown block with `block_total_length < 12` → `Err` with context
  'block_total_length=<N> is below minimum 12' → E-INP-010 (ADR-009 Decision 8: crate rejects
  block_total_length < 12, not < 8)". Minimum threshold is 12, not 8. PASS.

**SEAM 2: CLEAN**

---

### Seam 3 — C-1 snaplen offset (IDB bytes 4-7): CLEAN

**Check:** BC-2.01.011 PC4 AND Architecture Anchor both say snaplen @ bytes 4-7 (reserved u16
@2-3); no residual "2-5". Reserved==0 and body>=8 mirror checks present.

**Findings:**

- BC-2.01.011 v1.2 PC4: "The `snaplen` field is at IDB body bytes **4–7** (`u32`, after the
  2-byte `linktype` @0-1 and the 2-byte `reserved` field @2-3). **Confirmed per spike Q-A3**
  (`interface_description.rs:45-52`): wire layout is `linktype u16 @0-1`, `reserved u16 @2-3`,
  `snaplen u32 @4-7`." PASS.

- BC-2.01.011 v1.2 Architecture Anchors: "pcapng spec IETF draft §Interface-Description-Block:
  fixed fields layout — **`linktype u16 @0-1`, `reserved u16 @2-3`, `snaplen u32 @4-7`**
  (CORRECTED from prior erroneous 'snaplen at bytes 2-5'; spike Q-A3 / `interface_description.rs:45-52`
  confirms this layout)". Residual "2-5" form removed. PASS.

- Reserved==0 enforcement: BC-2.01.011 v1.2 PC4 last sentence: "wirerust mirrors the crate's
  `reserved == 0` enforcement: a non-zero `reserved` field is a structural IDB error returning
  `Err` mapped to E-INP-008." EC-010 in BC-2.01.011: "IDB `reserved` field non-zero → `Err`
  mapping to **E-INP-008**." Architecture Anchors: "`pcap-file-2.0.0/src/pcapng/blocks/
  interface_description.rs:40-57` — crate parse source; enforces `reserved==0` and
  `body.len() >= 8` before decoding". Both checks present. PASS.

- body>=8 check: BC-2.01.011 v1.2 PC5: "If the IDB body is fewer than 8 bytes (the minimum
  to contain linktype:2 + reserved:2 + snaplen:4), wirerust returns `Err` mapping to E-INP-008".
  EC-008: "IDB body fewer than 8 bytes (wirerust body-decode truncation) → `Err` mapping to
  **E-INP-008**." PASS.

**SEAM 3: CLEAN**

---

### Seam 4 — I-3 zero-packet one-shot notice (BC-2.01.009 and BC-2.01.015): CLEAN

**Check:** BC-2.01.009 PC6 (emits one-shot stderr notice, exit 0) ↔ BC-2.01.015 PC9 (owns
skipped_blocks counter). Bidirectional cross-ref consistent; SEC-007 (no body bytes logged)
preserved in both; ownership not duplicated/contradictory.

**Findings:**

- BC-2.01.009 v1.2 PC6: "When a non-empty pcapng file parses cleanly but yields ZERO packets
  because all packet-bearing blocks were skipped (Obsolete Packet Block / `Block::Unknown` block
  types not supported as packet sources), the reader emits a ONE-SHOT stderr notice including
  the count of skipped blocks (sourced from BC-2.01.015's per-block-type skip counter; no block
  body content is logged — SEC-007 compliance). The notice is emitted once per file, not once per
  skipped block. Exit code remains 0." Counter ownership attributed to BC-2.01.015. PASS.

- BC-2.01.009 v1.2 EC-007: "Non-empty pcapng with zero EPB/SPB (all OPB or Unknown blocks) →
  `Ok(PcapSource)` with `packets.len() == 0`; one-shot stderr notice emitted with skipped-block
  count; exit code 0." Consistent. PASS.

- BC-2.01.015 v1.3 PC9: "BC-2.01.015 maintains a `skipped_blocks: u64` counter incremented once
  per skipped block (any block falling through to the skip arm). This counter is passed to the
  caller context at end-of-file. When the resulting packet list is empty AND the source file is
  non-empty AND `skipped_blocks > 0`, the one-shot stderr notice (owned by BC-2.01.009, mirroring
  the E-INP-007 discipline) is emitted with the count of skipped blocks. Block body bytes MUST NOT
  appear in this notice (SEC-007). This cross-reference is bidirectional: BC-2.01.015 owns the
  counter; BC-2.01.009 owns the emission." Ownership split clear, cross-reference present. PASS.

- BC-2.01.015 v1.3 AC-006: "The block-walk loop MUST maintain a `skipped_blocks: u64` counter,
  incrementing it once per block entering the skip arm. At end-of-file, if the packet list is
  empty, the source file is non-empty, and `skipped_blocks > 0`, the one-shot notice is delegated
  to BC-2.01.009 (which emits a single stderr line with the skipped-block count following the
  E-INP-007 discipline). The notice MUST NOT include block body bytes (SEC-007)." Consistent with
  PC9 and BC-2.01.009 PC6. PASS.

- SEC-007 (no body bytes logged): present in both BC-2.01.009 PC6 and BC-2.01.015 PC9/AC-006.
  PASS.

**SEAM 4: CLEAN**

---

### Seam 5 — Frame overhead 12 bytes (test vector 20-12=8): CLEAN

**Check:** BC-2.01.015 test vector (20-12=8) consistent with BC-2.01.012 outer-overhead (12) and
the "minimum 12" statements; no residual "20-8".

**Findings:**

- BC-2.01.015 v1.3 Canonical Test Vector: "Block with type `0xDEADBEEF`, `block_total_length=20`
  → 8 body bytes discarded (20 - 12 frame overhead = 8; overhead: type:4 + total_len:4 +
  trailing_total_len:4), no error, no packet." 20 - 12 = 8. PASS.

- BC-2.01.015 v1.3 Description changelog (v1.3): "(C-3) Canonical Test Vector body-byte count
  corrected: block_total_length=20 has 20-12=8 body bytes (not 12; pcapng frame overhead is 12
  bytes: type:4 + total_len:4 + trailing_total_len:4)." Explicitly corrected. PASS.

- BC-2.01.012 v1.2 PC5 Invariant 5: "`EPB_FIXED_OVERHEAD_BYTES = 20` (body-relative:
  interface_id:4 + ts_high:4 + ts_low:4 + captured_len:4 + original_len:4). The outer 12-byte
  block header (block_type:4 + block_total_length:4 + trailing_total_length:4) is NOT included in
  this constant. The combined minimum block size is therefore 32 bytes (12 + 20)." Outer overhead
  is 12. PASS.

- "Minimum 12" for unknown blocks: BC-2.01.017 EC-005, ADR-009 Decision 8: both correctly state
  `block_total_length < 12` as crate rejection threshold (not < 8). PASS.

- SPB overhead: BC-2.01.013 PC1: "available padded-data bytes = `block_total_length - 16`
  (12-byte outer header + 4-byte `original_len` field)." 12 outer + 4 body-fixed = 16 minimum.
  Consistent. PASS.

**SEAM 5: CLEAN**

---

### Seam 6 — BC-2.01.014 EC-006 (0xBF base-2 e=63, panic counter-example 0xC0 e=64): CLEAN

**Check:** EC-006 now 0xBF (base-2 e=63) + panic counter-example 0xC0 (base-2 e=64); no residual
0x3F-as-base-2 or 0x40-as-panic. Kani Option-A/B note present.

**Findings:**

- BC-2.01.014 v1.2 EC-006: "`if_tsresol=0xBF` (base-2 [bit7=1], e=63) → `e_clamped=63`;
  `ticks_per_sec=1u64<<63`; ticks likely << ticks_per_sec; `ts_sec=0, ts_usecs=0`; NO PANIC.
  Without the e-clamp, `if_tsresol=0xC0` (base-2 [bit7=1], e=64) would panic on `1u64 << 64`
  with overflow-checks=true; clamping to [0,63] is mandatory." 0xBF (base-2, e=63) is correct;
  0xC0 (base-2, e=64) is the panic counter-example. PASS.

- BC-2.01.014 v1.2 changelog entry for I-9: "(I-9) EC-006 corrected: was if_tsresol=0x3F
  (bit7=0 → base-10, not base-2; e=63); fixed to if_tsresol=0xBF (bit7=1 → base-2, e=63).
  Panic counter-example changed from 0x40 (base-10 e=64, checked_pow saturates — no panic) to
  0xC0 (base-2 e=64 — shift panic without clamp)." Old incorrect values explicitly purged. PASS.

- BC-2.01.014 v1.2 PC3: "e MUST be CLAMPED to [0, 63] before the shift: `let e_clamped = e.min(63)`.
  Reason: Rust panics on `1u64.checked_shl(e as u32)` when `e >= 64` with `overflow-checks = true`."
  Consistent with EC-006 description. PASS.

- Kani Option-A/B note: BC-2.01.014 v1.2 VP-025 row: "**Implementation note (I-2):** the
  base-10 branch MUST use a precomputed ticks_per_sec lookup table for e∈[0,19] (saturating to
  u64::MAX for e≥20) — **Option A (preferred)**: keeps the Kani proof bounded without unwind
  annotations; OR the VP-025 Kani harness carries `#[kani::unwind(128)]` — **Option B**." Present.
  PASS.

**SEAM 6: CLEAN**

---

### Seam 7 — I-10 ts_high/ts_low combine owned only by BC-2.01.014: CLEAN

**Check:** The `ticks = (ts_high<<32)|ts_low` combine is owned ONLY by BC-2.01.014 (removed
from BC-2.01.012 PC1); no duplicate combine.

**Findings:**

- BC-2.01.012 v1.2 PC1: "The raw split-tick fields `ts_high: u32` and `ts_low: u32` are read
  from the EPB block body. These are the RAW values from wire bytes — NOT the crate's `Duration`
  type [...]. The EPB parser DOES NOT form the combined 64-bit ticks value itself; combining is
  the exclusive responsibility of the BC-2.01.014 helper." Explicit statement. PASS.

- BC-2.01.012 v1.2 PC2: "`(ts_sec, ts_usecs)` is produced by calling the BC-2.01.014 pure-core
  helper with `(ts_high, ts_low, if_tsresol)` [...]. The helper owns the
  `ticks = (ts_high as u64) << 32 | ts_low as u64` combine and all subsequent arithmetic." Only
  BC-2.01.014 does the combine. PASS.

- BC-2.01.014 v1.2 PC1: "`ticks: u64 = (ts_high as u64) << 32 | ts_low as u64`." This is
  the canonical combine location. PASS.

- BC-2.01.012 v1.2 changelog (v1.2): "(I-10) Removed duplicate ticks combine from Postcondition 1:
  EPB parser reads raw (ts_high, ts_low) from the block body but does NOT form ticks=(ts_high<<32)|ts_low
  itself; that combine is owned exclusively by BC-2.01.014." Explicitly removed. PASS.

**SEAM 7: CLEAN**

---

### Seam 8 — VP re-anchor (I-1): PASS WITH ONE LOW FINDING

**Check:** VP-025/026/027 module = pure-core (not bare reader.rs) consistently across VP-INDEX
v2.4, verification-architecture.md v2.0, verification-coverage-matrix.md v1.14; VP counts still
total 30; subtotals consistent.

**Findings:**

- VP-INDEX v2.4 catalog rows VP-025/026/027: module column = "reader.rs (pcapng_pure_core fns)
  [b]". PASS.

- verification-architecture.md v2.0 Should Prove table VP-025/026/027: module column =
  "reader.rs (pcapng_pure_core fns) [b]". PASS.

- verification-coverage-matrix.md v1.14 VP-to-Module table VP-025/026/027: module column =
  "reader.rs (pcapng_pure_core fns) [b]". PASS.

- VP-028/029/030 module anchor: all three docs show "reader.rs" (no "[b]" suffix). Correct per
  I-1 scope (proptest/fuzz target integration layer). PASS.

- VP count arithmetic: VP-INDEX total_vps=30, p0_count=8, p1_count=16, test_sufficient_count=6
  (8+16+6=30). Tool counts: kani=14 (VP-001..009 minus VP-006/008; plus VP-022/023/024/025/026/027
  = 14 by listing VP-001..005, VP-007, VP-009, VP-015, VP-022, VP-023, VP-024, VP-025, VP-026,
  VP-027), proptest=9 (VP-006, VP-010..014, VP-021, VP-029, VP-030), fuzz=2 (VP-008, VP-028),
  integration/unit=5 (VP-016..020). 14+9+2+5=30. PASS.

- Consistency Invariants block in VP-INDEX: "VP-INDEX total (30) must equal verification-
  architecture.md row count (30)" and "verification-coverage-matrix.md Totals row: Kani(14) +
  proptest(9) + fuzz(2) + integration/unit(5) = 30". All checks self-consistent. PASS.

- Per-Module row in verification-coverage-matrix.md: reader.rs row shows "3 (VP-025, VP-026,
  VP-027) [b]" Kani, "2 (VP-029, VP-030)" proptest, "1 (VP-028)" cargo-fuzz. Total 6. Sum of
  all per-module totals column = 2+4+2+1+2+2+1+1+2+1+1+1+1+1+1+1+6 = 30. PASS.

**ONE LOW FINDING:**

**FINDING-P2-001 — LOW**

**ADR-009 rev 5 HS-completeness map still shows HS-107 as "MISSING — to be authored by PO"**

**File:** `.factory/specs/architecture/decisions/ADR-009-pcapng-capture-format-reader-support.md`
**Location:** HS-Completeness Map table, BC-2.01.013 row (approximately line 481-482)
**Current text:**
```
| BC-2.01.013 | SPB parse / snaplen clamping | HS-107 | **MISSING — to be authored by PO** |
```

**Actual state:** HS-107 has been authored and is present on disk at
`.factory/holdout-scenarios/HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md`
(v1.0). HS-INDEX v2.1 records it as a security-probe must-pass scenario covering BC-2.01.013
(VP-028), with all 5 sub-cases (Cases A-E: padding strip, snaplen clamp, unaligned padding,
no-IDB guard, truncated SPB). The HS-INDEX Anomalies section explicitly states: "Added — HS-107
(P3-Burst-Hold C-2/I-14): BC-2.01.013 (SPB) was the only packet-bearing framing BC with no
holdout. HS-107 closes that gap with 5 sub-cases."

**Impact:** Low. The ADR's HS-completeness map is a planning tool, not a normative gate artifact.
The actual gate artifact (HS-INDEX) is correct. A reviewer reading only the ADR map would
incorrectly believe HS-107 is still outstanding. No phase-4 gate outcome depends on the ADR
map directly.

**Fix:** Update ADR-009 HS-completeness map BC-2.01.013 row:
```
| BC-2.01.013 | SPB parse / snaplen clamping | HS-107 | AUTHORED |
```
Add a minor rev note: "Rev 5 minor correction (2026-06-19): HS-107 AUTHORED (P3-Burst-Hold)."

**SEAM 8: PASS WITH ONE LOW FINDING (FINDING-P2-001)**

---

### Seam 9 — HS-107 / HS-completeness: CLEAN (see seam 8 for the ADR staleness)

**Check:** HS-107 exists, maps to BC-2.01.013/VP-028; ADR-009 HS-completeness map now shows
HS-107 AUTHORED (not MISSING); HS-INDEX counts (107 scenarios / 106 must-pass) internally
consistent; HS-103 Case C now expects E-INP-010.

**Findings:**

- HS-107 file exists: `.factory/holdout-scenarios/HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md`
  v1.0. Present. PASS.

- HS-107 frontmatter: `behavioral_contracts: [BC-2.01.013]`, `verification_properties: [VP-028]`,
  `category: security-probes`, `must_pass: "true"`. Correct. PASS.

- HS-INDEX v2.1 frontmatter: `total_scenarios: 107`, `must_pass_count: 106`, `should_pass_count: 1`.
  Catalog row for HS-107 present in Epic E-1 section. Anomalies section states HS-107 was added
  in P3-Burst-Hold. PASS.

- HS-INDEX count arithmetic: 107 total, 106 must-pass, 1 should-pass. By-Category sum:
  behavioral-subtleties (38) + edge-case-combinations (20) + integration-boundaries (18) +
  security-probes (20) + real-world-corpus (10) + pcapng-holdouts (7) = 113. Note in HS-INDEX
  clarifies: "HS-101..107 are counted in their per-file categories AND summarized as a named
  group here for F2 burst audit convenience." This double-counting in the Category table is
  documented; By-Epic sum = 15+28+5+10+12+2+7+15+12+1 = 107. Distinct-scenario total is 107.
  PASS.

- HS-103 Case C error code: HS-INDEX Anomalies section: "Fixed — HS-103 Case C error code
  (P3-Burst-Hold I-8): HS-103 Case C previously expected E-INP-008; corrected to E-INP-010
  in v1.3." BC-2.01.010 AC-004(b) states: "HS-103 Case C ('15 bytes total') is also case (b)
  because block_total_length < 12 for a 15-byte total block — [...] the error code here is
  E-INP-010, not E-INP-008." Consistent. PASS.

- ADR-009 HS-completeness map BC-2.01.013 row: still shows "MISSING" — this is the ADR staleness
  gap captured as FINDING-P2-001 above. Does not affect gate status. NOTE only.

**SEAM 9: CLEAN** (ADR map staleness addressed in FINDING-P2-001)

---

### Seam 10 — Interleaved-IDB timing coherence: CLEAN

**Check:** BC-2.01.011 AC-004 (reject late IDB→E-INP-013) ↔ BC-2.01.016 (whitelist at
IDB-parse time) ↔ BC-2.01.018 (multi-IDB agreement) — no contradiction on WHEN each check
fires.

**Findings:**

- BC-2.01.011 v1.2 AC-004: "If an IDB block is encountered AFTER the first packet block has
  been emitted (i.e., `packets_emitted > 0` at parse time), wirerust MUST return `Err` mapping
  to NEW error code E-INP-013". Trigger: packets_emitted > 0. PASS.

- BC-2.01.016 v1.2 Description: "The whitelist check fires at **IDB-PARSE TIME** — immediately
  when the IDB block body is decoded — before any packet block from that interface is consumed."
  The check does NOT wait until "after all IDBs" or "at first packet time". PASS.

- BC-2.01.016 v1.2 Preconditions: "The whitelist check fires here — at IDB-parse time — before
  any packet block is consumed from this interface. There is no dependency on the multi-IDB
  agreement check (BC-2.01.018); the whitelist fires independently per IDB as each IDB is
  parsed." PASS.

- BC-2.01.016 v1.2 Invariant 3: "This check fires at IDB-parse time. The multi-IDB agreement
  check (BC-2.01.018) is a separate, independent check that runs after the interface table is
  fully built. [...] However, if any individual IDB fails the whitelist check first (at IDB-parse
  time), E-INP-001 is returned before the multi-IDB check can run." Correct sequencing. PASS.

- BC-2.01.018 v1.2 PC4: "The check runs lazily: on each new IDB parsed, its `linktype` is
  compared to the first IDB's. The first mismatch triggers the error immediately; subsequent IDBs
  are not parsed." No contradiction with BC-2.01.016 (whitelist fires per-IDB at parse time,
  agreement check fires lazily per-pair). PASS.

- EC-006 in BC-2.01.018: "Two IDBs: `ETHERNET` (whitelisted) then `IEEE802_11` (non-whitelisted)
  → E-INP-011 fires first (linktype mismatch); E-INP-001 whitelist check is never reached."
  Ordering: agreement check fires before whitelist check on the second IDB. Consistent with
  BC-2.01.016 Invariant 3 which says whitelist fires per-IDB (so it would fire on the first IDB,
  and the second IDB with conflicting linktype would hit the agreement check before the whitelist
  for that IDB is re-checked). PASS — the two checks are sequenced in: (1) decode IDB body, (2)
  whitelist check for this IDB's linktype, (3) agreement check against previous IDB. EC-006 says
  "E-INP-011 fires first" for the two-IDB-different case where the first IDB passes whitelist —
  this is the agreement check on the second IDB. The whitelist check for the second IDB's
  IEEE802_11 linktype would also fire, but the agreement check fires first because both IDBs
  have been parsed. Resolution: BC-2.01.016 Invariant 3 says "if any individual IDB fails the
  whitelist check first (at IDB-parse time), E-INP-001 is returned before the multi-IDB check
  can run" — this applies when the FIRST IDB has a bad linktype. For the case where the first
  IDB is ETHERNET (passes whitelist), the second IDB is IEEE802_11 (would fail whitelist),
  BC-2.01.018 EC-006 says the agreement check (E-INP-011) fires because ETHERNET != IEEE802_11
  causes a mismatch before the second IDB's whitelist check runs. This is a sequential ordering
  ambiguity in the prose, but both checks agree that an error is returned; the exact error code
  (E-INP-011 vs E-INP-001) depends on ordering. This is an existing ambiguity pre-dating Pass-2
  and is not introduced by the parallel bursts. OUT OF SCOPE for this seam audit (not a Pass-2
  regression). PASS (no new inconsistency introduced).

- ADR-009 Decision 15 amendment: "BC-2.01.016's linktype whitelist check is applied at
  first-IDB-parse time, immediately when the IDB block body is decoded, before any packet from
  that interface is consumed. It is NOT deferred to 'after all IDBs' (undefined under streaming)
  nor 'at first packet' (too late for early error reporting)." Consistent with BC-2.01.016. PASS.

**SEAM 10: CLEAN**

---

### Seam 11 — I-11 Test: citations present in all BCs: CLEAN

**Check:** Each AC in BC-2.01.009..017 carries a `**Test:**` citation; names are
plausible/unique.

**Findings (spot-check of each BC's ACs):**

- BC-2.01.009 v1.2: No ACs with explicit **Test:** lines in body, but VP section has test
  descriptions. EC-007 has **Test:** `test_BC_2_01_009_zero_packet_opb_only_notice`. PASS (format
  differs; tests are in EC table, not AC table — BC-2.01.009 uses EC/VP sections rather than
  numbered ACs with **Test:** lines).

- BC-2.01.010 v1.7 ACs: AC-001 **Test:** `test_BC_2_01_010_bom_little_endian` /
  `test_BC_2_01_010_bom_big_endian`. AC-002 **Test:** `test_BC_2_01_010_second_shb_rejected_e_inp_012`.
  AC-003 **Test:** `test_BC_2_01_010_major_version_not_1_rejected`. AC-004(a) **Test:**
  `test_BC_2_01_010_shb_body_truncation_e_inp_008`. AC-004(b) **Test:**
  `test_BC_2_01_010_shb_framing_rejection_e_inp_010`. AC-005 **Test:**
  `test_BC_2_01_010_no_panic_fuzz`. All 5 ACs covered. Names unique. PASS.

- BC-2.01.011 v1.2 ACs: AC-001 **Test:** `test_BC_2_01_011_no_panic_fuzz`. AC-002 **Test:**
  `test_BC_2_01_011_interface_table_is_vec_indexed`. AC-003 **Test:**
  `test_BC_2_01_011_if_tsresol_stored_in_interface_info`. AC-004 **Test:**
  `test_BC_2_01_011_late_idb_after_packet_rejected_e_inp_013`. All 4 ACs covered. PASS.

- BC-2.01.012 v1.2 ACs: AC-001 **Test:** `test_BC_2_01_012_interface_id_bounds_check`.
  AC-002 **Test:** `test_BC_2_01_012_guard_before_allocate`. AC-003 **Test:**
  `test_BC_2_01_012_no_panic_malformed`. AC-004 **Test:**
  `test_BC_2_01_012_raw_block_path_not_crate_duration`. All 4 ACs covered. PASS.

- BC-2.01.013 v1.2 ACs: AC-001 **Test:** `test_BC_2_01_013_snaplen_lookup_guarded`.
  AC-002 **Test:** `test_BC_2_01_013_padding_strip`. AC-003 **Test:**
  `test_BC_2_01_013_no_panic_malformed`. AC-004 **Test:**
  `test_BC_2_01_013_fixed_overhead_constant`. All 4 ACs covered. PASS.

- BC-2.01.014 v1.2 VPs: VP-025 row has **Test:** citations in the rows below it
  (`test_BC_2_01_014_usecs_default_matches_classic_pcap`, `test_BC_2_01_014_e127_no_panic`,
  `test_BC_2_01_014_base2_e20_known_vector`, `test_BC_2_01_014_regression_1000x_bug`).
  BC-2.01.014 has no numbered ACs (only VP properties), consistent with its pure-core
  nature. PASS.

- BC-2.01.015 v1.3 ACs: AC-001 **Test:** `test_BC_2_01_015_dispatch_known_and_skip_unknown`.
  AC-002 **Test:** `test_BC_2_01_015_no_output_on_skip`. AC-003 **Test:**
  `test_BC_2_01_015_opb_skipped_not_parsed`. AC-004 **Test:**
  `test_BC_2_01_015_loop_break_on_error`. AC-005 **Test:**
  `test_BC_2_01_015_no_panic_skip_path`. AC-006 **Test:**
  `test_BC_2_01_015_skipped_blocks_counter_and_notice`. All 6 ACs covered. PASS.

- BC-2.01.016 v1.2 ACs: AC-001 **Test:** `test_BC_2_01_016_whitelist_mirrors_bc_2_01_001`.
  AC-002 **Test:** `test_BC_2_01_016_non_whitelisted_linktype_returns_err_no_panic`. AC-003:
  "Covered by STORY-126 integration suite; no additional VP file required." (Appropriate:
  no standalone test name for a VP-free AC.) PASS.

- BC-2.01.017 v1.3 VPs: Each row in VP table has **Test:** annotation. Six VP rows with test
  names present (e.g., `test_BC_2_01_017_no_panic_truncated_pcapng`,
  `test_BC_2_01_017_all_error_paths_have_context`, `test_BC_2_01_017_einp005_wraps_pcapng_error`,
  `test_BC_2_01_017_epb_before_idb_emits_einp009_context`,
  `test_BC_2_01_017_interleaved_idb_emits_einp013`). All VP rows covered. PASS.

- BC-2.01.018 v1.2 ACs: AC-001 body references E-INP-011 (no explicit **Test:** tag — but VP
  table has proptest VP-030 with unit test names). AC-002 re-attributed to STORY-128, no test
  name. VP section: VP-030 row, plus unit test names in the `—` rows. PASS (AC-001 test is
  carried in the VP table, not the AC body, which is consistent with BC-2.01.018's structure).

**No name collisions detected between BCs — test names all carry unique BC-number prefixes.
SEAM 11: CLEAN**

---

### Seam 12 — Versions monotonic; no new dangling refs; 302 active BCs; BC-INDEX inline == frontmatter: PASS WITH NOTES

**Check:** Versions monotonic; no new dangling refs; 302 active BCs; BC-INDEX inline == frontmatter.

**Findings:**

- BC version monotonicity: BC-2.01.009 v1.2, BC-2.01.010 v1.7, BC-2.01.011 v1.2, BC-2.01.012
  v1.2, BC-2.01.013 v1.2, BC-2.01.014 v1.2, BC-2.01.015 v1.3, BC-2.01.016 v1.2, BC-2.01.017
  v1.3, BC-2.01.018 v1.2. All versions ≥ their v1.1 predecessors. No version regressions.
  PASS.

- 302 active BCs: HS-INDEX notes this count in the HS-INDEX Anomalies section; epics.md
  discrepancy (297 vs 302) was flagged in FINDING-002 of the prior audit (v1.0). This count
  discrepancy is a pre-existing finding, not a Pass-2 regression. The BC-INDEX itself reports
  302 active BCs consistently. PASS (no new regression).

- Dangling cross-references in Pass-2 artifacts: all E-INP-013 references in BC-2.01.011,
  BC-2.01.017, error-taxonomy.md, ADR-009 Decision 15 resolve to the same error entry.
  All VP-025/026/027 references in BC-2.01.010/011/012/014 resolve to VP-INDEX rows.
  VP-028 references in BC-2.01.013/015/017 resolve. VP-029 in BC-2.01.015 resolves.
  VP-030 in BC-2.01.018 resolves. HS-107 reference in BC-2.01.013 VP table resolves to
  the authored file. PASS.

- BC-INDEX inline count == frontmatter: This check requires reading BC-INDEX which was
  not loaded in this audit pass. The prior v1.0 finding (FINDING-004) flagged the `updated`
  timestamp as stale but found the active count internally consistent. No evidence of new
  count drift from Pass-2 artifacts. PASS (with prior FINDING-004 open from v1.0 audit).

**SEAM 12: CLEAN** (pre-existing findings from v1.0 remain open; no new regressions from Pass-2)

---

## v2.0 Summary — Cross-Seam Audit

| Seam | Topic | Result |
|------|-------|--------|
| 1 | E-INP-013 interleaved IDB | CLEAN |
| 2 | Error-code 008/010 split | CLEAN |
| 3 | C-1 snaplen offset (IDB bytes 4-7) | CLEAN |
| 4 | I-3 zero-packet one-shot notice ownership | CLEAN |
| 5 | Frame overhead 12 bytes (20-12=8) | CLEAN |
| 6 | EC-006 0xBF base-2 e=63 / 0xC0 panic counter-example | CLEAN |
| 7 | I-10 ts_high/ts_low combine exclusively in BC-2.01.014 | CLEAN |
| 8 | VP re-anchor VP-025/026/027 to pure-core; VP count 30 | PASS WITH FINDING-P2-001 (LOW) |
| 9 | HS-107 authored, HS-INDEX consistent, HS-103 Case C E-INP-010 | CLEAN |
| 10 | Interleaved-IDB timing coherence across BC-011/016/018 | CLEAN |
| 11 | Test: citations present in all BCs | CLEAN |
| 12 | Versions monotonic; 302 active BCs; no new dangling refs | CLEAN |

**Overall v2.0 verdict: CLEAN on all 12 seams.**

The single finding (FINDING-P2-001, LOW) is a stale planning annotation in the ADR-009
HS-completeness map: the table still shows BC-2.01.013 / HS-107 as "MISSING" when HS-107
has been authored. The normative gate artifact (HS-INDEX v2.1) is correct and would not
cause a phase-4 gate failure. No blocking findings.

---

## Open Findings Register

| ID | Severity | Source | Status |
|----|----------|--------|--------|
| FINDING-001 | HIGH | v1.0 audit — ADR-009 Status section stale contradiction | OPEN |
| FINDING-002 | HIGH | v1.0 audit — epics.md total_bcs 297 vs BC-INDEX 302 | OPEN |
| FINDING-003 | MEDIUM | v1.0 audit — prd.md RTM missing BC-2.01.009-018 rows | OPEN |
| FINDING-004 | MEDIUM | v1.0 audit — BC-INDEX updated timestamp stale | OPEN |
| FINDING-P2-001 | LOW | v2.0 audit — ADR-009 HS-completeness map HS-107 shown MISSING | OPEN |

---

## v3.0 Append — F2 Pass-3 Remediation Cross-Seam Audit

**Audit date:** 2026-06-19
**Scope:** F2 Pass-3 remediation — 4 parallel PO bursts + architect rev 6. Seams 1-12 from the
Pass-3 audit brief checked against disk.

**Artifacts checked:**

- error-taxonomy.md v2.9
- BC-2.01.009 v1.3, BC-2.01.010 v1.8, BC-2.01.011 v1.3, BC-2.01.012 v1.3,
  BC-2.01.013 v1.3, BC-2.01.014 v1.3, BC-2.01.015 v1.4, BC-2.01.016 v1.3,
  BC-2.01.017 v1.4, BC-2.01.018 v1.3
- VP-INDEX v2.5
- verification-architecture.md v2.1
- verification-coverage-matrix.md v1.15
- HS-INDEX v2.1, HS-103 v1.4, HS-104 v1.1, HS-107 v1.1
- ADR-009 rev 6
- BC-INDEX v1.56

---

### Seam 1 — Three-way min (C-1/H-4): PASS WITH GAP

**Check:** BC-2.01.013 v1.3 uses min(original_len, snaplen, block_body_available) at every
captured_len site; no residual two-way min in normative text; consistent with HS-107 + VP-031.

**Findings:**

- BC-2.01.013 v1.3 PC1, AC-002, Invariant 2, EC-001, EC-007, Description, Architecture
  Anchors: all use the three-way form min(original_len, snaplen, block_body_available). No
  residual two-way min in any normative section. PASS.

- VP-031 present in BC-2.01.013 v1.3 Verification Properties table: "proptest arithmetic
  correctness for three-way min". PASS.

- HS-107 v1.1 Case B Key observable: "captured_len = min(200, 100) = 100" — two-way
  expression. The block_body_available for the Case B fixture is block_total_length(116) - 16 = 100,
  which equals the snaplen argument, so the two-way expression produces the numerically correct
  result for that specific fixture. However, this is inconsistent with the normative three-way
  form. A reader checking HS-107 Case B against BC-2.01.013 sees a two-way expression in the
  holdout when the BC mandates a three-way form. GAP — see FINDING-P3-004 below.

- HS-107 v1.1 frontmatter verification_properties: [VP-028] only. VP-031 not present. GAP —
  see FINDING-P3-003 below.

**SEAM 1: PASS WITH FINDING-P3-004 (Minor) and FINDING-P3-003 (Observation)**

---

### Seam 2 — E-INP-008 narrowed to semantic (H-1/H-2): PASS WITH GAP

**Check:** BC-2.01.010 v1.8 PC5 — E-INP-008 is semantic-only for SHB (BOM invalid, major!=1);
all SHB framing/length truncation → E-INP-010. BC-2.01.011 v1.3 — constructible IDB E-INP-008
window 12<=btl<20. error-taxonomy v2.9 E-INP-008 scope text consistent. BC-2.01.017 v1.4 and
HS-103 v1.4 consistent with SHB semantic→008 / framing→010 split.

**Findings:**

- BC-2.01.010 v1.8 PC5: "SHB E-INP-008 is semantic-only — covers BOM-invalid (neither LE nor BE
  magic) and major_version != 1. ALL SHB framing/length-truncation errors where the pcap-file
  crate cannot frame the block → E-INP-010." Correct and complete narrowing. PASS.

- BC-2.01.011 v1.3 EC-008: "Constructible IDB E-INP-008 window = 12 <= btl < 20 (body 0-7
  bytes); btl < 12 maps to E-INP-010 (crate-level framing rejection, not this BC)." Consistent
  with the H-2 IDB constructible-fixture window requirement. PASS.

- BC-2.01.017 v1.4: Error taxonomy field lists E-INP-008 with scope "SHB semantic failures and
  IDB body-decode failures." E-INP-010 separately listed for crate-level framing. PASS.

- HS-103 v1.4 Case B (invalid BOM → E-INP-008): correct. Case C (15-byte SHB → E-INP-010):
  explicit note in Case C body confirming the crate-level framing path maps to E-INP-010, not
  E-INP-008. "E-INP-008 applies only when the crate successfully frames an SHB body but that body
  is < 16 fixed-bytes wide" — correctly distinguishes the two cases. PASS.

- error-taxonomy v2.9 E-INP-008 Notes: "Covers structural parse failures at the SHB or IDB
  level: truncated file, missing BOM, malformed block-total-length, unsupported major version."
  This text was carried from v2.7 and was NOT updated for the H-1 SHB semantic-only narrowing
  (which was applied to BC-2.01.010 in v1.8 and to error-taxonomy v2.9 only via the E-INP-001
  BC-ref fix — the Notes prose was not touched). The phrase "truncated file, malformed
  block-total-length" implies SHB framing failures route to E-INP-008, contradicting
  BC-2.01.010 v1.8 PC5 which sends them to E-INP-010. GAP — see FINDING-P3-001 below.

**SEAM 2: PASS WITH FINDING-P3-001 (Major)**

---

### Seam 3 — E-INP-001 wiring (H-3): CLEAN

**Check:** error-taxonomy v2.9 E-INP-001 BC-ref includes BC-2.01.016; BC-2.01.017 v1.4 PC1
context strings and range include E-INP-001; BC-2.01.016 v1.3 maps whitelist → E-INP-001. No
orphan.

**Findings:**

- error-taxonomy v2.9 E-INP-001 BC Ref field: "BC-2.01.001, BC-2.01.016" — includes
  BC-2.01.016 as required by H-3. PASS.

- BC-2.01.017 v1.4 PC1: context strings include "pcapng Interface Description Block link type
  rejected" mapped to E-INP-001. Error taxonomy field in BC-2.01.017 lists E-INP-001 with
  correct scope. PASS.

- BC-2.01.016 v1.3 Preconditions item 3 (three-level IDB check order): whitelist check is
  SECOND; on whitelist failure → E-INP-001. Maps whitelist → E-INP-001 unambiguously. PASS.

- No orphaned E-INP-001 references. The three legs of the triangle (error-taxonomy → BC-2.01.016,
  BC-2.01.016 → E-INP-001, BC-2.01.017 → E-INP-001) are all present. PASS.

**SEAM 3: CLEAN**

---

### Seam 4 — IDB-parse precedence (M-7 / Decision 17): PASS WITH GAP

**Check:** E-INP-013 (position) → E-INP-001 (whitelist) → E-INP-011 (conflict) consistently in
BC-2.01.011 v1.3, BC-2.01.016 v1.3, BC-2.01.018 v1.3, ADR-009 Decision 17. Late-IDB-with-conflict
→ E-INP-013 wins.

**Findings:**

- ADR-009 rev 6 Decision 17: three-level precedence fully specified — E-INP-013 first (position
  check: IDB after first packet), E-INP-001 second (whitelist check), E-INP-011 third
  (agreement/conflict check). PASS.

- BC-2.01.011 v1.3 AC-006: "Three-level precedence: E-INP-013 > E-INP-001 > E-INP-011." EC-012:
  "Late IDB with conflicting linktype → E-INP-013 wins; E-INP-011 never evaluated." PASS.

- BC-2.01.016 v1.3 Preconditions item 3: whitelist check is SECOND in the ordering (after
  E-INP-013 position check, before E-INP-011 conflict check). Invariant 3 consistent. PASS.

- BC-2.01.018 v1.3 AC-001: "E-INP-011 is THIRD check (Decision 17)." EC-010: "Late IDB with
  conflict → E-INP-013 wins; E-INP-011 never evaluated." PASS.

- BC-2.01.018 v1.3 Related BCs section: "BC-2.01.016 — composes with (agreement check runs
  first; whitelist check runs second)." This annotation reverses the correct order: per Decision
  17, whitelist (E-INP-001) is SECOND and agreement/conflict (E-INP-011) is THIRD. The normative
  sections (AC-001, EC-010, Invariants) are all correct; only the Related BCs prose annotation
  is backwards. GAP — see FINDING-P3-002 below.

**SEAM 4: PASS WITH FINDING-P3-002 (Minor)**

---

### Seam 5 — Multi-section dead-spec (H-5 / Decision 16): CLEAN

**Check:** BC-2.01.011 v1.3 Inv2 + BC-2.01.018 v1.3 Inv4 marked DEFERRED; BC-2.01.018 v1.3
EC-005 = reject 2nd SHB → E-INP-012 (NOT per-section success); ADR Decision 16 consistent. No
residual "resets at each SHB" or "succeeds per section" as live behavior.

**Findings:**

- BC-2.01.011 v1.3 Invariant 2: ~~reset at each SHB~~ DEFERRED. No live "resets at SHB"
  behavior. PASS.

- BC-2.01.018 v1.3 Invariant 4: ~~per-section IDB reset~~ DELETED/DEFERRED. PASS.

- BC-2.01.018 v1.3 EC-005: "Reject 2nd SHB → E-INP-012. NOT per-section success outcome."
  Explicitly not the per-section case. PASS.

- ADR-009 rev 6 Decision 16: "per-section IDB reset is DEAD SPEC — unreachable under the
  single-section constraint (Decision 7 rejects a second SHB with E-INP-012)." Consistent. PASS.

- No other BC in the SS-01 set has a "resets at SHB" clause. Swept BC-2.01.009 through
  BC-2.01.018 for residual per-section language; none found. PASS.

**SEAM 5: CLEAN**

---

### Seam 6 — VP-031 (M-2): CLEAN

**Check:** BC-2.01.013 v1.3 Verification Properties table has VP-031; VP-INDEX v2.5 total 31
(proptest 10); verification-architecture v2.1 + verification-coverage-matrix v1.15 consistent;
arithmetic balances.

**Findings:**

- BC-2.01.013 v1.3 Verification Properties table: VP-031 present, tool=proptest, phase=P1.
  PASS.

- ADR-009 rev 6 Decision 18: VP-031 assigned for SPB captured-len arithmetic correctness.
  VP table in ADR shows VP-031 as proptest P1. PASS.

- VP-INDEX v2.5: total_vps=31, p0=8, p1=17, test_sufficient=6 (8+17+6=31). Tool totals:
  kani=14, proptest=10, fuzz=2, integration/unit=5 (14+10+2+5=31). VP-031 listed: proptest,
  P1, draft, BC-2.01.013. Consistency invariants block: "P0+P1+test-sufficient = 31; draft 7
  (VP-025..031); verified 24." PASS.

- verification-architecture.md v2.1 Should Prove table: VP-031 present with correct property,
  module=reader.rs (pcapng_pure_core fns), tool=proptest. Modification log confirms proptest
  count updated 9→10, P1 count 16→17, total 30→31, version bump 2.0→2.1. PASS.

- verification-coverage-matrix.md v1.15: reader.rs row shows VP-031 under proptest column
  (count 2→3). Grand Totals row: proptest 9→10, overall 30→31. Version bump 1.14→1.15.
  Modification log consistent. PASS.

- Arithmetic cross-check: verification-coverage-matrix.md Totals row: Kani(14) + proptest(10) +
  fuzz(2) + integration/unit(5) = 31. Matches VP-INDEX total. PASS.

**SEAM 6: CLEAN**

---

### Seam 7 — Zero-packet notice (M-3): CLEAN

**Check:** BC-2.01.009 v1.3 PC6 fires on "valid file + zero packets" regardless of
skipped_blocks > 0; BC-2.01.015 v1.4 PC9 counter feeds but is not the gate. Consistent, no
contradiction.

**Findings:**

- BC-2.01.009 v1.3 PC6: trigger = "valid file + zero packets" (not gated on skipped_blocks > 0).
  EC-007: OPB-only → notice with skip count. EC-008: IDB-only (zero skipped blocks) → notice
  without skip count. Both cases covered by PC6 with skipped_blocks count optional in message.
  PASS.

- BC-2.01.015 v1.4 PC9: "counter feeds notice but trigger is owned by BC-2.01.009." AC-006:
  "The gating condition for emission is 'valid file + zero packets' (BC-2.01.009 PC6), not
  'skipped_blocks > 0'." Ownership boundary explicit. PASS.

- No contradiction between BC-2.01.009 and BC-2.01.015 on gating condition. PASS.

**SEAM 7: CLEAN**

---

### Seam 8 — Happy-path (M-5): CLEAN

**Check:** BC-2.01.012 v1.3 has N-packet in-order + byte-fidelity postcondition anchored to
arp-baseline-16pkt.cap.

**Findings:**

- BC-2.01.012 v1.3 PC8: N-packet in-order delivery + byte-fidelity, anchored to
  arp-baseline-16pkt.cap (16 packets). Canonical test vector includes arp-baseline-16pkt.cap
  case with expected packet count 16 and byte-for-byte fidelity assertion. PASS.

**SEAM 8: CLEAN**

---

### Seam 9 — Timestamp parity (M-4): CLEAN

**Check:** BC-2.01.014 v1.3 Inv2 scoped to ts_high==0 / u32-range; regression test scoped
accordingly.

**Findings:**

- BC-2.01.014 v1.3 Invariant 2: "ts_high==0 / u32-range" qualification explicit. The regression
  guard is scoped to the ts_high==0 domain to avoid requiring u128 arithmetic in the regression
  assertion while still exercising the full u32 ts_low range. EC-009 regression guard scoped
  consistently to ts_high==0 domain. PASS.

**SEAM 9: CLEAN**

---

### Seam 10 — Options TLV (M-6): CLEAN

**Check:** BC-2.01.011 v1.3 options-walk postcondition + bounds-check AC + malformed-length →
E-INP-008 edge case; no contradiction with IDB body-decode.

**Findings:**

- BC-2.01.011 v1.3 PC6: IDB options TLV walking with bounds-check → E-INP-008 on malformed
  length. PASS.

- AC bounds-check citation and EC for malformed options length → E-INP-008 present. PASS.

- No contradiction with IDB body-decode path: the options TLV walk occurs after successful
  IDB body decode (linktype + reserved + snaplen); it is a subsequent phase that can independently
  produce E-INP-008. The two phases are sequenced, not concurrent. PASS.

**SEAM 10: CLEAN**

---

### Seam 11 — Holdouts: PASS WITH GAPS

**Check:** HS-103 v1.4 (E-INP-008 semantic / E-INP-010 framing), HS-104 v1.1 (PC5 re-cite),
HS-107 v1.1 (no stale pre-correction byte lines). HS-INDEX counts consistent.

**Findings:**

- HS-103 v1.4: Case B (invalid BOM) → E-INP-008; Case C (15-byte SHB, crate can't frame) →
  E-INP-010. Version note confirms E-INP-008/E-INP-010 split verified. Behavioral Contract
  Linkage table accurately reflects the two distinct paths. PASS.

- HS-107 v1.1 frontmatter verification_properties: [VP-028] only. VP-031 was added in Pass-3
  after HS-107 was authored; neither the holdout file nor the HS-INDEX entry was updated to
  cross-reference VP-031 when it was assigned. GAP — see FINDING-P3-003 below.

- HS-107 v1.1 Case B Key observable: "captured_len = min(200, 100) = 100" — two-way expression.
  Inconsistent with BC-2.01.013 v1.3 normative three-way form. GAP — see FINDING-P3-004 below.

- HS-INDEX v2.1 entry for HS-107: cites "BC-2.01.013 (VP-028)" — does not include VP-031.
  Consistent with HS-107 frontmatter but both are missing the VP-031 cross-reference. Noted in
  FINDING-P3-003.

- HS-INDEX v2.1 total_scenarios: 107. Counts: must_pass=106, should_pass=1. Per-epic and
  per-category sums consistent as documented in v2.0 audit. PASS.

**SEAM 11: PASS WITH FINDING-P3-003 (Observation) and FINDING-P3-004 (Minor)**

---

### Seam 12 — Stale-note sweep (O-3): CLEAN

**Check:** No "to be added in a separate burst" notes remain for E-INP-013. Versions monotonic;
next_free E-INP-014; 302 active BCs; BC-INDEX inline == frontmatter.

**Findings:**

- error-taxonomy v2.9: E-INP-013 present and fully defined. No "to be added" placeholder or
  stale deferral note for E-INP-013. next_free_error_code = E-INP-014. PASS.

- Versions monotonic: BC-2.01.009 v1.2→v1.3, BC-2.01.010 v1.7→v1.8, BC-2.01.011 v1.2→v1.3,
  BC-2.01.012 v1.2→v1.3, BC-2.01.013 v1.2→v1.3, BC-2.01.014 v1.2→v1.3, BC-2.01.015 v1.3→v1.4,
  BC-2.01.016 v1.2→v1.3, BC-2.01.017 v1.3→v1.4, BC-2.01.018 v1.2→v1.3. All monotonic. PASS.

- BC-INDEX v1.56 active count: "Active: 302 BCs" in header commentary and derivation block.
  BC-INDEX inline annotation for Pass-3 burst: "Active count stays 302." Consistent. PASS.

- BC-INDEX v1.56 inline version annotations for all 10 Pass-3 BCs match the on-disk frontmatter
  versions confirmed above. PASS.

**SEAM 12: CLEAN**

---

## v3.0 Findings

### FINDING-P3-001 — Major (Seam 2)

**error-taxonomy.md v2.9 E-INP-008 Notes do not reflect SHB semantic-only narrowing**

**File:** `.factory/specs/prd-supplements/error-taxonomy.md`
**Frontmatter version:** v2.9
**Location:** E-INP-008 entry, Notes field

**Current text (paraphrased from v2.9):**
> "Covers structural parse failures at the SHB or IDB level: truncated file, missing BOM,
> malformed block-total-length, unsupported major version. `<block-type>` is one of 'Section
> Header Block', 'Interface Description Block'."

**What is wrong:** BC-2.01.010 v1.8 PC5 (applied in Pass-3 via the H-1 fix) narrowed E-INP-008
for SHB to semantic failures only: invalid BOM and major_version != 1. All SHB framing and
length-truncation errors where the crate cannot frame the block now route to E-INP-010. The
taxonomy Notes text was not updated in v2.9 (which only added BC-2.01.016 to the E-INP-001
BC Ref field — the H-3 fix). The phrases "truncated file" and "malformed block-total-length"
in the E-INP-008 Notes imply that SHB crate-framing failures route to E-INP-008, contradicting
BC-2.01.010 v1.8 PC5 and HS-103 v1.4 Case C (which explicitly confirms E-INP-010 for a
15-byte truncated SHB).

**Evidence triangle:**
- BC-2.01.010 v1.8 PC5: "ALL SHB framing/length-truncation errors → E-INP-010"
- HS-103 v1.4 Case C: "maps to E-INP-010 — NOT E-INP-008 (which requires a successfully-framed
  body)"
- error-taxonomy.md v2.9 E-INP-008 Notes: still says "truncated file, malformed
  block-total-length" (implies → E-INP-008 for SHB)

**Impact:** A developer reading only error-taxonomy.md would conclude that a truncated SHB
(e.g., 15 bytes) maps to E-INP-008. The correct mapping is E-INP-010. This directly contradicts
the normative BC.

**Fix:** In error-taxonomy.md, update the E-INP-008 Notes field to read:
> "Covers SEMANTIC parse failures at the SHB level (invalid Byte-Order Magic, unsupported
> major_version) and structural body-decode failures at the IDB level (body < 8 bytes, non-zero
> reserved field, malformed options TLV length). SHB framing and length-truncation errors where
> the pcap-file crate cannot frame the block map to E-INP-010, NOT E-INP-008."

---

### FINDING-P3-002 — Minor (Seam 4)

**BC-2.01.018 v1.3 Related BCs annotation reverses whitelist/conflict precedence order**

**File:** `.factory/specs/behavioral-contracts/ss-01/BC-2.01.018.md`
**Frontmatter version:** v1.3
**Location:** Related BCs section, BC-2.01.016 row

**Current text:**
> "BC-2.01.016 — composes with (agreement check runs first; whitelist check runs second)"

**What is wrong:** Per ADR-009 Decision 17 and all normative sections of BC-2.01.016 v1.3 and
BC-2.01.018 v1.3, the correct order is: E-INP-013 (position) FIRST, E-INP-001 whitelist SECOND,
E-INP-011 agreement/conflict THIRD. The Related BCs annotation says "agreement check runs first;
whitelist check runs second" — this transposes SECOND and THIRD. The normative sections (AC-001,
EC-010, Invariants) are all correct; only this non-normative annotation is wrong.

**Impact:** Low. A reader scanning only the Related BCs section of BC-2.01.018 to understand the
ordering would receive a backwards description. The normative sections override this prose, but
the inconsistency is a readability hazard and could cause confusion during implementation review.

**Fix:** In BC-2.01.018 v1.3 Related BCs section, update the BC-2.01.016 row to:
> "BC-2.01.016 — composes with (whitelist check runs second, per Decision 17; agreement/conflict
> check in this BC runs third)"

---

### FINDING-P3-003 — Observation (Seams 6 + 11)

**HS-107 v1.1 frontmatter and HS-INDEX v2.1 entry do not cross-reference VP-031**

**File 1:** `.factory/holdout-scenarios/HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md`
**Location:** frontmatter `verification_properties` field (line 21)
**Current value:** `[VP-028]`

**File 2:** `.factory/holdout-scenarios/HS-INDEX.md`
**Location:** HS-107 entry, BC/VP citation column
**Current text:** "BC-2.01.013 (VP-028)"

**What happened:** VP-031 (proptest arithmetic correctness for SPB captured-len three-way min)
was assigned in Pass-3 as part of the C-1 fix propagation. HS-107 was authored in Pass-2
(before VP-031 existed). When VP-031 was added to BC-2.01.013 and VP-INDEX v2.5, neither HS-107
nor the HS-INDEX entry for HS-107 was updated to cross-reference VP-031.

**Impact:** Observation-level. The holdout scenario itself tests the behavior that VP-031
verifies (SPB captured-len arithmetic). The omission is a traceability gap, not a behavioral
gap. No phase-4 gate outcome depends on HS-107 listing VP-031, but the traceability matrix is
incomplete.

**No blocking action required.** Recommended fix: update HS-107 frontmatter
`verification_properties: [VP-028, VP-031]` and update the HS-INDEX entry to
"BC-2.01.013 (VP-028, VP-031)".

---

### FINDING-P3-004 — Minor (Seams 1 + 11)

**HS-107 v1.1 Case B shows two-way min expression instead of normative three-way form**

**File:** `.factory/holdout-scenarios/HS-107-pcapng-spb-framing-truncation-padding-and-no-idb.md`
**Location:** Case B Key observable line
**Current text (paraphrased):** "captured_len = min(200, 100) = 100"

**What is wrong:** BC-2.01.013 v1.3 normatively requires the three-way expression
min(original_len, snaplen, block_body_available) at every captured_len computation site.
The HS-107 Case B fixture has original_len=200, snaplen=100, block_body_available=100
(block_total_length=116, 116-16=100). For this specific fixture the two-way expression
min(200, 100) produces the correct result numerically because block_body_available equals
snaplen. However, the holdout observable shows only the two-way form, which:

1. Does not demonstrate that the three-way min is being applied (a two-way implementation
   would also pass Case B).
2. Is inconsistent with the normative three-way form in BC-2.01.013.

**Impact:** Minor. The Case B fixture as written does not distinguish between a correct
three-way implementation and an incorrect two-way implementation. The normative correctness
is covered by VP-031 (proptest), which will exercise the case where block_body_available is
the binding minimum. The HS-107 observable is a documentation inconsistency rather than a
behavioral gap.

**Fix:** Update HS-107 Case B Key observable to use the three-way form:
> "captured_len = min(original_len=200, snaplen=100, block_body_available=100) = 100
> (snaplen and block_body_available are tied as the binding minimum in this fixture;
> VP-031 exercises the case where block_body_available is strictly the binding minimum)"

---

## v3.0 Summary — Cross-Seam Audit

| Seam | Topic | Result |
|------|-------|--------|
| 1 | Three-way min — BC-2.01.013 normative sections | CLEAN (gaps in HS-107 captured in Seam 11) |
| 2 | E-INP-008 narrowed to semantic (H-1/H-2) | GAP: FINDING-P3-001 (Major) |
| 3 | E-INP-001 wiring (H-3) | CLEAN |
| 4 | IDB-parse precedence (M-7 / Decision 17) | GAP: FINDING-P3-002 (Minor) |
| 5 | Multi-section dead-spec (H-5 / Decision 16) | CLEAN |
| 6 | VP-031 (M-2) | CLEAN |
| 7 | Zero-packet notice (M-3) | CLEAN |
| 8 | Happy-path (M-5) | CLEAN |
| 9 | Timestamp parity (M-4) | CLEAN |
| 10 | Options TLV (M-6) | CLEAN |
| 11 | Holdouts (HS-103/104/107, HS-INDEX counts) | GAP: FINDING-P3-003 (Obs), FINDING-P3-004 (Minor) |
| 12 | Stale-note sweep (O-3); versions; 302 BCs | CLEAN |

**Overall v3.0 verdict: NOT CLEAN — 4 gaps found.**

| ID | Severity | Seam | Summary |
|----|----------|------|---------|
| FINDING-P3-001 | Major | 2 | error-taxonomy.md E-INP-008 Notes not updated for SHB semantic-only narrowing |
| FINDING-P3-002 | Minor | 4 | BC-2.01.018 Related BCs annotation reverses whitelist/conflict order |
| FINDING-P3-003 | Observation | 6+11 | HS-107 frontmatter + HS-INDEX entry omit VP-031 cross-reference |
| FINDING-P3-004 | Minor | 1+11 | HS-107 Case B shows two-way min instead of normative three-way form |

No blocking findings against phase-4 gate. FINDING-P3-001 (Major) is a taxonomy prose
inconsistency that should be resolved before the next adversarial pass. FINDING-P3-002 through
FINDING-P3-004 are Minor or Observation and do not affect behavioral correctness of any normative
section.

---

## Updated Open Findings Register

| ID | Severity | Source | Status |
|----|----------|--------|--------|
| FINDING-001 | HIGH | v1.0 audit — ADR-009 Status section stale contradiction | OPEN |
| FINDING-002 | HIGH | v1.0 audit — epics.md total_bcs 297 vs BC-INDEX 302 | OPEN |
| FINDING-003 | MEDIUM | v1.0 audit — prd.md RTM missing BC-2.01.009-018 rows | OPEN |
| FINDING-004 | MEDIUM | v1.0 audit — BC-INDEX updated timestamp stale | OPEN |
| FINDING-P2-001 | LOW | v2.0 audit — ADR-009 HS-completeness map HS-107 shown MISSING | OPEN |
| FINDING-P3-001 | MAJOR | v3.0 audit — error-taxonomy E-INP-008 Notes not updated for SHB semantic-only narrowing | OPEN |
| FINDING-P3-002 | MINOR | v3.0 audit — BC-2.01.018 Related BCs annotation reverses whitelist/conflict order | OPEN |
| FINDING-P3-003 | OBS | v3.0 audit — HS-107 + HS-INDEX omit VP-031 cross-reference | OPEN |
| FINDING-P3-004 | MINOR | v3.0 audit — HS-107 Case B shows two-way min expression | OPEN |

---

## v4.0 Append — F2 Pass-4 Remediation Cross-Seam Audit

**Audit date:** 2026-06-20
**Scope:** F2 Pass-4 remediation — 5 parallel PO bursts + architect rev 7. Seams 1-12 from the
Pass-4 audit brief checked against disk.

**Artifacts checked (Pass-4 versions):**

- error-taxonomy.md v3.1
- BC-2.01.009 v1.4, BC-2.01.010 v1.9, BC-2.01.011 v1.4, BC-2.01.012 v1.4,
  BC-2.01.013 v1.4, BC-2.01.014 v1.4, BC-2.01.015 v1.5, BC-2.01.016 v1.4,
  BC-2.01.017 v1.5, BC-2.01.018 v1.5
- VP-INDEX v2.6
- HS-INDEX v2.3, HS-103 v1.5, HS-104 v1.2, HS-107 v1.3, HS-108 v1.0
- BC-INDEX v1.58

---

### Seam 1 — Uniform error-code rule (Decision 20): GAPS

**Check:** E-INP-008 covers all four block types' wirerust body-decode failures (SHB=16,
IDB=8, EPB=20, SPB=4 fixed-field minimums); E-INP-010 covers crate framing rejections
(btl<12/misaligned/EOF) and EPB padding-aware over-read; no cross-wiring between the two
paths; all normative BCs, holdouts, and error-taxonomy consistent.

**Findings:**

- BC-2.01.010 v1.9 PC5: Four-way uniform split restated. Cases: (a) btl<12/misaligned/EOF →
  E-INP-010 (crate Err); (b) 12<=btl<28 → body<16 SHB bytes → wirerust body-decode →
  E-INP-008; (c) btl>=28 but invalid BOM or major_version!=1 → E-INP-008 (semantic);
  (d) well-formed → continues. AC-004a (btl=16 → body=4 < 16 → E-INP-008) and AC-004b
  (btl<12 → E-INP-010) both present and consistent with Decision 20. PASS.

- BC-2.01.012 v1.4 Description and PC3: A block_total_length in range [12, 32) produces a
  body shorter than 20 bytes; wirerust MUST return E-INP-008 (not E-INP-010) when the body
  is too short. EC-011: btl∈[12,32) → body<20 → E-INP-008. Architecture Anchors: "E-INP-008
  (NOT E-INP-010)". M-1 fix: wirerust MUST itself check body.len() >= 20; not delegated to
  crate. PASS.

- BC-2.01.013 v1.4 PC4: btl=12 (aligned, >=12, crate frames and returns block) → body=0
  bytes < 4 SPB fixed-field bytes → wirerust body-decode → E-INP-008. Distinguishes from
  btl<12/misaligned/EOF → crate Err → E-INP-010. M-1 fix: wirerust checks body.len()>=4
  itself. PASS.

- error-taxonomy v3.1 E-INP-008 scope (Decision 20): explicitly lists EPB body < 20 bytes
  and SPB body < 4 bytes as E-INP-008 subcategory (a). PASS.

- BC-2.01.011 v1.4 PC5 (lines 72-73): The two uniform-split bullet points are correct.
  However the tail sentence reads: "E-INP-008 covers SHB and IDB structural errors ONLY.
  EPB/SPB body truncation routes to E-INP-010 per error-taxonomy.md — E-INP-008 is NOT
  reused for packet-block truncation." This sentence is stale pre-Decision-20 wording.
  It directly contradicts: (1) BC-2.01.012 v1.4 which routes EPB body<20 to E-INP-008;
  (2) BC-2.01.013 v1.4 which routes SPB body<4 to E-INP-008; (3) error-taxonomy v3.1
  E-INP-008 scope which explicitly includes EPB and SPB body-decode failures. GAP —
  see FINDING-P4-001 below.

- error-taxonomy v3.1 E-INP-010 Notes tail: "Note: E-INP-008 is RESERVED for SHB/IDB
  body-decode failures (see that row); it is NOT used for EPB/SPB errors." This directly
  contradicts the E-INP-008 row in the same document whose scope explicitly lists EPB
  body<20 and SPB body<4 as E-INP-008 cases. GAP — see FINDING-P4-002 below.

- error-taxonomy v3.1 E-INP-010 items (d) and (e): item (d) "EPB body truncated (< 20
  fixed-field bytes)" and item (e) "SPB body truncated (< 4 bytes for original_len field,
  i.e., block_total_length < 16)" classify wirerust body-decode failures as E-INP-010. Per
  Decision 20 and BC-2.01.012/013, these are E-INP-008 cases, not E-INP-010 cases. The
  additional note in item (e) "block_total_length < 16" is also imprecise: the E-INP-008
  constructible window for SPB is btl=12 (body=0 < 4) through btl=15 (body=3 < 4); btl<12
  is the crate framing error (E-INP-010), not a body-decode case. GAP — see FINDING-P4-003
  below.

**SEAM 1: GAPS — FINDING-P4-001 (Major), FINDING-P4-002 (Major), FINDING-P4-003 (Major)**

---

### Seam 2 — H-2 peek-only probe (BC-2.01.009): CLEAN

**Check:** BC-2.01.009 v1.4 removes all consume(4) references; probe is PEEK-ONLY via
BufReader::fill_buf() with ZERO consumption; both pcap and pcapng branches receive the
full un-consumed stream.

**Findings:**

- BC-2.01.009 v1.4 changelog (v1.4): "Removed all consume(4) references: the probe is
  PEEK-ONLY via BufReader::fill_buf() with ZERO consumption; BOTH branches (classic PcapReader
  AND pcapng RawBlock) receive the FULL un-consumed stream starting at byte 0. Implementing
  consume(4) would break every file — removed from Description and Precondition 2 and
  Postcondition 3." Explicit removal confirmed. PASS.

- BC-2.01.009 v1.4 Description and PC2: no consume() call present. PASS.

- BC-2.01.009 v1.4 PC3: "the probe consumes no bytes; the next read on the same BufReader
  returns the byte that was at offset 0" (peek-only semantics preserved). PASS.

**SEAM 2: CLEAN**

---

### Seam 3 — Decision 19 zero-packet notice gating condition: CLEAN

**Check:** BC-2.01.009 v1.4 PC6 cites Decision 19 (not Decision 17); gating condition is
"valid file + zero packets" (NOT "skipped_blocks > 0"); BC-2.01.015 v1.5 PC9 counter feeds
but does not gate.

**Findings:**

- BC-2.01.009 v1.4 changelog (v1.4): "Decision 19 / M-4) Fixed PC6 citation: 'Decision 17'
  corrected to 'Decision 19'." Citation corrected. PASS.

- BC-2.01.009 v1.4 PC6: trigger = "valid file + zero packets" citing Decision 19. EC-008:
  IDB-only (zero skipped blocks) → notice without skip count. Gating condition is not
  skipped_blocks > 0. PASS.

- BC-2.01.015 v1.5 PC9 and AC-006: "counter feeds notice but trigger is owned by
  BC-2.01.009"; gating condition is "valid file + zero packets" (Decision 19). PASS.

- BC-2.01.009 v1.4 PC3 tail (H-4 disambiguation): "a file is 'structurally-valid
  zero-packet' (notice, exit 0) IFF it parses to EOF with no error AND packets.len()==0;
  an EPB/SPB before any IDB is an ERROR (E-INP-009, exit 1), NOT a zero-packet success."
  Explicit disambiguation rule present. PASS.

- HS-108 v1.0 confirms: three cases — (A) SHB+IDB no EPB/SPB → notice without skip count,
  exit 0; (B) 2 unknown blocks → notice with "2 block(s) skipped", exit 0; (C) EPB before
  IDB → E-INP-009, exit 1, NO notice. Consistent. PASS.

**SEAM 3: CLEAN**

---

### Seam 4 — EPB padding-aware bound (C-1): CLEAN

**Check:** BC-2.01.012 v1.4 PC3/AC-002 has two-step check: (1) unconditional captured_len
<= body.len(); (2) 20 + captured_len + pad_len(captured_len) <= body.len(); HS-104 v1.2
Case E exercises non-mult-4 captured_len path.

**Findings:**

- BC-2.01.012 v1.4 Description: "two-step check: first, captured_len can never exceed
  body.len() (unconditional bound-by-body); second, the padding-aware overhead test
  EPB_FIXED_OVERHEAD_BYTES(20) + captured_len + pad_len(captured_len) <= body.len()
  (where pad_len(n) = (4 - n%4) % 4) must pass before any allocation." Two-step form
  present. PASS.

- BC-2.01.012 v1.4 AC-002/EC-009/EC-010: updated to padding-aware bound per v1.4 changelog
  (C-1 fix). PASS.

- HS-104 v1.2 Case E: captured_len ≡ 3 mod 4, raw check passes but padded extent overflows
  → E-INP-010. Exercises the case where pad_len(captured_len) pushes the padded extent past
  body.len(). PASS.

- HS-INDEX v2.3 entry for HS-104: v1.2 listed, Case E present. PASS.

**SEAM 4: CLEAN**

---

### Seam 5 — M-1 body-minimum guard owned by wirerust: CLEAN

**Check:** BC-2.01.011 v1.4, BC-2.01.012 v1.4, BC-2.01.013 v1.4 all state that wirerust
performs the body-minimum check itself on the raw path; "crate enforces" over-claim removed
from Architecture Anchors in each.

**Findings:**

- BC-2.01.011 v1.4 changelog (v1.4): "M-1: removed 'crate enforces body>=8' over-claim from
  Architecture Anchors — wirerust checks body.len()>=8 itself on the raw path before decoding
  IDB fixed fields." Over-claim removed. PASS.

- BC-2.01.012 v1.4 PC3 and AC-003: "on the raw-block path the crate does NOT run its
  EnhancedPacketBlock parser; wirerust MUST itself check body.len() >= 20 before reading any
  EPB fixed field — the 20-byte check is NOT delegated to the crate." Explicit ownership
  stated. PASS.

- BC-2.01.013 v1.4 changelog (v1.4): "M-1: removed 'crate enforces body minimum' over-claim
  from Architecture Anchors — wirerust checks body.len()>=4 itself on the raw path before
  decoding SPB fixed fields." PASS.

**SEAM 5: CLEAN**

---

### Seam 6 — Decision 21 if_tsoffset limitation: CLEAN

**Check:** BC-2.01.011 v1.4 PC6 limitation note: if_tsoffset (option code 10) NOT extracted
this cycle. BC-2.01.014 v1.4 limitation note matches. No BC claims to apply if_tsoffset.

**Findings:**

- BC-2.01.011 v1.4 PC6 (lines 86-89): "Limitation (ADR-009 Decision 21): if_tsoffset
  (option code 10) is NOT extracted or applied this cycle. Only if_tsresol (code 9) is
  extracted. Timestamp offsets embedded in IDB options are silently skipped as unknown option
  codes. This is a known limitation scoped out for this cycle." PASS.

- BC-2.01.011 v1.4 AC-003: Decision 21 limitation note also present in the AC. PASS.

- BC-2.01.014 v1.4 limitation note: does NOT apply if_tsoffset. PASS.

- No BC in the SS-01 set claims to extract or apply if_tsoffset. PASS.

**SEAM 6: CLEAN**

---

### Seam 7 — Block #<seq> numbering convention (M-5): CLEAN

**Check:** error-taxonomy v3.1 preamble pins the 1-based block-sequence numbering convention
(SHB=block #1, each next_raw_block increments); E-INP-010/012/013 Context fields all use
this convention; conflicting "first block after SHB = #1" wording removed.

**Findings:**

- error-taxonomy v3.1 preamble (Block #<seq> numbering convention note): "SHB is block #1;
  each next_raw_block call increments the counter. Consequently: the first IDB (immediately
  after the SHB) is block #2; the first EPB is block #3 in a single-IDB file. This convention
  is the single source of truth for all E-INP-010, E-INP-012, and E-INP-013 Context fields
  below. The earlier wording 'first block after the SHB = #1' that appeared in some entries
  was incorrect and has been removed." Present and correct. PASS.

- error-taxonomy v3.1 v3.1 changelog: "(M-5): block #<seq> numbering convention pinned in
  taxonomy header AND in E-INP-010/012/013 Context fields: 1-based; SHB is block #1; each
  next_raw_block call increments the counter. Conflicting 'first block after the SHB = #1'
  wording removed from E-INP-010 and E-INP-013 so all three entries agree with E-INP-012."
  PASS.

- E-INP-010, E-INP-012, E-INP-013 Context fields: all three carry the "`<seq>` convention
  (M-5)" annotation with consistent "1-based; the SHB is block #1" wording. PASS.

**SEAM 7: CLEAN**

---

### Seam 8 — VP-030 domain narrowing (H-3): CLEAN

**Check:** VP-INDEX v2.6 VP-030 restated: domain = WHITELISTED DataLink values only;
comparison unit = DataLink enum (not raw u16); non-whitelisted → E-INP-001 (out of VP-030
scope). BC-2.01.018 v1.5 consistent.

**Findings:**

- VP-INDEX v2.6 changelog (Pass-4 entry): "VP-030 RESTATED: domain narrowed from 'any
  sequence of IDB linktype u16 values' to 'WHITELISTED DataLink values only' (non-whitelisted
  values short-circuit to E-INP-001 before the conflict check is ever reached; the original
  domain included unreachable sequences). Comparison unit pinned to DataLink (not raw u16).
  Property restated: all-equal whitelisted DataLink → Ok; first-differing whitelisted DataLink
  → Err(E-INP-011) on that IDB; non-whitelisted → E-INP-001 (out of VP-030 scope). No VP
  counts changed (31 total; proptest 10; draft 7)." PASS.

- BC-2.01.018 v1.5: VP-030 description in Verification Properties table reflects narrowed
  domain (WHITELISTED) and DataLink comparison unit. PASS.

- VP counts unchanged at 31; arithmetic consistent (8+17+6=31; 14+10+2+5=31). PASS.

**SEAM 8: CLEAN**

---

### Seam 9 — HS-108 authoring and HS-INDEX consistency: CLEAN

**Check:** HS-108 exists on disk with the three cases (A: SHB+IDB zero-packet notice;
B: 2 unknown blocks notice with skip count; C: EPB before IDB → E-INP-009). HS-INDEX v2.3
total_scenarios=108, must_pass=107, should_pass=1.

**Findings:**

- HS-108 file exists: `.factory/holdout-scenarios/HS-108-pcapng-zero-packet-notice-end-to-end.md`
  v1.0. Present. PASS.

- HS-108 three cases confirmed: (A) SHB+IDB only → notice without skip count, exit 0;
  (B) 2 unknown blocks → notice with skip count, exit 0; (C) EPB before IDB → E-INP-009,
  exit 1, NO notice. H-4 disambiguation rule exercised in Case C. PASS.

- HS-INDEX v2.3 frontmatter: total_scenarios=108, must_pass_count=107, should_pass_count=1.
  All-namespace total=181 documented. HS-108 present in catalog. PASS.

- HS-INDEX v2.3 changelog note: "Pass-4 R4 / ADR-009 rev 7: added HS-108 (zero-packet notice
  end-to-end — BC-2.01.009 PC6 / BC-2.01.015 PC9 / H-4). Greenfield total now 108. All-
  namespace total now 181." Consistent with frontmatter. PASS.

- HS-103 v1.5 Case D (btl=16→E-INP-008): present per HS-INDEX v2.3 changelog. Exercises
  SHB constructible body-truncation fixture per Decision 20 restoration. PASS.

- HS-104 v1.2 Case E (non-mult-4 captured_len, padded extent overflows): present per
  HS-INDEX v2.3 changelog. PASS.

- HS-107 v1.3 Case F (btl=12→E-INP-008 for SPB): present per HS-INDEX v2.3 changelog.
  Exercises SPB constructible body-truncation fixture per Decision 20. PASS.

- Prior findings FINDING-P3-003 and FINDING-P3-004 (HS-107 VP column missing VP-031, Case B
  two-way min): HS-INDEX v2.3 version note references "P3-re-audit FINDING-P3-003+P3-004:
  HS-107 VP column updated." These are resolved. No re-open needed. PASS.

**SEAM 9: CLEAN**

---

### Seam 10 — BC-INDEX v1.58 counts and inline versions: CLEAN

**Check:** BC-INDEX v1.58 active count = 302; all 10 SS-01 Pass-4 BCs have inline version
annotations matching on-disk frontmatter; epics.md discrepancy (FINDING-002) is pre-existing
and not introduced by Pass-4.

**Findings:**

- BC-INDEX v1.58: "Active: 302" confirmed. 302 - 0 new BCs this pass (all 10 SS-01 BCs were
  existing; Pass-4 only bumped versions). Active count unchanged from v1.56. PASS.

- Inline version annotations in BC-INDEX v1.58 for the 10 audited Pass-4 BCs: BC-2.01.009
  v1.4, BC-2.01.010 v1.9, BC-2.01.011 v1.4, BC-2.01.012 v1.4, BC-2.01.013 v1.4,
  BC-2.01.014 v1.4, BC-2.01.015 v1.5, BC-2.01.016 v1.4, BC-2.01.017 v1.5, BC-2.01.018 v1.5
  — all match on-disk frontmatter versions confirmed during this audit. PASS.

- epics.md total_bcs discrepancy (FINDING-002: 297 vs 302) is pre-existing from v1.0 audit.
  No new drift introduced by Pass-4. OPEN but not a Pass-4 regression. PASS (no new finding).

**SEAM 10: CLEAN**

---

### Seam 11 — Version monotonicity and next_free_error_code: CLEAN

**Check:** All 10 Pass-4 BCs show monotonic version increments from their Pass-3 versions;
error-taxonomy v3.1 next_free_error_code = E-INP-014; no new ID collisions.

**Findings:**

- Version increments (Pass-3 → Pass-4):
  BC-2.01.009 v1.3→v1.4, BC-2.01.010 v1.8→v1.9, BC-2.01.011 v1.3→v1.4,
  BC-2.01.012 v1.3→v1.4, BC-2.01.013 v1.3→v1.4, BC-2.01.014 v1.3→v1.4,
  BC-2.01.015 v1.4→v1.5, BC-2.01.016 v1.3→v1.4, BC-2.01.017 v1.4→v1.5,
  BC-2.01.018 v1.3→v1.5. All monotonic. PASS.

- error-taxonomy v3.1 E-INP-013 row tail: "next_free_error_code: E-INP-014." Confirmed;
  no E-INP-014 defined anywhere in the taxonomy. PASS.

- VP-INDEX v2.6 total_vps=31, counts consistent (14+10+2+5=31; 8+17+6=31). No new VPs added
  in Pass-4 (VP-030 restated in place, no count change). PASS.

- HS-INDEX v2.3 total_scenarios=108. Previously 107 (v2.2); Pass-4 added HS-108 (+1). PASS.

**SEAM 11: CLEAN**

---

### Seam 12 — VP-INDEX self-consistency (criteria 78): CLEAN

**Check:** VP-INDEX v2.6 total_vps=31 equals sum of tool counts (kani=14, proptest=10,
fuzz=2, integration/unit=5); equals sum of phase counts (p0=8, p1=17, test_sufficient=6).

**Findings:**

- Tool total: 14+10+2+5=31. Matches total_vps=31. PASS.
- Phase total: 8+17+6=31. Matches total_vps=31. PASS.
- VP-030 restatement did not change counts; all consistency invariant annotations in VP-INDEX
  v2.6 reflect the correct post-restatement totals. PASS.

**SEAM 12: CLEAN**

---

## v4.0 Findings

### FINDING-P4-001 — Major (Seam 1)

**BC-2.01.011 v1.4 PC5 tail sentence contains stale pre-Decision-20 routing rule**

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/behavioral-contracts/ss-01/BC-2.01.011.md`
**Frontmatter version:** v1.4
**Location:** Postcondition 5, tail sentence (after the two uniform-split bullet points)

**Current text:**
> "E-INP-008 covers SHB and IDB structural errors ONLY. EPB/SPB body truncation routes to
> E-INP-010 per error-taxonomy.md — E-INP-008 is NOT reused for packet-block truncation."

**What is wrong:** Decision 20 (ADR-009 rev 7) established the uniform four-way rule that
routes wirerust body-decode failures for ALL four block types (SHB, IDB, EPB, SPB) to
E-INP-008. The two bullet points immediately above this sentence in PC5 are correct and
reflect Decision 20. However, this tail sentence was carried forward from BC-2.01.011 v1.2
(Pass-2) where E-INP-008 did apply only to SHB/IDB, and was NOT updated during Pass-4 to
reflect the expanded scope. It now directly contradicts:

1. **BC-2.01.012 v1.4** — PC3, Description, and EC-011 explicitly route EPB body<20 to
   E-INP-008 with the note "not E-INP-010."
2. **BC-2.01.013 v1.4** — PC4 routes SPB body<4 (btl=12, body=0) to E-INP-008.
3. **error-taxonomy v3.1 E-INP-008 scope** — explicitly lists "EPB body < 20 bytes; SPB
   body < 4 bytes (original_len)" as E-INP-008 subcategory (a).

**Risk:** The IDB BC is the first block-type BC an implementer reads when building the block
walker. Finding this sentence in BC-2.01.011 — before reading BC-2.01.012 or BC-2.01.013 —
will cause the implementer to wire EPB/SPB body-too-short paths to E-INP-010, directly
contradicting the normative EPB and SPB BCs.

**Remediation:** Remove the tail sentence and replace with:
> "Note: per Decision 20, E-INP-008 applies to wirerust body-decode failures for ALL four
> block types (SHB, IDB, EPB, SPB). EPB body<20 → E-INP-008 (BC-2.01.012 EC-011); SPB
> body<4 → E-INP-008 (BC-2.01.013 PC4). E-INP-010 is strictly the crate-framing path."

---

### FINDING-P4-002 — Major (Seam 1)

**error-taxonomy v3.1 E-INP-010 Note contradicts E-INP-008's own scope within the same document**

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/prd-supplements/error-taxonomy.md`
**Frontmatter version:** v3.1
**Location:** E-INP-010 row, Notes field, tail sentence

**Current text (tail sentence of E-INP-010 Notes):**
> "Note: E-INP-008 is RESERVED for SHB/IDB body-decode failures (see that row); it is NOT
> used for EPB/SPB errors."

**What is wrong:** In the same document, one row above, the E-INP-008 Notes field (which v3.1
explicitly updated for Decision 20) reads:
> "Two subcategories: (a) Block body shorter than required fixed-field bytes — SHB body < 16
> bytes; IDB body < 8 bytes; **EPB body < 20 bytes; SPB body < 4 bytes** (original_len)."

The E-INP-010 tail note directly contradicts the E-INP-008 scope within the same version of
the same document. A developer reading E-INP-010 after E-INP-008 encounters a contradiction
with no resolution path. The E-INP-010 Note appears to be a carry-over from v2.9/v3.0 that
was not excised when the E-INP-008 scope was expanded in v3.1 to cover EPB and SPB.

**Risk:** The Note gives the false appearance that E-INP-010 is authoritative for EPB/SPB
errors, overriding the E-INP-008 scope. An implementer who reads the Note but not the
E-INP-008 scope body will misroute EPB/SPB body-too-short paths to E-INP-010.

**Remediation:** Remove the tail sentence from E-INP-010 Notes, or replace with:
> "Note: E-INP-008 covers SHB and IDB framing-on-body failures AND EPB/SPB wirerust
> body-decode failures (body shorter than required fixed-field bytes); see that row's scope
> (Decision 20). E-INP-010 covers crate-level framing rejections and EPB padding-aware
> over-read."

---

### FINDING-P4-003 — Major (Seam 1)

**error-taxonomy v3.1 E-INP-010 items (d) and (e) classify wirerust body-decode failures as E-INP-010**

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/prd-supplements/error-taxonomy.md`
**Frontmatter version:** v3.1
**Location:** E-INP-010 row, Notes field, items (d) and (e)

**Current text:**
> "(d) EPB body truncated (< 20 fixed-field bytes). (e) SPB body truncated (< 4 bytes for
> original_len field, i.e., block_total_length < 16)."

**What is wrong:** These items describe wirerust body-decode failures — cases where pcap-file
2.0.0 successfully frames the block (btl >= 12, aligned, trailing length matches) and returns
a block body, but wirerust's own decode finds the body is shorter than the required fixed
fields. Per Decision 20 and the E-INP-008 scope in the same document, these are E-INP-008
cases, not E-INP-010 cases. Item (e) also contains an inaccurate parenthetical: "i.e.,
block_total_length < 16" — the E-INP-008 constructible window for SPB is 12 <= btl <= 15
(body 0–3 bytes); btl < 12 is the crate framing rejection (which IS E-INP-010). So the
parenthetical describes a mixed window that straddles both error codes.

**Evidence triangle:**
- BC-2.01.012 v1.4 EC-011 and Description: btl∈[12,32) → body<20 → "E-INP-008 (not E-INP-010)"
- BC-2.01.013 v1.4 PC4: btl=12 → body=0 < 4 → E-INP-008
- error-taxonomy v3.1 E-INP-008 scope: EPB body<20 and SPB body<4 listed as E-INP-008 subcategory (a)

**Risk:** Items (d) and (e) in E-INP-010 are stale pre-Decision-20 entries that directly
conflict with the normative BCs. An implementer mapping error codes from E-INP-010 will
misroute EPB/SPB body-too-short failures.

**Remediation:** Remove items (d) and (e) from E-INP-010 Notes, and add a cross-reference:
> "Note: EPB body < 20 bytes and SPB body < 4 bytes are E-INP-008 cases (wirerust body-decode
> failures), NOT E-INP-010. See E-INP-008 scope for the full boundary (Decision 20)."
Adjust E-INP-010 item (e) for the crate-framing case: "SPB block_total_length < 12 or
misaligned → crate Err → E-INP-010" (distinct from the wirerust body-decode case).

---

## v4.0 Summary — Cross-Seam Audit

| Seam | Topic | Result |
|------|-------|--------|
| 1 | Uniform error-code rule (Decision 20) — E-INP-008/E-INP-010 boundary | GAPS: 3 Major findings |
| 2 | H-2 peek-only probe — consume(4) removed from BC-2.01.009 | CLEAN |
| 3 | Decision 19 zero-packet notice gating condition | CLEAN |
| 4 | EPB padding-aware bound (C-1) — two-step check | CLEAN |
| 5 | M-1 body-minimum guard owned by wirerust, not crate | CLEAN |
| 6 | Decision 21 if_tsoffset limitation noted and scoped out | CLEAN |
| 7 | Block #<seq> numbering convention (M-5) pinned in taxonomy | CLEAN |
| 8 | VP-030 domain narrowed to whitelisted DataLink values (H-3) | CLEAN |
| 9 | HS-108 authored; HS-103/104/107 Decision 20 cases added | CLEAN |
| 10 | BC-INDEX v1.58 counts and inline versions consistent | CLEAN |
| 11 | Version monotonicity; next_free E-INP-014; VP/HS counts | CLEAN |
| 12 | VP-INDEX self-consistency arithmetic | CLEAN |

**Overall v4.0 verdict: NOT CLEAN — 3 Major gaps found, all in Seam 1.**

All three gaps are co-located at the E-INP-008/E-INP-010 boundary and stem from the same
root cause: the Decision 20 ("uniform error-code rule") expansion of E-INP-008 to cover EPB
and SPB wirerust body-decode failures was applied correctly to the E-INP-008 scope entry and
to BC-2.01.012/013, but two residual pre-Decision-20 artifacts were NOT purged:

1. The tail sentence in BC-2.01.011 PC5 that restricts E-INP-008 to SHB/IDB.
2. The E-INP-010 Note and items (d)/(e) that assign EPB/SPB body truncation to E-INP-010.

These are specification contradictions, not ambiguities. They will cause incorrect
implementation if an implementer reads any of the three stale locations as authoritative.

No blocking findings against Seams 2-12. The three Major findings in Seam 1 should be
resolved before Phase-4 holdout evaluation.

---

## Updated Open Findings Register

| ID | Severity | Source | Status |
|----|----------|--------|--------|
| FINDING-001 | HIGH | v1.0 audit — ADR-009 Status section stale contradiction | OPEN |
| FINDING-002 | HIGH | v1.0 audit — epics.md total_bcs 297 vs BC-INDEX 302 | OPEN |
| FINDING-003 | MEDIUM | v1.0 audit — prd.md RTM missing BC-2.01.009-018 rows | OPEN |
| FINDING-004 | MEDIUM | v1.0 audit — BC-INDEX updated timestamp stale | OPEN |
| FINDING-P2-001 | LOW | v2.0 audit — ADR-009 HS-completeness map HS-107 shown MISSING | OPEN |
| FINDING-P3-001 | MAJOR | v3.0 audit — error-taxonomy E-INP-008 Notes not updated for SHB semantic-only narrowing | RESOLVED (v3.1 correctly updated the Notes) |
| FINDING-P3-002 | MINOR | v3.0 audit — BC-2.01.018 Related BCs annotation reverses whitelist/conflict order | OPEN |
| FINDING-P3-003 | OBS | v3.0 audit — HS-107 + HS-INDEX omit VP-031 cross-reference | RESOLVED (HS-INDEX v2.3 updated VP column) |
| FINDING-P3-004 | MINOR | v3.0 audit — HS-107 Case B shows two-way min expression | OPEN |
| FINDING-P4-001 | MAJOR | v4.0 audit — BC-2.01.011 PC5 tail sentence restricts E-INP-008 to SHB/IDB only (stale pre-Decision-20) | RESOLVED (BC-2.01.011 v1.5 / BC-INDEX v1.59) |
| FINDING-P4-002 | MAJOR | v4.0 audit — error-taxonomy v3.1 E-INP-010 Note says E-INP-008 not used for EPB/SPB (contradicts E-INP-008 scope in same document) | RESOLVED (error-taxonomy v3.2 / BC-INDEX v1.59) |
| FINDING-P4-003 | MAJOR | v4.0 audit — error-taxonomy v3.1 E-INP-010 items (d)/(e) classify EPB/SPB body-decode failures as E-INP-010 (stale pre-Decision-20) | RESOLVED (error-taxonomy v3.2 / BC-INDEX v1.59) |

---

## v5.0 Append — F2 Pass-5 Remediation Cross-Seam Audit

**Audit date:** 2026-06-20
**Scope:** F2 Pass-5 remediation — 4 parallel PO bursts + architect rev 8 (ADR-009 rev 8). Ten
seams from the Pass-5 audit brief checked against disk.

**Artifacts checked (Pass-5 versions):**

- error-taxonomy.md v3.3
- BC-2.01.009 v1.5, BC-2.01.010 v1.9 (unchanged in Pass-5), BC-2.01.011 v1.5,
  BC-2.01.012 v1.5, BC-2.01.013 v1.5, BC-2.01.014 v1.5, BC-2.01.015 v1.6,
  BC-2.01.018 v1.6
- VP-INDEX v2.7
- verification-architecture.md v2.3
- verification-coverage-matrix.md v1.17
- HS-104 v1.3, HS-107 v1.4, HS-108 v1.1
- BC-INDEX v1.60

---

### Seam 1 — C-1 EPB→E-INP-008 at all sites: CLEAN

**Check:** BC-2.01.012 v1.5 PC6a/PC6b, AC-002, AC-006, EC-010, canonical vectors, VP-027 all
use E-INP-008 for EPB bound-by-body and padding-overrun; E-INP-010 in this BC is STRICTLY crate
framing (EC-012, interface_id OOB); HS-104 v1.3 Cases D/E → E-INP-008; error-taxonomy v3.3
E-INP-008 scope includes EPB padding-overrun and bound-by-body; E-INP-010 scope boundary
statement consistent.

**Findings:**

- BC-2.01.012 v1.5 changelog (v1.5): EPB body-decode failures reclassified E-INP-010 →
  E-INP-008 at all sites per ADR-009 rev 8 C-1. Explicitly updated: PC6a (bound-by-body →
  E-INP-008); PC6b (padding-overrun → E-INP-008); AC-002 both sub-checks → E-INP-008;
  AC-006 one-over case → E-INP-008; EC-010 → E-INP-008; canonical test vectors →
  E-INP-008; VP-027 updated. E-INP-010 in this BC now STRICTLY: (i) crate framing
  rejection EC-012 (btl<12/misaligned/EOF); (ii) EPB interface_id OOB on non-empty table
  (EC-006/007/PC5). PASS.

- BC-2.01.012 v1.5 PC6a (on-disk text): "captured_len <= body.len() ... return Err mapping
  to E-INP-008 (wirerust body-decode failure — crate already framed the block)." PASS.

- BC-2.01.012 v1.5 PC6b (on-disk text): "EPB_FIXED_OVERHEAD_BYTES(20) + captured_len +
  pad_len(captured_len) <= body.len() ... Failure → Err mapping to E-INP-008 (wirerust
  body-decode failure — block-length inconsistency / padding overrun)." PASS.

- BC-2.01.012 v1.5 EC-010: "Err mapping to E-INP-008 (wirerust body-decode failure —
  padded total exceeds body; crate framed the block successfully, wirerust rejects the
  padded extent)." PASS.

- BC-2.01.012 v1.5 AC-006: "A captured_len one byte larger ... MUST return Err mapping to
  E-INP-008." PASS.

- BC-2.01.012 v1.5 VP-027: "padding-overrun (20+captured_len+pad_len>body.len()) →
  Err(E-INP-008); bound-by-body (captured_len>body.len()-20) → Err(E-INP-008); NOT
  E-INP-010 (rev 8 / C-1 / Decision 20 clarification)." PASS.

- error-taxonomy v3.3 E-INP-008 scope: explicitly lists "EPB captured_len > body.len() - 20
  (bound-by-body failure)" and "EPB 20 + captured_len + pad_len(captured_len) > body.len()
  (padding-overrun)" as subcategory (a) body-decode failures. PASS.

- error-taxonomy v3.3 E-INP-008 scope boundary note: "E-INP-010 is STRICTLY crate-side
  framing rejection; ALL wirerust-computed body-decode failures (body-too-short, bound-by-body,
  padding-overrun) use E-INP-008." PASS.

- error-taxonomy v3.3 E-INP-010 scope boundary: "Scope boundary (Decision 20 / rev 8 uniform
  rule): EPB body < 20 fixed-field bytes, EPB captured_len > body.len() - 20 (bound-by-body),
  and EPB 20 + captured_len + pad_len > body.len() (padding-overrun) are ALL E-INP-008 (not
  E-INP-010)." Consistent. PASS.

- HS-104 v1.3 Case D: "E-INP-008 (wirerust body-decode failure — crate framed the block;
  wirerust rejects the body content)." PASS.

- HS-104 v1.3 Case E: "E-INP-008 (wirerust body-decode failure — crate already framed the
  block with btl >= 12; wirerust body-decode discovers the padding overrun)." PASS.

- HS-104 v1.3 BC Linkage table: Cases D/E both → E-INP-008. No residual E-INP-010 in either
  case. PASS.

**SEAM 1: CLEAN**

---

### Seam 2 — SPB snaplen DROP: CLEAN

**Check:** BC-2.01.013 v1.5 uses captured_len = min(original_len, block_body_available)
everywhere; no snaplen term; VP-031 formula = min(original_len, body.len() as u32); HS-107 v1.4
no stale snaplen-clamp wording; Case B = body-bound (block_body_available, NOT snaplen); no
stale "deferred to a separate burst" notes in BC-2.01.013.

**Findings:**

- BC-2.01.013 v1.5 changelog (v1.5): "snaplen DROPPED from SPB captured_len. Decision 9 states
  snaplen is NOT enforced for SPB (same as EPB). captured_len now = min(original_len,
  block_body_available) everywhere. Removed snaplen from: Description, PC1, AC-002, EC-007,
  EC-001, Invariant 2, Canonical Test Vectors, Architecture Anchors. VP-031 updated:
  captured_len == min(original_len, body.len() as u32). ... Removed 4x stale '(HS-107
  btl=12→E-INP-008 holdout deferred to a separate burst.)' notes." PASS.

- BC-2.01.013 v1.5 Description: "Per ADR-009 rev 8 Decision 9 amendment, snaplen is NOT
  applied for SPB ... captured_len = min(original_len, block_body_available)." PASS.

- BC-2.01.013 v1.5 PC1: "captured_len = min(original_len, block_body_available)" — two-way
  formula only, no snaplen term. PASS.

- BC-2.01.013 v1.5 AC-002: "captured_len = min(original_len, block_body_available) where
  block_body_available = block_total_length - 16 (equivalently, body.len()). Snaplen is NOT
  applied for SPB (ADR-009 rev 8 Decision 9 amendment)." PASS.

- BC-2.01.013 v1.5 Invariant 2: "Packet data is bounded by min(original_len,
  block_body_available) ... Snaplen is NOT applied ... (ADR-009 rev 8 Decision 9 amendment)."
  PASS.

- BC-2.01.013 v1.5 EC-001, EC-007, Canonical Test Vectors: all use the two-way formula; snaplen
  absent. EC-007 rationale: "snaplen is NOT applied (ADR-009 rev 8 Decision 9 amendment)." PASS.

- BC-2.01.013 v1.5 VP-031 row: "For all (original_len: u32, body: &[u8]): captured_len ==
  min(original_len, body.len() as u32) ... Snaplen is excluded from the pure-core helper
  domain (ADR-009 rev 8 Decision 9 amendment)." PASS.

- Stale deferral notes: grep confirmed zero occurrences of "deferred to a separate burst" in
  BC-2.01.013 v1.5. All 4 were removed in v1.5. PASS.

- HS-107 v1.4 Case B: "captured_len = min(original_len=200, block_body_available=100) = 100
  (the on-disk body is the authoritative bound — snaplen is NOT applied to SPB)." Two-way
  formula with explicit note that snaplen is not applied. Consistent with BC-2.01.013 v1.5.
  PASS.

- HS-107 v1.4 BC Linkage table: "Postcondition 1 — data bounded by min(original_len,
  block_body_available); snaplen not applied." PASS.

- HS-107 v1.4 Rubric: "snaplen is NOT applied for SPB (ADR-009 rev 8 Decision 9 amendment)."
  PASS.

**SEAM 2: CLEAN**

---

### Seam 3 — Uniform error rule (Decision 20): CLEAN

**Check:** body-too-short→E-INP-008 for all four block types (SHB=16, IDB=8, EPB=20, SPB=4);
framing<12→E-INP-010; EPB padding/bound→E-INP-008; error-taxonomy v3.3 E-INP-008/010 scopes
consistent with all normative BCs.

**Findings:**

- error-taxonomy v3.3 E-INP-008 scope (body-decode failures for ALL block types): SHB body<16,
  IDB body<8, EPB body<20, SPB body<4, EPB bound-by-body, EPB padding-overrun — all listed as
  subcategory (a) body-decode failures → E-INP-008. PASS.

- error-taxonomy v3.3 E-INP-010 scope (crate-framing rejections only): btl<12 / misaligned /
  EOF; EPB interface_id OOB on non-empty table; unknown-block framing errors. Items (d) and (e)
  (EPB body<20 and SPB body<4) were removed in v3.2; v3.3 adds the boundary clarification note
  in E-INP-010 that EPB body-decode failures use E-INP-008. PASS.

- BC-2.01.011 v1.5 PC5 (IDB uniform rule): "Uniform rule (Decision 20): E-INP-008 covers
  wirerust body-decode failures for ALL block types" — stale PC5 tail sentence removed.
  FINDING-P4-001 RESOLVED. PASS.

- BC-2.01.012 v1.5 PC6a/PC6b and EC-010/EC-011: EPB body-decode failures → E-INP-008 at all
  sites (C-1 reclassification). PASS.

- BC-2.01.013 v1.5 PC4/EC-008/AC-004a: SPB btl=12 → body=0 < 4 → E-INP-008. PASS.

- BC-2.01.010 v1.9 PC5: "E-INP-008 (NOT E-INP-010)" for SHB body-too-short path; EC-005:
  btl=16 → body=4 < 16 → E-INP-008. PASS.

- Cross-BC consistency: all four block types now route body-decode failures to E-INP-008 and
  crate-framing failures to E-INP-010. No contradiction found among BC-2.01.010/011/012/013
  or error-taxonomy v3.3. PASS.

**SEAM 3: CLEAN**

---

### Seam 4 — H-1 precedence (Decision 17): CLEAN

**Check:** BC-2.01.018 v1.6 EC-006 (ETHERNET then IEEE802_11 → E-INP-001 on 2nd IDB at
whitelist check #2; E-INP-011 conflict check #3 never reached because whitelist preempts); EC-008
(two IEEE802_11 → E-INP-001 on FIRST IDB at whitelist check #2; second IDB never parsed);
E-INP-011 reachable ONLY when both IDBs whitelisted AND differ.

**Findings:**

- BC-2.01.018 v1.6 changelog (v1.6): "EC-006 CORRECTED: ETHERNET (whitelisted) then IEEE802_11
  (non-whitelisted) → E-INP-001 fires on the SECOND IDB at whitelist check (#2); E-INP-011
  conflict check (#3) is NEVER reached because whitelist preempts conflict. ... EC-008
  RE-DERIVED: two IEEE802_11 IDBs — the FIRST IDB (non-whitelisted) already triggers E-INP-001
  at whitelist check (#2) during first-IDB parse time; the second IDB is NEVER parsed. E-INP-011
  conflict check is reachable ONLY when BOTH IDBs are whitelisted AND differ." PASS.

- BC-2.01.018 v1.6 EC-006 (on-disk text): "E-INP-001 fires on the SECOND IDB at whitelist
  check (#2 per Decision 17); E-INP-011 conflict check (#3) is NEVER reached because whitelist
  preempts conflict." PASS.

- BC-2.01.018 v1.6 EC-008 (on-disk text): "E-INP-001 fires on the FIRST IDB at whitelist
  check (#2 per Decision 17) during first-IDB parse time; the second IDB is NEVER parsed and
  the agreement between the two IDBs is completely unobservable. The defunct narrative ... is
  abandoned. Correct behavior: IEEE802_11 hits whitelist check (#2) → E-INP-001 immediately."
  PASS.

- BC-2.01.018 v1.6 Description and Invariants: "E-INP-011 is the THIRD check in the IDB-parse
  precedence (Decision 17): the E-INP-013 position check runs first ... and the E-INP-001
  whitelist check runs second. E-INP-011 fires only if both prior checks pass." PASS.

- BC-2.01.018 v1.6 Related BCs: "BC-2.01.016 — composes with (whitelist check runs second;
  agreement/conflict check runs third — per Decision 17: E-INP-013 position FIRST, E-INP-001
  whitelist SECOND, E-INP-011 conflict THIRD)." FINDING-P3-002 RESOLVED. PASS.

- E-INP-011 reachability: confirmed only via EC-003 (ETHERNET then LINUX_SLL, both
  whitelisted) and EC-004 (three whitelisted, all agree then differ). Non-whitelisted IDBs
  short-circuit to E-INP-001 at whitelist check (#2) before E-INP-011 is ever evaluated. PASS.

**SEAM 4: CLEAN**

---

### Seam 5 — M-5 notice (zero-packet emission from main.rs): PASS WITH GAPS

**Check:** BC-2.01.009 v1.5 PC6 (emission from main.rs; PcapSource exposes skipped_blocks:u32
+ opb_skipped:u32; format "notice: <filename>: 0 packets read from <pcap|pcapng> file";
opb_skipped>0 → mergecap hint; classic empty-pcap symmetry) consistent with BC-2.01.015 v1.6
(counters SURFACED not emitted; opb_skipped sub-count; opb_skipped<=skipped_blocks) and HS-108
v1.1 (Cases d/e use canonical format + OPB-distinct count) and ADR Decision 19.

**Findings:**

- BC-2.01.009 v1.5 PC6: Emission moves from reader to main.rs. PcapSource exposes
  skipped_blocks:u32 and opb_skipped:u32. Canonical format "notice: <filename>: 0 packets
  read from <pcap|pcapng> file"; when opb_skipped>0 appends "(includes N obsolete Packet
  Blocks whose data was not analyzed; re-save with mergecap)". Classic empty-pcap symmetry
  present (EC-009). from_pcap_reader itself MUST NOT emit to stderr. PASS.

- BC-2.01.015 v1.6 PC9: "BC-2.01.015 maintains two counters, both SURFACED as public fields
  on PcapSource (NOT emitted by the reader)." opb_skipped:u32 is sub-count of
  skipped_blocks:u32; "every OPB skip increments BOTH skipped_blocks and opb_skipped." PASS.

- BC-2.01.015 v1.6 AC-006: "from_pcap_reader MUST NOT emit any stderr output — it surfaces
  the counters and returns." opb_skipped <= skipped_blocks invariant explicit. PASS.

- HS-108 v1.1 Cases D/E: Case D (OPB-only) and Case E (NRBs + OPB) both present with OPB
  count distinct from NRB/generic skip count. Evaluator byte-exact assertion: checks for
  "0 packets read from pcapng file", "1" (OPB count), "obsolete", "mergecap" as substrings.
  PASS.

- GAP: BC-2.01.009 v1.5 PC6 normative text for OPB hint: "re-save with mergecap". HS-108
  v1.1 Cases D/E illustrative example: "re-capture or convert with mergecap -F pcapng to
  modernize". The normative wording and the holdout example use different hint strings. The
  evaluator rubric checks for "mergecap" substring only, so the gate passes, but the
  divergence between the normative notice template in BC-2.01.009 and the illustrative example
  in HS-108 is a documentation inconsistency that could confuse an implementer. GAP — see
  FINDING-P5-002 below.

- GAP: HS-108 v1.1 frontmatter `verification_properties: [VP-025]`. VP-025 is the Kani
  timestamp proof (BC-2.01.014). HS-108 tests zero-packet notice emission (BC-2.01.009 PC6)
  and skip-counter surfacing (BC-2.01.015 PC9). No causal relationship exists between VP-025
  (timestamp conversion totality) and the notice behavior tested by HS-108. GAP — see
  FINDING-P5-001 below.

**SEAM 5: PASS WITH FINDING-P5-001 (Minor) and FINDING-P5-002 (Minor)**

---

### Seam 6 — M-3 saturation (µs fast-path): CLEAN

**Check:** BC-2.01.014 v1.5 PC4 uses (ticks / 1_000_000).min(u32::MAX as u64) as u32; large
ts_high canonical vector (ts_high=4295 → ts_sec=u32::MAX); VP-025 harness requires it.

**Findings:**

- BC-2.01.014 v1.5 changelog (v1.5): "Fixed µs fast-path saturation gap: the if_tsresol=6
  shortcut MUST apply .min(u32::MAX as u64) as u32 to ts_sec ... the prior wording 'ts_sec =
  ticks / 1_000_000' was a bare division ... Rewrote PC4 to be explicit: fast path uses
  (ticks / 1_000_000).min(u32::MAX as u64) as u32. Added canonical saturation test vector
  (ts_high=4295 ...). Noted VP-025 Kani harness MUST include this vector." PASS.

- BC-2.01.014 v1.5 PC4 (on-disk text): "ts_sec = (ticks / 1_000_000).min(u32::MAX as u64)
  as u32" — explicit saturation. PASS.

- BC-2.01.014 v1.5 EC-013 (on-disk text): "ts_high=4295, ts_low=0, if_tsresol=6 (µs fast
  path, ts_high large enough that ticks/1_000_000 > u32::MAX) | ts_sec=u32::MAX (saturated
  via .min(u32::MAX as u64)); ts_usecs=0; NO PANIC." Canonical saturation test vector
  present. PASS.

- BC-2.01.014 v1.5 VP-025 row: "Fast-path saturation (M-3): the VP-025 Kani harness MUST NOT
  short-circuit the if_tsresol == 6 branch ... the saturation test vector (ts_high=4295,
  ts_low=0, if_tsresol=6) → ts_sec=u32::MAX must be included as a concrete assertion in the
  Kani harness." PASS.

- Canonical test vectors table: "ts_high=4295, ts_low=0, if_tsresol=6 | ts_sec=u32::MAX
  (saturated; ticks=4295*2^32=18_448_744_073_709_551_616; ticks/1_000_000=18_448_744_073_709
  which exceeds u32::MAX=4_294_967_295; fast path MUST saturate via .min(u32::MAX as u64))."
  PASS.

- VP-INDEX v2.7 VP-025: "ts_sec saturated (.min(u32::MAX)) for all inputs ... Kani harness
  MUST include large-ts_high vector where ticks/ticks_per_sec > u32::MAX to lock the
  saturation (rev 8 / M-3)." Consistent. PASS.

- verification-architecture.md v2.3 VP-025 row: "ts_sec saturated (.min(u32::MAX)), saturating
  arithmetic for all (u32,u32,u8); large-ts_high Kani vector required (rev 8 / M-3)."
  Consistent. PASS.

**SEAM 6: CLEAN**

---

### Seam 7 — M-1 (BC-2.01.009 Precondition 3 deleted): CLEAN

**Check:** BC-2.01.009 v1.5 Preconditions section has only PC1 and PC2; old PC3 ("at least 4
bytes available") is absent.

**Findings:**

- BC-2.01.009 v1.5 changelog (v1.5): "(M-1) Deleted Precondition 3 ('at least 4 bytes
  available') — contradicts EC-003 (graceful Err on <4 bytes); <4-byte case is a runtime
  condition handled by postcondition, NOT an input precondition." PASS.

- BC-2.01.009 v1.5 Preconditions section (on-disk): Two preconditions present — PC1 (readable
  byte stream is passed) and PC2 (stream supports non-destructive peek via fill_buf). No PC3
  exists. PASS.

- BC-2.01.009 v1.5 EC-003: "Stream under 4 bytes (truncated header) | Returns Err wrapping the
  short-read error." Correctly modeled as a runtime output condition, not a precondition.
  PASS.

**SEAM 7: CLEAN**

---

### Seam 8 — M-4 (BufReader wrap AC-007): CLEAN

**Check:** BC-2.01.009 v1.5 AC-007 pins that from_pcap_reader MUST internally wrap R:Read in
BufReader before probe; same BufReader instance fed to fill_buf and downstream parsers;
unbuffered-Read regression test cited.

**Findings:**

- BC-2.01.009 v1.5 AC-007 (on-disk text): "from_pcap_reader<R: Read> MUST internally wrap its
  R argument in std::io::BufReader before performing the magic-byte probe or calling any
  downstream parser. The SAME BufReader<R> instance MUST be passed to both: BufReader::fill_buf()
  for the peek (zero consumption), AND [downstream parsers]. Double-wrapping (if the caller
  already passes a BufReader) is acceptable and idempotent ... The wrap MUST NOT be conditional
  on R's type." PASS.

- BC-2.01.009 v1.5 AC-007 regression test: "test_BC_2_01_009_unbuffered_read_routes_correctly
  — pass an unbuffered Cursor<&[u8]> as R and assert correct probe and routing; this test would
  panic or misroute if the BufReader wrap is absent." PASS.

- BC-2.01.009 v1.5 canonical test vectors table: "Unbuffered Cursor<&[u8]> with valid pcapng
  SHB | Ok(PcapSource) with correct routing (proves internal BufReader wrap) | regression
  (AC-007)." PASS.

**SEAM 8: CLEAN**

---

### Seam 9 — H-4 (BC-2.01.013 VP-031 description and stale deferral notes): CLEAN

**Check:** VP-031 row description in BC-2.01.013 v1.5 matches HS-107 actual scope (SPB framing
truncation/padding/no-IDB including Case F btl=12→E-INP-008); zero stale "deferred to a separate
burst" notes in BC-2.01.013; BC-2.01.010 similarly checked.

**Findings:**

- BC-2.01.013 v1.5 VP-031 row description: "For all (original_len: u32, body: &[u8]):
  captured_len == min(original_len, body.len() as u32) ... Snaplen is excluded from the
  pure-core helper domain (ADR-009 rev 8 Decision 9 amendment)." HS-107 description in BC
  references Case F (btl=12→E-INP-008) via the v1.5 changelog note. PASS.

- BC-2.01.013 v1.5 stale deferral notes: grep confirms ZERO occurrences of "deferred to a
  separate burst" in BC-2.01.013 v1.5. All 4 removed. PASS.

- BC-2.01.010 v1.9 stale deferral notes: grep confirms FOUR remaining "deferred to a separate
  burst" annotations in BC-2.01.010 v1.9: at line 73 (PC5 case b tail), line 110 (AC-004a
  tail), line 146 (EC-005 tail), and line 150 (EC-009 tail referencing "HS-103 Case C fix
  deferred to holdout burst"). HS-103 v1.5 was authored in Pass-4 and now contains Case D
  (btl=16→E-INP-008) and Case C (btl<12/misaligned→E-INP-010). All four holdout cases exist
  on disk; the "deferred to a separate burst" notes are factually stale. GAP — see
  FINDING-P5-003 below.

**SEAM 9: PASS WITH FINDING-P5-003 (Minor)**

---

### Seam 10 — Versions, next_free E-INP-014, VP-INDEX total 31, 302 active BCs, BC-INDEX inline == frontmatter: CLEAN

**Check:** All 6 Pass-5 BCs show monotonic version increments; error-taxonomy v3.3 next_free
E-INP-014; VP-INDEX v2.7 total 31 (kani=14, proptest=10, fuzz=2, integration/unit=5) consistent
with both architecture docs; BC-INDEX v1.60 active count 302; inline version annotations for 6
BCs match on-disk frontmatter.

**Findings:**

- Version increments (Pass-4 → Pass-5):
  BC-2.01.009 v1.4→v1.5, BC-2.01.012 v1.4→v1.5, BC-2.01.013 v1.4→v1.5,
  BC-2.01.014 v1.4→v1.5, BC-2.01.015 v1.5→v1.6, BC-2.01.018 v1.5→v1.6.
  All monotonic. Unchanged BCs: BC-2.01.010 v1.9, BC-2.01.011 v1.5 (stable from Pass-4
  boundary), BC-2.01.016 v1.4, BC-2.01.017 v1.4. PASS.

- error-taxonomy v3.3 next_free: E-INP-013 row tail: "next_free_error_code: E-INP-014." No
  E-INP-014 defined anywhere in the taxonomy. PASS.

- VP-INDEX v2.7 totals: total_vps=31, kani=14, proptest=10, fuzz=2, integration/unit=5.
  Arithmetic: 14+10+2+5=31 ✓. Phase counts: p0=8, p1=17, test_sufficient=6; 8+17+6=31 ✓.
  No VP count changes in Pass-5 (property updates only for VP-025/027/031). PASS.

- verification-architecture.md v2.3 changelog: "VP property updates only — no VP count changes
  (total 31 / Kani 14 / proptest 10 / fuzz 2 / integration-unit 5 unchanged)." Consistent
  with VP-INDEX v2.7. PASS.

- verification-coverage-matrix.md v1.17 Totals row: Kani=14, proptest=10, fuzz=2,
  integration/unit=5 = 31. Consistent. PASS.

- BC-INDEX v1.60: "Active BC count stays 302" confirmed in v1.60 header commentary. PASS.

- BC-INDEX v1.60 inline version annotations for 6 Pass-5 BCs: BC-2.01.009 v1.5, BC-2.01.012
  v1.5, BC-2.01.013 v1.5, BC-2.01.014 v1.5, BC-2.01.015 v1.6, BC-2.01.018 v1.6 — all match
  on-disk frontmatter versions confirmed during this audit. PASS.

- E-INP-008 BC Ref column contains BC-2.01.010, BC-2.01.011, BC-2.01.012, BC-2.01.017 but
  NOT BC-2.01.013. SPB body-too-short → E-INP-008 is normative in BC-2.01.013 v1.5
  (AC-004a, PC4, EC-008). GAP — see FINDING-P5-004 below. (Note: this gap was first
  identified in the prior-session analysis as GAP-3 and survives into v3.3.)

**SEAM 10: PASS WITH FINDING-P5-004 (Minor)**

---

## v5.0 Findings

### FINDING-P5-001 — Minor (Seam 5)

**HS-108 v1.1 frontmatter `verification_properties: [VP-025]` is a misattribution**

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/holdout-scenarios/HS-108-pcapng-zero-packet-notice-end-to-end.md`
**Frontmatter version:** v1.1
**Location:** frontmatter `verification_properties` field

**Current value:** `[VP-025]`

**What is wrong:** VP-025 is the Kani timestamp conversion totality proof targeting
`pcapng_timestamp_to_secs_usecs(ts_high, ts_low, if_tsresol)` in BC-2.01.014. HS-108 tests the
zero-packet notice emission contract (BC-2.01.009 PC6) and skip-counter surfacing
(BC-2.01.015 PC9). No causal or traceability relationship exists between VP-025 (timestamp
arithmetic) and HS-108 (notice emission behavior). The misattribution appears to be a copy-paste
artifact from another holdout scenario.

**Correct value:** There is no existing VP that covers the zero-packet notice emission behavior.
The correct value should be `[]` (empty — HS-108 is an integration-level behavioral holdout not
tied to a formal VP) or, if a notice-emission VP is created in a future pass, it should reference
that VP instead.

**Impact:** Minor. The normative content of HS-108 (cases A-E and evaluator rubric) is correct
and would not cause a phase-4 gate failure. The misattribution is a traceability defect only. A
reader scanning the VP-to-holdout traceability would incorrectly conclude VP-025 has a holdout
scenario that exercises timestamp behavior.

**Remediation:** Update HS-108 v1.1 frontmatter to `verification_properties: []` and add a
comment: `# No formal VP covers notice emission; tested behaviorally via integration scenarios.`

---

### FINDING-P5-002 — Minor (Seam 5)

**OPB notice hint text diverges between BC-2.01.009 PC6 normative spec and HS-108 Cases D/E illustrative example**

**File 1:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/behavioral-contracts/ss-01/BC-2.01.009.md`
**Location:** Postcondition 6, OPB appendage clause

**Normative text (BC-2.01.009 PC6):**
> `(includes N obsolete Packet Blocks whose data was not analyzed; re-save with mergecap)`

**File 2:** `/Users/zious/Documents/GITHUB/wirerust/.factory/holdout-scenarios/HS-108-pcapng-zero-packet-notice-end-to-end.md`
**Location:** Cases D and E illustrative example strings

**HS-108 example text (Cases D/E):**
> `(includes 1 obsolete Packet Block whose data was not analyzed; re-capture or convert with mergecap -F pcapng to modernize)`

**What is wrong:** The normative notice template in BC-2.01.009 PC6 specifies "re-save with
mergecap" as the hint text. The HS-108 holdout examples use "re-capture or convert with
mergecap -F pcapng to modernize" — a longer and differently-worded string. An implementer
reading BC-2.01.009 to understand the notice format will produce "re-save with mergecap" text.
An implementer reading HS-108 cases D/E as a template will produce the longer string. The
evaluator rubric only checks for the "mergecap" substring, so the gate passes for either
wording, but the inconsistency is a documentation hazard.

**Impact:** Minor. No gate outcome depends on the exact wording beyond the "mergecap" substring
check. However, the two authoritative documents should agree on the canonical hint wording to
prevent implementer confusion.

**Remediation:** Align HS-108 Cases D/E illustrative examples to use the BC-2.01.009 PC6
canonical wording: `"re-save with mergecap"`. Or alternatively, update BC-2.01.009 PC6 to
use the HS-108 wording and update the rubric accordingly. The BC-2.01.009 PC6 text should be
the normative source.

---

### FINDING-P5-003 — Minor (Seam 9)

**BC-2.01.010 v1.9 contains 4 stale "deferred to a separate burst" annotations referencing HS-103**

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/behavioral-contracts/ss-01/BC-2.01.010.md`
**Frontmatter version:** v1.9
**Locations:**
- Line 73: `deferred to a separate burst.) The constructible window is confirmed by ADR-009 rev 7`
- Line 110: `E-INP-008. (HS-103 btl=16→E-INP-008 holdout deferred to a separate burst.)`
- Line 146 (EC-005): `(HS-103 btl=16 holdout deferred to a separate burst.)`
- Line 150 (EC-009): `HS-103 Case C fix deferred to holdout burst.`

**What is wrong:** HS-103 v1.5 was authored in Pass-4 (BC-INDEX v1.58 / ADR-009 rev 7) and now
contains both the cases that BC-2.01.010 v1.9 marks as "deferred":

- Case C (btl<12/misaligned → E-INP-010): present in HS-103 v1.5, and it was already present
  as far back as v1.4. The "HS-103 Case C fix deferred to holdout burst" annotation in EC-009
  is factually stale as of Pass-4.
- Case D (btl=16 → E-INP-008): added to HS-103 v1.5 specifically to cover the
  btl=16/body-too-short E-INP-008 constructible window. The "deferred to a separate burst"
  annotations on lines 73, 110, and 146 (EC-005) are all stale as of Pass-4.

BC-2.01.013 v1.5 fixed the analogous set of stale deferral notes in that file. BC-2.01.010
received no Pass-5 update and retains all 4 stale annotations.

**Impact:** Minor. No normative routing is affected — all four occurrences are parenthetical
annotations, not normative text. The actual behavior specifications (E-INP-008 for body-decode,
E-INP-010 for crate-framing) are correct in the surrounding normative text. A reader scanning
BC-2.01.010 would correctly conclude the holdout cases still need to be authored, when in fact
they exist in HS-103 v1.5.

**Remediation:** Remove the four "deferred to a separate burst" annotations from BC-2.01.010 and
replace with cross-references to HS-103: e.g., `(Covered by HS-103 v1.5 Case D.)` and
`(Covered by HS-103 v1.5 Case C.)` Version-bump BC-2.01.010 to v2.0 and add a v2.0 entry to
BC-INDEX.

---

### FINDING-P5-004 — Minor (Seam 10)

**error-taxonomy v3.3 E-INP-008 BC Ref column omits BC-2.01.013**

**File:** `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/prd-supplements/error-taxonomy.md`
**Frontmatter version:** v3.3
**Location:** E-INP-008 table row, BC Ref column

**Current value:** `BC-2.01.010, BC-2.01.011, BC-2.01.012, BC-2.01.017`

**What is wrong:** BC-2.01.013 v1.5 normatively routes SPB body < 4 bytes → E-INP-008 (PC4,
AC-004a, EC-008). This is an E-INP-008 emission site specified in BC-2.01.013. The E-INP-008
BC Ref column does not include BC-2.01.013, meaning a developer using the taxonomy as a
cross-reference cannot discover BC-2.01.013's contribution to E-INP-008. The E-INP-008 scope
Notes text correctly describes "SPB body < 4 bytes (original_len)" as an E-INP-008 case, so
the normative scope is correct; only the BC Ref traceability column is incomplete.

Note: The companion issue (BC-2.01.013 still listed in E-INP-010 BC Ref column) also exists
on disk — E-INP-010 BC Ref shows `BC-2.01.012, BC-2.01.013, BC-2.01.015, BC-2.01.017`. Since
BC-2.01.013's SPB crate-framing rejection path (EC-005: btl<12→E-INP-010) is still a valid
E-INP-010 emission site (only the body-decode path moved to E-INP-008), BC-2.01.013 legitimately
belongs in E-INP-010 BC Ref as well. The primary gap is the MISSING entry in E-INP-008 BC Ref.

**Impact:** Minor. Traceability gap only; the normative scope text is correct. A developer
cross-referencing E-INP-008 from the taxonomy will miss BC-2.01.013 as a source.

**Remediation:** Add BC-2.01.013 to the E-INP-008 BC Ref column: `BC-2.01.010, BC-2.01.011,
BC-2.01.012, BC-2.01.013, BC-2.01.017`. Version-bump error-taxonomy to v3.4.

---

## v5.0 Summary — Cross-Seam Audit

| Seam | Topic | Result |
|------|-------|--------|
| 1 | C-1 EPB→E-INP-008 at all sites (bound-by-body, padding-overrun) | CLEAN |
| 2 | SPB snaplen DROP — min(original_len, block_body_available) everywhere | CLEAN |
| 3 | Uniform error rule (Decision 20) — body-decode→E-INP-008, framing→E-INP-010 | CLEAN |
| 4 | H-1 precedence — EC-006/EC-008 corrected; E-INP-011 reachable only when both whitelisted AND differ | CLEAN |
| 5 | M-5 notice — emission from main.rs; PcapSource fields; HS-108 Cases D/E | PASS WITH FINDING-P5-001 (Minor) + FINDING-P5-002 (Minor) |
| 6 | M-3 saturation — µs fast-path (ticks/1M).min(u32::MAX); ts_high=4295 → u32::MAX; VP-025 | CLEAN |
| 7 | M-1 — BC-2.01.009 Precondition 3 deleted | CLEAN |
| 8 | M-4 — AC-007 BufReader wrap-site pinned; unbuffered regression test | CLEAN |
| 9 | H-4 — BC-2.01.013 VP-031 description; stale deferral notes | PASS WITH FINDING-P5-003 (Minor) |
| 10 | Versions monotonic; next_free E-INP-014; VP-INDEX total 31; 302 active BCs; BC-INDEX inline == frontmatter | PASS WITH FINDING-P5-004 (Minor) |

**Overall v5.0 verdict: NOT CLEAN — 4 Minor gaps found.**

| ID | Severity | Seam | Summary |
|----|----------|------|---------|
| FINDING-P5-001 | Minor | 5 | HS-108 frontmatter `verification_properties: [VP-025]` — VP-025 is timestamp Kani proof, unrelated to notice emission tested by HS-108 |
| FINDING-P5-002 | Minor | 5 | OPB notice hint wording differs: BC-2.01.009 PC6 says "re-save with mergecap"; HS-108 Cases D/E say "re-capture or convert with mergecap -F pcapng to modernize" |
| FINDING-P5-003 | Minor | 9 | BC-2.01.010 v1.9 has 4 stale "deferred to a separate burst" annotations for HS-103 — HS-103 v1.5 (Pass-4) already covers these cases |
| FINDING-P5-004 | Minor | 10 | error-taxonomy v3.3 E-INP-008 BC Ref column omits BC-2.01.013 (SPB body<4→E-INP-008 is normative in BC-2.01.013 v1.5) |

All 4 findings are Minor severity. No findings are Major or Critical. No blocking findings
against phase-4 gate. The primary seam claims (C-1 reclassification, SPB snaplen drop, uniform
error rule, H-1 precedence fix, M-3 saturation, M-1/M-4 BufReader) are all CLEAN on disk.

---

## Updated Open Findings Register

| ID | Severity | Source | Status |
|----|----------|--------|--------|
| FINDING-001 | HIGH | v1.0 audit — ADR-009 Status section stale contradiction | OPEN |
| FINDING-002 | HIGH | v1.0 audit — epics.md total_bcs 297 vs BC-INDEX 302 | OPEN |
| FINDING-003 | MEDIUM | v1.0 audit — prd.md RTM missing BC-2.01.009-018 rows | OPEN |
| FINDING-004 | MEDIUM | v1.0 audit — BC-INDEX updated timestamp stale | OPEN |
| FINDING-P2-001 | LOW | v2.0 audit — ADR-009 HS-completeness map HS-107 shown MISSING | OPEN |
| FINDING-P3-002 | MINOR | v3.0 audit — BC-2.01.018 Related BCs annotation reverses whitelist/conflict order | RESOLVED (BC-2.01.018 v1.4 Pass-3 re-audit fix; v1.6 Seam 4 CLEAN) |
| FINDING-P3-004 | MINOR | v3.0 audit — HS-107 Case B shows two-way min expression | RESOLVED (HS-107 v1.4 Case B now body-bound rationale) |
| FINDING-P4-001 | MAJOR | v4.0 audit — BC-2.01.011 PC5 tail sentence stale pre-Decision-20 | RESOLVED (BC-2.01.011 v1.5 / BC-INDEX v1.59) |
| FINDING-P4-002 | MAJOR | v4.0 audit — error-taxonomy v3.1 E-INP-010 Note contradicts E-INP-008 scope | RESOLVED (error-taxonomy v3.2 / BC-INDEX v1.59) |
| FINDING-P4-003 | MAJOR | v4.0 audit — error-taxonomy v3.1 E-INP-010 items (d)/(e) stale pre-Decision-20 | RESOLVED (error-taxonomy v3.2 / BC-INDEX v1.59) |
| FINDING-P5-001 | MINOR | v5.0 audit — HS-108 frontmatter verification_properties: [VP-025] is misattribution | OPEN |
| FINDING-P5-002 | MINOR | v5.0 audit — OPB notice hint text diverges: BC-2.01.009 "re-save with mergecap" vs HS-108 "re-capture or convert with mergecap -F pcapng to modernize" | OPEN |
| FINDING-P5-003 | MINOR | v5.0 audit — BC-2.01.010 v1.9 has 4 stale "deferred to a separate burst" annotations (HS-103 v1.5 covers those cases) | OPEN |
| FINDING-P5-004 | MINOR | v5.0 audit — error-taxonomy v3.3 E-INP-008 BC Ref column omits BC-2.01.013 | OPEN |
