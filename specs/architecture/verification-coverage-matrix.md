---
artifact: architecture-section
section: verification-coverage-matrix
traces_to: ARCH-INDEX.md
version: "1.37"
status: verified
producer: spec-steward
timestamp: 2026-05-20T00:00:00Z
modified:
  - date: 2026-06-02
    actor: spec-steward
    reason: "Phase-6 gate close: status draft→verified (propagated from VP-INDEX, all 20 VPs locked). Counts unchanged at 20."
  - date: 2026-06-08
    actor: state-manager
    reason: "Feature Mode F2 (issue #100): VP-021 added (draft/unverified; integration+proptest; source BCs BC-2.09.007 + BC-2.04.055). Total 20→21. proptest 6→7."
  - date: 2026-06-09
    actor: spec-steward
    reason: "F6 lock propagation (FINDING-001): VP-021 Phase F4→test-sufficient, Status draft→verified; coverage note updated to reflect lock @256a490. Internal counts verified consistent."
  - date: 2026-06-09
    actor: spec-steward
    reason: "F7 consistency fix — VP-021 reclassified to proptest column to match VP-INDEX counting convention (proptest 7 / integration-unit 5); prior v1.2 placement contradicted VP-INDEX invariant (verification-coverage-matrix.md Totals must equal Kani 8 / proptest 7 / fuzz 1 / integration-unit 5 = 21)."
  - date: 2026-06-09
    actor: architect
    reason: "F2 delta (issue #7 Modbus TCP): VP-022 added (draft; Kani; P1; analyzer/modbus.rs). New module row added. Kani 8→9, Total 21→22."
  - date: 2026-06-09
    actor: spec-steward
    reason: "F7 consistency fix F1 — VP-022 locked/verified at F6 (Kani 4/4 SUCCESSFUL @ develop 68a3306); propagate lock: Status draft→verified. All 22 VPs now verified; draft count 0."
  - date: 2026-06-10
    actor: architect
    reason: "Issue #222 (MITRE ATT&CK-ICS v19.1 remap): no row-level changes — VP-007 row module/tool/phase/status unchanged. VP counts unchanged at 22 (Kani 9, proptest 7, fuzz 1, integration-unit 5). Coverage note updated to reference T1692.001/T1692.002 replacing revoked T0855/T0856."
  - date: 2026-06-10
    actor: architect
    reason: "F2 delta (issue #8 DNP3 TCP): VP-023 added (draft; Kani; P1; analyzer/dnp3.rs). New module row added. Kani 9→10, Total 22→23."
  - date: 2026-06-12
    actor: architect
    reason: "F2 delta ARP security analyzer (SS-16): VP-024 added (draft; Kani; P1; analyzer/arp.rs). New module row added. Kani 10→11, Total 23→24."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-18 A-02: VP-023 lock-evidence coverage note added (verified_at_commit e685664, 2026-06-12 per vp-023-dnp3-parse-safety.md frontmatter). VP-021 and VP-022 notes already present; VP-023 was the only missing parallel entry. Version bump 1.3→1.4."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-23 Slice-A fixes: A-01 — VP-024 coverage note story corrected STORY-112/F6→STORY-113/F6 (Sub-A Kani only lands STORY-112; Sub-B/C/D land STORY-113; full VP-024 lock earliest at STORY-113/F6 per arp-architecture-delta §6). A-02 — VP-024 arp.rs row annotated: Sub-A harnesses authored in decoder.rs #[cfg(kani)] block while umbrella VP anchors arp.rs. Version bump 1.5→1.6."
  - date: 2026-06-16
    actor: architect
    reason: "F7 consistency F1 — VP-024 locked/verified at F6 (all 5 Kani harnesses Sub-A ×3 + Sub-B + Sub-D VERIFICATION:- SUCCESSFUL @ develop 6e9f2cc, 2026-06-16); propagate lock: Status draft→verified in VP-to-Module table; coverage note replaced with verified-lock evidence matching VP-022/VP-023 entry style. Version bump 1.6→1.7."
  - date: 2026-06-16
    actor: architect
    reason: "E-17 F2 governance note — VP-024 row and coverage note unchanged (no new VP, no count change, no tool/phase/module reassignment). E-17 confirmed the QinQ/MACsec lax-path offset formula is outside VP-024 proof scope; existing cargo-fuzz VP-008 coverage (16.2M/0) + 10 new behavioral tests across 2 files (bc_2_16_qinq_macsec_offset_tests.rs: 4 tests incl. MACsec observe-only probe; bc_2_16_e17_macsec_offset_tests.rs: 6 tests incl. offset==22/30 assertions) are sufficient. These behavioral tests are not counted in VP totals (which track formal proof harnesses only). E-17 note added to Coverage Notes. Version bump 1.7→1.8."
  - date: 2026-06-17
    actor: product-owner
    reason: "Issue #259 F2 integrate (v0.8.0 collapse feature): 5 new BCs BC-2.11.025–029 added (test-sufficient per F1 analysis — no new formal VP). reporter/terminal.rs gains ~5 collapse UNIT TESTS (test-sufficient, not new formal VPs); VP-row total unchanged at 2 (VP-012 + VP-016); total VPs unchanged at 24 (Kani 11 / proptest 7 / fuzz 1 / integration-unit 5 = 24). Coverage note added. Version bump 1.8→1.9."
  - date: 2026-06-17
    actor: product-owner
    reason: "F2 adversarial pass-4 (F-F2-A02): fix stale 'collapse path calls same render_finding_prefix code path' claim in Issue #259 coverage note — corrected to reflect that terminal safety = escape_for_terminal FUNCTION invariant (VP-012); collapse path calls escape_for_terminal directly on each sampled evidence line, does NOT delegate to render_finding_prefix's evidence loop. Citations updated: BC-2.11.010 v1.7 / BC-2.11.027 v1.3 / ADR-0003. Version bump 1.9→1.10."
  - date: 2026-06-17
    actor: product-owner
    reason: "F2 adversarial pass-5 (F2): fix prose nit in Issue #259 v1.8→v1.9 reason field — 'reporter/terminal.rs row unit count grows 1→6' misstated the formal VP row; corrected to: '~5 collapse UNIT TESTS (test-sufficient, not new formal VPs); VP-row total unchanged at 2 (VP-012 + VP-016); total VPs unchanged at 24.' Version bump 1.10→1.11."
  - date: 2026-06-17
    actor: product-owner
    reason: "F2 adversarial passes 12-14 (F-C01): sync BC-2.11.010 citation in Issue #259 coverage note from v1.7 → v1.8 (live BC is v1.8; prior stamp was stale). Version bump 1.11→1.12."
  - date: 2026-06-19
    actor: architect
    reason: "F2 pcapng remediation (ADR-009 rev 4): VP-025 through VP-030 added (SS-01 pcapng, reader.rs). New module row reader.rs added. Kani 11→14 (VP-025, VP-026, VP-027); proptest 7→9 (VP-029, VP-030); cargo-fuzz 1→2 (VP-028). Total 24→30. Totals row updated. Version bump 1.12→1.13."
  - date: 2026-06-19
    actor: architect
    reason: "Pass-2 adversarial remediation (ADR-009 rev 5, I-1/I-2): VP-025/VP-026/VP-027 Module cell in VP-to-Module table re-anchored from 'reader.rs' to 'reader.rs (pcapng_pure_core fns) [b]'. reader.rs Per-Module row annotated [b]. Footnote [b] and coverage note added: Kani targets are pure-core sub-functions within reader.rs (not from_pcap_reader); VP-025 Kani unwind bound must be resolved before STORY-125 F3 decomposition. No VP counts, tool counts, or Totals row values changed. Version bump 1.13→1.14."
  - date: 2026-06-19
    actor: architect
    reason: "Pass-3 adversarial remediation (ADR-009 rev 6 / Decision 18 / M-2): VP-031 added to VP-to-Module table (proptest; P1; draft; reader.rs (pcapng_pure_core fns) [b]; BC-2.01.013). reader.rs Per-Module row proptest count 2→3; Total VPs 6→7. Grand Totals row proptest 9→10, overall 30→31. Coverage note added for VP-031. Version bump 1.14→1.15."
  - date: 2026-06-19
    actor: architect
    reason: "Pass-4 adversarial remediation (ADR-009 rev 7 / H-3): VP-030 Property cell in VP-to-Module table restated — domain narrowed to WHITELISTED DataLink values only; non-whitelisted values short-circuit to E-INP-001 before conflict check (out of VP-030 scope); comparison unit = DataLink not raw u16. No row additions, no count changes. Coverage note updated. Version bump 1.15→1.16."
  - date: 2026-06-20
    actor: architect
    reason: "Pass-5 adversarial remediation (ADR-009 rev 8): VP property updates only — no row additions, no count changes (Totals row unchanged: Kani 14 / proptest 10 / fuzz 2 / integration-unit 5 = 31). VP-025 Property cell: ts_sec saturation (.min(u32::MAX)) and large-ts_high Kani vector added (M-3). VP-027 Property cell: padding-overrun and bound-by-body → Err(E-INP-008) NOT E-INP-010 added explicitly (C-1). VP-031 Property cell: snaplen DROPPED; formula now min(original_len, body.len() as u32) (Decision 9 rev 8 / H-3 + M-2). Coverage note updated. Version bump 1.16→1.17."
  - date: 2026-06-20
    actor: architect
    reason: "Pass-6 adversarial remediation (ADR-009 rev 9): VP property updates only — no row additions, no count changes (Totals row unchanged: Kani 14 / proptest 10 / fuzz 2 / integration-unit 5 = 31). VP-027 Property cell: interface_id discriminant split — empty table → Err(E-INP-009); OOB non-empty table → Err(E-INP-010); slash notation removed; Kani harness must model table-size as symbolic boolean (Decision 22 / F-H4). VP-031 Property cell: formula CORRECTED from min(original_len, body.len() as u32) to min(original_len, body.len() as u32 - 4) = min(original_len, spb_data_available); canonical symbol spb_data_available = body.len()-4 = block_total_length-16; rev 8 formula was wrong by 4 bytes (included the original_len header in the data bound) (Decision 22 / F-H2 / F-H3). Coverage note updated. Version bump 1.17→1.18."
  - date: 2026-06-22
    actor: spec-steward
    reason: "F7 consistency fix FINDING-F7-001 — VP-025..VP-031 Status reconciled draft→verified to match VP-INDEX v2.10. All 7 pcapng BCs were locked/verified at the F6 lock gate (develop 1ca30a3, PRs #293+#294, 2026-06-21): VP-025 (Kani 4 harnesses, 59 checks each), VP-026 (Kani 272 checks), VP-027 (Kani 687 checks, status active→verified at lock), VP-028 (cargo-fuzz 2,340,242 execs/0 crashes), VP-029 (proptest counter exactness + DSB-no-log + termination), VP-030 (proptest 3 cases WHITELISTED domain), VP-031 (proptest body.len()-4 formula). Coverage note updated to replace stale draft-pending prose with verified-lock evidence block mirroring VP-021..VP-024 style. Aggregate counts remain unchanged: total 31, verified 31, draft 0. Version bump 1.18→1.19."
  - date: 2026-06-24
    actor: architect
    reason: "Feature Mode F2 (feature-enip-v0.11.0, issue #316): VP-032 added to VP-to-Module table (Kani; P1; draft; src/analyzer/enip.rs; BC-2.17.001/002/003/004/007). New module row analyzer/enip.rs added to Per-Module Coverage Totals. Kani column Totals 14→15. Overall Totals 31→32. Coverage note added for VP-032. Version bump 1.19→1.20."
  - date: 2026-06-27
    actor: spec-steward
    reason: "RULING-EDGECASE-001 (EC-X1/EC-X2 index propagation): VP-033 and VP-034 added to VP-to-Module table (proptest; P1; draft; analyzer/enip.rs). analyzer/enip.rs Per-Module row proptest count 0→2; Total VPs 1→3. Totals row proptest 10→12, overall 32→34. Coverage note added for VP-033/VP-034. Version bump 1.20→1.21."
  - date: 2026-06-27
    actor: spec-steward
    reason: "RULING-DNP3-SIBLING-001 (DNP3 carry-split + saturating_sub governance): VP-035 and VP-036 added to VP-to-Module table (proptest; P1; draft; analyzer/dnp3.rs). analyzer/dnp3.rs Per-Module row proptest count 0→2; Total VPs 1→3. Totals row proptest 12→14, overall 34→36. Coverage note added for VP-035/VP-036. Version bump 1.21→1.22."
  - date: 2026-06-28
    actor: spec-steward
    reason: "RULING-MODBUS-SIBLING-001 (Modbus carry-direction splice + clock-backwards window reset): VP-037 and VP-038 added to VP-to-Module table (proptest; P1; draft; analyzer/modbus.rs). analyzer/modbus.rs Per-Module row proptest count 0→2; Total VPs 1→3. Totals row proptest 14→16, overall 36→38. Coverage note added for VP-037/VP-038. Version bump 1.22→1.23."
  - date: 2026-06-29
    actor: architect
    reason: "fix-tls-clienthello-frag F2 spec evolution: VP-039 added to VP-to-Module table (proptest + 2 unit tests; P1; draft; analyzer/tls.rs). analyzer/tls.rs Per-Module row proptest count 1→2, Total VPs 2→3. Totals row proptest 16→17, overall 38→39. Coverage note added for VP-039. Version bump 1.23→1.24."
  - date: 2026-06-29
    actor: architect
    reason: "Pass-1 adversarial reconciliation (fix-tls-clienthello-frag F2): VP-039 Property cell updated — Sub-C renamed clear-and-recover + 3 unit tests (overflow, recovery, body_len-spoof), Sub-D adds findings_count pre/post assertion (BC-2.07.040 PC3), Sub-E split range = function of actual message length, Sub-F (proptest_vp039_carry_bounded_invariant) added (BC-2.07.039 Inv-1 bounded-carry regression guard). Per-Module row and Totals row UNCHANGED (VP counts are per-VP; no new VPs added). Version bump 1.24→1.25."
  - date: 2026-06-29
    actor: architect
    reason: "Pass-2 adversarial reconciliation (F-F2-001/F-F2-010/F-F2-011/F-F2-012): VP-039 Property cell updated — seam contract added (aggregate reads via TlsAnalyzer accessors; handshake_reassembly_overflows is TlsAnalyzer-level aggregate NOT TlsFlowState); Sub-B handshakes_seen==1 via analyzer.handshake_count() DIRECTLY; canonical-frame test (test_BC_2_07_038_canonical_frame_rfc8446_s4) and SNI-boundary deterministic test (test_vp039_sni_boundary_deterministic) added (6 total unit tests). Per-Module row and Totals row UNCHANGED (VP counts are per-VP; no new VPs). Version bump 1.25→1.26."
  - date: 2026-06-29
    actor: architect
    reason: "Pass-3 adversarial reconciliation (F-P3-002/F-P3-003/F-P3-004/F-P3-005/F-P3-006/F-P3-LOW): VP-039 Property cell updated — (F-P3-002) body_len-spoof corrected to 65537; (F-P3-003) canonical-frame test Frame B anti-shared-assumption discriminator added; (F-P3-004) test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key NEW (BC-2.07.039 PC-7); Sub-C unit test count 3→4 (7 total unit tests); (F-P3-005) overflow + body_len-spoof tests: findings_count pre/post snapshot assertions added (was comment-only); (F-P3-006) SNI-boundary: runtime scan replaces blind n/2; (F-P3-LOW) Sub-D: parse_errors post==pre replaces pre==0. Per-Module row and Totals row UNCHANGED (VP counts are per-VP; no new VPs). Version bump 1.26→1.27."
  - date: 2026-06-29
    actor: architect
    reason: "Pass-4 adversarial reconciliation (F-A1/F-A2/F-A3): VP-039 coverage note corrected — (F-A1) Frame B BE body_len arithmetic corrected 65,792→66,816 (bytes [0x01,0x05,0x00]: (0x01<<16)|(0x05<<8)|0x00 = 65536+1280+0 = 66816; prior value was wrong); (F-A2) VP-039 harness count corrected 6 unit tests/10 total → 7 unit tests/11 total, enumerating all 7: carry_overflow_clear_and_recover, carry_overflow_recovery, body_len_spoof, summarize_exposes_handshake_reassembly_overflows_key, truncated_carry_no_error, canonical_frame_rfc8446_s4, sni_boundary_deterministic; (F-A3) SNI-boundary description corrected from 'splits at n/2 (SNI-extension region)' to runtime scan for [0x00,0x00] type marker at sni_ext_start+1 (provably inside SNI extension type field) per F-P3-006 fix. VP total (39), tool counts (proptest 17, Kani 15, fuzz 2, integration/unit 5), and P1 count (25) UNCHANGED. Version bump 1.27→1.28."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-5 stale-prose sync: VP-039 coverage note P1 list corrected — 'Sub-C, 3 unit tests' → 'Sub-C, 4 unit tests' to match the authoritative coverage-note block (which already correctly enumerates all 7 unit tests at 4 proptest + 7 unit tests = 11 total). No table-row or Totals-row changes (VP counts unchanged). Version bump 1.28→1.29."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-6 (F-FRESH-001/F-CRITICAL-2/F-F2P-IMP-001/F-FRESH-002): VP-039 table row and coverage note updated — (F-CRITICAL-2) Sub-C overflow fixture corrected: valid-header body_len=65,500 ([0x01,0x00,0xFF,0xDC]) + accumulation records triggers Decision-5 buffer-fill guard (NOT Decision-4 body_len-spoof path); counter==overflows_before+1 TRUE (prior 0xCC fill hit Decision-4 4×; counter==overflows_before+4; assertion was FALSE); (F-FRESH-001) test_BC_2_07_038_malformed_assembled_body NEW — assembled length-complete body fails inner parse → parse_errors+1, exact-consume, no finding, no panic (ADR-011 Decision-4 error semantics; parse_tls_message_handshake path; PO must add BC postcondition/EC); (F-F2P-IMP-001) Sub-F proptest generator restructured: valid-header prefix (body_len drawn from 0..=65_536) ensures genuine carry accumulation rather than near-vacuous Decision-4 bypass; (F-FRESH-002) Frame C added to canonical-frame test: [0x01,0x00,0x01,0x00] body_len=256 mid-range dispatch-lane pin; unit test count 7→9; proptest 17, total 39 UNCHANGED. Version bump 1.29→1.30."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-7 (F-FRESH2-001/F-FRESH2-003): VP-039 table row and coverage note updated — (F-FRESH2-003) 2 orphaned unit tests authored: test_BC_2_07_040_empty_carry_flow_close (Sub-D-ext: on_flow_close with EMPTY carry, BC-2.07.040 degenerate case) and test_BC_2_07_042_exact_consume_no_double_dispatch (Sub-B-ext: deterministic coalesced ClientHello+Certificate, asserts handshake_count()==1, BC-2.07.042); (F-FRESH2-001) harness count corrected 9 unit tests → 10 unit tests = 14 total harnesses; all 10 unit test names enumerated; (F-FRESH2-004) Sub-F Decision-5 path coverage note: Decision-5 exercised DETERMINISTICALLY by test_vp039_carry_overflow_clear_and_recover, not probabilistically by Sub-F. VP/tool counts unchanged (total 39, proptest 17, p1 25). Version bump 1.30→1.31."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-10 (F-ADVF2-001 CRITICAL): VP-039 coverage note updated — Frame A description corrected from stale 'carry drains to 0 (benign)' to PC-9 malformed-body path: body_len=5 is length-complete but too short for a valid ClientHello; parse_tls_message_handshake Err → parse_errors==errors_before+1, carry exact-consumed (carry_len=0), client_hello_seen==false, no panic; cites BC-2.07.038 AC-CANONICAL-FRAME v2.5. Harness count (14) and VP/tool totals (39 total, proptest 17, Kani 15, fuzz 2, integration/unit 5) UNCHANGED. Version bump 1.31→1.32."
  - date: 2026-06-29
    actor: architect
    reason: "Fix-burst-11 (F-COMP-001/F-COMP-002/F-COMP-003/F-F2IMPL-001): VP-039 table row and coverage note updated — 3 new unit test skeletons added (unit count 10→13; total harnesses 14→17): (F-COMP-002) test_BC_2_07_041_cross_flow_isolation (two distinct FlowKeys: Flow A complete single-record SNI=a.example, Flow B same-shaped fragmented SNI=b.example; asserts sni_counts.len()==2, no bleed, both client_hello_seen, parse_errors==0; BC-2.07.041 PC-1/PC-4/Inv-1); (F-COMP-001) test_vp039_n_record_reassembly (ONE ClientHello across >=3 records; two scenarios; BC-2.07.038 PC-1/PC-2/PC-6+EC-003); (F-COMP-003) test_vp039_large_valid_hello_reassembly (~40 KB valid ClientHello fragmented across <=MAX_RECORD_PAYLOAD records; positive verification of 18,432→65,536 cap raise; BC-2.07.038 Inv-5); (F-F2IMPL-001) summarize_key description tightened: 'asserts key present' → 'asserts detail[handshake_reassembly_overflows].as_u64()==1 (value-equality, not mere key presence)' to match BC-2.07.039 PC-7 and actual test body. VP/tool totals UNCHANGED (39 total, proptest 17, Kani 15, fuzz 2, integration/unit 5). ARCH-INDEX ADR-011 row flagged: harness count should update '4 proptest + 10 unit tests = 14' → '4 proptest + 13 unit tests = 17'. Version bump 1.32→1.33."
  - date: 2026-06-29
    actor: architect
    reason: "F2 scope-addition (fix-tls-clienthello-frag) — VP-040 added to VP-to-Module table (unit; P1; draft; analyzer/tls.rs; BC-2.07.043 + BC-2.07.005). analyzer/tls.rs Per-Module row integration/unit count 0→1; Total VPs 3→4. Totals row integration/unit 5→6, overall 39→40. Coverage note added for VP-040. Version bump 1.33→1.34."
  - date: 2026-06-29
    actor: architect
    reason: "Adversary fix burst (F-2/F-3 PC-citations, C-1 borrow rationale, C-2 full-drop seam, I-2 value-equality): VP-040 row updated — PC citations corrected in row property cell: Sub-A→PC-1, Sub-A-full-drop added (test_BC_2_07_043_buffer_saturation_full_drop; fill_buf_for_testing seam; remaining==0 full-drop path; PO to deliver seam with BC-2.07.043 v1.1), Sub-B→PC-1 negative, Sub-C→PC-5 (not reset at flow close), Sub-D→PC-4 value-equality (NOT PC-2), Sub-E→PC-3 (both directions); increment condition corrected to data.len()>remaining; increment site after &mut state block (C-1 borrow note); test count updated 5→6 unit tests. Totals row UNCHANGED (VP total 40, integration/unit 6). Version bump 1.34→1.35."
  - date: 2026-06-29
    actor: architect
    reason: "DF-AC-TEST-NAME-SYNC reconciliation (fix-tls-clienthello-frag) — VP-040 Sub-D test renamed test_BC_2_07_043_summarize_exposes_buffer_saturation_drops_key → test_BC_2_07_043_summarize_value_equals_drop_count in VP-to-Module table row and coverage note. No VP counts changed (VP total 40, integration/unit 6). Version bump 1.35→1.36."
  - date: 2026-06-29
    actor: architect
    reason: "F2 adversary implementability fix burst (on_data sig / CloseReason / seam / ADR-011 C-3 propagation): VP-040 Coverage-Note PROSE block corrected — (1) harness count '5 deterministic unit tests' → '6 deterministic unit tests'; (2) Sub-A description corrected from stale 'pre-fill via on_data calls (~45 × 1,460-byte calls)' to correct '65,537-byte single slice to empty buffer (no seam required)'; (3) Sub-A-full-drop entry added explicitly with fill_buf_for_testing seam (FINAL); (4) increment condition stated as data.len()>remaining; (5) on_flow_close(CloseReason::Fin) noted in Sub-C; (6) all 6 canonical test names enumerated. The VP-040 table row (already correct from v1.35) and Totals row (VP total 40, integration/unit 6) UNCHANGED. Version bump 1.36→1.37."
