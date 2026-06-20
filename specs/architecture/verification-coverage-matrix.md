---
artifact: architecture-section
section: verification-coverage-matrix
traces_to: ARCH-INDEX.md
version: "1.18"
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
| VP-025 | pcapng timestamp conversion totality: no panic, ts_usecs in [0,999999], ts_sec saturated (.min(u32::MAX)), saturating arithmetic for all (u32,u32,u8); large-ts_high Kani vector required (rev 8 / M-3) | reader.rs (pcapng_pure_core fns) [b] | Kani | P1 | draft |
| VP-026 | pcapng SHB parse safety: no panic, byte-order BOM detection correct (LE/BE), Err for <28 bytes | reader.rs (pcapng_pure_core fns) [b] | Kani | P1 | draft |
| VP-027 | pcapng EPB parse safety: no panic; interface_id discriminant — empty table → Err(E-INP-009); OOB non-empty table → Err(E-INP-010); slash notation removed (rev 9 / Decision 22 / F-H4); guard-before-allocate; padding-overrun (20+captured_len+pad_len>body.len()) → Err(E-INP-008); bound-by-body (captured_len>body.len()-20) → Err(E-INP-008); NOT E-INP-010 (rev 8 / C-1) | reader.rs (pcapng_pure_core fns) [b] | Kani | P1 | draft |
| VP-028 | pcapng reader no-panic (cargo-fuzz fuzz_pcapng_reader, F6 hardening) | reader.rs | cargo-fuzz | P1 | draft |
| VP-029 | pcapng block-walk skip: always terminates, Err-breaks loop, cursor advances >= 12 bytes per Ok | reader.rs | proptest | P1 | draft |
| VP-030 | pcapng multi-IDB linktype agreement totality (RESTATED rev 7 / H-3): WHITELISTED DataLink domain only; all-equal → Ok, first-differing whitelisted DataLink → Err(E-INP-011); non-whitelisted → E-INP-001 (out of scope); comparison unit DataLink not raw u16 | reader.rs | proptest | P1 | draft |
| VP-031 | pcapng SPB captured-len arithmetic correctness (spb_data_available formula): captured_len == min(original_len, body.len() as u32 - 4) = min(original_len, spb_data_available); slice length == captured_len; no OOB for all (u32, &[u8] with len>=4) inputs; formula CORRECTED from rev 8 (body.len() → body.len()-4 per Decision 22 / F-H2 / F-H3); snaplen DROPPED (rev 8 / Decision 9) | reader.rs (pcapng_pure_core fns) [b] | proptest | P1 | draft |


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
| reader.rs | 3 (VP-025, VP-026, VP-027) [b] | 3 (VP-029, VP-030, VP-031) [b] | 1 (VP-028) | 0 | 7 |
| **Totals** | **14** | **10** | **2** | **5** | **31** |


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

- VP-025 through VP-031 (reader.rs) are status=draft pending BC revisions by the PO
  per ADR-009 rev 4/5/6/7/8 PO BC-Change Dispatch and F3 story decomposition. VP-028
  (cargo-fuzz) is explicitly an F6 hardening deliverable; it is NOT expected to be
  exercised in F3/F4. VP-025, VP-026, VP-027 (Kani) and VP-029, VP-030, VP-031
  (proptest) will transition to verified at F6 per the VP-022/VP-023/VP-024 lifecycle
  pattern. VP-031 (proptest) was added in Pass-3 (ADR-009 rev 6 / Decision 18) to
  provide the arithmetic correctness VP for BC-2.01.013 SPB body-clamping — VP-028
  cargo-fuzz covers no-panic but cannot assert the arithmetic relationship.
  **Rev 8 amendment (Decision 9 / H-3 + M-2):** VP-031 property narrowed — snaplen
  DROPPED; formula was `captured_len == min(original_len, body.len() as u32)` (corrected
  in rev 9 / Decision 22 — see below).
  VP-031 fills that gap per DF-CANONICAL-FRAME-HOLDOUT-001. BC-2.01.013 carries DUAL
  VP coverage: VP-031 (arithmetic correctness, proptest) + VP-028 (no-panic, cargo-fuzz).
  **Rev 8 property amendments also apply to VP-025** (ts_sec saturation + large-ts_high
  Kani vector, M-3) **and VP-027** (padding-overrun/bound-by-body → E-INP-008 explicit,
  C-1 / Decision 20 clarification). No VP counts changed.
  **Rev 9 amendment (Decision 22 / F-H2 / F-H3):** VP-031 formula CORRECTED —
  `min(original_len, body.len() as u32)` → `min(original_len, body.len() as u32 - 4)`.
  On the raw-block path `RawBlock.body` for SPB includes the 4-byte `original_len`
  fixed field followed by packet data; rev 8 incorrectly used the full body length as
  the data bound. The canonical symbol `spb_data_available = body.len() - 4` (= btl-16)
  is now defined in Decision 22. Property domain: (original_len: u32, body: &[u8])
  with body.len() >= 4.
  **Rev 9 amendment (Decision 22 / F-H4):** VP-027 interface_id check extended to
  assert error DISCRIMINANT — empty table → E-INP-009 specifically; OOB non-empty →
  E-INP-010 specifically. Slash notation removed. Kani harness must model table-size
  as symbolic boolean. No VP counts changed.

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
