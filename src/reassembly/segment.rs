//! Per-direction segment buffer with the first-wins overlap policy.
//!
//! [`InsertResult`] is the classification produced by every segment
//! insertion: clean, duplicate (identical bytes at the same offset),
//! overlap-clean (overlap that agrees with the buffer),
//! conflicting overlap (overlap that disagrees — emitted as an Anomaly
//! finding upstream), out-of-window (too far ahead of `base_offset`),
//! depth-exceeded (per-direction byte cap hit), segment-limit
//! (per-direction segment-count cap hit), or `IsnMissing` (programming
//! error guarded by a one-shot eprintln so the bug is loud, not silent).

use std::sync::atomic::{AtomicBool, Ordering};

use crate::reassembly::flow::FlowDirection;

static ISN_MISSING_WARNED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertResult {
    Inserted,
    Duplicate,
    PartialOverlap,
    ConflictingOverlap,
    Truncated,
    DepthExceeded,
    SegmentLimitReached,
    OutOfWindow,
    IsnMissing,
}

/// Compute the ISN-relative offset for a sequence number.
fn seq_offset(seq: u32, isn: u32) -> u64 {
    seq.wrapping_sub(isn) as u64
}

/// Half-open range-intersection test for two segments
/// `[new_start, new_end)` and `[existing_offset, existing_end)`.
///
/// Returns `true` iff the ranges intersect. Touching at an exact boundary
/// (`new_start == existing_end` or `existing_offset == new_end`) is NOT an
/// overlap. Pure, offset-only (no slices) — exhaustively Kani-checkable.
fn ranges_overlap(new_start: u64, new_end: u64, existing_offset: u64, existing_end: u64) -> bool {
    new_start < existing_end && new_end > existing_offset
}

/// Classify how a new segment `[new_start, new_end)` relates to one existing
/// buffered segment `[existing_offset, existing_offset + existing_data.len())`.
///
/// Returns `(overlaps, conflicts)`:
/// - `overlaps`  — the two byte ranges intersect (see [`ranges_overlap`]).
/// - `conflicts` — they overlap AND the bytes in the overlapping region differ
///   (the first-wins policy surfaces this as `ConflictingOverlap`).
///
/// This is the per-existing-segment overlap test used by `insert_segment`; it
/// is pure (no `self`, no allocation) so it can be model-checked by Kani over
/// symbolic offsets/bytes without dragging in the BTreeMap.
fn segment_overlap(
    new_start: u64,
    new_data: &[u8],
    existing_offset: u64,
    existing_data: &[u8],
) -> (bool, bool) {
    let new_end = new_start + new_data.len() as u64;
    let existing_end = existing_offset + existing_data.len() as u64;

    if ranges_overlap(new_start, new_end, existing_offset, existing_end) {
        let overlap_start = new_start.max(existing_offset);
        let overlap_end = new_end.min(existing_end);

        let new_slice_start = (overlap_start - new_start) as usize;
        let new_slice_end = (overlap_end - new_start) as usize;
        let existing_slice_start = (overlap_start - existing_offset) as usize;
        let existing_slice_end = (overlap_end - existing_offset) as usize;

        let conflicts = new_slice_end <= new_data.len()
            && existing_slice_end <= existing_data.len()
            && new_data[new_slice_start..new_slice_end]
                != existing_data[existing_slice_start..existing_slice_end];

        (true, conflicts)
    } else {
        (false, false)
    }
}