---

# Verification Coverage Matrix

## VP-to-Module Mapping

| VP-ID | Property (short) | Module | Tool | Phase | Status |
|-------|-----------------|--------|------|-------|--------|
| VP-001 | FlowKey canonical ordering | reassembly/flow.rs | Kani | P0 | verified |
| VP-002 | First-wins overlap policy | reassembly/segment.rs | Kani | P0 | verified |
| VP-003 | MAX_FINDINGS cap + finalize bypass | reassembly/mod.rs | Kani | P0 | verified |
| VP-004 | Content-first dispatch precedence | dispatcher.rs | Kani | P0 | verified |
| VP-005 | SNI 4-way ordered match (INV-5 boundary) | analyzer/tls.rs | Kani | P0 | verified |
| VP-006 | HTTP poison monotonicity | analyzer/http.rs | proptest | P1 | verified |
| VP-007 | MITRE technique ID format completeness | mitre.rs | Kani | P0 | verified |
| VP-008 | decode_packet no-panic | decoder.rs | cargo-fuzz | P0 | verified |
| VP-009 | FlowState machine validity | reassembly/flow.rs | Kani | P0 | verified |
| VP-010 | buffered_bytes invariant | reassembly/segment.rs | proptest | P1 | verified |
| VP-011 | flush_contiguous monotonicity | reassembly/segment.rs | proptest | P1 | verified |
| VP-012 | escape_for_terminal correctness | reporter/terminal.rs | proptest | P1 | verified |
| VP-013 | JA3 GREASE filter | analyzer/tls.rs | proptest | P1 | verified |
| VP-014 | HttpAnalyzer cross-flow isolation | analyzer/http.rs | proptest | P1 | verified |
| VP-015 | TCP sequence wraparound | reassembly/segment.rs | Kani | P1 | verified |
| VP-016 | MITRE tactic grouping order | reporter/terminal.rs | integration | test-sufficient | verified |
| VP-017 | JsonReporter key determinism | reporter/json.rs | integration | test-sufficient | verified |
| VP-018 | CLI mutual exclusion (reassemble flags) | cli.rs | integration | test-sufficient | verified |
| VP-019 | DNS statistics-only (no findings) | analyzer/dns.rs | unit | test-sufficient | verified |
| VP-020 | CSV injection neutralization | reporter/csv.rs | unit | test-sufficient | verified |
| VP-021 | Timestamp provenance threading | reassembly/mod.rs | integration+proptest | test-sufficient | verified |
| VP-022 | Modbus MBAP parse safety + FC boundary classification | analyzer/modbus.rs | Kani | P1 | verified |
| VP-023 | DNP3 DL frame parse safety + FC classification + frame_len arithmetic | analyzer/dnp3.rs | Kani | P1 | verified |
| VP-024 | ARP frame parse safety (extract_arp_frame) + GARP totality + binding-table cap | analyzer/arp.rs | Kani | P1 | verified |
| VP-025 | pcapng timestamp conversion totality: no panic, ts_usecs in [0,999999], ts_sec saturated (.min(u32::MAX)), saturating arithmetic for all (u32,u32,u8); large-ts_high Kani vector required (rev 8 / M-3) | reader.rs (pcapng_pure_core fns) [b] | Kani | P1 | verified |
| VP-026 | pcapng SHB parse safety: no panic, byte-order BOM detection correct (LE/BE), Err for <28 bytes | reader.rs (pcapng_pure_core fns) [b] | Kani | P1 | verified |
| VP-027 | pcapng EPB parse safety: no panic; interface_id discriminant — empty table → Err(E-INP-009); OOB non-empty table → Err(E-INP-010); slash notation removed (rev 9 / Decision 22 / F-H4); guard-before-allocate; padding-overrun (20+captured_len+pad_len>body.len()) → Err(E-INP-008); bound-by-body (captured_len>body.len()-20) → Err(E-INP-008); NOT E-INP-010 (rev 8 / C-1) | reader.rs (pcapng_pure_core fns) [b] | Kani | P1 | verified |
| VP-028 | pcapng reader no-panic (cargo-fuzz fuzz_pcapng_reader, F6 hardening) | reader.rs | cargo-fuzz | P1 | verified |
| VP-029 | pcapng block-walk skip: always terminates, Err-breaks loop, cursor advances >= 12 bytes per Ok | reader.rs | proptest | P1 | verified |
| VP-030 | pcapng multi-IDB linktype agreement totality (RESTATED rev 7 / H-3): WHITELISTED DataLink domain only; all-equal → Ok, first-differing whitelisted DataLink → Err(E-INP-011); non-whitelisted → E-INP-001 (out of scope); comparison unit DataLink not raw u16 | reader.rs | proptest | P1 | verified |
| VP-031 | pcapng SPB captured-len arithmetic correctness (spb_data_available formula): captured_len == min(original_len, body.len() as u32 - 4) = min(original_len, spb_data_available); slice length == captured_len; no OOB for all (u32, &[u8] with len>=4) inputs; formula CORRECTED from rev 8 (body.len() → body.len()-4 per Decision 22 / F-H2 / F-H3); snaplen DROPPED (rev 8 / Decision 9) | reader.rs (pcapng_pure_core fns) [b] | proptest | P1 | verified |
| VP-032 | EtherNet/IP + CIP frame parse safety and command/service classification: (Sub-A) parse_enip_header no-panic, None<24b, Some with correct LE fields; (Sub-B) classify_enip_command total over all 65,536 u16 inputs, Unknown reachable; (Sub-C) is_valid_enip_frame biconditional iff command in known-set; (Sub-D) classify_cip_service total over all 256 u8 inputs, response-bit mask (0x80→Response) proven; 4 sub-properties (Sub-A..Sub-D); 5 Kani harnesses (Sub-D = totality + request-partition) | analyzer/enip.rs | Kani | P1 | draft |
| VP-033 | EtherNet/IP carry-buffer direction isolation (EC-X1): Harness-A direction-isolation pdu_count — interleaved c2s/s2c deliveries produce pdu_count==2 with carry buffers never mixed; Harness-B independent-run equivalence — interleaved pdu_count equals sum of independent same-direction runs; traces BC-2.17.016 v2.0 Inv-7 | analyzer/enip.rs | proptest | P1 | draft |
| VP-034 | EtherNet/IP window backwards-timestamp no-spurious-reset (EC-X2): Sub-A T0836 write-burst backwards-ts no-reset (BC-2.17.012 v1.2 EC-009); Sub-B T0888 error-rate backwards-ts no-reset (BC-2.17.008 v1.3 EC-009); Sub-C T0814 malformed backwards-ts no-reset + EC-X4 operator pin (elapsed==300 NOT > 300; BC-2.17.018 v1.1 EC-008); Sub-D genuine u32 rollover deterministic unit test; traces BC-2.17.008 v1.3 / BC-2.17.012 v1.2 / BC-2.17.018 v1.1 | analyzer/enip.rs | proptest | P1 | draft |
| VP-035 | DNP3 carry-buffer direction isolation (DRIFT-DNP3-DIRECTION-001): proptest_vp035_direction_isolation_frame_count — interleaved c2s/s2c deliveries produce correct frame_count with carry_c2s/carry_s2c never mixed; proptest_vp035_independent_run_equivalence — interleaved frame_count equals sum of independent runs; traces BC-2.15.016 v2.0 Inv-6 | analyzer/dnp3.rs | proptest | P1 | draft |
| VP-036 | DNP3 window backwards-timestamp no-spurious-reset (DRIFT-DNP3-CLOCK-001): Sub-A T1692.001 60s backwards-ts no-reset (BC-2.15.010 v1.8 EC-012); Sub-B T1691.001 10s block-timeout backwards-ts no-spurious-fire (BC-2.15.014 v2.1 EC-009); Sub-C T0827/T0814 300s correlation-window backwards-ts no-reset + DRIFT-DNP3-OP-001 operator pin (elapsed==300 NOT > 300; BC-2.15.015 v2.0 EC-010); Sub-D genuine u32 rollover deterministic unit test (all three windows); traces BC-2.15.010 v1.8 / BC-2.15.014 v2.1 / BC-2.15.015 v2.0 | analyzer/dnp3.rs | proptest | P1 | draft |
| VP-037 | Modbus carry-buffer direction isolation (DRIFT-MODBUS-DIRECTION-001): proptest_vp037_direction_isolation_fn_code_counts — interleaved c2s/s2c deliveries produce correct fn_code_counts with carry_c2s/carry_s2c never mixed; parse_errors==0; proptest_vp037_independent_run_equivalence — interleaved fn_code_counts equal those of independent same-direction runs; traces BC-2.14.002 v2.0 Inv-4 + EC-007 | analyzer/modbus.rs | proptest | P1 | draft |
| VP-038 | Modbus window backwards-timestamp no-spurious-reset (DRIFT-MODBUS-CLOCK-001): Sub-A T0831 5s backwards-ts no-reset (BC-2.14.016 v2.3 EC-010/EC-011); Sub-B T0806 burst 1s backwards-ts no-reset (BC-2.14.017 v2.7 EC-010/EC-012); Sub-C T0806 sustained >=2s minimum-duration gate — >= INTENTIONALLY PRESERVED (RULING-MODBUS-SIBLING-001 §2.3 — fires AT 2s mark; not a pin); Sub-D T0888 exception 10s backwards-ts no-reset (BC-2.14.019 v1.5 EC-009); Sub-E genuine u32 rollover deterministic unit test (all four Modbus windows); traces BC-2.14.016 v2.3 / BC-2.14.017 v2.7 / BC-2.14.019 v1.5 | analyzer/modbus.rs | proptest | P1 | draft |
| VP-040 | TLS per-direction buffer saturation observability (F-EV-001 defense-in-depth, fix-tls-clienthello-frag F2 scope-addition — adversary fix burst applied): buffer_saturation_drops TlsAnalyzer aggregate; SEAM — reads via buffer_saturation_drop_count() accessor; INCREMENT CONDITION data.len()>remaining (covers partial-drop AND full-drop); INCREMENT SITE after &mut state block closes (borrow constraint; C-1); (Sub-A test_BC_2_07_043_buffer_saturation_observable) PARTIAL-DROP: 65,537-byte slice to empty buffer, counter+1, parse_errors unchanged (PC-1, PC-6; no seam needed); (Sub-A-full-drop test_BC_2_07_043_buffer_saturation_full_drop) FULL-DROP: fill_buf_for_testing seam to remaining==0, deliver non-empty slice, counter+1 (PC-1, EC-002; seam from PO BC-2.07.043 v1.1); (Sub-B test_BC_2_07_043_no_drop_no_counter) small data fits, counter unchanged (PC-1 negative); (Sub-C test_BC_2_07_043_counter_persists_across_flows) drop then on_flow_close, counter unchanged — NOT reset (PC-5); (Sub-D test_BC_2_07_043_summarize_value_equals_drop_count) summarize() detail["buffer_saturation_drops"].as_u64()==1 — value-equality, not key presence (PC-4; NOT PC-2); (Sub-E test_BC_2_07_043_both_directions_increment_same_counter) c2s drop + s2c drop == initial+2 (PC-3: both directions); PC-6: parse_errors NOT incremented (BC-2.07.005); DISTINCT from VP-039; 6 unit tests total | analyzer/tls.rs | unit | P1 | draft |
| VP-039 | TLS handshake reassembly (fix-tls-clienthello-frag, F-P3/F-burst-6/F-burst-7 fixes): SEAM CONTRACT — aggregate reads (parse_errors, sni_counts, ja3_counts, handshakes_seen, handshake_reassembly_overflows) via TlsAnalyzer accessors ONLY; NEVER off TlsFlowState; (Sub-A) proptest_vp039_carry_reassembly_two_record — split range = function of actual hello length via prop_oneof![1..4, 4..n]; partial-header {1,2,3} reachable; SNI-region guaranteed by test_vp039_sni_boundary_deterministic; client_hello_seen==true, parse_errors==0 via analyzer.parse_error_count() (BC-2.07.038); (Sub-B) proptest_vp039_exact_consume_coalesced — two coalesced messages (second with NON-ZERO body_len), carry_len==0, handshakes_seen==1 asserted DIRECTLY via analyzer.handshake_count() (BC-2.07.042); (Sub-B-ext — F-FRESH2-003) test_BC_2_07_042_exact_consume_no_double_dispatch: deterministic coalesced ClientHello + Certificate (type=0x0B), asserts handshake_count()==1 (no double-dispatch), carry drained, parse_errors==0 (BC-2.07.042); (Sub-C, 4 unit tests — F-CRITICAL-2 fixture corrected) test_vp039_carry_overflow_clear_and_recover: valid-header body_len=65,500 ([0x01,0x00,0xFF,0xDC]) + accumulation records trigger Decision-5 buffer-fill guard once (carry.len()+payload>MAX_BUF), carry cleared to len==0, analyzer.handshake_reassembly_overflow_count()+1 [TlsAnalyzer aggregate; prior 0xCC fill hit Decision-4 body_len-spoof 4× — assertion was FALSE], parse_errors unchanged, findings_count pre==post [BC-2.07.039 PC-4; F-P3-005]; test_vp039_carry_overflow_recovery: post-overflow ClientHello dispatched normally, SNI+JA3 via analyzer.sni_counts()/ja3_counts() (BC-2.07.039 recovery assertion); test_vp039_body_len_spoof: body_len=65537>MAX_BUF [65536 would NOT trigger strict > guard; F-P3-002] triggers Decision-4 clear-and-recover, findings_count pre==post [BC-2.07.039 PC-4; F-P3-005]; test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key: triggers overflow, calls summarize(), asserts detail["handshake_reassembly_overflows"].as_u64()==1 — value-equality NOT mere key presence [BC-2.07.039 PC-7; F-P3-004; F-F2IMPL-001]; (Sub-D) test_vp039_truncated_carry_no_error — on_flow_close with partial carry: findings_count pre==post, parse_errors post==pre snapshot [NOT pre==0; F-P3-LOW] via analyzer.parse_error_count() (BC-2.07.040 PC3); (Sub-D-ext — F-FRESH2-003) test_BC_2_07_040_empty_carry_flow_close: on_flow_close with EMPTY carry (after full consume) has no observable effect beyond flow removal; parse_errors unchanged, findings unchanged, active_flows==0 (BC-2.07.040 degenerate case); (Sub-E) proptest_vp039_direction_isolation — interleaved c2s/s2c fragmented hellos == independent same-direction runs; parse_errors via analyzer.parse_error_count(); carry_c2s/carry_s2c never mixed (BC-2.07.041); (Sub-F — F-F2P-IMP-001 generator restructured; F-FRESH2-004 Decision-5 note) proptest_vp039_carry_bounded_invariant — generator draws body_len from 0..=65_536 (valid-header prefix via prop_flat_map) ensuring genuine carry accumulation; prior arbitrary-u8 generator was near-vacuous (Decision-4 fired on nearly every record); carry.len()≤MAX_BUF after every call (BC-2.07.039 Inv-1); NOTE: Decision-5 buffer-fill path exercised DETERMINISTICALLY by test_vp039_carry_overflow_clear_and_recover, not probabilistically by Sub-F; (Canonical-frame F-P3-003/F-FRESH-002) test_BC_2_07_038_canonical_frame_rfc8446_s4 — Frame A: [0x01,0x00,0x00,0x05] body_len=5; Frame B discriminator: [0x01,0x01,0x05,0x00] BE=66816>MAX_BUF→carry_len=0; LE=1281→carry_len=4; pins decode direction (DF-CANONICAL-FRAME-HOLDOUT-001); Frame C (F-FRESH-002): [0x01,0x00,0x01,0x00] body_len=256 mid-range dispatch-lane — asserts carry drains to 0, parse_errors+1 (malformed all-zeros body via ADR-011 Decision-4); pins BE decode in dispatch lane, not only at overflow boundary; (SNI-boundary F-P3-006) test_vp039_sni_boundary_deterministic — runtime scan for [0x00,0x00] SNI type marker; splits at sni_ext_start+1 (provably inside extension); asserts sni_ext_start>4 and split<n; replaces blind n/2; (Malformed-assembled-body F-FRESH-001) test_BC_2_07_038_malformed_assembled_body — assembled length-complete handshake body (body_len=6, header [0x01,0x00,0x00,0x06]) with malformed body (version OK but missing Random field) fails parse_tls_message_handshake → parse_errors+1, exact-consume 4+6=10 bytes, no finding, no panic; parity with single-record parse_errors discipline (ADR-011 Decision-4 error semantics); total 4 proptest + 13 unit tests = 17 harnesses (fix-burst-11 +3); the 13 unit tests: (1) test_vp039_carry_overflow_clear_and_recover; (2) test_vp039_carry_overflow_recovery; (3) test_vp039_body_len_spoof; (4) test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key; (5) test_vp039_truncated_carry_no_error; (6) test_BC_2_07_038_canonical_frame_rfc8446_s4; (7) test_vp039_sni_boundary_deterministic; (8) test_BC_2_07_038_malformed_assembled_body; (9) test_BC_2_07_040_empty_carry_flow_close (F-FRESH2-003: Sub-D-ext, BC-2.07.040 empty-carry degenerate); (10) test_BC_2_07_042_exact_consume_no_double_dispatch (F-FRESH2-003: Sub-B-ext, BC-2.07.042 deterministic coalesce); (11) test_BC_2_07_041_cross_flow_isolation (F-COMP-002: Sub-E-ext, two distinct FlowKeys, BC-2.07.041 PC-1/PC-4/Inv-1); (12) test_vp039_n_record_reassembly (F-COMP-001: Sub-A-ext-N, >=3-record re-entrancy, BC-2.07.038 PC-1/PC-2/PC-6+EC-003); (13) test_vp039_large_valid_hello_reassembly (F-COMP-003: Sub-C-ext-large, ~40 KB valid ClientHello, BC-2.07.038 Inv-5) | analyzer/tls.rs | proptest | P1 | draft |


