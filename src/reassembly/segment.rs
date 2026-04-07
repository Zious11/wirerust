use crate::reassembly::flow::FlowDirection;

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
}

/// Compute the ISN-relative offset for a sequence number.
fn seq_offset(seq: u32, isn: u32) -> u64 {
    seq.wrapping_sub(isn) as u64
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
                eprintln!("wirerust: insert_segment called with no ISN set");
                return InsertResult::DepthExceeded;
            }
        };

        let offset = seq_offset(seq, isn);

        // Reject segments too far ahead of base_offset (before overlap/depth checks)
        if offset > self.base_offset.saturating_add(max_receive_window as u64) {
            return InsertResult::OutOfWindow;
        }

        // Enforce max segments per direction to prevent BTreeMap overhead explosion
        if self.segments.len() >= max_segments {
            return InsertResult::SegmentLimitReached;
        }

        // Track small segments (cumulative, not consecutive)
        if data.len() < 8 {
            self.small_segment_count += 1;
        }

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
            let existing_end = existing_offset + existing_data.len() as u64;

            if new_start < existing_end && new_end > existing_offset {
                has_overlap = true;

                let overlap_start = new_start.max(existing_offset);
                let overlap_end = new_end.min(existing_end);

                // Use slice comparison (SIMD-optimized) instead of byte-by-byte
                let new_slice_start = (overlap_start - new_start) as usize;
                let new_slice_end = (overlap_end - new_start) as usize;
                let existing_slice_start = (overlap_start - existing_offset) as usize;
                let existing_slice_end = (overlap_end - existing_offset) as usize;

                if new_slice_end <= segment_data.len()
                    && existing_slice_end <= existing_data.len()
                    && segment_data[new_slice_start..new_slice_end]
                        != existing_data[existing_slice_start..existing_slice_end]
                {
                    has_conflict = true;
                }

                trimmed_ranges.push((existing_offset, existing_end));
            }
        }

        if has_overlap {
            self.overlap_count += 1;

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
                            "gap_start {} collided with existing segment",
                            gap_start
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
            "offset {} collided with existing segment in no-overlap path",
            offset
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