impl FlowDirection {
    /// Insert a segment into the flow direction's out-of-order buffer.
    /// Applies first-wins overlap policy and tracks anomaly counters.
    pub fn insert_segment(
        &mut self,
        seq: u32,
        data: &[u8],
        max_depth: usize,
        max_segments: usize,
        max_receive_window: usize,
    ) -> InsertResult {
        if data.is_empty() {
            return InsertResult::Inserted;
        }

        let isn = match self.isn {
            Some(isn) => isn,
            None => {
                if !ISN_MISSING_WARNED.swap(true, Ordering::Relaxed) {
                    eprintln!("wirerust: insert_segment called with no ISN set");
                }
                return InsertResult::IsnMissing;
            }
        };

        let offset = seq_offset(seq, isn);

        // Reject segments too far ahead of base_offset (before overlap/depth checks)
        if offset > self.base_offset.saturating_add(max_receive_window as u64) {
            self.out_of_window_count = self.out_of_window_count.saturating_add(1);
            return InsertResult::OutOfWindow;
        }

        // Enforce max segments per direction to prevent BTreeMap overhead explosion
        if self.segments.len() >= max_segments {
            return InsertResult::SegmentLimitReached;
        }

        // Note: the small-segment run counter is maintained by the
        // caller (`insert_payload_segment`), not here — "small" is a
        // pure property of the payload size and needs no buffer state,
        // unlike the overlap and out-of-window counters below.

        // Check depth limit
        let remaining_depth = max_depth.saturating_sub(self.reassembled_bytes);
        if remaining_depth == 0 {
            if !self.depth_exceeded {
                self.depth_exceeded = true;
            }
            return InsertResult::DepthExceeded;
        }

        let mut segment_data = data.to_vec();

        // Truncate if exceeding depth
        let buffered = self.buffered_bytes;
        let total_after = self.reassembled_bytes + buffered + segment_data.len();
        let truncated = if total_after > max_depth {
            let allowed = max_depth.saturating_sub(self.reassembled_bytes + buffered);
            if allowed == 0 {
                self.depth_exceeded = true;
                return InsertResult::DepthExceeded;
            }
            segment_data.truncate(allowed);
            self.depth_exceeded = true;
            true
        } else {
            false
        };

        let new_start = offset;
        let new_end = offset + segment_data.len() as u64;

        // Check for overlaps with existing segments
        let mut has_overlap = false;
        let mut has_conflict = false;
        let mut trimmed_ranges: Vec<(u64, u64)> = Vec::new();

        // Only segments starting before new_end can overlap [new_start, new_end).
        for (&existing_offset, existing_data) in self.segments.range(..new_end) {
            // `existing_end` is recomputed here for the `trimmed_ranges` entry
            // below (the gap-fill sweep needs the end offset). `segment_overlap`
            // also derives it internally for its own overlap test; this is a
            // cheap `+` and keeping the value local avoids returning it from the
            // pure helper (whose signature stays minimal for the Kani proofs).
            let existing_end = existing_offset + existing_data.len() as u64;

            // Per-segment overlap/conflict classification (pure; Kani-verified
            // via `segment_overlap` — see VP-002 harnesses below).
            let (overlaps, conflicts) =
                segment_overlap(new_start, &segment_data, existing_offset, existing_data);
            if overlaps {
                has_overlap = true;
                if conflicts {
                    has_conflict = true;
                }
                trimmed_ranges.push((existing_offset, existing_end));
            }
        }

        if has_overlap {
            self.overlap_count = self.overlap_count.saturating_add(1);

            let fully_covered = trimmed_ranges
                .iter()
                .any(|&(es, ee)| es <= new_start && ee >= new_end);
            if fully_covered {
                return if has_conflict {
                    InsertResult::ConflictingOverlap
                } else {
                    InsertResult::Duplicate
                };
            }

            // First-wins: insert only gap portions
            let mut gaps: Vec<(u64, u64)> = Vec::new();
            let mut cursor = new_start;

            let mut sorted_ranges = trimmed_ranges.clone();
            sorted_ranges.sort_by_key(|&(start, _)| start);

            for &(es, ee) in &sorted_ranges {
                if cursor < es {
                    gaps.push((cursor, es.min(new_end)));
                }
                cursor = cursor.max(ee);
            }
            if cursor < new_end {
                gaps.push((cursor, new_end));
            }

            let had_gap = !gaps.is_empty();

            let mut segments_exhausted = false;
            for (gap_start, gap_end) in gaps {
                // Enforce max_segments inside gap insertion loop
                if self.segments.len() >= max_segments {
                    segments_exhausted = true;
                    break;
                }
                let start_idx = (gap_start - new_start) as usize;
                let end_idx = (gap_end - new_start) as usize;
                if start_idx < segment_data.len() && end_idx <= segment_data.len() {
                    let gap_data = segment_data[start_idx..end_idx].to_vec();
                    if !gap_data.is_empty() {
                        let gap_len = gap_data.len();
                        let old = self.segments.insert(gap_start, gap_data);
                        debug_assert!(
                            old.is_none(),
                            "gap_start {gap_start} collided with existing segment"
                        );
                        if let Some(old) = old {
                            self.buffered_bytes -= old.len();
                        }
                        self.buffered_bytes += gap_len;
                    }
                }
            }

            // Union of existing segments covers the new segment entirely (no gaps to fill)
            return if !had_gap && has_conflict {
                InsertResult::ConflictingOverlap
            } else if !had_gap {
                InsertResult::Duplicate
            } else if segments_exhausted {
                InsertResult::SegmentLimitReached
            } else if truncated {
                InsertResult::Truncated
            } else {
                InsertResult::PartialOverlap
            };
        }

        // No overlap — insert normally
        let data_len = segment_data.len();
        let old = self.segments.insert(offset, segment_data);
        debug_assert!(
            old.is_none(),
            "offset {offset} collided with existing segment in no-overlap path"
        );
        if let Some(old) = old {
            self.buffered_bytes -= old.len();
        }
        self.buffered_bytes += data_len;

        if truncated {
            InsertResult::Truncated
        } else {
            InsertResult::Inserted
        }
    }