## Per-Module Coverage Totals

| Module | Kani | proptest | cargo-fuzz | integration/unit | Total VPs |
|--------|------|----------|------|-----------------|-----------|
| reassembly/flow.rs | 2 (VP-001, VP-009) | 0 | 0 | 0 | 2 |
| reassembly/segment.rs | 2 (VP-002, VP-015) | 2 (VP-010, VP-011) | 0 | 0 | 4 |
| reassembly/mod.rs | 1 (VP-003) | 1 (VP-021) | 0 | 0 | 2 |
| dispatcher.rs | 1 (VP-004) | 0 | 0 | 0 | 1 |
| analyzer/tls.rs | 1 (VP-005) | 2 (VP-013, VP-039) | 0 | 1 (VP-040) | 4 |
| analyzer/http.rs | 0 | 2 (VP-006, VP-014) | 0 | 0 | 2 |
| mitre.rs | 1 (VP-007) | 0 | 0 | 0 | 1 |
| decoder.rs | 0 | 0 | 1 (VP-008) | 0 | 1 |
| reporter/terminal.rs | 0 | 1 (VP-012) | 0 | 1 (VP-016) | 2 |
| reporter/json.rs | 0 | 0 | 0 | 1 (VP-017) | 1 |
| cli.rs | 0 | 0 | 0 | 1 (VP-018) | 1 |
| analyzer/dns.rs | 0 | 0 | 0 | 1 (VP-019) | 1 |
| reporter/csv.rs | 0 | 0 | 0 | 1 (VP-020) | 1 |
| analyzer/modbus.rs | 1 (VP-022) | 2 (VP-037, VP-038) | 0 | 0 | 3 |
| analyzer/dnp3.rs | 1 (VP-023) | 2 (VP-035, VP-036) | 0 | 0 | 3 |
| analyzer/arp.rs | 1 (VP-024) [a] | 0 | 0 | 0 | 1 |
| analyzer/enip.rs | 1 (VP-032) | 2 (VP-033, VP-034) | 0 | 0 | 3 |
| reader.rs | 3 (VP-025, VP-026, VP-027) [b] | 3 (VP-029, VP-030, VP-031) [b] | 1 (VP-028) | 0 | 7 |
| **Totals** | **15** | **17** | **2** | **6** | **40** |


