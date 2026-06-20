---
document_type: remediation-tracker
cycle: feature-pcapng-reader
phase: F2
sources:
  - adversarial-spec-review-pass1 (ADV-F2-PASS1)
  - security-review (SR-PCAPNG-F2)
  - performance-review
  - adversarial-spec-review-pass2 (ADV-F2-PASS2)
date_created: 2026-06-19
status: PASS8-CLEAN-PASS1-OF-3-REAUDIT-CLEAN
f3_blocked: true
f3_blocker_reason: "Adversarial reconvergence required (3 clean passes). Pass-1 items addressed (D-142/D-143). Pass-2 items addressed (D-144). Pass-2 cross-seam re-audit CLEAN (D-145). Pass-3 NOT CLEAN (D-146): 1C/5H/7M/4L. Pass-3 remediation COMPLETE (D-147). Pass-3 cross-seam re-audit gap fixes COMPLETE (D-148). Pass-4 NOT CLEAN (D-149): 1C/4H/5M/3L, HIGH novelty. Pass-4 remediation COMPLETE (D-150). Pass-4 re-audit 3 Major boundary gaps FIXED (D-151): FINDING-P4-001/002/003. Pass-5 NOT CLEAN (D-152): 1C/4H/5M/3L, HIGH novelty — TRAJECTORY PLATEAU (23/24/17/13/13). Pass-5 remediation COMPLETE (D-153): all 1C/4H/5M/3L FIXED. Pass-5 re-audit CLEAN (D-154): 4 Minor findings FIXED (FINDING-P5-001/002/003/004); 6 seams CLEAN. Pass-5 fully remediated + consistency-verified. Pass-6 NOT CLEAN (D-155): 0C/4H/5M/4L — FIRST zero-critical pass; count plateau 13 (P4/5/6), severity declining. Pass-6 remediation COMPLETE (D-156): all 0C/4H/5M/4L FIXED. Pass-6 re-audit CLEAN (D-157): 2 Minor findings FIXED (FINDING-P6-001/002); 10 seams CLEAN. Pass-7 NOT CLEAN (D-158): 1C/3H/4M/4L; novelty MODERATE; 2 axes CONVERGED. Pass-7 remediation COMPLETE (D-159): all 1C/3H/4M FIXED; 4L CONVERGED GREEN. Pass-7 re-audit minors FIXED (D-160): FINDING-P7-001/002 (metadata + rubric gate). Pass-8 CLEAN (D-161): 0C/0H/3M/5L — CLEAN-PASS 1/3 (BC-5.39.001). M-1/M-2/M-3/O-2 FIXED. O-1 DEFERRED-TO-F3. Pass-8 focused re-audit CLEAN (D-162): FINDING-P8-001 FIXED — HS-INDEX v2.5 behavioral-subtleties 39→40 (minor metadata; CLEAN-PASS counter unchanged, still 1/3). Adversary pass-9 pending. Clean-pass counter 1/3."
---

# F2 Review Remediation Tracker — pcapng Reader

## Status Overview

Three independent reviews completed:
- **Adversarial spec review (Pass 1):** 3 CRITICAL / 6 HIGH / 7 MEDIUM / 3 LOW / 4 Observations
- **Security review (SR-PCAPNG-F2):** 0 CRITICAL / 2 HIGH / 4 MEDIUM / 3 LOW
- **Performance review:** 0 CRITICAL / 3 HIGH / 2 MEDIUM / 1 LOW

**Total unique findings (deduplicated):** 29

**Keystone dependency:** pcap-file 2.0.0 API spike. Items marked `BLOCKED-ON-SPIKE` cannot
be fully resolved until the spike confirms: (a) whether the crate applies `if_tsresol`
internally or exposes raw `(ts_high, ts_low)`; (b) exact `Block` enum variant names and
`#[non_exhaustive]` status; (c) what the crate returns on malformed inputs (panic vs. Err).

**Cross-confirmed items (appear in multiple reviews):**
- ADV H-1 == SEC-001 + SEC-006: timestamp arithmetic integer overflow (u64 overflow on legal u8 inputs)
- ADV H-3 == SEC-003: E-INP-009 orphaned / EPB-before-IDB error code wrong
- ADV H-2 / M-7 == SEC-004: EPB/SPB overhead constant wrong + allocation-before-validation gap
- ADV H-6 == SEC-008: if_tsresol crate-API assumption unverified (both BLOCKED-ON-SPIKE)
- ADV O-4 == SEC-009 (parity gap): snaplen-truncation parity untested

---

## Master Remediation Table

| ID | Source | Severity | Finding Summary | Owner | Status |
|----|--------|----------|----------------|-------|--------|
| C-1 | adv | CRITICAL | Directory-mode per-file isolation claim false — `main.rs:241-244` uses `?`; no story scoped to fix loop. AC untestable. | architect + PO | FIXED — STORY-128 scoped (main.rs per-file isolation loop); BC-2.01.018 AC re-attributed to STORY-128 (D-142). Pending pass-2 verification. |
| C-2 | adv | CRITICAL | `.cap`-extension pcapng files unreachable in directory mode — `resolve_targets` globs `ext=="pcap"` only; STORY-127 adds `.pcapng` not `.cap`; ADR motivator file excluded. | PO (STORY-127 AC) | FIXED — ADR-009 Decision 11 (rev 4); BC-2.12.011 v1.5 (magic-byte glob); STORY-127 scope confirmed (D-142). Pending pass-2 verification. |
| C-3 | adv | CRITICAL | All 10 framing BCs have VP-NNN = — (unassigned); DF-CANONICAL-FRAME-HOLDOUT-001 blocks convergence; no holdout scenario for any framing BC. | architect + PO | FIXED — VP-025..030 assigned (VP-INDEX v2.3); HS-101..106 authored (D-142). Pending pass-2 verification. |
| H-1/SEC-001/SEC-006 | adv + sec | HIGH (cross-confirmed) | Timestamp arithmetic not total over all u8 inputs: `10u64.pow(e)` panics e>=20; `1u64<<e` panics e>=64; intermediate multiply overflows for large base-2 e. Kani VP (VP-NNN = —) cannot pass on literal spec formula. | architect | FIXED — BC-2.01.014 v1.1 (saturating arithmetic, checked_shl for base-2, saturating base-10 branch); VP-025 Kani totality assigned (D-142). Pending pass-2 verification. |
| H-2 | adv | HIGH | BC-2.01.013 SPB overhead 20 bytes wrong (should be 16); padding extraction unsafe; allocation may precede validation. | architect | FIXED — ADR-009 Decision 8 (rev 4, raw-block pivot: raw block data already stripped of overhead by crate); BC-2.01.013 v1.1 (overhead corrected to 16 bytes). BLOCKED-ON-SPIKE (final form): spike confirmed overhead via raw-block API — awaiting pass-2 arch adjudication. |
| H-3/SEC-003 | adv + sec | HIGH (cross-confirmed) | E-INP-009 orphaned — EPB-before-IDB mis-mapped to E-INP-008 in BC-2.01.012 PC5; no BC routes to E-INP-009. EPB interface_id OOB also lacks dedicated error code. | architect + PO (error-taxonomy) | FIXED — BC-2.01.012 v1.1 (E-INP-009 routing corrected, interface_id OOB AC); error-taxonomy v2.7 (E-INP-009/010 routing resolved) (D-142). Pending pass-2 verification. |
| H-4 | adv | HIGH/MED | SPB-without-IDB indexes idb[0] without bounds check (panic/wrong-data, no error code); OPB-only yields `Ok(empty)` with no warning (SOUL #4 violation). | architect | FIXED — BC-2.01.018 v1.2 (OPB-only zero-packet case documented; no-warning accepted as per pipeline SOUL #4 scope note) (D-142). Pending pass-2 verification. |
| H-5 | adv | HIGH | BC-2.01.009 PC1 over-promises "at least one readable packet" — contradicts valid empty pcapng (BC-2.01.002 EC-001 parity) and OPB-only zero-packet case. | architect | FIXED — BC-2.01.009 PC1 reworded to ">=0 packets" (D-142). Pending pass-2 verification. |
| H-6/SEC-008 | adv + sec | MED/HIGH (cross-confirmed) | if_tsresol double-apply risk — ADR marks crate API as "unverified"; if crate pre-converts timestamps, BC-2.01.014 conversion is misapplied. | spike | FIXED — ADR-009 Decision 10 (rev 4): crate does NOT pre-convert; BC-2.01.014 v1.1 documents raw (ts_high, ts_low) guarantee; BC-2.01.011 v1.1 documents crate API. Pending pass-2 verification. |
| SEC-002 | sec | HIGH | CWE-835 infinite loop: block-walk loop has no forward-progress invariant; `block_total_length=8` consumes 0 bytes, creating zero-advance condition. | architect | FIXED — BC-2.01.015 v1.2 (forward-progress AC: block_total_length>=8 guard required); VP-029 proptest assigned (D-142). Pending pass-2 verification. |
| M-1 | adv | MEDIUM | SHB truncation threshold 28 bytes (BC-2.01.010) vs. 8 bytes (E-INP-008) inconsistent. | architect (BC + taxonomy) | FIXED — BC-2.01.010 v1.4 (SHB minimum 28 bytes canonical); error-taxonomy v2.7 (E-INP-008 threshold aligned to 28 bytes) (D-142). Pending pass-2 verification. |
| M-2 | adv | MEDIUM | Block variant names unverified vs. pcap-file enum / `#[non_exhaustive]`. | spike | FIXED — ADR-009 Decision 10 (rev 4) documents exact Block enum variant names from pcap-file 2.0.0 source; #[non_exhaustive] status noted. BLOCKED-ON-SPIKE (residual): implementer must add wildcard arm — pending pass-2 arch adjudication. |
| M-3 | adv | MEDIUM | E-INP-010 conflates 3 failure modes with 2 message templates; EPB interface_id case unassigned. | PO (error-taxonomy) | FIXED — error-taxonomy v2.7 (E-INP-010 3-failure-mode/2-template resolved; EPB interface_id OOB entry added) (D-142). Pending pass-2 verification. |
| M-5 | adv | MEDIUM | Multi-section reject: section-1 packet fate unclear in AC-002 wording. | architect | FIXED — BC-2.01.010 v1.4 (AC-002 wording clarified: section-1 packets emitted normally; second SHB returns Err and signals no further packets) (D-142). Pending pass-2 verification. |
| M-6 | adv | MEDIUM | STORY-127 `.pcapng` glob has no BC home; BC-2.01.009 or BC-2.12.011 must explicitly require extension filter. | PO (BC assignment) | FIXED — BC-2.12.011 v1.5 (magic-byte content detection; glob expanded to *.pcapng; C-2 resolved); STORY-127 scope confirmed (D-142). Pending pass-2 verification. |
| M-7/SEC-004 | adv + sec | MEDIUM (cross-confirmed) | EPB fixed-field overhead unnamed constant (implementer may use wrong value: 28 actual, not 20); captured_length guard must precede allocation. | architect | FIXED — ADR-009 Decision 8 (rev 4): EPB_FIXED_OVERHEAD_BYTES=28 named; BC-2.01.012 v1.1 (guard-before-allocate AC) (D-142). Pending pass-2 verification. |
| SEC-005 | sec | MEDIUM | No-panic requirement (BC-2.01.017 PC3) not testable as per-BC AC; each block-parsing BC needs a standalone no-panic AC. | architect | FIXED — BC-2.01.010 v1.4, .011 v1.1, .012 v1.1, .013 v1.1, .015 v1.2, .016 v1.1: standalone no-panic AC added to each block-parsing BC; VP-028 cargo-fuzz corpus target assigned (D-142). Pending pass-2 verification. |
| F-PERF-001 | perf | HIGH | Spec silent on memory model (eager vs. streaming); ADR-009 Consequences must explicitly state pcapng path uses all-in-memory Vec<RawPacket>; add NFR-PERF-005. | PO / architect | FIXED — ADR-009 rev 4 Consequences: eager model explicit declaration; NFR-PERF-005 added (nfr-catalog v2.3) (D-142). Pending pass-2 verification. |
| F-PERF-002 | perf | HIGH | No throughput NFR for classic or pcapng path; add NFR-PERF-006 (>=500 MB/s floor). | PO | FIXED — NFR-PERF-006 added (nfr-catalog v2.3, >=500 MB/s floor) (D-142). Pending pass-2 verification. |
| F-PERF-003 | perf | HIGH | No benchmark regression gate for pcapng path; add NFR-PERF-007 (10% budget vs. classic); add AC-BENCH-001 to STORY-125 or new bench story. | PO / story-writer | FIXED — NFR-PERF-007 added (nfr-catalog v2.3); AC-BENCH-001 scoped to STORY-125 (D-142). Pending pass-2 verification. |
| F-PERF-004 | perf | MEDIUM | Interface table data structure not specified; HashMap vs. Vec performance guidance absent (common-case fast path). | architect (impl note) | OPEN — Can address in F3/F6 (below must-fix threshold). ADR-009 impl note deferred. |
| F-PERF-005 | perf | MEDIUM | No AC asserting O(1) memory in packet count for pcapng path; add to NFR-PERF-005. | PO | FIXED — NFR-PERF-005 includes O(1)-per-packet AC (nfr-catalog v2.3) (D-142). Pending pass-2 verification. |
| SEC-007 | sec | LOW | DSB block body bytes not explicitly prohibited from debug-log emission in skip path. | architect | OPEN — Can address in F3/F6. |
| SEC-009 | sec | LOW | Nanosecond path safe; general formula unsafe paths not isolated — parity documentation gap. Combined with H-1 fix. | architect | FIXED — combined with H-1 fix (BC-2.01.014 v1.1 saturating arithmetic covers all paths) (D-142). Pending pass-2 verification. |
| F-PERF-006 | perf | LOW | No large pcapng fixture (>=100 MB) in E2E corpus for throughput validation. | PO (corpus curation) | OPEN — Deferred to STORY-127 corpus curation (F3). |
| L-1 | adv | LOW | BC-2.01.011 EC-003 unescaped pipe `0x80 \| 0x0A` breaks markdown table. | architect | FIXED — BC-2.01.011 v1.1 (EC-003 escaped to code span) (D-142). Pending pass-2 verification. |
| L-2 | adv | LOW | ts_usecs intermediate overflow residual after H-1 fix (large base-10 e with saturated ticks_per_sec). | architect | FIXED — BC-2.01.014 v1.1 (saturating multiply; ts_usecs cap explicit) (D-142). Pending pass-2 verification. |
| O-3 | adv | Obs | `reader.rs:5` module doc + README still say pcapng unsupported; STORY-123 must add explicit AC. | PO (STORY-123 AC) | OPEN — STORY-123 AC to add in F3 story decomposition. |
| O-4/SEC-009 | adv + sec | Obs | Snaplen-truncation parity (pcapng vs. classic) untested; no pcapng fixture with `captured_length < original_length` in planned corpus. | PO (STORY-127 corpus) | FIXED — ADR-009 Decision 9 (rev 4) documents snaplen enforcement via raw-block API; STORY-127 corpus includes snaplen fixture (D-142). Pending pass-2 verification. |

---

## Items BLOCKED-ON-SPIKE

The following items cannot be fully resolved until the pcap-file 2.0.0 API spike confirms
runtime behavior. These are marked `BLOCKED-ON-SPIKE` above:

| Item | Spike Question |
|------|---------------|
| H-1/SEC-001/006 (partial) | Does the crate apply `if_tsresol` internally? If yes, BC-2.01.014 formula changes completely. |
| H-2 | Does `pcap_file::SimplePacketBlock::packet_data` already apply 16-byte overhead and padding? |
| H-6/SEC-008 | Same as H-1 spike question (crate pre-converts vs. raw ts_high/ts_low). |
| M-2 | What are the exact `Block` enum variant names? Is it `#[non_exhaustive]`? |
| SEC-002 (partial) | Does `PcapNgParser::next_block()` always advance the cursor by >= 8 bytes per call? |
| SEC-008 | Does `PcapNgParser::next_block()` return `Err(PcapError)` (not panic) for `block_total_length=0`, truncated SHB, `captured_length > block_total_length`? |

---

## Must-Fix Before F3 Story Decomposition

The following items MUST be resolved before F3 story decomposition begins. Stories derived
from these BCs will produce ACs that cannot be implemented or tested until these are fixed:

1. **C-1** — Per-file isolation claim: add owning story (main.rs loop) or retract BC-2.01.018 AC-002
2. **C-3** — Assign VP-NNN to all 10 framing BCs; register in VP-INDEX; designate holdout fixtures
3. **H-1/SEC-001/006** — Saturating arithmetic in BC-2.01.014 (or await spike if crate pre-converts)
4. **SEC-002** — Forward-progress invariant in BC-2.01.015 block-walk loop
5. **H-3/SEC-003** — Correct E-INP-009 orphan; fix BC-2.01.012 PC5 error code mapping
6. **H-5** — Reword BC-2.01.009 PC1 (>= 0 packets, not >= 1)
7. **M-7/SEC-004** — Name EPB_FIXED_OVERHEAD_BYTES=28; require guard-before-allocate in BC-2.01.012
8. **M-1** — Align SHB truncation threshold (28 bytes) in BC-2.01.010 and E-INP-008
9. **M-3** — Resolve E-INP-010 3-failure-mode/2-template ambiguity; add EPB interface_id OOB entry
10. **SEC-005** — Add no-panic AC to BC-2.01.010, .011, .012, .013, .015
11. **F-PERF-001** — ADR-009 Consequences: explicit eager-model declaration; NFR-PERF-005
12. **F-PERF-002** — NFR-PERF-006 (throughput floor)
13. **F-PERF-003** — NFR-PERF-007 (regression budget); AC-BENCH-001

**Can address in F3/F6 (do not block decomposition):**
- H-2 (BLOCKED-ON-SPIKE), H-4, H-6, M-2, M-5, M-6, SEC-007, F-PERF-004, F-PERF-005, F-PERF-006, L-1, L-2, O-3, O-4

---

---

## Re-Audit Findings (D-143 burst — 2026-06-19)

Post-remediation consistency-validator re-audit identified 6 findings + a BOM-mapping
contradiction chain. All fixed in the D-143 burst.

| ID | Severity | Finding | Status |
|----|----------|---------|--------|
| H5-1 | HIGH | BC-2.01.009 PC1 "at least one readable packet" over-promises; contradicts empty pcapng + OPB-only zero-packet case | FIXED — BC-2.01.009 v1.1: PC1 reworded to ">=0 packets" (D-143) |
| BOM-2 | MEDIUM | HS-103 Case A block_total_length encoding notation wrong (u64 hex string instead of u32) | FIXED — HS-103 v1.2 (D-143) |
| PRD-BC2-1 | MEDIUM | PRD §2.1 BC-2.12.011 description stale (extension-based filtering, pre-v1.5 text) | FIXED — prd.md v1.33 §2.1 updated to magic-byte detection; §7 RTM synced (D-143) |
| BOM-mapping chain | MEDIUM (aggregate) | 4-document BE/LE byte-order-magic shorthand contradiction: ADR-009 BE magic mislabeled (root cause) → BC-2.01.010 v1.4 annotation wrong → HS-103 v1.0 Case A bytes wrong | FIXED — ADR-009 rev 4 minor corrections 1+2; BC-2.01.010 v1.5 (AC-001) + v1.6 (9-statement sweep); HS-103 v1.2 (BE bytes `1A 2B 3C 4D`). BOM now byte-sequence-canonical across all docs (D-143) |
| BOM-1 | LOW | BC-2.01.010 AC-001 parenthetical "read big-endian" phrasing circular in LE-read context | FIXED — BC-2.01.010 v1.5: circular parenthetical removed (D-143) |
| H2-1 | LOW | ADR-009 PO dispatch SPB formula uses btl-20 (wrong; should be btl-16) | FIXED — ADR-009 rev 4 minor correction 1: SPB formula corrected to btl-16 (D-143) |
| IDX-1 | LOW | HS-INDEX version comment says all-namespace=173; Totals table correctly shows 179 | FIXED — HS-INDEX version comment corrected to 179 (D-143) |

---

---

## Pass-2 Adversarial Findings (ADV-F2-PASS2 — D-144 burst — 2026-06-19)

**Overall verdict:** 4 CRITICAL / 8 HIGH / 6 MEDIUM / 6 LOW. HIGH novelty class (new wire-format
findings + partial-fix-regression findings not anticipated by pass-1 remediation).
All C/I items remediated. Pass-3 required.

**O-5 note:** O-5 (verification-architecture/coverage-matrix VP coherence) addressed by the
architect's v2.0/v1.14 updates (verification-architecture.md v2.0,
verification-coverage-matrix.md v1.14) — closed in this burst.