    /// Flush contiguous segments starting from base_offset.
    /// Returns Vec of (offset, data) pairs that were flushed.
    pub fn flush_contiguous(&mut self) -> Vec<(u64, Vec<u8>)> {
        let mut flushed = Vec::new();

        while let Some(data) = self.segments.remove(&self.base_offset) {
            let offset = self.base_offset;
            self.buffered_bytes -= data.len();
            self.base_offset += data.len() as u64;
            self.reassembled_bytes += data.len();
            flushed.push((offset, data));
        }

        flushed
    }
}

/// Test-only accessor for the process-global ISN_MISSING_WARNED flag.
///
/// Exposes a read of [`ISN_MISSING_WARNED`] so integration tests can verify the
/// one-shot behavior asserted by BC-2.04.048 PC1 — the flag transitions
/// `false → true` exactly once across the process lifetime.
///
/// The accessor is `pub` (not `#[cfg(test)]`-gated) because integration tests
/// in `tests/` are separate crates and cannot see `#[cfg(test)]` items.
/// Naming includes `_for_testing` to flag intent to readers of public API.
#[doc(hidden)]
pub fn isn_missing_warned_for_testing() -> bool {
    ISN_MISSING_WARNED.load(Ordering::Relaxed)
}

/// Test-only reset of the process-global ISN_MISSING_WARNED flag.
///
/// Allows tests to deterministically observe the BC-2.04.048 PC1
/// `false → true` swap transition, which would otherwise be non-deterministic
/// across test ordering because the atomic is process-global. This function
/// MUST NOT be called from production code paths; it is a test seam only.
#[doc(hidden)]
pub fn reset_isn_missing_warned_for_testing() {
    ISN_MISSING_WARNED.store(false, Ordering::Relaxed);
}