## Coverage Notes

- reassembly/segment.rs has 2 Kani proofs (VP-002 first-wins overlap, VP-015 sequence
  wraparound) and 2 proptest proofs (VP-010 buffered_bytes invariant, VP-011
  flush_contiguous monotonicity). Row total = 4 VPs.

- VP-001 through VP-020 statuses are `verified` as of Phase-6 gate close (2026-06-02 @ develop 0855f25).
  verification_lock=true is set on all those VP documents.
- VP-021 is `verified` — locked at F6 formal hardening gate (2026-06-09 @ develop 256a490). verification_lock=true.
  Proof evidence: tests/timestamp_threading_tests.rs (integration + proptest). 1147 tests green.
- VP-022 is `verified` — locked at F6 formal hardening gate (2026-06-09 @ develop 68a3306). verification_lock=true.
  Proof evidence: Kani 4/4 harnesses SUCCESSFUL (verify_parse_mbap_header_safety, verify_is_valid_modbus_adu_gate,
  verify_classify_fc_total, verify_classify_fc_exception_iff_high_bit). See .factory/phase-f6-hardening/kani-results.md.
- VP-023 is `verified` — locked at F6 formal hardening gate (2026-06-12 @ develop e685664). verification_lock=true.
  Proof evidence: Kani harnesses SUCCESSFUL (DNP3 DL header parse safety, FC classification totality, validity gate
  biconditional, frame_len arithmetic). See vp-023-dnp3-parse-safety.md frontmatter (verified_at_commit: e685664).
