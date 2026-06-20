---
document_type: adversarial-spec-review
review_id: ADV-F2-PASS1
phase: F2
feature: FE-001-pcapng-reader-support
cycle: feature-pcapng-reader
pass: 1
date: 2026-06-19
reviewer: adversary
model: claude-sonnet-4-6
total_findings: 23
critical: 3
high: 6
medium: 7
low: 3
observations: 4
convergence_status: NOT-CONVERGED
spec_state_reviewed:
  - "factory-artifacts mounted at .factory/"
  - "ADR-009 rev 3 (proposed)"
  - "BC-2.01.009 v1.0 / .010 v1.3 / .011 v1.0 / .012 v1.0 / .013 v1.0 / .014 v1.0 / .015 v1.1 / .016 v1.0 / .017 v1.1 / .018 v1.1"
  - "BC-2.01.004 v1.5 retired"
  - "BC-2.01.001 v1.7 / .002 v1.6"
  - "error-taxonomy v2.6"
  - "nfr-catalog v2.2"
  - "BC-INDEX 303 on disk / 302 active"
top_5_priority: [C-1, H-1, C-3, H-2, C-2]
---

# Adversarial Spec Review — F2 pcapng Reader, Pass 1

## Scope

Spec state reviewed: factory-artifacts mounted at .factory/, ADR-009 rev 3 (proposed),
BC-2.01.009 v1.0/.010 v1.3/.011 v1.0/.012 v1.0/.013 v1.0/.014 v1.0/.015 v1.1/.016 v1.0/.017
v1.1/.018 v1.1, BC-2.01.004 v1.5 retired, .001 v1.7, .002 v1.6, error-taxonomy v2.6, nfr-catalog
v2.2, BC-INDEX 303 on disk / 302 active.

## Summary

**Findings: CRITICAL 3 | HIGH 6 | MEDIUM 7 | LOW 3 | Observations 4**

**Top-5 priority order:** C-1, H-1, C-3, H-2, C-2

**Convergence verdict:** NOT CONVERGED. Three critical defects exist that are directly
untestable as written. H-1 is cross-confirmed by the security review (SEC-001/006). Remediation
required before F3 story decomposition. Adversarial reconvergence (3 clean passes) required before
F3 proceeds.

---

## Critical Findings

### C-1 [CRITICAL, process-gap]: Directory-mode per-file isolation claim is false

**Severity:** CRITICAL
**Category:** process-gap / false-AC

