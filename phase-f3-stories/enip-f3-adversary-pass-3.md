# Adversarial Story Review — feature-enip-v0.11.0 (SS-17 stories), Pass 3

VERDICT: FAIL — 0 CRITICAL, 2 HIGH, 3 MEDIUM, 2 LOW. Novelty: MEDIUM. Pass-1 + Pass-2 fixes HELD (single-arg is_valid_enip_frame, EnipCommandClass, general_status byte-2, no raw &0x80, T0846 one-shot, T0814 windowed, parse_errors). Severity decay: P1 4C/6H → P2 1C/3H → P3 0C/2H.

- F3-301 (HIGH): dependency-graph edge text references nonexistent types EnipCommand/EnipFrame (→ EnipHeader/EnipCommandClass). REMEDIATED.
- F3-302 (HIGH): EnipAnalyzer.write_count/error_count consumed by summarize() (BC-2.17.021) but no story increments them (BC-2.17.012 Post2 write_count++, BC-2.17.008 Inv2 error_count++). REMEDIATED (STORY-135 write_count++, STORY-134 error_count++).
- F3-303 (MED): STORY-135 write_window_start:Option<u64> → write_window_start_ts:u32 (BC-2.17.012). REMEDIATED.
- F3-304 (MED): STORY-132 CipHeader 4 fields → BC-2.17.006 2 fields (general_status at call-site, request_path_size local). REMEDIATED.
- F3-305 (MED): STORY-137 file-structure "frame-walk in process_pdu" → on_data (BC-2.17.016). REMEDIATED.
- F3-306 (LOW): STORY-132 CpfItem length field removed (BC-2.17.005 2 fields). REMEDIATED.
- F3-307 (LOW, process-gap): STORY-133 EMITTED/SEEDED baselines must be re-verified vs src/mitre.rs HEAD at F4 (post-STORY-129). NOTE ADDED.

Verified-clean axes: BC completeness (all 26 once), VP obligations, holdout coverage (HS-110/111/114/117/119), scope, type/signature consistency, wave/dependency acyclic.