- VP-024 is `verified` — locked at F6 formal hardening gate (2026-06-16 @ develop 6e9f2cc). verification_lock=true.
  Proof evidence: all 5 Kani harnesses VERIFICATION:- SUCCESSFUL: Sub-A ×3
  (verify_extract_arp_frame_safety, verify_extract_arp_frame_eth_ipv4_correctness,
  verify_extract_arp_frame_none_on_bad_size in src/decoder.rs #[cfg(kani)]) + Sub-B
  (verify_classify_garp_total) + Sub-D (verify_binding_table_cap, array surrogate
  insert_binding_lru_array) in src/analyzer/arp.rs #[cfg(kani)]. Sub-C (proptest
  test_binding_table_last_write_wins) is test-sufficient. See vp-024-arp-parse-safety.md
  frontmatter (verified_at_commit: 6e9f2cc). See .factory/phase-f6-hardening/kani-results.md.
  [a] VP-024 umbrella is anchored to analyzer/arp.rs (Sub-B/C/D targets). Sub-A Kani harnesses
  (verify_extract_arp_frame_safety, verify_extract_arp_frame_eth_ipv4_correctness,
  verify_extract_arp_frame_none_on_bad_size) are authored in the src/decoder.rs #[cfg(kani)]
  block because extract_arp_frame lives in src/decoder.rs (per vp-024-arp-parse-safety.md §Proof
  Harness Skeleton and arp-architecture-delta §6 STORY-112). The module row anchor (arp.rs) is
  correct for the umbrella VP; the harness file split is a Sub-A implementation detail.