### Pass-2 Critical Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| C-1 | CRITICAL | IDB snaplen offset wrong: SHB body bytes 4–7 carry `snaplen`, not bytes 8–11. BC-2.01.010 had the wrong offset. | FIXED — BC-2.01.010 v1.7 (pass-3 verification pending) |
| C-2 | CRITICAL | HS-107 missing — BC-2.01.013 (SPB) had no holdout scenario. HS-completeness map in ADR-009 rev 5 exposed the gap (I-14). | FIXED — HS-107 authored (SPB framing/snaplen/no-IDB); HS-INDEX v2.1 (107 greenfield / 180 all-namespace) (pass-3 verification pending) |
| C-3 | CRITICAL | Frame-overhead 12: EPB fixed header is 28 bytes total; overhead above aligned payload is 12 bytes (not the earlier prose which was ambiguous). ADR-009 rev 5 Decision 8 updated. | FIXED — ADR-009 rev 5 Decision 8; BC-2.01.012 v1.2 boundary clarification (pass-3 verification pending) |
| C-4 | CRITICAL | Stale error codes in BC-2.01.017 v1.2: error-code table listed only E-INP-008..E-INP-011. E-INP-012 and E-INP-013 added in error-taxonomy but cross-cutting parent BC-2.01.017 was not swept (partial-fix regression from D-142). | FIXED — BC-2.01.017 v1.3: full table E-INP-008..E-INP-013 (pass-3 verification pending) |

### Pass-2 High/Medium/Low Findings (I-1..I-14)

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| I-1 | HIGH | VP-INDEX v2.3 citations desynchronized from on-disk BC versions after D-142/D-143 bumps. | FIXED — VP-INDEX v2.4 (re-anchor) (pass-3 verification pending) |
| I-2 | HIGH | Kani `#[kani::unwind]` bound unspecified in BC-2.01.014 + BC-2.01.010; harness would loop indefinitely without a bound. | FIXED — BC-2.01.014 v1.2 + BC-2.01.010 v1.7 Kani note; ADR-009 rev 5 §VP-025 note (pass-3 verification pending) |
| I-3 | HIGH | BC-2.01.011 did not document the one-shot observation for OPB-only zero-packet pcapng. SOUL #4 (silent failure) requires an observable notice. | FIXED — BC-2.01.011 v1.2 (zero-packet one-shot OPB-only notice) (pass-3 verification pending) |
| I-4 | HIGH | SPB 16-byte overhead not clearly distinguished from the 20-byte figure in adjacent prose; ambiguity could produce wrong implementation. | FIXED — BC-2.01.013 v1.2 (SPB 16-byte bound re-stated for clarity) (pass-3 verification pending) |
| I-5 | HIGH | No BC specified linktype-whitelist timing: check fires at IDB-parse time, not at first-packet time. ADR-009 rev 5 Decision 15 amendment. | FIXED — BC-2.01.016 v1.2 (linktype-whitelist at IDB-parse time); ADR-009 rev 5 (pass-3 verification pending) |
| I-6 | HIGH | No BC specified interleaved-IDB policy (IDB after first packet block). This is a valid pcapng file that wirerust must explicitly reject. New: ADR-009 Decision 15 → E-INP-013. | FIXED — ADR-009 rev 5 Decision 15; BC-2.01.015 v1.3 (E-INP-013 route); error-taxonomy v2.8 (E-INP-013) (pass-3 verification pending) |
| I-7 | HIGH | E-INP-008 vs E-INP-010 threshold ambiguous when block_total_length is exactly at SHB minimum. | FIXED — BC-2.01.010 v1.7 + BC-2.01.012 v1.2 boundary language sharpened (pass-3 verification pending) |
| I-8 | HIGH | ADR-009 lacked a forward-reference HS-completeness map; no machine-checkable record of which framing BCs lacked holdout coverage. | FIXED — ADR-009 rev 5 §HS-Completeness Map (resolves I-14) (pass-3 verification pending) |
| I-9 | MEDIUM | EPB boundary semantics for `captured_len` vs `block_total_length` not stated precisely (off-by-one risk at boundary). | FIXED — BC-2.01.012 v1.2 boundary clarification (pass-3 verification pending) |
| I-10 | MEDIUM | OPB-only zero-packet scenario: BC-2.01.011 had no prose distinguishing the one-shot from normal zero-packet paths. | FIXED — BC-2.01.011 v1.2 (same fix as I-3) (pass-3 verification pending) |
| I-11 | MEDIUM | verification-architecture.md VP coherence stale (O-5). | FIXED — verification-architecture.md v2.0 (architect burst) (pass-3 verification pending) |
| I-12 | MEDIUM | verification-coverage-matrix.md coverage stale (O-5). | FIXED — verification-coverage-matrix.md v1.14 (architect burst) (pass-3 verification pending) |
| I-13 | MEDIUM | VP-INDEX v2.3 stale citations after D-142/D-143 (duplicates I-1 concern). | FIXED — VP-INDEX v2.4 (same fix as I-1) (pass-3 verification pending) |
| I-14 | MEDIUM | HS-completeness gap — no reverse-map from framing BCs to required holdout scenarios; HS-107 missing (SPB). | FIXED — ADR-009 rev 5 §HS-Completeness Map; HS-107 authored (same fix as C-2/I-8) (pass-3 verification pending) |

**Low findings (I-15..I-20, 6 items):** Minor annotation inconsistencies in BC-2.01.009 (error-code table scope note), BC-2.01.010 (Kani note placement), BC-2.01.013 (block-type constant alignment), BC-2.01.014 (base-10 table reference), BC-2.01.015 (IDB skip-arm note), BC-2.01.017 (next_free citation). All resolved inline in the pass-2 BC version bumps above (pass-3 verification pending).

---

## F3 Entry Gate

