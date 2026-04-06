use crate::reassembly::flow::FlowDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertResult {
    Inserted,
    Duplicate,
    PartialOverlap,
    ConflictingOverlap,
    Truncated,
    DepthExceeded,
}

/// Compute the ISN-relative offset for a sequence number.
fn seq_offset(seq: u32, isn: u32) -> u64 {
    seq.wrapping_sub(isn) as u64
}

/// Insert a segment into the flow direction's out-of-order buffer.
/// Applies first-wins overlap policy and tracks anomaly counters.
pub fn insert_segment(
    dir: &mut FlowDirection,
    seq: u32,
    data: &[u8],
    max_depth: usize,
) -> InsertResult {
    if data.is_empty() {
        return InsertResult::Inserted;
    }

    let isn = match dir.isn {
        Some(isn) => isn,
        None => return InsertResult::Inserted,
    };

    // Track small segments
    if data.len() < 8 {
        dir.small_segment_count += 1;
    } else {
        dir.small_segment_count = 0;
    }

    // Check depth limit
    let remaining_depth = max_depth.saturating_sub(dir.reassembled_bytes);
    if remaining_depth == 0 {
        if !dir.depth_exceeded {
            dir.depth_exceeded = true;
        }
        return InsertResult::DepthExceeded;
    }

    let offset = seq_offset(seq, isn);
    let mut segment_data = data.to_vec();

    // Truncate if exceeding depth
    let buffered: usize = dir.segments.values().map(|v| v.len()).sum();
    let total_after = dir.reassembled_bytes + buffered + segment_data.len();
    let truncated = if total_after > max_depth {
        let allowed = max_depth.saturating_sub(dir.reassembled_bytes + buffered);
        if allowed == 0 {
            dir.depth_exceeded = true;
            return InsertResult::DepthExceeded;
        }
        segment_data.truncate(allowed);
        dir.depth_exceeded = true;
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

    for (&existing_offset, existing_data) in dir.segments.iter() {
        let existing_end = existing_offset + existing_data.len() as u64;

        if new_start < existing_end && new_end > existing_offset {
            has_overlap = true;

            let overlap_start = new_start.max(existing_offset);
            let overlap_end = new_end.min(existing_end);

            for pos in overlap_start..overlap_end {
                let new_idx = (pos - new_start) as usize;
                let existing_idx = (pos - existing_offset) as usize;
                if new_idx < segment_data.len()
                    && existing_idx < existing_data.len()
                    && segment_data[new_idx] != existing_data[existing_idx]
                {
                    has_conflict = true;
                    break;
                }
            }

            trimmed_ranges.push((existing_offset, existing_end));
        }
    }

    if has_overlap {
        dir.overlap_count += 1;

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

        for (gap_start, gap_end) in gaps {
            let start_idx = (gap_start - new_start) as usize;
            let end_idx = (gap_end - new_start) as usize;
            if start_idx < segment_data.len() && end_idx <= segment_data.len() {
                let gap_data = segment_data[start_idx..end_idx].to_vec();
                if !gap_data.is_empty() {
                    dir.segments.insert(gap_start, gap_data);
                }
            }
        }

        // Only report ConflictingOverlap when fully covered (no gap was inserted)
        return if !had_gap && has_conflict {
            InsertResult::ConflictingOverlap
        } else if truncated {
            InsertResult::Truncated
        } else {
            InsertResult::PartialOverlap
        };
    }

    // No overlap — insert normally
    dir.segments.insert(offset, segment_data);

    if truncated {
        InsertResult::Truncated
    } else {
        InsertResult::Inserted
    }
}

/// Flush contiguous segments starting from base_offset.
/// Returns Vec of (offset, data) pairs that were flushed.
pub fn flush_contiguous(dir: &mut FlowDirection) -> Vec<(u64, Vec<u8>)> {
    let mut flushed = Vec::new();

    while let Some(data) = dir.segments.remove(&dir.base_offset) {
        let offset = dir.base_offset;
        dir.base_offset += data.len() as u64;
        dir.reassembled_bytes += data.len();
        flushed.push((offset, data));
    }

    flushed
}