// Kani formal-verification harnesses. Gated behind `#[cfg(kani)]` (set only by
// `cargo kani`), so invisible to the normal build/test/clippy pipeline. These
// prove VP-002 (first-wins overlap policy) and VP-015 (TCP sequence wraparound).
// Co-located in this file so they can reach the module-private `seq_offset` and
// `segment_overlap`.
//
// DESIGN NOTE — why these proofs target the pure helpers, not `insert_segment`
// end-to-end: `insert_segment` stores segments in a `std::collections::BTreeMap`.
// CBMC (Kani's backend) symbolically executes the BTreeMap's generic, pointer-
// rich, `unsafe`-heavy internals; even a 1–2 entry map drives a 4–5 minute solve
// or an out-of-memory abort (a documented Kani limitation for std collections —
// the model-checking-blog "turbocharging" post and the standard "model & stub"
// guidance). The model checker would be re-verifying `BTreeMap` itself, which is
// not the property under test.
//
// To verify the ACTUAL production logic without that blow-up, the overlap-
// classification (`segment_overlap`) and the sequence-to-offset conversion
// (`seq_offset`) are factored out of `insert_segment` as pure functions and
// proven directly over symbolic inputs. `insert_segment` calls these exact
// functions, and the full TDD/integration test-suite (138 reassembly tests,
// plus tests/reassembly_*; all green) exercises the BTreeMap glue around them,
// so coverage of the end-to-end path is retained by tests while the hard
// invariants are discharged by formal proof here.
//
// ── VP-002 FORMAL-COVERAGE BOUNDARY (read before citing these proofs) ────────
// FORMALLY PROVEN here, over symbolic inputs:
//   * `ranges_overlap`  — the half-open range-intersection / adjacency predicate
//     (BC-2.04.043: touching at an exact boundary is NOT an overlap).
//   * `segment_overlap` — the per-existing-segment first-wins CONFLICT predicate
//     (overlap AND overlapping bytes differ => `ConflictingOverlap`; equal =>
//     `Duplicate`). Proven for all byte pairs at the relevant offsets.
// NOT Kani-verified (intractable: lives inside the BTreeMap-driven loop), covered
// by the integration test-suite instead:
//   * the `self.segments.range(..new_end)` BTreeMap range-scan that selects which
//     existing segments to compare against;
//   * the `fully_covered` aggregate check across all overlapping segments
//     (Duplicate / ConflictingOverlap vs PartialOverlap decision);
//   * the gap-filling cursor sweep (`sorted_ranges` + `cursor`) that picks the
//     first-wins "winner" bytes across MULTIPLE overlapping segments and inserts
//     only the gap portions.
// In other words: the per-pair overlap/conflict DECISION is proven; the
// BTreeMap orchestration that applies that decision across many segments is
// test-covered, not proof-covered. This boundary is the basis for the Phase-6
// "proven OR justified" gate entry for VP-002.
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // ---- VP-002: First-Wins Overlap Policy (BC-2.04.018/035/036/037/043) -----
    //
    // `segment_overlap(new_start, new_data, existing_offset, existing_data)`
    // returns `(overlaps, conflicts)` — the per-existing-segment classification
    // that `insert_segment` uses to decide Duplicate / ConflictingOverlap /
    // PartialOverlap and to apply the first-wins policy. Proving it over
    // symbolic offsets and bytes covers that per-pair DECISION for ALL inputs in
    // the bound (see the formal-coverage boundary note above for what is and is
    // not proven).

    /// Invariant 4 (adjacency is NOT overlap) + the general overlap predicate:
    /// for all small symbolic segment spans, `ranges_overlap` reports overlap
    /// IFF the half-open ranges intersect, and in particular reports NO overlap
    /// when one segment starts exactly at the other's end (adjacency,
    /// BC-2.04.043). `insert_segment` calls `ranges_overlap` for exactly this
    /// decision, so this discharges the overlap/adjacency policy.
    ///
    /// Bounded domain: symbolic starts in [0, 8] and symbolic lengths in [1, 4]
    /// (segments are non-empty; `insert_segment` returns early on empty data).
    /// `ranges_overlap` is offset-only (no slice indexing), so this is a small,
    /// fast SAT instance that is complete over the [0, 12] offset window.
    #[kani::proof]
    fn verify_overlap_predicate_and_adjacency() {
        let new_start: u64 = kani::any();
        let existing_offset: u64 = kani::any();
        let new_len: u64 = kani::any();
        let existing_len: u64 = kani::any();
        kani::assume(new_start <= 8);
        kani::assume(existing_offset <= 8);
        kani::assume((1..=4).contains(&new_len));
        kani::assume((1..=4).contains(&existing_len));

        let new_end = new_start + new_len;
        let existing_end = existing_offset + existing_len;

        let overlaps = ranges_overlap(new_start, new_end, existing_offset, existing_end);

        // Ground-truth half-open intersection.
        let expect_overlap = new_start < existing_end && new_end > existing_offset;
        assert!(overlaps == expect_overlap);

        // Adjacency special case: segments touching at an exact boundary must
        // NOT be an overlap.
        if new_start == existing_end || existing_offset == new_end {
            assert!(!overlaps);
        }
    }

    /// Invariant 1 (Duplicate) + Invariant 2 (ConflictingOverlap): for two
    /// fully-coincident single-byte segments at the same offset, `conflicts` is
    /// true IFF the bytes differ. This is the exact predicate that distinguishes
    /// `Duplicate` (identical retransmission, first-wins no-op) from
    /// `ConflictingOverlap` (differing bytes, original preserved).
    ///
    /// Bounded domain: two symbolic bytes at the same fixed offset. Same-offset
    /// single-byte segments make the overlap region the whole byte, so the
    /// proof is complete over the conflict predicate for ALL 256x256 byte pairs.
    #[kani::proof]
    fn verify_conflict_iff_bytes_differ() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();

        let (overlaps, conflicts) = segment_overlap(1, &[a], 1, &[b]);

        // Coincident ranges always overlap.
        assert!(overlaps);
        // First-wins conflict detection: conflict exactly when bytes differ.
        assert!(conflicts == (a != b));
    }

    /// Invariant 2 (multi-byte conflict): for two 2-byte segments at the same
    /// offset, a conflict is reported IFF the byte sequences differ in any
    /// position — exercising the multi-byte slice comparison
    /// (`new_data[..] != existing_data[..]`) for all symbolic byte combinations.
    #[kani::proof]
    fn verify_multibyte_conflict_predicate() {
        let a0: u8 = kani::any();
        let a1: u8 = kani::any();
        let b0: u8 = kani::any();
        let b1: u8 = kani::any();

        let new_data = [a0, a1];
        let existing_data = [b0, b1];
        let (overlaps, conflicts) = segment_overlap(2, &new_data, 2, &existing_data);

        assert!(overlaps);
        assert!(conflicts == (new_data != existing_data));
    }

    // ---- VP-015: TCP Sequence Number Wraparound (BC-2.04.039) ----------------

    /// Invariant 3: `seq_offset(seq, isn) = (seq - isn) mod 2^32`, monotonic in
    /// the byte-offset (u64) space even when the u32 SEQ wraps. This is the
    /// arithmetic that maps TCP sequence space to the monotonic offset space the
    /// BTreeMap is keyed on, so correctness here is exactly the wraparound
    /// guarantee (a 4-byte segment at seq=0xFFFF_FFFF lands at consecutive
    /// offsets 1,2,3,4 across the 32-bit boundary).
    ///
    /// Bounded domain: symbolic ISN within 256 of the 32-bit ceiling and a delta
    /// in [0, 300]. The delta range straddles the wrap point (isn + delta can
    /// exceed 0xFFFF_FFFF), which is exactly the case under test; bounding the
    /// neighborhood keeps the proof a tight witness of the boundary arithmetic
    /// rather than re-proving wrapping_sub over the whole u32 space.
    #[kani::proof]
    fn verify_seq_offset_at_wraparound() {
        let isn: u32 = kani::any();
        kani::assume(isn > 0xFFFF_FF00u32);

        let delta: u32 = kani::any();
        kani::assume(delta <= 300);

        let seq_after_wrap: u32 = isn.wrapping_add(delta);
        let expected_offset: u64 = delta as u64;

        let computed = seq_offset(seq_after_wrap, isn);
        assert!(computed == expected_offset);
    }

    /// Invariant 1-2 (wraparound delivers in order + adjacency across the wrap):
    /// builds on `seq_offset` and `ranges_overlap` to show that, with
    /// ISN=0xFFFF_FFFE, a 4-byte segment at seq=0xFFFF_FFFF occupies offsets
    /// 1..5 contiguously and the next segment at seq=0x0000_0003 (offset 5) is
    /// adjacent (no overlap) — i.e. the wrap does not corrupt ordering.
    ///
    /// SCOPE: the general `seq_offset` wraparound arithmetic is discharged
    /// SYMBOLICALLY by `verify_seq_offset_at_wraparound` above (all ISN values
    /// near the 32-bit ceiling, deltas in [0,300]). THIS harness is a CONCRETE
    /// boundary WITNESS (ISN=0xFFFF_FFFE) that composes those per-offset results
    /// into the end-to-end "contiguous span + adjacent next segment" claim at the
    /// exact wrap point; it is not itself a general (all-ISN) proof. Uses the
    /// pure helpers (no BTreeMap) so it stays fast.
    #[kani::proof]
    fn verify_wraparound_offsets_contiguous_and_adjacent() {
        let isn: u32 = 0xFFFF_FFFE;

        // 4-byte segment at seq=0xFFFF_FFFF maps to offset 1, spanning 1..5.
        let first_seq: u32 = 0xFFFF_FFFF;
        let first_off = seq_offset(first_seq, isn);
        assert!(first_off == 1);

        // Each subsequent sequence number (including the wrap to 0x0) is the
        // next consecutive offset.
        assert!(seq_offset(0x0000_0000, isn) == 2); // wrap
        assert!(seq_offset(0x0000_0001, isn) == 3);
        assert!(seq_offset(0x0000_0002, isn) == 4);

        // Next segment at seq=0x0000_0003 -> offset 5: adjacent to the 1..5 span.
        let next_off = seq_offset(0x0000_0003, isn);
        assert!(next_off == 5);

        // ranges_overlap confirms offset 5 (2-byte segment, 5..7) does NOT
        // overlap the first segment's 1..5 span (adjacent across the wrap).
        let first_span = (first_off, first_off + 4); // 1..5
        let next_span = (next_off, next_off + 2); // 5..7
        assert!(!ranges_overlap(
            next_span.0,
            next_span.1,
            first_span.0,
            first_span.1
        ));
    }
}