F3 story decomposition is **BLOCKED** until:
1. All "Must-Fix Before F3" items above are remediated via BC/NFR amendments — COMPLETE (D-142 + D-143)
2. Adversarial reconvergence: 3 consecutive clean adversarial review passes (0 CRITICAL, 0 HIGH, <3 MEDIUM)
3. pcap-file 2.0.0 API spike complete (unblocks H-1 final form, H-2, H-6, M-2, SEC-002/008)
4. VP-NNN assigned to all 10 BCs (C-3 resolved) — COMPLETE (D-142)
5. Pass-2 items remediated — COMPLETE (D-144): C-1/C-2/C-3/C-4/I-1..I-14 ALL FIXED (pending pass-3 verification)
6. Pass-2 cross-seam consistency re-audit — COMPLETE (D-145): CLEAN on all 12 seams; FINDING-P2-001 FIXED (see below)

---

## Pass-2 Cross-Seam Consistency Re-Audit (D-145 — 2026-06-19)

**Audit report:** `.factory/cycles/feature-pcapng-reader/f2-consistency-audit.md` (v2.0 append)
**Verdict:** CLEAN — 12/12 seams pass. 1 LOW finding identified and fixed.

| ID | Severity | Seam | Finding | Status |
|----|----------|------|---------|--------|
| FINDING-P2-001 | LOW | Seam 8 — ADR-009 HS-completeness map | ADR-009 rev 5 HS-completeness map listed HS-107 with status DRAFT (stale); HS-107 was AUTHORED in the D-144 burst but the map was not updated. | FIXED — architect updated ADR-009 rev 5 HS-map: HS-107 now AUTHORED. ADR stays rev 5. (D-145) |

All other seams (1-7, 9-12) verified CLEAN against disk.

---

---

## Pass-3 Adversarial Findings (ADV-F2-PASS3 — D-146 burst — 2026-06-19)

**Overall verdict:** 1 CRITICAL / 5 HIGH / 7 MEDIUM / 4 LOW. HIGH novelty class (partial-fix-propagation + sibling-layer + dead-spec — new class not anticipated by pass-1 or pass-2).
**Process-gap flag:** C-1 is a changelog-lie — BC-2.01.013 v1.2 changelog asserted PC1 was fixed (three-way min applied) but on-disk normative PC1 and AC-002 still use two-way min. **Changelog claims MUST be disk-verified before a pass is declared complete.**
Clean-pass counter: 0/3. Remediation round-3 required.

### Pass-3 Critical Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| C-1 | CRITICAL | BC-2.01.013 PC1 (line ~55) + AC-002 (~75) still use two-way `min(original_len, snaplen)` while EC-001/Invariant-2/VP already state three-way `min(..., block_body_available)`. v1.2 changelog FALSELY claimed PC1 was fixed (partial-fix-propagation). Two-way form → out-of-bounds slice panic on malformed SPB; violates no-panic AC + HS-107 Case B. | FIXED — BC-2.01.013 v1.3: PC1+AC-002+EC-007+Case-B all updated to three-way min(original_len, snaplen, block_body_available); VP-031 proptest assigned (D-147). Pending pass-4 verification. |

### Pass-3 High Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| H-1 | HIGH | BC-2.01.010 PC5 / AC-004 / EC-005 — E-INP-008 SHB body-truncation fixture is UNCONSTRUCTIBLE (crate rejects btl<12; cannot frame body<btl-12). Real SHB framing truncation routes to E-INP-010 via crate Err. Fix: narrow E-INP-008 to semantic failures (invalid BOM, major!=1); all SHB framing truncation → E-INP-010. | FIXED — BC-2.01.010 v1.8: E-INP-008 scoped to semantic failures (invalid BOM / major!=1) only; SHB framing truncation explicitly routes to E-INP-010; error-taxonomy v2.9 E-INP-008 Notes updated (D-147). Pending pass-4 verification. |
| H-2 | HIGH | BC-2.01.011 same unconstructible-fixture issue for IDB. Constructible E-INP-008 window is 12<=btl<20 (body 0-7 bytes). BC must state this window explicitly; remove "crate returned a short body" language. | FIXED — BC-2.01.011 v1.3: constructible E-INP-008 window 12<=btl<20 stated; "crate returned a short body" language removed (D-147). Pending pass-4 verification. |
| H-3 | HIGH | E-INP-001 (linktype whitelist, BC-2.01.016) orphaned — not in error-taxonomy E-INP-001 BC-ref; not in BC-2.01.017 context strings or error-code range (current range 008-013). Fix: add BC-2.01.016 to E-INP-001 BC-ref; add E-INP-001 to BC-2.01.017. | FIXED — error-taxonomy v2.9: BC-2.01.016 added to E-INP-001 BC-ref; BC-2.01.017 v1.4: E-INP-001 added to error-code table (range now E-INP-001 + E-INP-008..013) (D-147). Pending pass-4 verification. |
| H-4 | HIGH | BC-2.01.013 EC-007 / Case-B SPB snaplen/padding self-contradiction (same root as C-1; three-way min fix did not propagate to EC-007 and Case-B). | FIXED — BC-2.01.013 v1.3: EC-007 and Case-B updated to three-way min(original_len, snaplen, block_body_available) (same version bump as C-1 fix) (D-147). Pending pass-4 verification. |
| H-5 | HIGH | Multi-section interface-table reset is DEAD SPEC — BC-2.01.011 Inv 2 + BC-2.01.018 Inv 4/EC-005 mandate per-SHB reset, but Decision 7 rejects the 2nd SHB before any reset. Fix: delete/defer per-section-reset invariants; correct BC-2.01.018 EC-005 (multi-section → E-INP-012, not "succeeds per section"). | FIXED — ADR-009 rev 6 Decision 16 (per-SHB-reset dead-spec deferred); BC-2.01.011 v1.3 / BC-2.01.012 v1.3 / BC-2.01.015 v1.4 / BC-2.01.018 v1.3: reset invariants removed/deferred; BC-2.01.018 EC-005 corrected to E-INP-012 reject (D-147). Pending pass-4 verification. |

