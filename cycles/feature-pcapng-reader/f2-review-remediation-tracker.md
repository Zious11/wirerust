---
document_type: remediation-tracker
cycle: feature-pcapng-reader
phase: F2
sources:
  - adversarial-spec-review-pass1 (ADV-F2-PASS1)
  - security-review (SR-PCAPNG-F2)
  - performance-review
date_created: 2026-06-19
status: REAUDIT-FIXED
f3_blocked: true
f3_blocker_reason: "Adversarial reconvergence required (3 clean passes). Must-fix items addressed (D-142). Re-audit 6 findings + BOM-mapping chain ALL FIXED (D-143). Pass 2 not yet dispatched. BLOCKED-ON-SPIKE items (H-2 final form, H-6/M-2/partial SEC-002) pending pass-2 verification."
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

## F3 Entry Gate

F3 story decomposition is **BLOCKED** until:
1. All "Must-Fix Before F3" items above are remediated via BC/NFR amendments — COMPLETE (D-142 + D-143)
2. Adversarial reconvergence: 3 consecutive clean adversarial review passes (0 CRITICAL, 0 HIGH, <3 MEDIUM)
3. pcap-file 2.0.0 API spike complete (unblocks H-1 final form, H-2, H-6, M-2, SEC-002/008)
4. VP-NNN assigned to all 10 BCs (C-3 resolved) — COMPLETE (D-142)
