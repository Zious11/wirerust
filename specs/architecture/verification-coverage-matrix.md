---
artifact: architecture-section
section: verification-coverage-matrix
traces_to: ARCH-INDEX.md
version: "1.8"
status: verified
producer: architect
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


## Per-Module Coverage Totals

| Module | Kani | proptest | cargo-fuzz | integration/unit | Total VPs |
|--------|------|----------|------|-----------------|-----------|
| reassembly/flow.rs | 2 (VP-001, VP-009) | 0 | 0 | 0 | 2 |
| reassembly/segment.rs | 2 (VP-002, VP-015) | 2 (VP-010, VP-011) | 0 | 0 | 4 |
| reassembly/mod.rs | 1 (VP-003) | 1 (VP-021) | 0 | 0 | 2 |
| dispatcher.rs | 1 (VP-004) | 0 | 0 | 0 | 1 |
| analyzer/tls.rs | 1 (VP-005) | 1 (VP-013) | 0 | 0 | 2 |
| analyzer/http.rs | 0 | 2 (VP-006, VP-014) | 0 | 0 | 2 |
| mitre.rs | 1 (VP-007) | 0 | 0 | 0 | 1 |
| decoder.rs | 0 | 0 | 1 (VP-008) | 0 | 1 |
| reporter/terminal.rs | 0 | 1 (VP-012) | 0 | 1 (VP-016) | 2 |
| reporter/json.rs | 0 | 0 | 0 | 1 (VP-017) | 1 |
| cli.rs | 0 | 0 | 0 | 1 (VP-018) | 1 |
| analyzer/dns.rs | 0 | 0 | 0 | 1 (VP-019) | 1 |
| reporter/csv.rs | 0 | 0 | 0 | 1 (VP-020) | 1 |
| analyzer/modbus.rs | 1 (VP-022) | 0 | 0 | 0 | 1 |
| analyzer/dnp3.rs | 1 (VP-023) | 0 | 0 | 0 | 1 |
| analyzer/arp.rs | 1 (VP-024) [a] | 0 | 0 | 0 | 1 |
| **Totals** | **11** | **7** | **1** | **5** | **24** |


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

- `module-criticality.md` defines kill-rate targets that constrain the minimum proof
  depth for each module. CRITICAL modules (reassembly/segment.rs, reassembly/flow.rs,
  analyzer/tls.rs) require Kani proofs, not just proptest.
- VP-007 (mitre.rs / Kani): catalog now contains ICS sub-technique IDs `T1692.001` and
  `T1692.002` (replacing revoked `T0855` and `T0856` per MITRE ATT&CK-ICS v19.1, issue #222).
  Both IDs satisfy the `T[0-9]{4}(\.[0-9]{3})?` format invariant and must be present in
  `SEEDED_TECHNIQUE_IDS` when the harness is updated. VP-007 proof status remains `verified`;
  the harness will be updated atomically with the `src/mitre.rs` source change in the same
  commit (implementer responsibility, issue #222 code fix).