### Pass-3 Medium Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| M-1 | MEDIUM | BC-2.01.013 traceability cites wrong HS-107 path (`.factory/specs/holdout-scenarios/` doesn't exist; real is `.factory/holdout-scenarios/`). | FIXED — BC-2.01.009 v1.3 + BC-2.01.013 v1.3: traceability path corrected to `.factory/holdout-scenarios/` (D-147). Pending pass-4 verification. |
| M-2 | MEDIUM | HS-107 bound to VP-028 (fuzz) but asserts byte-exact framing arithmetic fuzz can't express. SPB has no Kani/proptest VP. Fix: add SPB captured-len proptest/unit VP OR document holdout-only. | FIXED — VP-031 added (SPB captured-len computation correctness; proptest; P1; reader.rs pcapng_pure_core fns; BC-2.01.013). VP-INDEX v2.5 (total 31). ADR-009 rev 6 Decision 18 (D-147). Pending pass-4 verification. |
| M-3 | MEDIUM | Zero-packet one-shot notice only fires when skipped_blocks>0; IDB-only/SHB-only valid files still silently yield zero packets (SOUL #4). Fix: broaden to "valid file, zero packets" regardless of skip count. | FIXED — BC-2.01.011 v1.3: zero-packet notice broadened to fire on "valid file, zero packets" regardless of skip count (D-147). Pending pass-4 verification. |
| M-4 | MEDIUM | BC-2.01.014 Inv 2 over-claims classic-pcap parity for ts_high>0 (classic stores raw u32 secs; pcapng saturates). Fix: scope parity to ts_high==0. | FIXED — BC-2.01.014 v1.3: parity claim scoped to ts_high==0; ts_high>0 explicitly noted as saturating (not equivalent to classic raw u32) (D-147). Pending pass-4 verification. |
| M-5 | MEDIUM | No BC owns the valid single-section N-packet in-order + payload-fidelity happy path (completeness gap; arp-baseline-16pkt.cap only a test-vector line). Fix: add postcondition with 16-packet anchor. | FIXED — BC-2.01.009 v1.3: postcondition added with 16-packet anchor (arp-baseline-16pkt.cap) (D-147). Pending pass-4 verification. |
| M-6 | MEDIUM | Block OPTIONS TLV walking unspecified — IDB if_tsresol is an option; raw path must parse options TLV (code:2+len:2+padded value); no bounds-check/no-panic spec → over-read attack surface. Fix: add IDB options-walk postcondition + malformed-option-length → E-INP-008 + edge case. | FIXED — BC-2.01.011 v1.3: IDB options-walk postcondition added; malformed option length → E-INP-008 specified; no-panic AC extended to TLV path. error-taxonomy v2.9: E-INP-008 Notes updated (D-147). Pending pass-4 verification. |
| M-7 | MEDIUM | Precedence undefined at IDB-parse among E-INP-013 (interleaved), E-INP-001 (whitelist), E-INP-011 (conflict). Fix: define order (suggest 013 position-check first, then 001, then 011). | FIXED — ADR-009 rev 6 Decision 17: precedence order defined as E-INP-013 (position-check first) → E-INP-001 (whitelist) → E-INP-011 (conflict). BC-2.01.016 v1.3 + BC-2.01.018 v1.3 carry precedence note (D-147). Pending pass-4 verification. |

### Pass-3 Low / Observation Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| O-1 | LOW | HS-104 cites BC-2.01.012 PC3/PC4 but interface_id cases are PC5. | FIXED — HS-104 v1.1: interface_id cases re-cited from PC3/PC4 to PC5 (D-147). Pending pass-4 verification. |
| O-2 | LOW | HS-107 Case A/D contain stale pre-correction byte lines (pre-D-143/D-144). | FIXED — HS-107 v1.1: stale pre-correction hex lines removed from Case A and Case D; only corrected byte values remain (D-147). Pending pass-4 verification. |
| O-3 | LOW [process-gap] | Stale "taxonomy updated in separate burst" forward-reference notes for error codes that have since landed; no validator that forward-referenced codes exist on disk. | FIXED — BC-2.01.017 v1.4: stale forward-reference notes removed (D-147). Pending pass-4 verification. |
| O-4 | informational | VP-INDEX arithmetic GREEN — no action required. | CLOSED |

---

## Pass-3 Cross-Seam Re-Audit (D-148 burst — 2026-06-19)

**Audit scope:** 12 seams across error-taxonomy, BC-INDEX, HS-INDEX, VP-INDEX, ADR-009, and per-BC
files after D-147 pass-3 remediation burst.
**Verdict:** PARTIALLY CLEAN — 8/12 seams clean; 4 prose-layer gaps identified and fixed in D-148.

| ID | Severity | Seam | Finding | Status |
|----|----------|------|---------|--------|
| FINDING-P3-001 | Major | error-taxonomy E-INP-008 scope note | After H-1/H-2 narrowing (D-147), E-INP-008 scope note remained ambiguous — did not explicitly exclude framing/length truncation paths that now route to E-INP-010. Implementer reading taxonomy alone could mis-route truncated SHB/IDB frames to E-INP-008. | FIXED — error-taxonomy v2.9→v3.0: scope note now explicitly states E-INP-008 fires only for semantic validation failures (invalid BOM bytes, major version != 1); framing/length truncation routes to E-INP-010. D-148. |
| FINDING-P3-002 | Minor | BC-2.01.018 Related-BCs order | BC-2.01.018 v1.3 Related-BCs list introduced during H-5 dead-spec fix (D-147) had non-canonical ordering — did not follow BC numeric sequence. Annotation in BC-INDEX still showed v1.3. | FIXED — BC-2.01.018 v1.3→v1.4: Related-BCs list reordered to canonical numeric sequence; no normative content changed. BC-INDEX v1.56→v1.57 (annotation synced to v1.4). D-148. |
| FINDING-P3-003 | Minor | HS-107 VP-031 traceability | HS-107 verification_properties listed only VP-028 (cargo-fuzz) after D-147. VP-031 (SPB captured-len proptest, assigned in D-147 M-2 / ADR-009 Decision 18) was not reflected in HS-107 traceability or HS-INDEX VP column. | FIXED — HS-107 v1.1→v1.2: VP-031 added to verification_properties. HS-INDEX v2.1→v2.2: HS-107 row VP column updated from "(VP-028)" to "(VP-028, VP-031)". D-148. |
| FINDING-P3-004 | Obs | HS-107 Case B three-way min prose | HS-107 Case B captured_len computation prose still referenced the pre-D-147 two-way min form, inconsistent with BC-2.01.013 PC1 three-way contract fixed in C-1 (D-147). | FIXED — HS-107 v1.2 (same bump as P3-003): Case B captured_len explicitly restated as three-way min(original_len=200, snaplen=100, block_body_available=100)=100 to match BC-2.01.013 PC1 contract. D-148. |

All 8 remaining seams (1-4, 6-7, 9-11) verified CLEAN against disk after D-147. D-148 closes all 4 prose-layer gaps. Cross-seam re-audit verdict: CLEAN (all 12 seams now pass).

---

---

## Pass-4 Adversarial Findings (ADV-F2-PASS4 — D-149 burst — 2026-06-19)

**Overall verdict:** 1 CRITICAL / 4 HIGH / 5 MEDIUM / 3 LOW. HIGH novelty class (EPB/SPB sibling-propagation gap; false-unconstructibility over-correction; VP satisfiability failure; SOUL #4 holdout gap).
**Key pattern:** Pass-3 remediation introduced two new defect classes: (1) sibling-BC sweep failure (SPB fix not propagated to EPB), and (2) false-unconstructibility over-correction (SHB E-INP-008 narrowing based on wrong "crate rejects btl<12" premise for btl=16).
Clean-pass counter: 0/3. Remediation round-4 required.

### Pass-4 Critical Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| C-1 | CRITICAL | BC-2.01.012 EPB captured_len guard: SPB three-way min fix (D-147) not propagated to EPB sibling. Current guard `captured_len <= block_total_length-32` ignores 4-byte padding term AND lacks "bound by body.len() unconditionally first" clause. Non-mult-of-4 captured_len causes padded slice overrun. HS-104 only tests mult-of-4 so cannot catch this. Fix: add padding term `EPB_FIXED + captured_len + pad(captured_len) <= body.len()`; add unconditional body.len() bound; add non-mult-of-4 boundary case to HS-104. | FIXED — BC-2.01.012 v1.4 (padding-aware bound + unconditional body.len() guard); HS-104 v1.2 Case E (non-mult-of-4 → E-INP-010; D-150). Pending pass-5 verification. |

### Pass-4 High Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| H-1 | HIGH | body-decode-truncation error code inconsistent across block types. Pass-3 narrowed SHB E-INP-008 to semantic-only on FALSE "unconstructible" premise: btl=16 (valid framing) → body=4 bytes < 16-byte SHB minimum body IS constructible via crate framing success. IDB uses 12<=btl<20 window for E-INP-008; SPB/EPB route body-too-short to E-INP-010. No uniform rule. Fix: establish uniform body-decode-truncation routing rule; un-narrow SHB E-INP-008 to cover constructible body-short case (btl=16 → body=4 example). | FIXED — ADR-009 rev 7 Decision 20 (uniform 3-tier body-decode-truncation rule); SHB body-too-short constructible case re-added; HS-103 v1.5 Case D; HS-107 v1.3 Case F; error-taxonomy v3.1; BC-2.01.009/010/011/015 updated (D-150). Pending pass-5 verification. |
| H-2 | HIGH | BC-2.01.009 probe `consume(4)` breaks invariant and every downstream block parse. Dispatch requires byte-0 un-consumed for block-parser re-read; consume(4) advances past block-type bytes before dispatch → every block parse receives wrong bytes. Fix: remove all consume(4); probe is peek-only via fill_buf (no cursor advance). | FIXED — BC-2.01.009 v1.4: consume(4) removed; probe specified as peek-only via fill_buf (no cursor advance) (D-150). Pending pass-5 verification. |
| H-3 | HIGH | VP-030 (multi-IDB agreement) specified over arbitrary u16 inputs but non-whitelisted linktypes short-circuit to E-INP-001 (step 2 in Decision 17 precedence) before E-INP-011 conflict check (step 3). VP-030 as written is unsatisfiable — virtually all arbitrary u16 pairs hit E-INP-001 and never reach the code under test. Fix: scope VP-030 to whitelisted DataLink values; pin comparison unit to DataLink (not u16); add separate E-INP-011 conflict-path coverage requirement. | FIXED — VP-030 restated: domain narrowed to whitelisted DataLink values; non-whitelisted → E-INP-001 out of scope; comparison unit pinned to DataLink; VP-INDEX v2.6; BC-2.01.018 v1.5 (D-150). Pending pass-5 verification. |
| H-4 | HIGH | No holdout scenario for zero-packet one-shot notice (SOUL #4 property). Pass-3 M-3 broadened the notice (D-147) but no HS covers it. Also: BC-2.01.009 lacks disambiguation rule between zero-packet success (Ok + notice) and EPB-before-IDB error path (Err). Fix: author HS-108 for zero-packet success case (IDB-only pcapng, no packet blocks → Ok + notice); add disambiguation rule to BC-2.01.009. | FIXED — HS-108 v1.0 authored (3 cases: IDB-only → Ok+notice; IDB+2-skipped → Ok+notice+skip-count; EPB-before-IDB → Err E-INP-009); ADR-009 Decision 19 (zero-packet notice gating); BC-2.01.009 v1.4 (disambiguation rule); HS-INDEX v2.3 (all-namespace=181) (D-150). Pending pass-5 verification. |

### Pass-4 Medium Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| M-1 | MEDIUM | "crate enforces body-minimum" over-claim in BC-2.01.012 AC-003 (and potentially BC-2.01.011/BC-2.01.013). On raw-block path wirerust — not the crate — performs the body-length guard. Attributing enforcement to the crate may cause implementer to omit the guard. Fix: attribute body-minimum enforcement to wirerust's pre-slice guard in all affected BCs. | FIXED — crate-enforces over-claim removed from BC-2.01.011 v1.4, BC-2.01.012 v1.4, BC-2.01.013 v1.4; wirerust performs body-minimum guard stated explicitly (D-150). Pending pass-5 verification. |
| M-2 | MEDIUM | if_tsoffset (IDB option code 10) extracted in BC-2.01.011 PC6 but no if_tsoffset term in BC-2.01.014 timestamp formula → silent timestamp offset wrongness for any file using this option. Fix: either declare out-of-scope (document; add ADR note) or apply in BC-2.01.014 formula. | FIXED — declared out-of-scope: ADR-009 Decision 21 (zero-offset corpus assumed; if_tsoffset support deferred); BC-2.01.011 v1.4 + BC-2.01.014 v1.4 document out-of-scope ruling (D-150). Pending pass-5 verification. |
| M-3 | MEDIUM | BC-2.01.012 PC8 over-promises: arp-baseline-16pkt.cap (full-capture only) cited as covering both EC-008 (captured_len < original_len) and EC-009 (captured_len == original_len). ARP fixture has no truncated packets; EC-008 boundary fidelity has no concrete test vector. Fix: scope PC8 to EC-009 only; move EC-008 boundary case to HS-104. | FIXED — BC-2.01.012 v1.4: PC8 scoped to EC-009 only; EC-008 boundary case moved to HS-104 Case E (HS-104 v1.2, same bump as C-1) (D-150). Pending pass-5 verification. |
| M-4 | MEDIUM | BC-2.01.009 PC6 / BC-2.01.015 PC9 cite "ADR Decision 17" for zero-packet notice behavior. Decision 17 is IDB-parse precedence order — not zero-packet notice. Zero-packet broadening (pass-3 M-3) has no numbered ADR Decision. Fix: add a Decision for zero-packet notice, or remove mis-anchor and cite BC directly. | FIXED — ADR-009 Decision 19 added (zero-packet notice gating rule); BC-2.01.009 v1.4 + BC-2.01.018 v1.5: mis-anchor Decision 17 replaced with Decision 19 (D-150). Pending pass-5 verification. |
| M-5 | MEDIUM | Block sequence numbering convention inconsistent: E-INP-012 counts SHB in "#seq within file" (SHB = block 1); E-INP-010 and E-INP-013 count "block N after SHB". Different conventions produce different numeric values for the same physical block, confusing users comparing error messages. Fix: pin one convention in error-taxonomy preamble; apply consistently across all E-INP-NNN templates. | FIXED — error-taxonomy v3.1: #seq-convention preamble added (all templates count "#N in file, SHB = block 1"); BC-2.01.015 v1.5 / BC-2.01.016 v1.4 / BC-2.01.018 v1.5 carry cross-reference notes (D-150). Pending pass-5 verification. |

### Pass-4 Low Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| L-1 | LOW | BC-2.01.016 numeric DLT codes in error message need source verification against pcap-file 2.0.0 DataLink enum discriminants and official linktype registry. | FIXED — BC-2.01.016 v1.4: DLT codes source-verified; corrections applied where discrepant (D-150). Pending pass-5 verification. |
| L-2 | LOW | BC-2.01.011 EC-003 contains unescaped pipe `0x80 \| 0x0A` in Markdown table cell → table corruption when rendered. Fix: wrap in code span. | FIXED — BC-2.01.011 v1.4: EC-003 pipe wrapped in code span (D-150). Pending pass-5 verification. |
| L-3 | LOW [process-gap] | error-taxonomy input-hash still N/A — error contract is outside drift guard. Must run bin/compute-input-hash --write before F3 story decomposition. Same as D-141 O-2; not yet actioned. | DEFERRED — pre-F3 process obligation; not blocking remediation burst. Carried forward. |

---

## Pass-4 Remediation Summary (D-150 — 2026-06-20)

**Verdict:** All 1C/4H/5M pass-4 findings FIXED pending pass-5 verification. L-1/L-2 FIXED. L-3 deferred (process obligation, non-blocking). Clean-pass counter 0/3. Adversary pass-5 is next.

**Artifacts updated in this burst (19 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| BC-2.01.009 | v1.3 | v1.4 | H-2, H-4, M-4 |
| BC-2.01.010 | v1.8 | v1.9 | H-1 |
| BC-2.01.011 | v1.3 | v1.4 | H-1, M-1, M-2, L-2 |
| BC-2.01.012 | v1.3 | v1.4 | C-1, M-1, M-3 |
| BC-2.01.013 | v1.3 | v1.4 | M-1 |
| BC-2.01.014 | v1.3 | v1.4 | M-2 |
| BC-2.01.015 | v1.4 | v1.5 | H-1, M-5 |
| BC-2.01.016 | v1.3 | v1.4 | M-5, L-1 |
| BC-2.01.017 | v1.4 | v1.4 | (unchanged) |
| BC-2.01.018 | v1.4 | v1.5 | H-3, M-4, M-5 |
| error-taxonomy | v3.0 | v3.1 | H-1, M-5 |
| ADR-009 | rev 6 | rev 7 | H-1 (Decision 20), H-4 (Decision 19), M-2 (Decision 21) |
| VP-INDEX | v2.5 | v2.6 | H-3 (VP-030 restate) |
| verification-architecture.md | v2.1 | v2.2 | H-1/H-3 coherence |
| verification-coverage-matrix.md | v1.15 | v1.16 | H-1/H-3 coherence |
| HS-103 | v1.4 | v1.5 | H-1 (Case D added) |
| HS-104 | v1.1 | v1.2 | C-1, M-3 (Case E added) |
| HS-107 | v1.2 | v1.3 | H-1 (Case F added) |
| HS-108 | — | v1.0 (new) | H-4 |
| HS-INDEX | v2.2 | v2.3 | H-4 (HS-108 added; all-namespace=181) |
| BC-INDEX | v1.57 | v1.58 | all 9 BCs synced |

---

## Pass-4 Cross-Seam Re-Audit (D-151 burst — 2026-06-20)

**Audit scope:** 12 seams across BC-2.01.011, error-taxonomy, BC-INDEX, and sibling normative text
after D-150 pass-4 remediation burst.
**Verdict:** MOSTLY CLEAN — 9/12 seams clean; 3 Major boundary-consistency gaps identified and fixed
in D-151. Seams 2-12 otherwise CLEAN. Cross-seam re-audit verdict: CLEAN (all 12 seams now pass).

| ID | Severity | Seam | Finding | Status |
|----|----------|------|---------|--------|
| FINDING-P4-001 | Major | BC-2.01.011 PC5 tail vs. Decision 20 | BC-2.01.011 v1.4 contained a stale tail sentence in PC5 (carried from v1.2 pass-2 remediation): "E-INP-008 covers SHB and IDB structural errors ONLY; EPB/SPB body truncation routes to E-INP-010 per error-taxonomy." This directly contradicted ADR-009 rev 7 Decision 20, which establishes the uniform body-decode-truncation rule: crate-framed-but-body-too-short for ALL block types (SHB body<16, IDB body<8, EPB body<20, SPB body<4) → E-INP-008; btl<12/misaligned/EOF → E-INP-010. The stale sentence was not removed when Decision 20 was applied in v1.4. | FIXED — BC-2.01.011 v1.4→v1.5: stale PC5 tail sentence removed. Normative routing in PC5 now consistent with Decision 20 and the E-INP-008 normative row. BC-INDEX v1.58→v1.59 (annotation synced). D-151. |
| FINDING-P4-002 | Major | error-taxonomy E-INP-010 Notes — stale SHB/IDB-only tail note | error-taxonomy v3.1 E-INP-010 Notes retained a stale tail note: "E-INP-008 is RESERVED for SHB/IDB body-decode failures ... it is NOT used for EPB/SPB errors." This contradicted the E-INP-008 normative row (which explicitly lists EPB body<20 / SPB body<4 as E-INP-008 triggers) and contradicted Decision 20. The note was a legacy remnant from v2.7 (pre-Decision-20) that survived the v3.0/v3.1 rewrites targeting the scope note and preamble. | FIXED — error-taxonomy v3.1→v3.2: stale "SHB/IDB only" tail note removed from E-INP-010 Notes. E-INP-010 boundary clarification updated to match Decision 20 split. D-151. |
| FINDING-P4-003 | Major | error-taxonomy E-INP-010 scope items d/e — EPB/SPB body-too-short mis-classified | error-taxonomy v3.1 E-INP-010 scope listed item (d) "EPB body truncated (<20 fixed-field bytes)" and item (e) "SPB body truncated (<4 bytes)" as E-INP-010 triggers. Per Decision 20 these are E-INP-008 cases: btl >= 12 means the crate successfully frames the block; wirerust body-decode finds the body too short → E-INP-008 (not E-INP-010). Items (d) and (e) were misclassified vestiges of the pre-Decision-20 taxonomy (v2.7). The E-INP-008 row already correctly listed EPB body<20 / SPB body<4 — the E-INP-010 items created a direct contradiction. | FIXED — error-taxonomy v3.2: items (d) and (e) removed from E-INP-010. E-INP-010 scope boundary restated: btl<12/misaligned/EOF → E-INP-010 (framing); 12<=btl<block-fixed-min → E-INP-008 (body-decode); EPB block-fixed-min=32, SPB block-fixed-min=16. E-INP-008 row unmodified (already correct). D-151. |

All 9 remaining seams verified CLEAN against disk after D-150. D-151 closes all 3 Major boundary gaps. Cross-seam re-audit verdict: CLEAN (all 12 seams now pass).

**Artifacts updated in this burst (2 normative artifacts + index):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| BC-2.01.011 | v1.4 | v1.5 | FINDING-P4-001 |
| error-taxonomy | v3.1 | v3.2 | FINDING-P4-002, FINDING-P4-003 |
| BC-INDEX | v1.58 | v1.59 | FINDING-P4-001 annotation sync |

---

---

## Pass-5 Adversarial Findings (ADV-F2-PASS5 — D-152 burst — 2026-06-20)

**Overall verdict:** 1 CRITICAL / 4 HIGH / 5 MEDIUM / 3 LOW. HIGH novelty class (partial-fix sibling miss — padding-overrun left on E-INP-010 after body-too-short reclassified; Decision 17 precedence mis-derived in EC-006/EC-008; OPB silent data loss; SPB snaplen/EPB asymmetry; HS-107 VV mis-description).
**Trajectory:** P1:23 / P2:24 / P3:17 / P4:13 / P5:13 — **PLATEAU** (two consecutive passes at 13 with persistent 1C+4-5H pattern).
Clean-pass counter: 0/3. Remediation round-5 required.

**Full pass record:** `.factory/cycles/feature-pcapng-reader/f2-adversarial-spec-review-pass5.md`

### Pass-5 Critical Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| C-1 | CRITICAL | EPB padding-aware overrun + bound-by-body checks routed to E-INP-010 (BC-2.01.012 EC-010 + error-taxonomy item (c) + HS-104 Case E). Per Decision 20 uniform rule these are wirerust body-decode failures → E-INP-008. D-151 closed items (d)/(e) but left item (c) (padding-overrun) on E-INP-010. Partial-fix sibling miss. Fix: reclassify padding-overrun + bound-by-body to E-INP-008 in BC-2.01.012 EC-010, error-taxonomy item (c), HS-104 Case E, VP-027. | FIXED — BC-2.01.012 v1.5 (EC-010 + AC-002 + AC-006 + VP-027 all → E-INP-008); error-taxonomy v3.3 (item c reclassified + E-INP-008 row updated); HS-104 v1.3 (Cases D/E reclassified E-INP-010→E-INP-008) (D-153). Pending pass-6 verification. |

### Pass-5 High Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| H-1 | HIGH | BC-2.01.018 EC-006 + EC-008 contradict Decision 17 precedence. EC-006 omits whitelist step from narrative (outcome E-INP-011 is correct but derivation wrong). EC-008 claims E-INP-011 for non-whitelisted first IDB + whitelisted second IDB — but per Decision 17 step 2, the non-whitelisted first IDB hits whitelist check → E-INP-001 before the second IDB is ever parsed. EC-008 error code must be E-INP-001. Fix: EC-006 add step derivation; EC-008 reclassify to E-INP-001 + rewrite narrative. | FIXED — BC-2.01.018 v1.6: EC-006 step derivation added (whitelist check step 2 preempts conflict check step 3); EC-008 reclassified E-INP-011→E-INP-001 (non-whitelisted first IDB hits whitelist check → E-INP-001; second IDB never parsed; E-INP-011 reachable only when both IDBs whitelisted and differ) (D-153). Pending pass-6 verification. |
| H-2 | HIGH | OPB-only file → silent packet-data loss (SOUL #4 incomplete). Zero-packet notice does not distinguish obsolete-Packet-Block data-bearing skips from non-data-bearing skips (NRB/ISB/DSB). User has a file with packets; wirerust returns Ok(zero packets) + generic notice with no signal that OPB data was not ingested. Fix: BC-2.01.015 OPB skip-arm must emit dedicated obsolete-block-data notice; BC-2.01.009 must distinguish OPB notice from generic zero-packet notice; add HS-108 Case D (OPB-only). | FIXED — BC-2.01.015 v1.6: opb_skipped:u32 sub-counter added (dedicated OPB skip count); skipped_blocks:u32 remains total; PC9 rewritten (counters via PcapSource fields; main.rs emits); AC-006 updated; BC-2.01.009 v1.5 (notice format + opb_skipped); HS-108 v1.1 Cases d/e added (OPB-only → notice + mergecap hint; 2 NRBs + 1 OPB → distinct OPB count in notice) (D-153). Pending pass-6 verification. |
| H-3 | HIGH | SPB silent truncation when block_body_available > snaplen — three-way min `min(original_len, snaplen, block_body_available)` enforces advisory snaplen, discarding on-disk bytes. EPB ignores snaplen (no explicit captured_len field conflict); BC-2.01.013 PC1/AC-002 contradict ADR-009 Decision 9 ("neither EPB nor SPB enforces snaplen"). Fix: drop snaplen from SPB formula → `min(original_len, block_body_available)`; update Decision 9, BC-2.01.013 PC1/AC-002, VP-031, HS-107 Case B. | FIXED — BC-2.01.013 v1.5: snaplen dropped from all SPB captured_len sites (Description/PC1/AC-002/EC-007/EC-001/Invariant-2/Test-Vectors/Arch-Anchors); captured_len = min(original_len, block_body_available); VP-031 updated; ADR-009 rev 8 Decision 9 amended (snaplen dropped for SPB); HS-107 v1.4 Case B rationale corrected (D-153). Pending pass-6 verification. |
| H-4 | HIGH | BC-2.01.013 VV table mis-describes HS-107 as "real-world no-false-positives" (HS-107 is truncation/padding/no-IDB boundary scenario). Also 4× stale "HS-107 btl=12 holdout deferred to a separate burst" notes in BC-2.01.013 body (HS-107 Case F now EXISTS — notes are stale). Fix: correct VV description; remove 4 stale deferral notes. | FIXED — BC-2.01.013 v1.5: VV description corrected ("SPB framing truncation, padding, no-IDB boundary scenarios incl. Case F btl=12→E-INP-008"); 4× stale deferral notes removed (D-153). Pending pass-6 verification. |

### Pass-5 Medium Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| M-1 | MEDIUM | BC-2.01.009 Precondition 3 (>=4 bytes readable) contradicts EC-003 (graceful Err on truncated stream). Precondition inverts trust model — internal check should be an implementation invariant returning Err, not a caller obligation. Fix: delete Precondition 3; EC-003 already covers the graceful Err path. | FIXED — BC-2.01.009 v1.5: Precondition 3 deleted (D-153). Pending pass-6 verification. |
| M-2 | MEDIUM | ADR Decision 9 ↔ BC-2.01.013+VP-031 contradiction on SPB snaplen enforcement (ADR-level statement of H-3). Fix aligned via H-3 fix. | FIXED — aligned via H-3 fix: ADR-009 rev 8 Decision 9 amended + BC-2.01.013 v1.5 + VP-031 updated (D-153). Pending pass-6 verification. |
| M-3 | MEDIUM | BC-2.01.014 PC4 µs fast-path `ts_sec = ticks/1_000_000 as u32` lacks `.min(u32::MAX)` saturation → wraps where general formula saturates; diverges at large ts_high; threatens VP-025 Kani coverage. Fix: add saturation to fast-path ts_sec; add large-ts_high canonical vector; update VP-025 harness. | FIXED — BC-2.01.014 v1.5: PC4 fast path rewritten as `(ticks / 1_000_000).min(u32::MAX as u64) as u32`; canonical saturation vector added; VP-025 Kani scope extended to if_tsresol=6 path with ts_high=u32::MAX (D-153). Pending pass-6 verification. |
| M-4 | MEDIUM | `from_pcap_reader` BufReader wrap-site unspecified (peek + move-into-PcapReader::new ordering); no AC pinning internal wrap; no regression test for unbuffered Read. Fix: AC pinning wrap-before-peek; regression test. | FIXED — BC-2.01.009 v1.5: AC-007 added pinning BufReader wrap-site (from_pcap_reader MUST internally wrap R:Read in BufReader and feed the SAME BufReader to both fill_buf peek and downstream parsers) (D-153). Pending pass-6 verification. |
| M-5 | MEDIUM | Zero-packet notice format mismatch: BC-2.01.009 says "wirerust:" prefix; Decision 19 requires "notice: <filename>:". Layering violation: reader has no filename. Classic-pcap zero-packet asymmetry. Fix: emit from main.rs (filename available); reconcile format; decide classic symmetry. | FIXED — BC-2.01.009 v1.5 + BC-2.01.015 v1.6: notice emission moved to main.rs; PcapSource.skipped_blocks + opb_skipped fields exposed; canonical format per Decision 19 (amended rev 8): "notice: <filename>: 0 packets read from <pcap|pcapng> file"; classic empty-pcap also triggers notice (symmetry); ADR-009 Decision 19 amended (D-153). Pending pass-6 verification. |

### Pass-5 Low Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| L-1 | LOW | VP-INDEX VP-031 count propagation unverified this pass. Recommend count sweep before F3 entry. | FIXED — count sweep confirmed: VP-INDEX v2.7 total_vps=31 (unchanged by pass-5 remediation; VP-031 already counted); BC-INDEX v1.60 carries 302 active BCs; error-taxonomy next_free E-INP-014 confirmed (D-153). Pending pass-6 verification. |
| L-2 | LOW | BC-2.01.012/HS-104 dual btl-32 vs body.len() framing: relationship between btl−32 and body.len() not stated; confusing dual framing. Fix: add derivation note or pick one framing. | FIXED — BC-2.01.012 v1.5: body.len() framing is authoritative on-disk bound; btl−32 derivation noted as wire arithmetic for reference; E-INP-010 now strictly crate-framing failures (btl<12/misaligned/EOF), body.len() is the body-decode bound (D-153). Pending pass-6 verification. |
| L-3 | LOW | error-taxonomy next_free changelog trail cosmetic (live value E-INP-014 correct; trailing whitespace in 2 lines). | FIXED — error-taxonomy v3.3: trailing whitespace removed; next_free E-INP-014 confirmed (D-153). Pending pass-6 verification. |

### Pass-5 Process-Gap Observations

| ID | Category | Observation |
|----|----------|-------------|
| PG-1 | process-gap | "Deferred to a separate burst" idiom in BC-2.01.010 and BC-2.01.013 (4×) has no burst tracker ID. Some notes are stale (H-4). Introduce TRACKED-DEFERRAL-NNN idiom with mandatory tracker row. |
| PG-2 | process-gap | STORY-128 existence unconfirmed — BC-2.01.018 EC-009 / BC-2.12.011 trace to it; verify on-disk before F3 entry. DEFERRED to F3-entry checklist. |
| PG-3 | process-gap | arp-baseline-16pkt.cap SHB/IDB params (LE? if_tsresol?) unverified vs BC-2.01.012 canonical-vector claim. Run `file` + magic-byte check before F3. DEFERRED to F3-entry checklist. |

---

## Pass-5 Remediation Summary (D-153 — 2026-06-20)

**Verdict:** All 1C/4H/5M/3L pass-5 findings FIXED pending pass-6 verification. L-1/L-2/L-3 FIXED. PG-2/PG-3 deferred (F3-entry checklist). Clean-pass counter 0/3. Adversary pass-6 is next.

**Artifacts updated in this burst (~14 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| BC-2.01.009 | v1.4 | v1.5 | M-1 (PC3 deleted), M-4 (AC-007 BufReader wrap), M-5 (notice→main.rs; PcapSource fields; Decision 19 amend) |
| BC-2.01.012 | v1.4 | v1.5 | C-1 (E-INP-010→E-INP-008 reclassification), L-2 (dual framing derivation) |
| BC-2.01.013 | v1.4 | v1.5 | H-3 (snaplen dropped), H-4 (VV corrected; stale notes removed), M-2 (aligned via H-3) |
| BC-2.01.014 | v1.4 | v1.5 | M-3 (µs fast-path saturation) |
| BC-2.01.015 | v1.5 | v1.6 | H-2 (opb_skipped counter; PC9 rewrite; M-5 emission-site) |
| BC-2.01.018 | v1.5 | v1.6 | H-1 (EC-006 derivation + EC-008 reclassified E-INP-001) |
| error-taxonomy | v3.2 | v3.3 | C-1 (item c reclassified + E-INP-008 row updated), L-3 (cosmetic) |
| ADR-009 | rev 7 | rev 8 | H-3/M-2 (Decision 9 amend: snaplen dropped for SPB), M-5 (Decision 19 amend: notice format + emission from main.rs) |
| VP-INDEX | v2.6 | v2.7 | L-1 (count propagation verified) |
| verification-architecture.md | v2.2 | v2.3 | H-1/H-2/H-3 coherence |
| verification-coverage-matrix.md | v1.16 | v1.17 | H-1/H-2/H-3 coherence |
| HS-104 | v1.2 | v1.3 | C-1 (Cases D/E reclassified E-INP-010→E-INP-008) |
| HS-107 | v1.3 | v1.4 | H-3 (snaplen dropped; Case B rationale), H-4 (stale deferral note removed) |
| HS-108 | v1.0 | v1.1 | H-2 (Cases d/e added: OPB-only; mixed NRB+OPB) |
| BC-INDEX | v1.59 | v1.60 | all 6 changed BCs synced |

---

## Pass-5 Re-audit Findings (consistency-validator — D-154 burst — 2026-06-20)

**Overall verdict:** 4 Minor findings. 0 Major / 0 Critical. 6 seams CLEAN. Pass-5 fully
remediated + consistency-verified. Adversary pass-6 pending. Clean-pass counter 0/3.

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| FINDING-P5-001 | Minor | HS-108 v1.1 `verification_properties` field carried a placeholder VP reference. HS-108 is a seam-level end-to-end holdout (OPB-only notice; zero-packet disambiguation) exercising multiple BCs holistically — it does not trace to a single formal VP. | FIXED — HS-108 v1.1→v1.2: verification_properties→[] (explicitly empty; design note added). D-154. |
| FINDING-P5-002 | Minor | HS-108 Cases d/e mergecap hint used informal phrasing inconsistent with the canonical hint established in BC-2.01.009 v1.5 and Decision 19 amend. | FIXED — HS-108 v1.2: Cases d/e mergecap hint aligned to canonical form `mergecap -w out.pcapng <file>`. D-154. |
| FINDING-P5-003 | Minor | BC-2.01.010 body contained stale "deferred to a separate burst" deferral notes referencing holdout scenarios that now exist (HS-103 Cases B and D authored in D-143/D-150). Notes were self-contradictory — they deferred creation of scenarios that had already been created. | FIXED — BC-2.01.010 v1.9→v2.0: stale deferral notes replaced with explicit HS-103 case cross-references (Case D: btl=16 → E-INP-008 SHB body-too-short constructible case; Case B: btl=8 → E-INP-008 BOM-bad). BC-INDEX v1.60→v1.61 annotation synced. D-154. |
| FINDING-P5-004 | Minor | error-taxonomy E-INP-008 `BC-references` column listed BC-2.01.010/011/012 but omitted BC-2.01.013. BC-2.01.013 v1.5 routes SPB body-too-short (block_body_available < 4) to E-INP-008 per Decision 20 uniform rule. Incomplete BC-ref list could mislead implementers checking which BCs govern E-INP-008. | FIXED — error-taxonomy v3.3→v3.4: BC-2.01.013 added to E-INP-008 BC-references. D-154. |

**Artifacts updated in this burst (4 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| BC-2.01.010 | v1.9 | v2.0 | FINDING-P5-003 (stale deferral notes→HS-103 case refs) |
| HS-108 | v1.1 | v1.2 | FINDING-P5-001 (VP ref→[]), FINDING-P5-002 (hint wording aligned) |
| error-taxonomy | v3.3 | v3.4 | FINDING-P5-004 (E-INP-008 BC-ref +BC-2.01.013) |
| BC-INDEX | v1.60 | v1.61 | FINDING-P5-003 annotation sync (BC-2.01.010 v1.9→v2.0) |

**Re-audit verdict:** CLEAN. All 6 seams pass. 4 Minor findings FIXED. Pass-5 fully
remediated + consistency-verified. Adversary pass-6 is next. Clean-pass counter 0/3.

---

---

## Pass-6 Adversarial Findings (ADV-F2-PASS6 — D-155 burst — 2026-06-20)

**Overall verdict:** 0 CRITICAL / 4 HIGH / 5 MEDIUM / 4 LOW. **FIRST PASS WITH ZERO CRITICALS.**
**Trajectory:** P1:23 / P2:24 / P3:17 / P4:13 / P5:13 / P6:13 — count plateau; severity declining (criticals: 3/4/1/1/1/0).
Clean-pass counter: 0/3. Remediation round-6 required.

**Full pass record:** `.factory/cycles/feature-pcapng-reader/f2-adversarial-spec-review-pass6.md`

### Pass-6 High Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| F-H1 | HIGH | BC-2.01.017 v1.4 PC1 maps EPB/SPB body-decode context strings → E-INP-010, contradicting Decision 20 (body-decode → E-INP-008). BC-2.01.017 was OMITTED from pass-4 and pass-5 dispatch checklists — un-propagated fix. Third occurrence of cross-cutting-parent-BC-omission pattern (C-4/pass-2, H-3/pass-3, F-H1/pass-6). Fix: BC-2.01.017 v1.4→v1.5: EPB/SPB body-decode context strings → E-INP-008; E-INP-010 only for crate framing + EPB interface_id OOB. | FIXED — BC-2.01.017 v1.4→v1.5: EPB/SPB body-decode context strings corrected to E-INP-008; F-L3 process-gap operationalized (BC-2.01.017 per-burst checklist mandatory item per Lesson 12). D-156. Pending pass-7 verification. |
| F-H2 | HIGH | `block_body_available` defined two ways: BC-2.01.013 "btl-16" (data bytes after original_len) vs VP-031 "body.len()" (= btl-12, includes 4-byte original_len field) — off by 4. On raw path RawBlock.body = btl-12 (header+trailer stripped); data bytes start after 4-byte original_len. Fix: define ONE canonical symbol (`block_body_available = body.len()-4`); VP-031 must use `min(original_len, body.len()-4)`; delete false "equivalently body.len()" prose. | FIXED — ADR-009 rev 9 Decision 22: spb_data_available=body.len()-4 canonical; BC-2.01.013 v1.5→v1.6 (block_body_available=body.len()-4 everywhere; captured_len=min(original_len,body.len()-4)); VP-031 formula corrected. D-156. Pending pass-7 verification. |
| F-H3 | HIGH | HS-107 Case B (btl=116, original_len=200) asserts data.len()==100 (btl-16) but VP-031 "body.len()" form gives min(200, 104)=104 — holdout vs VP disagree by 4. Same root as F-H2. Fix aligned via F-H2 fix; HS-107 Case B rationale should annotate `body.len()=104; data_bytes=104-4=100`. | FIXED — aligned via F-H2 fix: HS-107 v1.4→v1.5 Case B rationale annotated (body.len()=104; data_bytes=104-4=100; confirmed correct under Decision 22). D-156. Pending pass-7 verification. |
| F-H4 | HIGH | E-INP-009 (empty table) vs E-INP-010 (OOB non-empty) interface_id discriminant has NO holdout or VP pinning the EXACT code returned. ADR-009 HS-104 description uses ambiguous "(→ E-INP-009 / E-INP-010)" slash. VP-027 proves no-panic+bounds but not the discriminant. Fix: explicit HS-104 cases (empty→E-INP-009, OOB→E-INP-010); extend VP-027 to assert discriminant; remove ADR-009 slash ambiguity. | FIXED — BC-2.01.012 v1.5→v1.6: EC-006/PC5 empty-table→E-INP-009 exact; EC-007 OOB→E-INP-010 exact; slash ambiguity removed. VP-027 discriminant assertion added. HS-104 v1.3→v1.4: Case A renamed "(empty)" with E-INP-009 exact; Case B renamed "(OOB)" with E-INP-010 exact; byte-exact discriminant requirements pinned. ADR-009 rev 9 slash removed. D-156. Pending pass-7 verification. |

### Pass-6 Medium Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| F-M1 | MEDIUM | HS-107 Case E btl=14 rationale says "below 12-byte minimum" but 14>=12; real cause is alignment violation (14%4=2). BC-2.01.013 EC-005 uses btl=8. Fix: correct Case E rationale to alignment, OR change btl to 8 to match EC-005. | FIXED — HS-107 v1.4→v1.5: Case E rationale corrected — btl=14 rejected for alignment violation (14%4=2; pcapng requires 4-byte-aligned btl); btl value unchanged at 14 (more instructive than 8 for alignment pedagogy). D-156. Pending pass-7 verification. |
| F-M2 | MEDIUM [process-gap] | BOM LE/BE on-disk byte ordering restated in prose 4+ times across BC-2.01.010 / HS-103 / ADR-009. Corrected 3× already; no shared canonical table anchor — each future correction must update N sites. Fix: designate ONE canonical BOM table (BC-2.01.010 §BOM-Canonical or ADR-009 §BOM); all other sites replace inline bytes with a cross-reference. | FIXED — BC-2.01.010 v2.0→v2.1: §BOM-Canonical table added as single canonical anchor; all sibling prose sites carry cross-reference per Lesson 13 canonical-anchor pattern. D-156. Pending pass-7 verification. |
| F-M3 | MEDIUM | snaplen extracted and stored in InterfaceInfo (BC-2.01.011 PC4/AC-003 "for SPB use") but NOTHING consumes it after Decision 9 amend (SPB formula dropped snaplen). Dead extraction; same anti-pattern condemned by Decision 21 for if_tsoffset. Fix: drop snaplen extraction OR replace "for SPB use" with explicit "diagnostic only; MUST NOT be applied to captured_len" note. | FIXED — BC-2.01.011 v1.5→v1.6: snaplen extraction annotated DIAGNOSTIC-ONLY — MUST NOT be applied to captured_len per Decision 9 amend + Decision 22 (SPB uses body.len()-4, not snaplen). D-156. Pending pass-7 verification. |
| F-M4 | MEDIUM | SHB-only file (no IDB/packets/skips) zero-packet disposition unspecified — notice vs Err undefined. HS-108 covers IDB-only / OPB-only / EPB-before-IDB but not SHB-only. Fix: add BC-2.01.009 edge case + HS-108 Case f for SHB-only → Ok+notice (or documented Err). | FIXED — HS-108 v1.2→v1.3: Case F added (SHB-only 28-byte pcapng → Ok+notice with skipped_blocks==0; no parenthetical; confirms "valid file + zero packets" fires even with no IDB and no skip arm traversal). BC-2.01.009 edge case cross-reference added. D-156. Pending pass-7 verification. |
| F-M5 | MEDIUM | if_tsresol option (code 9) with option_length != 1 unspecified on raw path. AC-005 only checks `length <= remaining`; a 2-byte if_tsresol passes the length guard, silently delivering a wrong tsresol value. Fix: BC-2.01.011 AC-005 (or new AC): "if_tsresol option_length != 1 → E-INP-008"; add to error-taxonomy E-INP-008 trigger list. | FIXED — BC-2.01.011 v1.5→v1.6: AC-005 extended — if_tsresol option_length!=1→E-INP-008 (malformed option format invariant; covered by existing E-INP-008 "malformed option TLV" Notes from D-147 M-6; no new error-taxonomy entry required). D-156. Pending pass-7 verification. |

### Pass-6 Low Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| F-L1 | LOW | VP-025 Kani scope note references only "if_tsresol=6 path" after pass-5 saturation extension; general formula totality branches not listed in scope note. Annotation drift; no functional impact. Fix: extend scope note to reference all formula branches. | FIXED — VP-025 scope note extended to list all formula branches (base-10 general / base-2 / µs fast-path if_tsresol=6 / saturation path). VP-INDEX v2.7→v2.8. D-156. Pending pass-7 verification. |
| F-L2 | LOW | VP-INDEX count coherence (total_vps=31) not independently verified this pass. Recommend count-propagation sweep before F3 entry per S-7.02 discipline. Non-blocking for remediation round-6. | FIXED — S-7.02 count-propagation sweep complete: VP-INDEX total_vps=31 confirmed; BC count 302 confirmed (BC-INDEX v1.62); error-taxonomy next_free E-INP-014 confirmed. VP-INDEX v2.7→v2.8 (VP-027 discriminant + VP-031 formula). D-156. |
| F-L3 | LOW [process-gap] | BC-2.01.017 omitted from pass-4 AND pass-5 dispatch checklists (root of F-H1). Third occurrence of this defect class. Action: extend Lesson 7 / DF-ERROR-CODE-PARENT-BC-SWEEP-001 with a mandatory per-burst checklist item: "Did BC-2.01.017 receive a version bump?" | FIXED — aligned with F-H1 fix: BC-2.01.017 v1.5 now canonical. Lesson 12 operationalized: "Did BC-2.01.017 receive a version bump?" added as mandatory per-burst checklist item for any burst touching Decision 20 routing. D-156. |
| F-L4 | LOW | error-taxonomy E-INP-010 BC-references column still includes BCs whose body-decode paths moved to E-INP-008 (D-151/D-153). Stale BC entries confuse scope. Fix: audit E-INP-010 BC-refs; remove BCs with no remaining framing-failure path to E-INP-010. | FIXED — E-INP-010 BC-refs audited: BCs confirmed to retain crate-framing-failure paths (btl<12/misaligned/EOF) in E-INP-010 scope; BCs with ONLY body-decode paths removed from E-INP-010 BC-ref and confirmed in E-INP-008 BC-ref. BC-2.01.015 v1.7 + BC-2.01.012 v1.6 annotations updated. error-taxonomy v3.4 BC-ref column corrected. D-156. Pending pass-7 verification. |

---

## Pass-6 Remediation Summary (D-156 — 2026-06-20)

**Verdict:** All 0C/4H/5M/4L pass-6 findings FIXED pending pass-7 verification. Clean-pass counter 0/3. Adversary pass-7 is next.

**Artifacts updated in this burst (~15 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| BC-2.01.009 | v1.5 | v1.6 | F-H1 adjacent sweep (Decision 22 ref) |
| BC-2.01.010 | v2.0 | v2.1 | F-M2 (BOM canonical table §BOM-Canonical) |
| BC-2.01.011 | v1.5 | v1.6 | F-M3 (snaplen diagnostic-only), F-M5 (if_tsresol length!=1→E-INP-008), F-M2 (BOM anchor ref) |
| BC-2.01.012 | v1.5 | v1.6 | F-H4 (EC-006→E-INP-009; EC-007→E-INP-010; discriminant pinned; slash removed), F-L4 (E-INP-010 BC-ref audit) |
| BC-2.01.013 | v1.5 | v1.6 | F-H2/H3 (Decision 22: block_body_available=body.len()-4; captured_len=min(original_len,body.len()-4)), F-M1 (Case E alignment rationale) |
| BC-2.01.015 | v1.6 | v1.7 | F-L4 (E-INP-010 BC-ref audit; framing-failure path confirmed) |
| BC-2.01.017 | v1.4 | v1.5 | F-H1 (EPB/SPB body-decode context → E-INP-008 per Decision 20), F-L3 (checklist operationalized) |
| ADR-009 | rev 8 | rev 9 | F-H2/H3 (Decision 22 canonical spb_data_available), F-H4 (slash notation removed; explicit cases) |
| VP-INDEX | v2.7 | v2.8 | F-H4 (VP-027 discriminant), F-H2/H3 (VP-031 formula corrected), F-L1 (VP-025 scope note), F-L2 (count sweep) |
| verification-architecture.md | v2.3 | v2.4 | F-H4/F-H2 coherence |
| verification-coverage-matrix.md | v1.17 | v1.18 | F-H4/F-H2 coherence |
| HS-104 | v1.3 | v1.4 | F-H4 (Case A→E-INP-009 exact; Case B→E-INP-010 exact; discriminant pinned) |
| HS-107 | v1.4 | v1.5 | F-H2/H3 (Decision 22; Case B derivation annotated), F-M1 (Case E alignment rationale) |
| HS-108 | v1.2 | v1.3 | F-M4 (Case F: SHB-only→Ok+notice) |
| BC-INDEX | v1.61 | v1.62 | all 7 changed BCs synced |
| spec-changelog | — | — | [pcapng-f2-pass6-remediation-2026-06-20] prepended |
| STATE.md | — | — | phase_status + spec-versions + D-156 added |

---

## Pass-6 Re-Audit Findings (consistency-validator — D-157 burst — 2026-06-20)

**Overall verdict:** 2 Minor findings. 0 Major / 0 Critical. 10 seams CLEAN. Pass-6 fully
remediated + consistency-verified. Adversary pass-7 pending. Clean-pass counter 0/3.

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| FINDING-P6-001 | Minor | BC-2.01.017 v1.5 Related-BCs section listed BC-2.01.012 and BC-2.01.013 with annotations referencing E-INP-009 and E-INP-010 only. Per Decision 20 and D-156, BC-2.01.012 routes EPB body-decode failures to E-INP-008 and BC-2.01.013 routes SPB body-decode failures to E-INP-008. Annotations were incomplete — omitting E-INP-008 contradicted this BC's own PC1 error-code split and Error Taxonomy field (which explicitly includes E-INP-008 for EPB/SPB body-decode). | FIXED — BC-2.01.017 v1.5→v1.6: Related-BCs annotations for BC-2.01.012 and BC-2.01.013 updated to include E-INP-008 (EPB/SPB body-decode failures) alongside E-INP-009 and E-INP-010. D-157. |
| FINDING-P6-002 | Minor | BC-2.01.017 v1.5 PC1 per-block body-too-short window descriptions were incorrect: SPB window stated "[btl 16<=btl<20]" (this is the IDB window; btl=16 is minimal valid SPB with body=4 >= SPB_FIXED_OVERHEAD_BYTES=4); EPB window stated "[btl 32<=btl<52]" (wrong: EPB_FIXED_OVERHEAD_BYTES=20, so window is 12<=btl<32). Correct per-block windows per Decision 20 + block fixed minimums: SHB 12<=btl<28 (body 0-15); IDB 12<=btl<20 (body 0-7); EPB 12<=btl<32 (body 0-19); SPB btl=12 only (body=0 < 4; btl=16 is minimal valid SPB body=4). | FIXED — BC-2.01.017 v1.5→v1.6: PC1 per-block body-too-short windows corrected. BC-INDEX v1.62→v1.63 (annotation synced). D-157. |

**Artifacts updated in this burst (2 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| BC-2.01.017 | v1.5 | v1.6 | FINDING-P6-001 (Related-BCs +E-INP-008), FINDING-P6-002 (per-block body-too-short windows corrected) |
| BC-INDEX | v1.62 | v1.63 | FINDING-P6-002 annotation sync (BC-2.01.017 v1.5→v1.6) |

**Re-audit verdict:** CLEAN. All 10 seams pass. 2 Minor findings FIXED. Pass-6 fully
remediated + consistency-verified. Adversary pass-7 is next. Clean-pass counter 0/3.

---

---

## Pass-7 Adversarial Findings (ADV-F2-PASS7 — D-158 burst — 2026-06-20)

**Overall verdict:** 1 CRITICAL / 3 HIGH / 4 MEDIUM / 4 LOW. Novelty MODERATE (down from HIGH).
**Trajectory:** P1:23 / P2:24 / P3:17 / P4:13 / P5:13 / P6:13 / P7:12 — count declining, convergence approaching.
**Two axes CONVERGED:** SPB body.len()-4 arithmetic (Decision 22 / VP-031 formula); VP-INDEX self-consistency (total_vps=31).
**Open cluster:** OPB zero-packet-notice subsystem (rev-8/9 propagation lag: BC-2.01.015 PC9 vs HS-108 Cases D/E) + symbol-rename incompleteness (block_body_available → spb_data_available).
Clean-pass counter: 0/3. Remediation round-7 required.

**Full pass record:** `.factory/cycles/feature-pcapng-reader/f2-adversarial-spec-review-pass7.md`

### Pass-7 Critical Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| F-1 | CRITICAL | OPB counter model contradiction: BC-2.01.015 PC9/AC-003/AC-006 establishes dual-counter model — OPB skip arm increments BOTH `skipped_blocks` AND `opb_skipped` (invariant: opb_skipped <= skipped_blocks, i.e., OPBs are a SUBSET of total skips). HS-108 Case D (3 OPBs) asserts `skipped_blocks=0, opb_skipped=3` — violates the subset invariant. HS-108 Case E (2 NRBs + 1 OPB) asserts `skipped_blocks=2, opb_skipped=1` — NRBs NOT included in skipped_blocks, making it a non-OPB counter not a total counter. A test suite written against BC-2.01.015 will FAIL the HS-108 holdout. Root cause: HS-108 authored in D-150 (pass-4 H-4) before opb_skipped sub-counter added in D-153 (pass-5 H-2); D-153 updated the BC but not HS-108 Cases D/E. Fix: keep BC "both" model; HS-108 Case D skipped_blocks=1/opb_skipped=1 (1 OPB → both counters increment once); Case E skipped_blocks=3/opb_skipped=1 (2 NRBs + 1 OPB). | FIXED — BC-2.01.015 v1.7→v1.8 (subset invariant canonical in PC9/AC-003/AC-006); HS-108 v1.3→v1.4 (Case D: skipped_blocks=1/opb_skipped=1 (1 OPB); Case E: skipped_blocks=3/opb_skipped=1 (2 NRBs + 1 OPB)). D-159. P7-001 metadata correction applied D-160. Pending pass-8 verification. |

### Pass-7 High Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| F-2 | HIGH | HS-108 uses non-existent field `obsolete_packet_blocks` (6× in Cases D/E expected outputs and notice assertions). Canonical field name per BC-2.01.015 AC-003 + BC-2.01.009 v1.5 is `opb_skipped`. The name `obsolete_packet_blocks` is a stale draft artifact from HS-108 v1.0 (D-150) not updated when D-153 introduced canonical naming. An implementation against the BCs would use `opb_skipped`; holdout uses non-existent field → wrong / compile-fail. Fix: rename all 6 occurrences `obsolete_packet_blocks` → `opb_skipped` in HS-108. | FIXED — HS-108 v1.3→v1.4: all 6 occurrences of `obsolete_packet_blocks` renamed to `opb_skipped`. D-159. Pending pass-8 verification. |
| F-3 | HIGH | Zero-packet notice DISPLAY arithmetic undefined for mixed-skip case. BC-2.01.009 PC6 only specifies appending an OPB clause when opb_skipped>0; it does NOT specify subtracting opb_skipped from the generic count to produce the non-OPB display segment. HS-108 Case E shows "2 generic + 1 OPB" but this is not derivable from the BC without knowing the subtraction convention — an implementation could emit "3 blocks skipped (incl. 1 OPB)" (double-counting). Fix: BC-2.01.009 PC6 define generic segment = (skipped_blocks - opb_skipped), emitted only when > 0; normalize 3-way notice format (OPB-only / generic-only / mixed). | FIXED — BC-2.01.009 v1.6→v1.7: PC6 display arithmetic explicit (G=skipped_blocks-opb_skipped emitted when G>0); 3-way notice format normalized. HS-108 v1.4: Case D G=0 (no generic segment); Case E G=2 ("2" as generic segment). D-159. Pending pass-8 verification. |
| F-4 | HIGH | EPB decode precedence unpinned + BC-2.01.012 Precondition 1 contradicts PC5a. Precondition 1 asserts "interface table is non-empty" (caller obligation); PC5a defines behavior when it IS empty (E-INP-009). These cannot both be correct — a precondition violation is undefined behavior, making PC5a dead spec. Also, EPB body-decode check ordering not stated as a formal precedence postcondition (body.len() guard, read interface_id, empty-table, OOB, captured_len/padding). Fix: (1) Remove "non-empty" from Precondition 1; (2) Add precedence postcondition to BC-2.01.012 making check order explicit and binding. | FIXED — BC-2.01.012 v1.6→v1.7: Precondition 1 "non-empty" obligation removed; EPB decode precedence postcondition added (binding order: body.len() guard → read interface_id → empty-table-E-INP-009 → OOB-E-INP-010 → captured_len/padding). D-159. Pending pass-8 verification. |

### Pass-7 Medium Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| F-5 | MEDIUM | HS-107 v1.5 still uses retired symbol `block_body_available` in Cases A/B/C/D/E rationale and assertions (numerically equal to body.len()-4 but using the retired name). Decision 22 (ADR-009 rev 9, D-156) established canonical rename to `spb_data_available`. BC-2.01.013 v1.6 uses `body.len()-4`; holdout uses retired symbol → assertion mismatch against correct implementation. Fix: rename `block_body_available` → `spb_data_available` throughout HS-107. | FIXED — HS-107 v1.5→v1.6: all remaining `block_body_available` → `spb_data_available` (Scenario header, Cases A–E, key-observables, Behavioral Contract Linkage table, Evaluation Rubric, Verification Approach). D-159. Pending pass-8 verification. |
| F-6 | MEDIUM [process-gap] | HS-104/107/108 input-hash "tbd" — drift tripwire disabled on three must-pass holdouts covering F2 convergence gate scenarios (EPB interface_id discriminant / SPB framing / zero-packet notice). Decisions 15/17/19/20/22 are governing decisions; ADR-009 not listed as holdout input for any of these three. Fix: run `bin/compute-input-hash --write` on HS-104/107/108; add ADR-009 to each holdout's inputs list. | DEFERRED-TO-F2-CONVERGENCE — Rationale: computing input-hash on a still-churning spec goes stale each pass; each remediation round changes governing BCs and ADR-009, making any hash computed mid-convergence immediately stale. Action: run `bin/compute-input-hash --write` on HS-104/107/108 AND add ADR-009 to each holdout's inputs list ONCE F2 reaches 3-clean-pass convergence, before the F2 human gate. Added to F2-gate / F3-entry checklist (STATE.md Section D, item 8). D-159. |
| F-7 | MEDIUM | BC-2.01.013 EC-001/EC-002/EC-003 + Canonical Test Vectors still use retired `block_body_available` (D-156 v1.6 changelog claimed "everywhere" rename complete but EC/vector sections lagged — same changelog-lie defect class as pass-3 C-1 / Lesson 8). Fix: rename `block_body_available` → `spb_data_available` in BC-2.01.013 EC-001/002/003 and Canonical Test Vectors. | FIXED — BC-2.01.013 v1.6→v1.7: `block_body_available` → `spb_data_available` in EC-001/EC-002/EC-003 and Canonical Test Vectors. D-159. Pending pass-8 verification. |
| F-8 | MEDIUM | BC-2.01.013 lacks the btl=14 misaligned-alignment-violation crate-rejection fixture. HS-107 Case E uses btl=14 rejected for alignment violation (14%4=2); D-156 F-M1 corrected the Case E rationale. But BC-2.01.013 EC-005 only illustrates btl=8 (<12, below minimum) — the alignment class is absent from the BC's canonical test vectors. An alignment-violation E-INP-010 case exists only in HS-107, not in the governing BC. Fix: add btl=14 misaligned→E-INP-010 example to BC-2.01.013 EC-005. | FIXED — BC-2.01.013 v1.7: btl=14 misaligned (14%4=2 → E-INP-010 alignment rejection) fixture added to EC-005, complementing existing btl=8 below-minimum fixture. D-159. Pending pass-8 verification. |

### Pass-7 Low Findings (all CLOSED / CONVERGED)

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| F-L1 | LOW | VP-INDEX total_vps=31 count propagation and self-consistency. | CONVERGED GREEN — no action |
| F-L2 | LOW | SPB formula `min(original_len, body.len()-4)` (Decision 22 / BC-2.01.013 / VP-031) consistency. | CONVERGED GREEN — no action |
| F-L3 | LOW | Section-wide endianness coverage for option code/length TLV fields. | CONVERGED GREEN — no action |
| F-L4 | LOW | BC-2.01.018 VP-030/STORY-128 anchor note (pending-intent deferral per Lesson 11). | INFORMATIONAL — intentional tracked deferral |

---

## Pass-7 Remediation Summary (D-159 — 2026-06-20)

**Verdict:** All 1C/3H/4M pass-7 findings FIXED pending pass-8 verification. 4L findings CONVERGED GREEN (no action). F-6 [process-gap] DEFERRED-TO-F2-CONVERGENCE. Clean-pass counter 0/3. Adversary pass-8 is next.

**Artifacts updated in this burst (6 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| BC-2.01.009 | v1.6 | v1.7 | F-3 (notice display arithmetic / PC6 subtraction convention / 3-way format) |
| BC-2.01.012 | v1.6 | v1.7 | F-4 (PC1 contradiction removed; EPB decode precedence postcondition added) |
| BC-2.01.013 | v1.6 | v1.7 | F-7 (block_body_available→spb_data_available in EC-001/002/003 + test vectors), F-8 (btl=14 alignment fixture → EC-005) |
| BC-2.01.015 | v1.7 | v1.8 | F-1 ("both" model canonical; subset invariant opb_skipped<=skipped_blocks pinned) |
| HS-107 | v1.5 | v1.6 | F-5 (block_body_available→spb_data_available throughout all cases) |
| HS-108 | v1.3 | v1.4 | F-1 (Case D/E counters corrected), F-2 (obsolete_packet_blocks→opb_skipped 6×), F-3 (display arithmetic explicit: G=skipped_blocks-opb_skipped) |
| BC-INDEX | v1.63 | v1.64 | 4 BC annotations synced (BC-2.01.009/012/013/015) |
| spec-changelog | — | — | [pcapng-f2-pass7-remediation-2026-06-20] prepended |
| STATE.md | — | — | phase_status updated; spec-versions synced; D-159 added; F2-gate checklist item 8 added |

---

## Pass-7 Re-Audit Minor Findings (D-160 — 2026-06-20)

**Overall verdict:** 0 CRITICAL / 0 HIGH / 0 MEDIUM / 2 LOW (metadata/rubric). No normative BC content changed this burst.

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| FINDING-P7-001 | Minor (metadata) | BC-INDEX v1.64 inline annotation for BC-2.01.015 v1.8 described Case D as "(3 OPBs) skipped_blocks=3/opb_skipped=3". STATE.md D-159 entry + session checkpoint repeated the same wrong description. Correct per normative HS-108 v1.5 on disk + BC-2.01.015 v1.8 "both" model: Case D = 1 OPB → skipped_blocks=1, opb_skipped=1. The D-159 remediation fixed HS-108 on disk correctly but propagated the adversary's original (pre-fix) "3 OPBs" label into index/state prose. | FIXED — BC-INDEX v1.64→v1.65: Case D annotation corrected to "Case D (1 OPB) skipped_blocks=1/opb_skipped=1". STATE.md D-159 decision text corrected. spec-changelog D-159 entry corrected. Tracker F-1 status cell corrected. D-160. |
| FINDING-P7-002 | Minor (rubric gate) | HS-108 v1.4 Cases B and F rubric gates used bare `skipped_blocks > 0` as the condition for emitting the generic skip-count segment. Canonical gate per BC-2.01.009 PC6 is `(skipped_blocks - opb_skipped) > 0` (G > 0). Numerically equivalent for Cases B/F (both have opb_skipped==0) but the bare form would incorrectly emit the generic segment for an OPB-only input (skipped_blocks=1, opb_skipped=1, G=0 → no generic segment should appear). | FIXED — HS-108 v1.4→v1.5: Case B rubric gate and Case F body gate corrected to `(skipped_blocks - opb_skipped) > 0`. BC-INDEX v1.65 changelog records HS-108 v1.4→v1.5. D-160. |

**Artifacts updated in this burst (5 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| BC-INDEX | v1.64 | v1.65 | FINDING-P7-001 (Case D metadata corrected in v1.64 annotation + new v1.65 changelog) |
| HS-108 | v1.4 | v1.5 | FINDING-P7-002 (Case B/F rubric gates → canonical G > 0 form) |
| STATE.md | — | — | FINDING-P7-001 (D-159 entry + checkpoint Case D corrected); phase_status + spec-versions updated; D-160 added |
| spec-changelog | — | — | [pcapng-f2-pass7-reaudit-minors-2026-06-20] prepended; D-159 body Case D text corrected |
| f2-review-remediation-tracker.md | — | — | FINDING-P7-001/002 sections added; F-1 status cell corrected |

**Verdict:** All 2 Minor pass-7 re-audit findings FIXED. 0 Major/Critical. Clean-pass counter 0/3. Adversary pass-8 pending.

---

---

## Pass-8 Adversarial Findings (ADV-F2-PASS8 — D-161 burst — 2026-06-20)

**Overall verdict:** 0 CRITICAL / 0 HIGH / 3 MEDIUM / 5 LOW. **CLEAN — 0C/0H.**
**Clean-pass counter: 1/3 (BC-5.39.001).**
**Trajectory:** P1:23 / P2:24 / P3:17 / P4:13 / P5:13 / P6:13 / P7:12 / P8:8

Full pass record: `.factory/cycles/feature-pcapng-reader/f2-adversarial-spec-review-pass8.md`

### Pass-8 Medium Findings

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| M-1 | MEDIUM | error-taxonomy SPB-fixed-min prose gap — E-INP-008 SPB body-too-short entry omitted SPB_FIXED_MIN=16 anchor; btl=12/btl=16 boundary not stated. | FIXED — error-taxonomy v3.4→v3.5: SPB_FIXED_MIN=16 cited; btl=12 (body=0<4) vs btl=16 (body=4, min valid SPB) boundary clarified; EC-008 cross-ref added. D-161. |
| M-2 | MEDIUM | IDB body-decode holdout gap — BC-2.01.011 was the only framing BC without a dedicated body-decode error-path holdout; no HS covered constructible IDB body-short window (12<=btl<20), malformed options-TLV, or if_tsresol option_length mismatch. | FIXED — HS-109 v1.0 authored (5 cases: btl=16 body<8→E-INP-008; reserved!=0→E-INP-008; options-TLV OOB→E-INP-008; if_tsresol option_length=4→E-INP-008; positive control). HS-INDEX v2.3→v2.4 (greenfield 108→109; all-namespace 181→182; must_pass 108). D-161. |
| M-3 | MEDIUM | BC-2.01.013 AC-001 test name `test_BC_2_01_013_snaplen_lookup_guarded` stale — snaplen removed from SPB path in D-153 (ADR-009 rev 8 Decision 9 amend); AC-001 guards empty-interface-table (E-INP-009), not snaplen; DF-AC-TEST-NAME-SYNC-001 violation. | FIXED — BC-2.01.013 v1.7→v1.8: AC-001 test name → `test_BC_2_01_013_empty_interface_table_guarded`; scope note clarified; AC-004a/EC-008 body-too-short non-redundancy noted; no normative change. BC-INDEX v1.65→v1.66. D-161. |

### Pass-8 Low Findings (all CONVERGED GREEN or FIXED via M-2)

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| L-1 | LOW | VP-INDEX total_vps=31 count propagation. | CONVERGED GREEN — VP-INDEX v2.8 total 31 confirmed; no action. |
| L-2 | LOW | error-taxonomy next_free E-INP-014 confirmed. | CONVERGED GREEN — no new error codes added in pass-7/8; no action. |
| L-3 | LOW | BC count 302 propagation. | CONVERGED GREEN — BC-INDEX v1.66 confirms 302; no action. |
| L-4 | LOW | HS-INDEX all-namespace count. | FIXED (via M-2) — HS-INDEX v2.4 all-namespace=182 (was 181). |
| L-5 | LOW | ADR-009 status field. | FIXED (via O-2) — ADR-009 status: proposed→accepted. |

### Pass-8 Process-Gap Observations

| ID | Category | Observation | Status |
|----|----------|-------------|--------|
| O-1 | process-gap | No machine-checkable framing-constant validator — 6 documents independently state same per-block constants; no cross-doc validator to catch future drift. Root-cause class of H-2/C-3/SPB-three-way-min failures. | DEFERRED-TO-F3 — bin/ script addition out of scope for F2 spec-only phase; F3 story decomposition checklist should evaluate `bin/framing-constant-validator` scope. |
| O-2 | observation | ADR-009 status: proposed (stale); should be accepted since rev 1 adoption. | FIXED — ADR-009 rev 9 status: proposed→accepted. D-161. |

---

## Pass-8 Remediation Summary (D-161 — 2026-06-20)

**Verdict:** Pass-8 CLEAN (0C/0H/3M/5L). All 3 MEDIUM findings FIXED. O-2 FIXED. O-1 DEFERRED-TO-F3. **CLEAN-PASS 1/3.** Adversary pass-9 is next.

**Artifacts updated in this burst (6 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| error-taxonomy | v3.4 | v3.5 | M-1 (SPB-fixed-min prose gap) |
| HS-109 | — | v1.0 (new) | M-2 (IDB body-decode holdout — 5 cases) |
| HS-INDEX | v2.3 | v2.4 | M-2 (greenfield 108→109; all-namespace 181→182) |
| BC-2.01.013 | v1.7 | v1.8 | M-3 (AC-001 test name DF-AC-TEST-NAME-SYNC-001) |
| ADR-009 | rev 9 | rev 9 (status) | O-2 (status: proposed→accepted) |
| BC-INDEX | v1.65 | v1.66 | M-3 annotation sync; clean-pass 1/3 recorded |
| spec-changelog | — | — | [pcapng-f2-pass8-clean-and-medium-remediation-2026-06-20] prepended |
| STATE.md | — | — | phase_status + spec-versions + D-161 added; clean-pass 1/3 recorded |

---

**F-6 Deferral Note (DEFERRED-TO-F2-CONVERGENCE):**
Rationale: HS-104/107/108 input-hashes computed on a still-churning spec go stale after each
remediation pass. Each burst changes governing BCs (currently BC-2.01.009/012/013/015) and
ADR-009 is not yet in any holdout's inputs list. Computing now produces a hash that will be
stale after pass-8 regardless of outcome. Action: once F2 achieves 3-clean-pass convergence,
run `bin/compute-input-hash --write` on HS-104/107/108 and add ADR-009 to each holdout's
inputs list, before the F2 human gate. This is tracked in STATE.md Section D item 8 and in
the F3-entry checklist so it cannot be lost.

**F2-Gate / F3-Entry Checklist Addendum (D-159):**
- [ ] **Item 8 (pre-F2-gate):** Run `bin/compute-input-hash --write` on HS-104, HS-107, HS-108.
  Add ADR-009 to each holdout's `inputs:` list before running the hash. Execute only after F2
  reaches 3-clean-pass convergence. (F-6 DEFERRED-TO-F2-CONVERGENCE — D-159)

**Clean-pass counter as of D-161: 1/3. Adversary pass-9 pending (targeting clean-pass 2/3).**

---

## Pass-8 Focused Re-Audit Findings (D-162 — 2026-06-20)

**Overall verdict:** CLEAN. 1 MINOR metadata finding (FINDING-P8-001) identified and FIXED.
**CLEAN-PASS counter: still 1/3.** A metadata-only fix does not reset the clean-pass counter — pass-8 remains the 1st clean pass. Adversary pass-9 pending.

| ID | Severity | Finding Summary | Status |
|----|----------|----------------|--------|
| FINDING-P8-001 | MINOR (metadata) | HS-INDEX By-Category table behavioral-subtleties cell showed 39 but correct count is 40. The 5 category rows summed to 108 but TOTAL=109. Root cause: the pcapng-holdouts note said "+3 (HS-101, HS-105, HS-108; HS-106 already included)" — the "already included" phrasing implied HS-106 was counted in the base before pcapng rows, when in fact HS-106 IS one of the 4 behavioral-subtleties pcapng additions. The TOTAL row (109) and the 109-scenario index were correct throughout. | FIXED — HS-INDEX v2.4→v2.5: behavioral-subtleties cell 39→40; pcapng-holdouts note corrected to "+4 (HS-101, HS-105, HS-106, HS-108)". Verification: 40+20+18+21+10=109=TOTAL. spec-changelog [pcapng-f2-pass8-reaudit-hsindex-count-2026-06-20] prepended. D-162. |

**Artifacts updated in this burst (3 artifacts):**

| Artifact | Before | After | Findings addressed |
|----------|--------|-------|--------------------|
| HS-INDEX | v2.4 | v2.5 | FINDING-P8-001 (behavioral-subtleties cell 39→40; pcapng-holdouts note corrected) |
| spec-changelog | — | — | [pcapng-f2-pass8-reaudit-hsindex-count-2026-06-20] prepended |
| STATE.md | — | — | D-162 added; HS-INDEX v2.5 noted; phase_status + checkpoint updated |

**Clean-pass counter as of D-162: 1/3. CLEAN-PASS 1/3 confirmed. Adversary pass-9 pending (targeting clean-pass 2/3).**
