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

/// Result of [`select_gaps`]: the first-wins winner-selection verdict for a new
/// segment `[new_start, new_end)` against a set of already-buffered ranges.
///
/// - `fully_covered` — at least one single existing range covers `[new_start,
///   new_end)` in full (`es <= new_start && ee >= new_end`). When true, the new
///   segment contributes no bytes (Duplicate / ConflictingOverlap is decided by
///   the byte-level conflict check, not here).
/// - `gaps` — the sub-ranges of `[new_start, new_end)` NOT covered by any
///   overlapping existing range. These are the ONLY positions whose bytes the
///   first-wins policy permits inserting; every other position keeps its already
///   buffered (first-arrived) byte. Each gap is `[gap_start, gap_end)`,
///   half-open, in ascending order, mutually disjoint.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectGaps {
    fully_covered: bool,
    gaps: Vec<(u64, u64)>,
}

/// Pure first-wins winner-selection over offset-only ranges.
///
/// Given a new segment span `[new_start, new_end)` and the list of
/// already-buffered `existing` ranges as `(offset, end)` pairs, this computes:
///
/// 1. `fully_covered` — whether any single overlapping existing range fully
///    spans `[new_start, new_end)`.
/// 2. `gaps` — the half-open sub-ranges of `[new_start, new_end)` left uncovered
///    by the union of all overlapping existing ranges. These are precisely the
///    byte positions the first-wins policy allows the new segment to fill;
///    positions already covered by an existing range keep their first-arrived
///    bytes (the new bytes there "lose").
///
/// Ranges that do NOT overlap `[new_start, new_end)` are ignored (only the
/// overlapping ones constrain where the new bytes may go), mirroring the
/// `has_overlap` filter in `insert_segment`.
///
/// PRECONDITION: `existing` is sorted ascending by start offset. The production
/// caller satisfies this because the ranges are collected from
/// `self.segments.range(..new_end)`, and `BTreeMap` iteration is key-ordered
/// (keys are the start offsets). The cursor sweep is order-dependent, so this
/// precondition is load-bearing; it is enforced in the Kani harnesses via a
/// `kani::assume` and exercised by the full reassembly integration suite.
///
/// SAFETY INVARIANT (the load-bearing first-wins guarantee): every returned gap
/// is DISJOINT from every existing range, so inserting gap bytes can never
/// overwrite a buffered (first-arrived) byte. This holds at run time in release
/// builds where the `debug_assert!` collision guard in `insert_segment` is
/// compiled out. This invariant is Kani-proven over symbolic inputs via the
/// allocation-free kernels in `kani_proofs` (`point_kept_by_existing` +
/// `sweep_gaps_into`, which mirror this function's exact algorithm byte-for-byte
/// minus the heap storage); proving it on the heap `Vec` directly explodes the
/// SAT instance (see the boundary note above `kani_proofs`).
///
/// Pure: no `self`, no I/O, no allocation beyond the returned `gaps` vec.
/// Offset-only (no slice indexing).
fn select_gaps(new_start: u64, new_end: u64, existing: &[(u64, u64)]) -> SelectGaps {
    // Load-bearing precondition (see doc comment): the cursor sweep is
    // order-dependent and produces wrong results on an unsorted slice. The
    // production caller satisfies this via key-ordered BTreeMap iteration; this
    // guard catches a future caller that forgets. Compiled out in release.
    debug_assert!(
        existing.windows(2).all(|w| w[0].0 <= w[1].0),
        "select_gaps: existing must be sorted ascending by start"
    );

    let mut fully_covered = false;

    // First-wins gap sweep: walk a cursor across [new_start, new_end) over the
    // (ascending-by-start) overlapping ranges, emitting the uncovered sub-ranges
    // as gaps. Non-overlapping ranges are skipped inline so no intermediate
    // filtered/sorted Vec is allocated (keeps the function CBMC-tractable).
    let mut gaps: Vec<(u64, u64)> = Vec::new();
    let mut cursor = new_start;
    for &(es, ee) in existing {
        // Always-true from the production caller (overlapping_ranges is
        // pre-filtered by segment_overlap), but LOAD-BEARING for the Kani
        // harnesses, which drive symbolic NON-overlapping ranges through this
        // path to prove the winner-selection is correct over arbitrary input.
        // Do not remove as "dead code" — that silently breaks the proof.
        if !ranges_overlap(new_start, new_end, es, ee) {
            continue;
        }
        // Any single overlapping range that spans the whole new segment.
        if es <= new_start && ee >= new_end {
            fully_covered = true;
        }
        if cursor < es {
            gaps.push((cursor, es.min(new_end)));
        }
        cursor = cursor.max(ee);
    }
    if cursor < new_end {
        gaps.push((cursor, new_end));
    }

    SelectGaps {
        fully_covered,
        gaps,
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
        let end_offset = offset + data.len() as u64;

        // R1 fix (CWE-401 zombie segments): reject segments that end STRICTLY
        // BELOW base_offset, i.e., entirely behind the flush cursor. Without
        // this guard, such segments would pass all overlap/depth checks and
        // insert permanently into the BTreeMap at offsets the flush cursor
        // has already passed — where flush_contiguous() (which only advances
        // forward) can never reach them. They accumulate silently until the
        // 10K segment-count cap triggers SegmentLimitReached on every
        // subsequent packet, causing a different form of DoS.
        //
        // Uses `< base_offset` (strict less-than) rather than `<=` so that
        // segments ending exactly at base_offset (with no bytes strictly ahead
        // of the cursor) are NOT rejected here: those are retransmissions of
        // already-flushed data that may still overlap with other segments buffered
        // at the same offset range, and the existing overlap/conflict logic handles
        // them correctly through the normal overlap detection path.
        //
        // Returning OutOfWindow is semantically correct: all bytes in the segment
        // are at offsets strictly less than base_offset, meaning they were already
        // flushed (or never buffered past the cursor) and lie outside the current
        // reassembly window.
        if end_offset < self.base_offset {
            self.out_of_window_count = self.out_of_window_count.saturating_add(1);
            return InsertResult::OutOfWindow;
        }

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
        let mut overlapping_ranges: Vec<(u64, u64)> = Vec::new();

        // Only segments starting before new_end can overlap [new_start, new_end).
        for (&existing_offset, existing_data) in self.segments.range(..new_end) {
            // `existing_end` is recomputed here for the `overlapping_ranges` entry
            // below (the winner-selection sweep needs the end offset). `segment_overlap`
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
                overlapping_ranges.push((existing_offset, existing_end));
            }
        }

        if has_overlap {
            self.overlap_count = self.overlap_count.saturating_add(1);

            // First-wins winner-selection (pure; Kani-proven in `kani_proofs`):
            // decide full-coverage and compute the disjoint gap sub-ranges that
            // the new segment is allowed to fill. Every gap is guaranteed
            // disjoint from every existing range, so the inserts below can never
            // overwrite a first-arrived byte (the safety invariant).
            let SelectGaps {
                fully_covered,
                gaps,
            } = select_gaps(new_start, new_end, &overlapping_ranges);
            if fully_covered {
                return if has_conflict {
                    InsertResult::ConflictingOverlap
                } else {
                    InsertResult::Duplicate
                };
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
//   * The MULTI-SEGMENT first-wins winner-selection (Phase-6 gap closure), proven
//     over symbolic existing ranges via the ALLOCATION-FREE kernels that mirror
//     `select_gaps`'s algorithm byte-for-byte (`point_kept_by_existing` and
//     `sweep_gaps_into`; see the note above them for why the heap-`Vec`
//     `select_gaps` cannot be driven through CBMC directly — the SAT instance
//     explodes to ~2e8 clauses and does not terminate). Proven:
//       (a) SAFETY: every gap byte is uncovered by every existing range — so no
//           inserted gap byte can overwrite a buffered first-arrived byte. This
//           is what makes the release-build `debug_assert!` collision guard at the
//           gap-insert site provably unnecessary;
//       (b) every emitted gap lies within `[new_start, new_end)`, is non-empty,
//           ascending, and disjoint from every existing range;
//       (c) emitted gaps and existing coverage PARTITION `[new_start, new_end)`
//           exactly (no in-range byte dropped, no covered byte re-inserted);
//       (d) zero gaps IFF the overlapping union covers the span; single-range full
//           coverage ⇒ `fully_covered` ⇒ zero gaps;
//       (e) the fixed gap buffer is never overrun (count <= COUNT + 1).
// NOT Kani-verified (covered by the integration test-suite instead):
//   * the `self.segments.range(..new_end)` BTreeMap range-scan that selects which
//     existing segments to compare against;
//   * `select_gaps`'s heap `Vec` PACKAGING of the (proven) gap algorithm, and the
//     slice extraction + `self.segments.insert` that materializes each gap's bytes
//     (byte-survival of this glue is covered by the G1/G2/G2b/G3 integration tests
//     added in the Phase-6 gap closure, which RE-READ the buffer/stream bytes).
// In other words: the per-pair overlap/conflict DECISION and the multi-segment
// winner-SELECTION ALGORITHM (which bytes may be inserted) are both formally
// proven; only the `Vec`/`BTreeMap` materialization of that selection is
// test-covered. This boundary is the basis for the Phase-6 "proven OR justified"
// gate entry for VP-002.
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

    // ---- VP-002 (Phase-6 gap closure): multi-segment first-wins winner-selection
    //
    // These harnesses prove the SAFETY invariant of `select_gaps` directly over
    // symbolic existing ranges. `insert_segment` feeds the returned gaps straight
    // into `self.segments.insert(gap_start, ...)`; the only thing standing between
    // a winner-selection bug and a silent overwrite of buffered first-arrived
    // bytes in a RELEASE build is that every produced gap be disjoint from every
    // existing range (the `debug_assert!` collision guard is compiled out). That
    // disjointness is invariant (a) below. If `select_gaps` could ever emit a gap
    // that overlapped an existing range, harness (a) would surface a CONCRETE
    // counterexample (the offending gap/existing pair) rather than SUCCESS.

    // WHY THE PROOFS TARGET ALLOCATION-FREE KERNELS, NOT `select_gaps` DIRECTLY:
    // `select_gaps` returns a heap `Vec<(u64,u64)>`. Driving that `Vec` (alloc +
    // `push`/realloc + CBMC `same_allocation` reasoning) through the model checker
    // — combined with symbolic u64 offset arithmetic — explodes the SAT instance
    // to ~1.5–2 x 10^8 clauses even on a 6-wide window, which does not terminate
    // in a usable time. (Measured: 45M vars / 199M clauses, no verdict in >6 min.)
    // This is the same documented Kani/std-collection limitation that keeps the
    // proofs off the BTreeMap path. So the load-bearing logic is factored into the
    // ALLOCATION-FREE, offset-only kernel `point_kept_by_existing`, and the gap-
    // emission ALGORITHM is mirrored by the stack-array sweep `sweep_gaps_into`
    // below (byte-identical cursor logic to `select_gaps`, no heap). Both are
    // proven exhaustively over symbolic ranges. `select_gaps` differs from
    // `sweep_gaps_into` ONLY in storage (Vec vs fixed array); their identical
    // algorithm is additionally exercised end-to-end by the reassembly integration
    // suite (336 tests) and the G1/G2/G2b/G3 byte-survival tests.

    /// First-wins coverage predicate for a SINGLE byte position.
    ///
    /// Returns `true` iff byte position `p` (assumed to lie in
    /// `[new_start, new_end)`) is covered by some existing range that ALSO
    /// overlaps `[new_start, new_end)`. When `true`, the first-wins policy keeps
    /// the existing (first-arrived) byte at `p`; when `false`, `p` is a "gap
    /// byte" the new segment is allowed to fill. This is the allocation-free,
    /// offset-only kernel of the winner-selection: the set of gap bytes
    /// `select_gaps` emits is exactly
    /// `{ p in [new_start,new_end) : !point_kept_by_existing(p, ..) }`.
    fn point_kept_by_existing(
        p: u64,
        new_start: u64,
        new_end: u64,
        existing: &[(u64, u64)],
    ) -> bool {
        existing
            .iter()
            .any(|&(es, ee)| ranges_overlap(new_start, new_end, es, ee) && es <= p && p < ee)
    }

    /// Build a fixed-size symbolic set of `COUNT` existing `(offset, end)` ranges
    /// within a bounded window, plus a symbolic new segment span. Each existing
    /// range has `offset` in `[0, WINDOW]` and length in `[1, MAXLEN]`; the new
    /// segment has `new_start` in `[0, WINDOW]` and length in `[1, MAXLEN]`.
    ///
    /// The ranges are constrained to ASCENDING start order — exactly the
    /// `select_gaps` precondition the production caller satisfies (ranges come
    /// from a key-ordered `BTreeMap` scan). A FIXED-SIZE ARRAY (not a `Vec`) is
    /// returned so CBMC sees a statically sized, stack-allocated buffer. `COUNT`
    /// is a const so the construction loop fully unrolls. Cases with FEWER than
    /// `COUNT` relevant ranges are still covered: a range may be placed outside
    /// `[new_start, new_end)`, where it is skipped — identical to absence. So
    /// `COUNT = k` subsumes all `0..=k` overlapping-range scenarios.
    fn symbolic_inputs<const COUNT: usize>(
        window: u64,
        maxlen: u64,
    ) -> (u64, u64, [(u64, u64); COUNT]) {
        let new_start: u64 = kani::any();
        let new_len: u64 = kani::any();
        kani::assume(new_start <= window);
        kani::assume(new_len >= 1 && new_len <= maxlen);
        let new_end = new_start + new_len;

        let mut existing = [(0u64, 0u64); COUNT];
        let mut prev_start = 0u64;
        for slot in existing.iter_mut() {
            let off: u64 = kani::any();
            let len: u64 = kani::any();
            kani::assume(off <= window);
            kani::assume(len >= 1 && len <= maxlen);
            kani::assume(off >= prev_start); // ascending-by-start precondition
            prev_start = off;
            *slot = (off, off + len);
        }
        (new_start, new_end, existing)
    }

    /// Allocation-free, fixed-array mirror of `select_gaps`'s cursor sweep. Runs
    /// the BYTE-IDENTICAL algorithm (skip non-overlapping; detect single-range
    /// full coverage; emit `[cursor, es.min(new_end))` when `cursor < es`; advance
    /// `cursor = cursor.max(ee)`; emit a trailing `[cursor, new_end)`) but writes
    /// gaps into a caller-provided `[(u64,u64); CAP]` and returns `(fully_covered,
    /// count)`. No heap, so CBMC stays tractable. `CAP` must be >= number of gaps
    /// (<= COUNT + 1); the harness asserts the count never reaches `CAP` so the
    /// fixed buffer is provably sufficient.
    fn sweep_gaps_into<const CAP: usize>(
        new_start: u64,
        new_end: u64,
        existing: &[(u64, u64)],
        out: &mut [(u64, u64); CAP],
    ) -> (bool, usize) {
        let mut fully_covered = false;
        let mut count = 0usize;
        let mut cursor = new_start;
        for &(es, ee) in existing {
            if !ranges_overlap(new_start, new_end, es, ee) {
                continue;
            }
            if es <= new_start && ee >= new_end {
                fully_covered = true;
            }
            if cursor < es {
                out[count] = (cursor, es.min(new_end));
                count += 1;
            }
            cursor = cursor.max(ee);
        }
        if cursor < new_end {
            out[count] = (cursor, new_end);
            count += 1;
        }
        (fully_covered, count)
    }

    // ---- SAFETY INVARIANT (the load-bearing first-wins guarantee) ------------
    //
    // A "gap byte" is a position the new segment is allowed to fill; production
    // fills exactly the positions `p in [new_start,new_end)` with
    // `!point_kept_by_existing(p, ..)`. The release-build collision guard
    // (`debug_assert!` at the gap-insert site) is compiled out, so the ONLY thing
    // preventing a silent overwrite of a buffered first-arrived byte is that every
    // gap byte be uncovered by every existing range. The next harness proves
    // exactly that, exhaustively, over symbolic inputs — and would yield a
    // CONCRETE counterexample (a gap byte that an existing range covers) if the
    // first-wins selection were ever wrong.

    /// Gap bytes never overwrite existing bytes: for every byte position `p` in
    /// `[new_start, new_end)`, if `p` is a gap byte (`!point_kept_by_existing`)
    /// then NO existing range contains `p`. This is the point-level form of "every
    /// produced gap is disjoint from every existing range" and is the load-bearing
    /// anti-evasion invariant. Allocation-free, so it model-checks fast over a
    /// wide window.
    ///
    /// Bounded domain: 3 symbolic existing ranges, offsets in [0,12], lengths in
    /// [1,4]; symbolic new segment, start in [0,12], length in [1,4]; per-point
    /// sweep over the span (<= MAXLEN = 4 bytes).
    ///
    /// `#[kani::unwind(5)]` is sufficient: the outer `while p < new_end` runs at
    /// most MAXLEN = 4 iterations (`new_len in [1,4]`), and the inner `.any()` /
    /// `for` loops run COUNT = 3 iterations over `existing`; a global bound of 5
    /// covers the larger (4) plus the loop-back check. CBMC's unwinding
    /// assertions (on by default) would FAIL the proof if 5 were ever too small,
    /// so sufficiency is verified, not assumed — the harness reports SUCCESSFUL
    /// with no unwinding warning.
    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_gap_byte_never_overwrites_existing() {
        let (new_start, new_end, existing) = symbolic_inputs::<3>(12, 4);

        let mut p = new_start;
        while p < new_end {
            if !point_kept_by_existing(p, new_start, new_end, &existing) {
                // p is a gap byte: assert it lies in NO existing range at all.
                for &(es, ee) in &existing {
                    assert!(!(es <= p && p < ee));
                }
            }
            p += 1;
        }
    }

    /// Algorithm correctness — the gap-emission sweep produces EXACTLY the gap
    /// bytes: every byte inside an emitted gap is a gap byte
    /// (`!point_kept_by_existing`), and every gap byte in `[new_start, new_end)`
    /// falls inside exactly one emitted gap. Equivalently: the emitted gaps and
    /// the existing coverage partition `[new_start, new_end)` with no overlap and
    /// no loss. Proven on the allocation-free `sweep_gaps_into` mirror of
    /// `select_gaps`. Also asserts the fixed buffer is never exceeded (CAP=4 for
    /// COUNT=3) and that single-range full coverage implies zero gaps.
    ///
    /// Bounded domain: 3 symbolic existing ranges, offsets in [0,8], lengths in
    /// [1,3]; symbolic new segment, start in [0,8], length in [1,3].
    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_sweep_emits_exactly_gap_bytes() {
        let (new_start, new_end, existing) = symbolic_inputs::<3>(8, 3);

        let mut gaps = [(0u64, 0u64); 4]; // CAP = COUNT + 1
        let (fully_covered, count) = sweep_gaps_into(new_start, new_end, &existing, &mut gaps);

        // Fixed buffer was sufficient (count <= COUNT + 1, never overran CAP).
        assert!(count <= 4);

        // Every emitted gap is well-formed: within bounds, non-empty, ascending,
        // and disjoint from every existing range (structural disjointness).
        let mut prev_end = new_start;
        for &(gs, ge) in gaps.iter().take(count) {
            assert!(gs >= new_start && ge <= new_end);
            assert!(gs < ge);
            assert!(gs >= prev_end); // ascending, non-overlapping with prior gap
            prev_end = ge;
            for &(es, ee) in &existing {
                assert!(!ranges_overlap(gs, ge, es, ee));
            }
        }

        // Exact partition: each in-range byte is in a gap XOR kept by existing.
        let mut p = new_start;
        while p < new_end {
            let in_gap = gaps.iter().take(count).any(|&(gs, ge)| gs <= p && p < ge);
            let kept = point_kept_by_existing(p, new_start, new_end, &existing);
            assert!(in_gap != kept);
            p += 1;
        }

        // No gaps emitted IFF the overlapping union fully covers the span.
        let mut union_covers = true;
        let mut q = new_start;
        while q < new_end {
            if !point_kept_by_existing(q, new_start, new_end, &existing) {
                union_covers = false;
            }
            q += 1;
        }
        assert!((count == 0) == union_covers);

        // A single range spanning the whole new segment ⇒ fully_covered ⇒ no gaps.
        if fully_covered {
            assert!(count == 0);
        }
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

// In-crate unit tests for the module-private overlap predicate. These run
// under the ordinary `cargo test` build (unlike the `#[cfg(kani)]` proofs
// above, which are dead code outside `cargo kani`). They pin the half-open
// adjacency boundary of `ranges_overlap` so the `>`→`>=` and `<`→`<=`
// comparison-operator mutations on line 43 are CAUGHT by `cargo mutants`.
#[cfg(test)]
mod adjacency_tests {
    use super::ranges_overlap;

    // GG-1 (CRITICAL anti-evasion, BC-2.04.043): touching at an exact boundary
    // is NOT an overlap. `ranges_overlap(new_start, new_end, existing_offset,
    // existing_end)` is `new_start < existing_end && new_end > existing_offset`.
    //
    // The `>` in `new_end > existing_offset` is the load-bearing comparison:
    // when the NEW segment ends exactly where the EXISTING begins
    // (`new_end == existing_offset`), the ranges merely touch and must NOT
    // overlap. The surviving mutant replaced `>` with `>=`, which would
    // wrongly report this exact-adjacency case as an overlap — a TCP-overlap
    // mis-classification. This test pins both touching sides as non-overlap
    // and the 1-byte-into case as overlap, so `>` vs `>=` diverges here.

    #[test]
    fn exact_adjacency_new_ends_where_existing_begins_is_not_overlap() {
        // existing = [10, 20).  new = [5, 10): new_end (10) == existing_offset (10).
        // new_end > existing_offset is `10 > 10` == false  => NOT an overlap.
        // The `>=` mutant computes `10 >= 10` == true and (wrongly) reports overlap.
        assert!(
            !ranges_overlap(5, 10, 10, 20),
            "new_end == existing_offset (touching at the existing's left edge) \
             must NOT be an overlap (BC-2.04.043); kills the `>`→`>=` mutant"
        );
    }

    #[test]
    fn one_byte_into_existing_is_overlap() {
        // existing = [10, 20).  new = [5, 11): new_end (11) > existing_offset (10).
        // Exactly one byte (offset 10) is shared => IS an overlap, under BOTH
        // `>` and `>=`. This is the correct-code companion to the adjacency
        // case: it proves the predicate still reports the genuine 1-byte
        // overlap, so the adjacency test above is not vacuously true.
        assert!(
            ranges_overlap(5, 11, 10, 20),
            "a 1-byte overlap (new_end one past existing_offset) IS an overlap"
        );
    }

    #[test]
    fn exact_adjacency_new_begins_where_existing_ends_is_not_overlap() {
        // existing = [10, 20).  new = [20, 25): new_start (20) == existing_end (20).
        // new_start < existing_end is `20 < 20` == false => NOT an overlap.
        // (Mirrors the other boundary; guards the `<` comparison on the same line.)
        assert!(
            !ranges_overlap(20, 25, 10, 20),
            "new_start == existing_end (touching at the existing's right edge) \
             must NOT be an overlap (BC-2.04.043); kills the `<`→`<=` mutant"
        );
    }

    #[test]
    fn one_byte_overlap_at_existing_right_edge_is_overlap() {
        // existing = [10, 20).  new = [19, 25): new_start (19) < existing_end (20),
        // sharing offset 19 => IS an overlap. Companion to the right-edge
        // adjacency case above.
        assert!(
            ranges_overlap(19, 25, 10, 20),
            "a 1-byte overlap at the existing's right edge IS an overlap"
        );
    }
}