- E-17 (2026-06-16) QinQ/MACsec offset governance note: VP-024 row, tool assignment, phase, and status are unchanged. E-17 confirmed the stacked-link-extension offset formula (`14 + Σ ext.header_len()` in `decode_packet`'s lax-None arm) is outside VP-024's proof scope — it is an effectful etherparse lax-parse path, not a pure-core function target for Kani. Existing coverage is sufficient: cargo-fuzz VP-008 (16.2M iterations / 0 panics, covering `decode_packet` including the lax-None ARP arm) + 10 new behavioral/assertion tests across 2 files (E-17 test delta — NOT counted in the VP-unit totals above, which track formally-verified VP proof harnesses only): `tests/bc_2_16_qinq_macsec_offset_tests.rs` (4 tests: QinQ behavioral, QinQ model-pin, QinQ malformed→D11, and MACsec observe-only probe `test_BC_2_16_015_macsec_arp_lax_parse_probe` — asserts no offset value) and `tests/bc_2_16_e17_macsec_offset_tests.rs` (6 tests: `test_BC_2_16_015_macsec_no_sci_unmodified_arp_truncated_offset_22` asserts arp_offset==22, `test_BC_2_16_015_macsec_sci_present_unmodified_arp_truncated_offset_30` asserts arp_offset==30, malformed→D11 for no-SCI/SCI, Modified/opaque-unreachable security guards; branch test/arp-qinq-macsec-fixtures, extends PR #258, committed in F4). The offset==22 and offset==30 arithmetic assertions reside ONLY in `bc_2_16_e17_macsec_offset_tests.rs`; the qinq file's MACsec test is observe-only. These 10 behavioral tests are separate from the VP proof-harness count; the VP totals (Kani 11 / proptest 7 / fuzz 1 / integration-unit 5 = 24) are unchanged. No new VP warranted. BC cross-references: BC-2.16.009 v1.8 EC-009, BC-2.16.015 v1.7 EC-009. arp-architecture-delta.md bumped to v1.18 with the per-variant offset table and etherparse source citations.

- Issue #259 (v0.8.0 terminal finding collapse): 5 new BCs (BC-2.11.025–029) are
  **test-sufficient** — no new formal VP warranted per F1 delta analysis §8 rationale:
  (1) count correctness = Vec.len(), unit test sufficient; (2) no-loss invariant (JSON/CSV
  unchanged) = enforced by code structure (collapse is private to TerminalReporter) + integration
  test; (3) terminal safety (escape_for_terminal) = VP-012 unchanged; the `escape_for_terminal`
  FUNCTION invariant is unchanged — the collapse path calls `escape_for_terminal` directly on
  each sampled evidence line and does NOT delegate to `render_finding_prefix`'s evidence loop
  (BC-2.11.010 v1.8 / BC-2.11.027 v1.3 / ADR-0003). New unit tests mandated by BC-2.11.025–029 Verification
  Properties sections (test_BC_2_11_025_*, test_BC_2_11_026_*, test_BC_2_11_027_*,
  test_BC_2_11_028_*, test_BC_2_11_029_*). These are behavioral unit tests, NOT formal VP
  harnesses; they are not counted in the VP totals above. VP-012 (proptest, P1, verified) is
  the sole formal VP touching reporter/terminal.rs; its scope is unchanged.

- VP-030 RESTATED (ADR-009 rev 7 / H-3): VP-030 domain was narrowed from "any sequence of
  IDB linktype u16 values" to "WHITELISTED DataLink values only." The original domain was
  unsatisfiable: non-whitelisted u16 values trigger E-INP-001 at IDB-parse time (Decision 17
  step 2 — whitelist check) before the E-INP-011 multi-IDB agreement check (step 3) is ever
  reached. A proptest with arbitrary u16 values would saturate on E-INP-001 rejections and never
  exercise the agreement property. The restated domain (whitelisted DataLink values) is exactly
  the domain where the conflict check is reachable. Comparison unit is DataLink (not raw u16).
  Non-whitelisted values are out of VP-030 scope; they are covered by BC-2.01.016 integration tests.
  Tool/phase/status/counts unchanged.

- VP-025 through VP-031 (reader.rs) are `verified` — all 7 pcapng BCs locked at the
  F6 lock gate (2026-06-21 @ develop 1ca30a3, PRs #293 + #294). verification_lock=true
  on all seven VP documents. Lock evidence by VP:
  - VP-025: harnesses `vp025_timestamp_totality` (µs fast-path, M-3 saturation guard) +
    `vp025_timestamp_totality_base10` + `vp025_timestamp_totality_base10_saturating` +
    `vp025_timestamp_totality_base2` — all cargo kani VERIFICATION SUCCESSFUL (59 checks
    each), non-vacuity confirmed; per-divisor-constant split resolves I-2 unwind note.
  - VP-026: harness `vp026_shb_parse_safety` (#[kani::unwind(21)]), 272 checks
    VERIFICATION SUCCESSFUL, non-vacuity confirmed; twin-drift tripwire
    `tests/sec_shb_twin_equivalence_tests.rs` (6 unit tests + 2000-case proptest) guards
    pure `parse_shb_body_discriminant` against divergence from production `parse_shb_body`.
  - VP-027: harness `reader::kani_proofs::vp027_epb_parse_safety`, 687 checks
    VERIFICATION SUCCESSFUL (proof fixed F-F5P1-001 @ develop 97c66b0; re-confirmed SUCCESSFUL
    @ 1ca30a3); status active→verified at lock gate. SEC-001 twin-drift risk remains a tracked
    follow-up obligation (see VP-INDEX v2.10 note).
  - VP-028: fuzz target `fuzz/fuzz_targets/fuzz_pcapng_reader.rs`, 2,340,242 execs / 121s / 0
    crashes.
  - VP-029: proptest suite including `proptest_VP_029_skip_arm_counter_exactness_and_dsb_no_log`
    (counter exactness + DSB-no-log + termination).
  - VP-030: proptest `proptest_VP_030_all_equal_whitelisted_idbs_ok` +
    `proptest_VP_030_first_differing_whitelisted_idb_errs_e_inp_011` +
    `proptest_VP_030_comparison_unit_is_datalink` (WHITELISTED domain, rev 7 / H-3 restatement).
  - VP-031: existing proptest confirmed correct against the `body.len()-4` formula (Decision 22 /
    F-H2 / F-H3 correction). BC-2.01.013 carries DUAL VP coverage: VP-031 (arithmetic
    correctness, proptest) + VP-028 (no-panic, cargo-fuzz).
  Aggregate counts are unchanged: total 31 / verified 31 / draft 0. Grand Totals row
  (Kani 14 / proptest 10 / fuzz 2 / integration-unit 5 = 31) is unchanged.

  [b] **VP-025 / VP-026 / VP-027 module anchor: reader.rs (pcapng_pure_core fns)**
  (I-1 resolution, ADR-009 rev 5). Kani targets are pure-core helper functions
  colocated in `src/reader.rs` as private functions, NOT the effectful
  `from_pcap_reader<R: Read>` top-level entry point (which cannot be Kani-proved due
  to I/O). Per-module row anchor is `reader.rs` (the compilation unit); the `[b]`
  annotation signals the proof harnesses target pure-core sub-functions within it,
  mirroring the VP-024 `[a]` annotation (umbrella anchor to arp.rs, Sub-A harnesses
  authored in decoder.rs). VP-028 (cargo-fuzz) correctly targets `from_pcap_reader`
  — the effectful entry point, appropriate for fuzzing. VP-029 and VP-030 (proptest)
  target pure predicate/aggregation logic extracted from the block-walk and multi-IDB
  policy layers within reader.rs.

  VP-025 Kani unwind note (I-2): the base-10 branch of
  `pcapng_timestamp_to_secs_usecs` uses `checked_pow` (iterative). The harness MUST
  either (A) use a precomputed power-of-ten lookup (preferred — no loop, trivially
  bounded) or (B) carry `#[kani::unwind(128)]`. Without one of these, Kani's default
  unwind=1 produces a vacuous proof. See ADR-009 rev 5 VP-025 Kani Provability Note.

- `module-criticality.md` defines kill-rate targets that constrain the minimum proof
  depth for each module. CRITICAL modules (reassembly/segment.rs, reassembly/flow.rs,
  analyzer/tls.rs) require Kani proofs, not just proptest.
- VP-007 (mitre.rs / Kani): catalog now contains ICS sub-technique IDs `T1692.001` and
  `T1692.002` (replacing revoked `T0855` and `T0856` per MITRE ATT&CK-ICS v19.1, issue #222).
  Both IDs satisfy the `T[0-9]{4}(\.[0-9]{3})?` format invariant and must be present in
  `SEEDED_TECHNIQUE_IDS` when the harness is updated. VP-007 proof status remains `verified`;
  the harness will be updated atomically with the `src/mitre.rs` source change in the same
  commit (implementer responsibility, issue #222 code fix).

- VP-033 and VP-034 (analyzer/enip.rs / proptest): draft; lock gate at F6. These two VPs
  were authored as part of RULING-EDGECASE-001 (EC-X1/EC-X2) spec adjudication. VP-033
  guards BC-2.17.016 v2.0 Inv-7 (carry-buffer direction isolation): Harness-A
  (proptest_vp033_direction_isolation_pdu_count) confirms interleaved c2s/s2c deliveries
  produce pdu_count==2 with carry_c2s and carry_s2c never mixed; Harness-B
  (proptest_vp033_independent_run_equivalence) confirms the interleaved pdu_count equals the
  sum of independent same-direction runs. VP-034 guards the backwards-timestamp no-spurious-
  reset property across all three windowed detections (BC-2.17.008 v1.3 / BC-2.17.012 v1.2 /
  BC-2.17.018 v1.1) introduced by EC-X2: Sub-A (T0836 write-burst window backwards-ts no-reset;
  BC-2.17.012 v1.2 EC-009), Sub-B (T0888 error-rate window backwards-ts no-reset; BC-2.17.008
  v1.3 EC-009), Sub-C (T0814 malformed-frame window backwards-ts no-reset + EC-X4 operator pin:
  elapsed==300 NOT > 300; BC-2.17.018 v1.1 EC-008), Sub-D (genuine u32 rollover deterministic
  unit test — window_start near u32::MAX, post-rollover now_ts near 0; saturating_sub returns 0).
  These are proptest (not Kani) because the window state machines operate over the stateful
  EnipFlowState — suitable for property-based testing but not for bounded Kani model-checking
  at the whole-flow-state level. The analyzer/enip.rs row now carries 1 Kani + 2 proptest = 3
  total VPs. Grand Totals at time of authoring: Kani(15) + proptest(12) + fuzz(2) + integration/unit(5) = 34.

- VP-035 and VP-036 (analyzer/dnp3.rs / proptest): draft; lock gate at F6. These two VPs
  were authored as part of RULING-DNP3-SIBLING-001 (DRIFT-DNP3-DIRECTION-001 and
  DRIFT-DNP3-CLOCK-001) spec adjudication. VP-035 guards BC-2.15.016 v2.0 Invariant 6
  (carry-buffer direction isolation, DNP3 sibling of VP-033/ENIP): harness
  `proptest_vp035_direction_isolation_frame_count` confirms interleaved c2s/s2c frame
  deliveries produce correct frame_count with carry_c2s and carry_s2c never mixed; harness
  `proptest_vp035_independent_run_equivalence` confirms interleaved frame_count equals the sum
  of independent same-direction runs. DNP3 frames use the 10-byte minimum DL header
  (sync [0x05, 0x64], LENGTH, CTRL with direction bit-7, DEST/SRC/CRC); split_offset 1..9
  (not 1..23 as in ENIP). VP-036 guards the backwards-timestamp no-spurious-reset property
  across all three DNP3 windowed detections introduced by DRIFT-DNP3-CLOCK-001 and
  DRIFT-DNP3-OP-001: Sub-A (T1692.001 direct-operate 60s window backwards-ts no-reset;
  BC-2.15.010 v1.8 EC-012), Sub-B (T1691.001 block-command-timeout 10s window backwards-ts
  no-spurious-fire; BC-2.15.014 v2.1 EC-009), Sub-C (T0827/T0814 correlation-window 300s
  backwards-ts no-reset + DRIFT-DNP3-OP-001 operator pin: elapsed==300 NOT > 300;
  BC-2.15.015 v2.0 EC-010), Sub-D (genuine u32 rollover deterministic unit test — all three
  DNP3 windows; saturating_sub returns 0, no spurious reset). These are proptest (not Kani)
  for the same reason as VP-034: DNP3 window state machines operate over stateful
  Dnp3FlowState. The analyzer/dnp3.rs row now carries 1 Kani (VP-023) + 2 proptest
  (VP-035, VP-036) = 3 total VPs. Grand Totals: Kani(15) + proptest(14) + fuzz(2) +
  integration/unit(5) = 36.

- VP-039 (analyzer/tls.rs / proptest + unit tests): draft; lock gate at F6. VP-039 was
  authored as part of the fix-tls-clienthello-frag F2 spec evolution (AMENDED Pass-1 through
  Pass-4 + Fix-burst-5 + Fix-burst-6 adversarial reconciliation). It covers reassembly
  correctness (Sub-A), exact-consume (Sub-B), clear-and-recover overflow policy (Sub-C, 4
  unit tests), truncation-safety (Sub-D), direction isolation (Sub-E), bounded-carry invariant
  (Sub-F), canonical-frame RFC 8446 §4 decode correctness (including Frame C dispatch-lane
  pin F-FRESH-002), SNI-boundary deterministic coverage (F-F2-011), and malformed-assembled-body
  error semantics (F-FRESH-001).
  SEAM CONTRACT (Pass-2 F-F2-001 fix): aggregate counters (parse_errors, sni_counts,
  ja3_counts, handshakes_seen, handshake_reassembly_overflows) are TlsAnalyzer-level fields
  and MUST be read via analyzer accessors (parse_error_count(), sni_counts(), ja3_counts(),
  handshake_count(), handshake_reassembly_overflow_count()) — NEVER off TlsFlowState.
  Sub-A (proptest_vp039_carry_reassembly_two_record) verifies BC-2.07.038: partial-header
  splits {1,2,3} guaranteed via prop_oneof; SNI-region guaranteed deterministically by
  test_vp039_sni_boundary_deterministic. Sub-B (proptest_vp039_exact_consume_coalesced)
  verifies BC-2.07.042: coalesced messages in one record (non-zero body_len) each dispatched
  independently; handshakes_seen==1 asserted DIRECTLY via analyzer.handshake_count() (F-F2-012
  fix — not inferred from ja3_counts.len()==1). Sub-C verifies BC-2.07.039 clear-and-recover
  policy (F-CRITICAL-2 fixture corrected): test_vp039_carry_overflow_clear_and_recover —
  valid-header body_len=65,500 ([0x01,0x00,0xFF,0xDC]) followed by accumulation records until
  carry.len()+next_payload>MAX_BUF triggers Decision-5 buffer-fill guard exactly once → carry
  cleared to len==0, analyzer.handshake_reassembly_overflow_count()+1 [TlsAnalyzer aggregate,
  NOT TlsFlowState field], parse_errors unchanged [prior 0xCC fill fixture decoded
  body_len=0xCCCCCC>>MAX_BUF — hit Decision-4 body_len-spoof guard 4× making counter==
  overflows_before+4; assertion ==overflows_before+1 was provably FALSE]; test_vp039_carry_overflow_recovery
  (post-overflow ClientHello IS dispatched — distinguishes Policy A from rejected Policy B);
  test_vp039_body_len_spoof (body_len=65537>MAX_BUF triggers Decision-4 clear-and-recover);
  test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key (overflow + summarize()
  asserts detail["handshake_reassembly_overflows"].as_u64()==1 — value-equality NOT mere key presence; BC-2.07.039 PC-7; F-P3-004; F-F2IMPL-001). Sub-D
  (test_vp039_truncated_carry_no_error) verifies BC-2.07.040 PC3: on_flow_close with partial
  carry produces no finding (findings_count pre==post) and no parse_errors increment. Sub-E
  (proptest_vp039_direction_isolation) verifies BC-2.07.041: interleaved c2s/s2c deliveries
  are direction-isolated, carry_c2s and carry_s2c never mixed, split range = function of
  actual message length; parse_errors via analyzer.parse_error_count(). Sub-F
  (proptest_vp039_carry_bounded_invariant — F-F2P-IMP-001 generator restructured): verifies
  BC-2.07.039 Inv-1 — carry.len()≤MAX_BUF after every on_data call; generator now draws
  body_len from 0..=65_536 (valid-header prefix via prop_flat_map) ensuring genuine carry
  accumulation; prior arbitrary-u8 generator was near-vacuous (first 4 bytes decoded
  body_len>MAX_BUF on ~99.6% of cases → Decision-4 fired immediately, carry never accumulated).
  Canonical-frame test (test_BC_2_07_038_canonical_frame_rfc8446_s4 — F-FRESH-002 Frame C
  added; fix-burst-10 Frame A corrected): Frame A [0x01,0x00,0x00,0x05] body_len=5 — pins
  BE decode correctness AND PC-9 malformed-body path: body_len=5 is length-complete but too
  short for a valid ClientHello; parse_tls_message_handshake Err → parse_errors==errors_before+1,
  carry exact-consumed (carry_len=0), client_hello_seen==false, no panic (BC-2.07.038
  AC-CANONICAL-FRAME v2.5); Frame B discriminator [0x01,0x01,0x05,0x00]
  BE=66816>MAX_BUF→carry_len=0, LE=1281→carry_len=4, pins decode direction
  (DF-CANONICAL-FRAME-HOLDOUT-001); Frame C NEW [0x01,0x00,0x01,0x00] body_len=256
  mid-range dispatch-lane — carry drains to 0, parse_errors+1 (malformed all-zeros body),
  pins BE decode in dispatch lane not only at overflow boundary. SNI-boundary deterministic
  test (test_vp039_sni_boundary_deterministic): scans the built ClientHello bytes at runtime
  for the [0x00,0x00] SNI extension type marker (after the compression block), splits at
  sni_ext_start+1 (provably inside the SNI extension type field), and asserts the split offset
  falls within the SNI extension byte range (sni_ext_start>4 and split<n); replaces blind n/2
  (F-P3-006 fix); asserts SNI populated — deterministic guarantee, not probabilistic.
  Malformed-assembled-body test (test_BC_2_07_038_malformed_assembled_body — F-FRESH-001 NEW):
  delivers fragmented handshake with body_len=6 header consistent but body malformed (version
  OK, missing Random field); assembled length-complete body fails parse_tls_message_handshake
  → parse_errors+1 (parity with single-record parse_errors discipline), exact-consume 4+6=10
  bytes, no finding, no panic; ADR-011 Decision-4 error semantics; PO must author matching BC
  postcondition/EC for Red-Gate test name test_BC_2_07_038_malformed_assembled_body.
  Total VP-039 harnesses: 4 proptest + 13 unit tests = 17 (fix-burst-7 corrected count; fix-burst-10 Frame A semantics corrected; fix-burst-11 +3 unit tests). The 13 unit tests are:
  (1) test_vp039_carry_overflow_clear_and_recover; (2) test_vp039_carry_overflow_recovery;
  (3) test_vp039_body_len_spoof; (4) test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key;
  (5) test_vp039_truncated_carry_no_error; (6) test_BC_2_07_038_canonical_frame_rfc8446_s4;
  (7) test_vp039_sni_boundary_deterministic; (8) test_BC_2_07_038_malformed_assembled_body;
  (9) test_BC_2_07_040_empty_carry_flow_close (F-FRESH2-003: Sub-D-ext, BC-2.07.040 empty-carry degenerate);
  (10) test_BC_2_07_042_exact_consume_no_double_dispatch (F-FRESH2-003: Sub-B-ext, BC-2.07.042 deterministic coalesce);
  (11) test_BC_2_07_041_cross_flow_isolation (F-COMP-002: Sub-E-ext, two distinct FlowKeys, BC-2.07.041 PC-1/PC-4/Inv-1);
  (12) test_vp039_n_record_reassembly (F-COMP-001: Sub-A-ext-N, >=3-record re-entrancy, BC-2.07.038 PC-1/PC-2/PC-6+EC-003);
  (13) test_vp039_large_valid_hello_reassembly (F-COMP-003: Sub-C-ext-large, ~40 KB valid ClientHello, BC-2.07.038 Inv-5).
  Sub-F (proptest_vp039_carry_bounded_invariant) confirms the bounded-carry invariant (carry.len()<=MAX_BUF)
  generatively; Decision-5 buffer-fill path is exercised DETERMINISTICALLY by test (1) above, not
  probabilistically by Sub-F (F-FRESH2-004). The analyzer/tls.rs row NOW carries 1 Kani (VP-005) +
  2 proptest (VP-013, VP-039) + 1 unit (VP-040) = 4 total VPs (after VP-040 addition).

- VP-040 (analyzer/tls.rs / unit): draft; lock gate at F6. Authored as part of
  fix-tls-clienthello-frag F2 scope-addition (BC-2.07.043 / F-EV-001 defense-in-depth).
  VP-040 is DISTINCT from VP-039: VP-039 covers the handshake-carry layer
  (client_hs_carry/server_hs_carry MAX_BUF overflow → handshake_reassembly_overflows counter);
  VP-040 covers the TCP-segment buffer layer (client_buf/server_buf MAX_BUF tail-drop →
  buffer_saturation_drops counter). These are separate buffer layers with different overflow
  semantics and different counter fields. The two VPs cannot be merged: conflating them would
  obscure which primitive lost data. VP-040 has 6 deterministic unit tests with exact fixture
  control; proptest is not warranted because the property is fully deterministic (increment
  condition data.len()>remaining = exactly +1). INCREMENT CONDITION: data.len() > remaining
  (single condition; covers partial-drop where remaining>0 AND full-drop where remaining==0;
  to_copy is undefined when remaining==0 so to_copy<data.len() is WRONG). SEAM CONTRACT:
  buffer_saturation_drops is a TlsAnalyzer-level aggregate, read via buffer_saturation_drop_count()
  accessor — NEVER off TlsFlowState. SEAM for full-drop: fill_buf_for_testing on TlsAnalyzer
  (FINAL — BC-2.07.043 Architecture Anchor; NOT the TlsFlowState alternative).
  Sub-A (test_BC_2_07_043_buffer_saturation_observable): PARTIAL-DROP path — deliver a single
  65,537-byte slice to an empty buffer (remaining==65,536); data.len()>remaining: 65,537>65,536
  → 1 byte dropped; counter+1, parse_errors unchanged, no finding. No test seam required.
  Sub-A-full-drop (test_BC_2_07_043_buffer_saturation_full_drop): FULL-DROP path — use
  fill_buf_for_testing seam to set remaining==0; deliver any non-empty slice; entire slice
  dropped; counter+1 (EC-002; BC-2.07.043 PC-1). Seam is FINAL: fill_buf_for_testing.
  Sub-B (test_BC_2_07_043_no_drop_no_counter): 6-byte small record — no drop, counter unchanged.
  Sub-C (test_BC_2_07_043_counter_persists_across_flows): trigger drop, call
  on_flow_close(CloseReason::Fin), assert counter==1 — NOT reset (aggregate NOT per-flow,
  mirrors truncated_records).
  Sub-D (test_BC_2_07_043_summarize_value_equals_drop_count): trigger overflow,
  call summarize(), assert detail["buffer_saturation_drops"].as_u64()==1 — value-equality NOT
  mere key presence (mirrors VP-039 Sub-C summarize pattern; BC-2.07.043 PC-4).
  Sub-E (test_BC_2_07_043_both_directions_increment_same_counter): one c2s drop + one s2c drop
  (use two independent flows or trigger s2c drop on second flow); assert counter==initial+2.
  The 6 canonical test names (FINAL per DF-AC-TEST-NAME-SYNC):
  (1) test_BC_2_07_043_buffer_saturation_observable; (2) test_BC_2_07_043_buffer_saturation_full_drop;
  (3) test_BC_2_07_043_no_drop_no_counter; (4) test_BC_2_07_043_counter_persists_across_flows;
  (5) test_BC_2_07_043_summarize_value_equals_drop_count; (6) test_BC_2_07_043_both_directions_increment_same_counter.
  The analyzer/tls.rs row now carries 1 Kani (VP-005) + 2 proptest (VP-013, VP-039) +
  1 unit (VP-040) = 4 total VPs. Grand Totals (post VP-040 addition):
  Kani(15) + proptest(17) + fuzz(2) + integration/unit(6) = 40.

- VP-037 and VP-038 (analyzer/modbus.rs / proptest): draft; lock gate at F6. These two VPs
  were authored as part of RULING-MODBUS-SIBLING-001 (DRIFT-MODBUS-DIRECTION-001 and
  DRIFT-MODBUS-CLOCK-001) spec adjudication; they are the Modbus siblings of VP-033/VP-034
  (ENIP) and VP-035/VP-036 (DNP3). VP-037 guards BC-2.14.002 v2.0 Invariant 4 (carry-buffer
  direction isolation, Modbus): harness `proptest_vp037_direction_isolation_fn_code_counts`
  confirms interleaved c2s/s2c deliveries produce correct fn_code_counts with carry_c2s and
  carry_s2c never mixed; harness `proptest_vp037_independent_run_equivalence` confirms the
  interleaved count equals the sum of independent same-direction runs. Modbus ADUs use a
  minimal 13-byte frame (8-byte MBAP header + 5-byte PDU); split_offset 1..7 (partial MBAP
  header below the 8-byte minimum). EC-007 (direction non-contamination) is jointly guarded.
  VP-038 guards the backwards-timestamp no-spurious-reset property across all four Modbus
  windowed detections (BC-2.14.016 v2.3 / BC-2.14.017 v2.7 / BC-2.14.019 v1.5) introduced
  by DRIFT-MODBUS-CLOCK-001: Sub-A (T0831 5s inactivity window backwards-ts no-reset;
  BC-2.14.016 v2.3 EC-010), Sub-B (T0806 burst 1s window backwards-ts no-reset; BC-2.14.017
  v2.7 EC-010), Sub-C (T0806 sustained >=2s window — the >= is INTENTIONAL minimum-duration
  gate per RULING-MODBUS-SIBLING-001 §2.3; this sub-harness validates the operator is correct,
  not a defect; BC-2.14.017 v2.7 EC-012), Sub-D (T0888 exception 10s window backwards-ts
  no-reset; BC-2.14.019 v1.5 EC-009), Sub-E (genuine u32 rollover deterministic unit test —
  all four Modbus windows; saturating_sub returns 0, no spurious reset). These are proptest
  (not Kani) for the same reason as VP-034 and VP-036: Modbus window state machines operate
  over stateful ModbusFlowState. STORY-141 implements the harnesses; STORY-142 (DNP3 desync-
  latch) does not add a new VP per RULING-DNP3-DESYNC-001 (regression test only). The
  analyzer/modbus.rs row now carries 1 Kani (VP-022) + 2 proptest (VP-037, VP-038) = 3 total
  VPs. Grand Totals: Kani(15) + proptest(16) + fuzz(2) + integration/unit(5) = 38.