BC-2.01.018 AC-002 and E-INP-011/012 notes describe directory-mode per-file isolation (one
file's error does not abort others). This claim is false as the codebase stands.
`main.rs:241-244` uses the `?` operator, meaning the first reader error propagates to `main()`
and aborts the entire run. No F2 story is scoped to fix the loop — stories are scoped to
`reader.rs`; the loop lives in `main.rs`.

The AC is untestable in its current form because no story owns the fix. F3 story decomposition
would produce implementation against an AC that the implementer cannot satisfy within the story
scope assigned.

**Resolution options:** (a) Add a story/AC to refactor `main.rs` with catch-and-continue error
handling in the directory traversal loop, OR (b) retract the per-file isolation claim from
BC-2.01.018 AC-002 and E-INP-011/012 notes.

---

### C-2 [CRITICAL]: `.cap`-extension pcapng files unreachable in directory mode

**Severity:** CRITICAL

The lead motivator file (`arp-baseline-16pkt.cap`) has a `.cap` extension. In directory mode,
`resolve_targets` at `main.rs:530` globs only `ext == "pcap"`. STORY-127 adds `.pcapng`
extension support but not `.cap`. The original motivator file is excluded by extension filtering
and will never be processed in directory mode.

**Resolution:** Either (a) glob by magic-byte/content-type detection (not by extension) to
handle `.cap` files with pcapng bytes, or (b) explicitly add `.cap` to the extension allowlist
alongside `.pcapng` in STORY-127's AC. Assign to a BC anchored to STORY-127 so the requirement
is traceable.

---

### C-3 [CRITICAL, DF-CANONICAL-FRAME-HOLDOUT-001]: Framing BCs have no VP assignments and no holdout scenarios

**Severity:** CRITICAL

All ten framing BCs (BC-2.01.010, .012, .014, .015, .018) have `VP-NNN = —` (unassigned) and
no associated holdout scenario. The convergence rubric (DF-CANONICAL-FRAME-HOLDOUT-001) blocks
convergence for any BC without a VP. Every framing BC must have: a VP-NNN assigned, the VP
registered in VP-INDEX, and a designated holdout fixture.

**Resolution:** Assign VP-NNN to each framing BC. Register each in VP-INDEX. Designate holdout
fixtures per framing BC (can share fixture files but must have distinct VP entries). This is
prerequisite to any F6 or convergence gate passing.

---

## High Findings

### H-1 [HIGH]: BC-2.01.014 timestamp arithmetic not total over all u8 inputs — integer overflow

**Severity:** HIGH
**Cross-confirmed:** SEC-001 (security review) and SEC-006 (security review)

`10u64.pow(e)` panics for base-10 `e >= 20` (overflow-checks=true; u64::MAX ≈ 1.8×10^19).
`1u64 << e` panics for base-2 `e >= 64`. Both are reachable from the u8 `if_tsresol` input
space. BC-2.01.014 Invariant 1 ("no panic for any (u32, u32, u8) input") is therefore false
as specified. The Kani verification property (`VP-NNN = —`, not yet assigned) cannot pass when
implemented literally.

Additionally, the intermediate expression `(ticks % ticks_per_sec) * 1_000_000` overflows u64
for large base-2 exponents (e.g., `e = 62` → `ticks_per_sec = 2^62 ≈ 4.6×10^18`; product ≈
`4.6×10^24`, exceeding u64::MAX).

**Resolution:** Require checked/saturating arithmetic throughout:
- Base-10: `10u64.checked_pow(e as u32).unwrap_or(u64::MAX)` (clamp overflow to u64::MAX; ts_usecs → 0)
- Base-2: clamp `e` to [0, 63] before shift; `1u64.checked_shl(e as u32).unwrap_or(u64::MAX)`
- Intermediate multiply: use `u128` intermediate or saturating_mul
- Cover edge cases: e=20/127 base-10, e=63/64/127 base-2

---

### H-2 [HIGH]: BC-2.01.013 SPB length overhead wrong (16 bytes, not 20) + padding unsafe

**Severity:** HIGH
**Cross-confirmed:** SEC-004 (security review) on the allocation-before-validation vector

BC-2.01.013 specifies SPB fixed-field overhead as 20 bytes. The correct value is 16 bytes
(type:4 + total_length:4 + original_length:4 + captured_length:4). The mismatch means the
captured_length validation check (captured_length <= block_total_length - 20) will reject valid
SPBs with small bodies (e.g., any SPB where the body is between 1 and 4 bytes is falsely
rejected). Padding alignment to 4-byte boundaries is not addressed in the padding extraction;
callers may slice into padding bytes.

**Resolution:** Correct the overhead constant to 16 in BC-2.01.013. Add explicit padding
handling: `packet_data` slice is `captured_length` bytes from the block body, where the body
total is `block_total_length - 16` bytes including padding. Defer to the `pcap-file` crate's
SPB `packet_data` field if it already applies this correctly (requires API spike to confirm).

---

### H-3 [HIGH]: E-INP-009 orphaned — EPB-before-IDB mis-mapped

**Severity:** HIGH
**Cross-confirmed:** SEC-003 (security review)

E-INP-009 ("EPB encountered before any IDB") is defined in the error taxonomy but is not routed
to by any BC. BC-2.01.012 Postcondition 5 maps EPB with out-of-range `interface_id` to
E-INP-008, but that entry covers SHB/IDB structural failures — semantically wrong. E-INP-009 is
the correct entry for the EPB-before-IDB case (empty interface table at EPB parse time), but no
BC references it. The result is an orphaned error code that implementers cannot know to emit.

**Resolution:** BC-2.01.012 Postcondition 5 must be corrected to route empty-table EPB to
E-INP-009. If `interface_id` is out of range on a non-empty table, route to E-INP-010 with a
context string. Add an explicit AC to BC-2.01.012 requiring a bounds check before any
`interface_id`-based indexing operation.

---

### H-4 [HIGH/MEDIUM]: Silent zero-packet traps

**Severity:** HIGH (SPB-without-IDB); MEDIUM (OPB-only)

Two silent failure modes exist in the spec:

1. **SPB-without-IDB:** SPB block processing indexes `idb[0]` (as implied by SPB inheriting
   interface context). If no IDB has been encountered yet, this is an out-of-bounds index (panic
   in Rust, or incorrect data). No error code is assigned.

2. **OPB-only file:** A file containing only Obsolete Packet Blocks (OPB, type 0x2) correctly
   skips all blocks per BC-2.01.015 AC-001, yielding `Ok(empty Vec)`. No warning is emitted.
   This violates SOUL #4 (no silent failures) — the caller receives an empty result with no
   indication that all blocks were skipped.

**Resolution:**
- SPB-without-IDB: define an error code (new E-INP entry or extension of E-INP-009) and add
  an AC to BC-2.01.013 requiring it.
- OPB-only: add a one-shot stderr warning ("0 packets extracted; N OPB blocks skipped") when
  the result Vec is empty due entirely to skipped blocks.

---

### H-5 [HIGH]: BC-2.01.009 PC1 over-promises "at least one readable packet"

**Severity:** HIGH

BC-2.01.009 Postcondition 1 implies that a valid pcapng file always yields at least one readable
packet. This contradicts: (a) BC-2.01.002 EC-001, which recognizes a valid empty pcap as
returning `Ok(PcapSource { packets: [] })` without error; and (b) the OPB-only zero-packet case
(H-4 above). An OPB-only file is structurally valid pcapng and should return `Ok` with an empty
Vec, not `Err`.

**Resolution:** Reword BC-2.01.009 PC1 to match the classic-pcap contract: a successfully parsed
pcapng file returns `Ok(PcapSource)` where `packets` may be empty. Keep ">0 packets" only as a
fixture-specific test vector assertion, not a general postcondition.

---

### H-6 [MEDIUM]: if_tsresol double-apply risk — crate API behavior unverified

**Severity:** MEDIUM (elevated from LOW by API uncertainty)

The BCs assume `pcap-file 2.0.0` exposes raw `(ts_high, ts_low)` values from EPB timestamp
fields, requiring wirerust to apply `if_tsresol` conversion. ADR-009 Trade-offs section
marks this as "unverified" — if the crate already applies `if_tsresol` internally and returns
pre-converted timestamps, applying the conversion a second time in `pcapng_timestamp_to_secs_usecs`
would produce wrong timestamps without error.

**Resolution:** The pcap-file API spike (dispatched, keystone) must confirm which interface
the crate exposes: raw `(ts_high, ts_low)` u32 fields or pre-converted epoch seconds/micros.
If pre-converted, the entire BC-2.01.014 conversion function is unnecessary. If raw, the
BCs are correct. Mark H-6 as BLOCKED-ON-SPIKE.

---

## Medium Findings

### M-1: SHB truncation threshold inconsistency (28 bytes vs. 8 bytes)

**Severity:** MEDIUM

BC-2.01.010 specifies the SHB minimum valid length as 28 bytes (standard SHB fixed fields),
but the error taxonomy entry E-INP-008 references an 8-byte truncation threshold for "truncated
SHB." These two values are inconsistent. A 9-byte SHB (truncated after the block length field)
would pass the 8-byte check but fail the 28-byte check. The test vector derived from this must
choose one or the other as the canonical "truncated" boundary.

**Resolution:** Align BC-2.01.010 and E-INP-008 on a single minimum length. The pcapng RFC
defines the SHB fixed-field block body as 16 bytes (magic:4 + major:2 + minor:2 + section_length:8),
so the minimum total block_total_length is 12 (block header) + 16 (minimum body) = 28. Update
E-INP-008 to use 28 as the truncation threshold, not 8.

---

### M-2: Block variant names unverified against pcap-file enum / #[non_exhaustive]

**Severity:** MEDIUM

The BC match-arm names (SectionHeaderBlock, InterfaceDescriptionBlock, EnhancedPacketBlock,
SimplePacketBlock, etc.) are written as if they match the pcap-file 2.0.0 `Block` enum variant
names exactly. If the enum uses different names or is marked `#[non_exhaustive]`, the match arms
in the implementation will not compile or will miss variants. This is not verifiable at spec time
without an API spike.

**Resolution:** Mark M-2 as BLOCKED-ON-SPIKE. The spike must confirm the exact variant names and
`#[non_exhaustive]` status of `pcap_file::pcapng::blocks::Block`. Update the BCs to use canonical
names post-spike.

---

### M-3: E-INP-010 conflates three failure modes with two message templates

**Severity:** MEDIUM

E-INP-010 covers: (a) EPB captured_length > block_total_length - overhead (truncation), (b)
block_total_length < 8 for unknown blocks, and (c) EPB interface_id out of range (if routed here
per SEC-003 fix). The error taxonomy provides two message templates but three distinct failure
modes. The implementer must guess which template to use for the interface_id case.

**Resolution:** Either add a third message template to E-INP-010 explicitly for interface_id
out-of-range, or create a distinct E-INP-013 entry for that case. Align with the SEC-003
resolution from the security review.

---

### M-4 [Withdrawn — reclassified LOW as L-3]

This finding (arithmetic calculation concern) was reviewed and determined to be correct upon
re-examination. Reclassified as L-3 (confusing parenthetical in EC-003).

---

### M-5: Multi-section reject discards section-1 packets — AC wording implies otherwise

**Severity:** MEDIUM

BC-2.01.010 AC-002 rejects multi-section pcapng files (E-INP-012). However, the wording
implies that any packets encountered before the second SHB in section 1 would be discarded along
with the error. This may be surprising behavior — a caller that expected "as many packets as
possible" would expect section-1 results to be returned alongside the error. The current
AC-002 wording does not clarify whether section-1 packets are returned before the `Err` or
discarded.

**Resolution:** Clarify AC-002: on encountering a second SHB, the reader MUST return
`Err(E-INP-012)` immediately. No partial results are returned. This makes the behavior
deterministic and testable.

---

### M-6: BC story anchors are provisional / forward-reference; STORY-127 has no BC home for glob

**Severity:** MEDIUM

Several BCs reference stories as `STORY-NNN (provisional)`. The `.pcapng` extension glob is
described in ADR-009 and mentioned in story planning notes but has no BC home — no BC
explicitly requires the extension filter to include `.pcapng`. STORY-127 is described as
implementing the glob but no BC's AC can be cited by the story's traceability section.

**Resolution:** Assign a BC (or extend an existing one, e.g., BC-2.01.009 or BC-2.12.011) to
explicitly require the extension filter to accept `.pcapng` (and `.cap` if C-2 is resolved via
allowlist). Remove "provisional" labels from all story anchors once F3 story decomposition
assigns canonical STORY-NNN numbers.

---

### M-7: EPB fixed-field overhead undefined

**Severity:** MEDIUM
**Cross-confirmed:** SEC-004 (security review) on allocation-before-validation

BC-2.01.012 references "EPB fixed-field overhead" in the captured_length validation expression
(Postcondition 6) but does not define the value as a named constant or enumerate the fields.
The EPB fixed fields are: block_type:4 + block_total_length:4 + interface_id:4 + timestamp_high:4
+ timestamp_low:4 + captured_length:4 + original_length:4 = 28 bytes. Without a named constant,
the implementer may use an incorrect value (20 is a common miscount).

**Resolution:** Define `EPB_FIXED_OVERHEAD_BYTES = 28` as an explicit named constant in
BC-2.01.012 and require its use in the captured_length validation expression. Verify against
the pcapng RFC (RFC 7468-equivalent, clause 4.3).

---

## Low Findings

### L-1: BC-2.01.011 EC-003 unescaped pipe character

**Severity:** LOW

BC-2.01.011 EC-003 contains `0x80 | 0x0A` — the `|` character in the body renders incorrectly
in markdown tables (splits the cell). Use `0x80 \| 0x0A` or reformulate as prose.

---

### L-2: ts_usecs intermediate overflow at large base-10 exponent (distinct from H-1)

**Severity:** LOW

Even after fixing `ticks_per_sec` computation (H-1), the expression
`(ticks % ticks_per_sec) * 1_000_000` can overflow u64 at very large base-10 exponents if
`ticks_per_sec` itself overflows to `u64::MAX` (the saturated value). When `ticks_per_sec =
u64::MAX`, `ticks % ticks_per_sec = ticks`, and `ticks * 1_000_000` overflows for any `ticks
> ~1.8×10^13`. This is a residual edge case after applying the H-1 fix and must be explicitly
handled in the spec's saturating-arithmetic prescription.

---

### L-3: EC-003 confusing parenthetical (reclassified from M-4)

**Severity:** LOW

The EC-003 parenthetical in BC-2.01.014 is confusing as written and may cause implementers to
misread the intended behavior. Rephrase for clarity without changing the arithmetic.

---

## Observations (Process Gaps)

### O-1 [process-gap]: All 10 BCs reached WRITTEN status with all-`—` VP cells

All ten new BCs (BC-2.01.009..018) were advanced to `[WRITTEN]` status with VP-NNN = `—`
(unassigned) in every verification-properties cell. The factory pipeline's convergence rubric
requires VP assignments before any BC can proceed past WRITTEN. This is a process gap: the
spec-evolution phase advanced BCs to WRITTEN without completing the VP-assignment step.
This is the root cause of C-3.

---

### O-2 [process-gap]: error-taxonomy input-hash listed as N/A

The error-taxonomy file's `input-hash` field is set to `N/A` rather than a computed 7-char
hash per DF-INPUT-HASH-CANONICAL-001. The error-taxonomy is an input to multiple story files
(any story whose BC references an E-INP-NNN code). Stories that list error-taxonomy as an
input will have stale input-hashes if the taxonomy changes after F2 and before the story
generates its hash.

**Resolution:** Compute and set the error-taxonomy input-hash before F3 story decomposition.
Run `bin/compute-input-hash --scan` to detect any stale story hashes.

---

### O-3 [doc-gap]: reader.rs:5 module doc + README still describe pcapng as unsupported

**Severity:** Observation

`src/reader.rs` line 5 module doc and the README still state pcapng format is unsupported.
STORY-123 must update these. If this is not tracked in STORY-123's ACs, it will be missed.
Add an explicit AC to STORY-123: "Update `reader.rs` module doc and README to remove 'pcapng
unsupported' language."

---

### O-4 [test-gap]: snaplen-truncation parity (pcapng vs. classic) untested — needs fixture

**Severity:** Observation

The classic-pcap path has a test fixture for snaplen-truncated packets (`incl_len < orig_len`).
No equivalent pcapng fixture is called out in the planned STORY-123..127 corpus. The pcapng
reader path must handle EPB `captured_length < original_length` identically to the classic path.
A pcapng fixture with a snaplen-truncated EPB should be added to the E2E corpus and cited in
STORY-127 (or a new test-vectors story).

---

## Finding Classification Table

| ID | Severity | Category | One-liner | BLOCKED-ON-SPIKE? |
|----|----------|----------|-----------|-------------------|
| C-1 | CRITICAL | process-gap | Directory-mode per-file isolation false (main.rs uses `?`) | No |
| C-2 | CRITICAL | missing-coverage | .cap extension unreachable in directory mode | No |
| C-3 | CRITICAL | DF-CANONICAL-FRAME-HOLDOUT-001 | All framing BCs have VP-NNN = — | No |
| H-1 | HIGH | overflow | Timestamp arithmetic overflows for legal u8 if_tsresol values | No |
| H-2 | HIGH | spec-error | SPB overhead 20 bytes wrong (should be 16); padding unsafe | Yes (crate spike) |
| H-3 | HIGH | orphaned-error-code | E-INP-009 orphaned; EPB-before-IDB mis-mapped | No |
| H-4 | HIGH/MED | silent-failure | SPB-without-IDB panics; OPB-only yields silent empty result | No |
| H-5 | HIGH | over-promise | BC-2.01.009 PC1 contradicts valid empty-pcapng contract | No |
| H-6 | MED | unverified-assumption | if_tsresol double-apply risk if crate pre-converts timestamps | Yes (API spike) |
| M-1 | MEDIUM | inconsistency | SHB truncation threshold 28 vs. 8 inconsistent | No |
| M-2 | MEDIUM | unverified | Block variant names unverified vs pcap-file enum | Yes (API spike) |
| M-3 | MEDIUM | ambiguity | E-INP-010 conflates 3 failure modes with 2 templates | No |
| M-4 | — | WITHDRAWN | Arithmetic re-examined; correct. Reclassified as L-3 | — |
| M-5 | MEDIUM | wording | Multi-section reject: section-1 packet fate unclear | No |
| M-6 | MEDIUM | traceability | STORY-127 glob has no BC home | No |
| M-7 | MEDIUM | undefined-constant | EPB fixed-field overhead value not named in BC | No |
| L-1 | LOW | formatting | EC-003 unescaped pipe char in markdown | No |
| L-2 | LOW | overflow | ts_usecs intermediate overflow residual after H-1 fix | No |
| L-3 | LOW | wording | EC-003 confusing parenthetical (reclassified from M-4) | No |
| O-1 | Obs | process-gap | All 10 BCs reached WRITTEN with VP-NNN = — | No |
| O-2 | Obs | process-gap | error-taxonomy input-hash is N/A | No |
| O-3 | Obs | doc-gap | reader.rs + README still say pcapng unsupported | No |
| O-4 | Obs | test-gap | Snaplen-truncation parity fixture not planned | No |
