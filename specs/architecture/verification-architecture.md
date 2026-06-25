---
artifact: architecture-section
section: verification-architecture
traces_to: ARCH-INDEX.md
version: "2.5"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
modified:
  - date: 2026-06-01
    actor: product-owner
    reason: "Fix VP-015 table entry: correct seq value from 0xFFFF_FFFE (ISN) to isn+1=0xFFFF_FFFF (offset 1) to match VP-015 v1.1 and Kani harness"
  - date: 2026-06-02
    actor: spec-steward
    reason: "Phase-6 gate close: status draft→verified (propagated from VP-INDEX, all 20 VPs locked). Counts unchanged at 20."
  - date: 2026-06-08
    actor: state-manager
    reason: "Feature Mode F2 (issue #100): VP-021 added (timestamp-provenance-threading; draft; integration+proptest). Total 20→21."
  - date: 2026-06-09
    actor: spec-steward
    reason: "F6 lock propagation (FINDING-001): VP-021 moved from Should Prove table to Test Sufficient table (verified @256a490); Test Sufficient count five→six."
  - date: 2026-06-09
    actor: architect
    reason: "F2 delta (issue #7 Modbus TCP): VP-022 added to Should Prove table (P1, Kani, analyzer/modbus.rs). P1 count 7→8. Total 21→22."
  - date: 2026-06-10
    actor: architect
    reason: "Issue #222 (MITRE ATT&CK-ICS v19.1 remap): VP-007 row description updated to note ICS sub-technique acceptance explicitly (T1692.001/T1692.002 replace revoked T0855/T0856). VP count unchanged at 22."
  - date: 2026-06-10
    actor: architect
    reason: "F2 delta (issue #8 DNP3 TCP): VP-023 added to Should Prove table (P1, Kani, analyzer/dnp3.rs). P1 count 8→9. Total 22→23."
  - date: 2026-06-10
    actor: architect
    reason: "Pass-1 adversarial remediation (issue #8 F2): Kani Tooling Selection table row was missing VP-023; appended VP-023 to complete the Kani VP list."
  - date: 2026-06-12
    actor: architect
    reason: "F2 delta ARP security analyzer (SS-16): VP-024 added to Should Prove table (P1, Kani, analyzer/arp.rs). P1 count 9→10. Total 23→24. Tooling table Kani row updated 10→11 VPs."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-13 anchor correction (F-A13-003, label-only — proof unaffected): VP-005 harness skeleton line references updated: fn extract_sni 246→247; 4-way match range 251-265→252-266. Verified against live src/analyzer/tls.rs."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-23 A-03: VP-005 proof harness skeleton Markdown fencing corrected — block was missing opening ```rust fence and closing ``` fence; now properly fenced to match sibling VP-001 and VP-002 skeletons. Proof logic unchanged. Version bump 1.6→1.7."
  - date: 2026-06-14
    actor: architect
    reason: "Pass-22 F3-convergence FIX-1: VP-024 Module cell in Should Prove table updated from 'analyzer/arp.rs' to 'analyzer/arp.rs + decoder.rs [a]' to match VP-INDEX.md:76 authoritative module listing and align with verification-coverage-matrix.md footnote [a] documenting the Sub-A dual-module split (extract_arp_frame lives in src/decoder.rs). Footnote [a] added below Should Prove table. FIX-2: VP-008 proof harness skeleton annotated with forward-reference note mirroring VP-008 v2.2: current signature is pre-STORY-111; STORY-111 changes return type to Result<DecodedFrame>. FIX-3: VP-008 fuzz target filename corrected from decode_packet.rs to fuzz_decode_packet.rs to match delivered harness (VP-008 v1.1, STORY-003 AC-011)."
  - date: 2026-06-14
    actor: architect
    reason: "F3-convergence sweep FIX-1: VP-006 row moved from Must Prove table to Should Prove table — VP-INDEX is authoritative (VP-006=P1); Must Prove table is now 8 rows (P0 VPs only), consistent with P0 enumeration list. FIX-2: Tooling Selection proptest row updated VP-006..014 (6) → VP-006, VP-010..014, VP-021 (7) to match VP-INDEX proptest_count=7. Version bump 1.7→1.8."
  - date: 2026-06-19
    actor: architect
    reason: "F2 pcapng remediation (ADR-009 rev 4): VP-025 through VP-030 added to Should Prove table (all P1, SS-01 pcapng reader). VP-025 Kani (timestamp totality); VP-026 Kani (SHB parse safety); VP-027 Kani (EPB parse safety + interface_id bounds); VP-028 cargo-fuzz (pcapng reader no-panic, F6 deliverable); VP-029 proptest (block-walk skip + forward progress); VP-030 proptest (multi-IDB linktype totality). Tooling Selection table Kani row updated 11→14; proptest row updated 7→9; fuzz row updated 1→2. P1 count 10→16. Total 24→30. Version bump 1.8→1.9."
  - date: 2026-06-19
    actor: architect
    reason: "Pass-2 adversarial remediation (ADR-009 rev 5, I-1/I-2): VP-025/VP-026/VP-027 Module cell in Should Prove table updated from 'reader.rs' to 'reader.rs (pcapng_pure_core fns) [b]' — Kani targets pure-core helper functions, not the effectful from_pcap_reader entry point. VP-028/029/030 module unchanged. Footnote [b] added documenting the pure-core anchor and VP-025 Kani unwind-bound requirement (Option A: lookup table preferred; Option B: #[kani::unwind(128)]). Version bump 1.9→2.0."
  - date: 2026-06-19
    actor: architect
    reason: "Pass-3 adversarial remediation (ADR-009 rev 6 / Decision 18 / M-2): VP-031 added to Should Prove table (P1, proptest, reader.rs (pcapng_pure_core fns) [b], BC-2.01.013). Fills SPB framing VP gap: VP-028 cargo-fuzz covers no-panic but cannot assert arithmetic correctness of captured_len = min(original_len, snaplen, body.len()); VP-031 provides the proptest arithmetic invariant per DF-CANONICAL-FRAME-HOLDOUT-001. Tooling table proptest row updated VP-029/030 (9) → VP-029/030/031 (10). P1 count 16→17. Total 30→31. Version bump 2.0→2.1."
  - date: 2026-06-19
    actor: architect
    reason: "Pass-4 adversarial remediation (ADR-009 rev 7 / H-3): VP-030 row in Should Prove table restated — domain narrowed to WHITELISTED DataLink values only; comparison unit pinned to DataLink not raw u16; non-whitelisted values short-circuit to E-INP-001 before the conflict check (out of VP-030 scope). No VP counts, tool assignments, or Totals changed. Version bump 2.1→2.2."
  - date: 2026-06-20
    actor: architect
    reason: "Pass-5 adversarial remediation (ADR-009 rev 8): VP property updates only — no VP count changes (total 31 / Kani 14 / proptest 10 / fuzz 2 / integration-unit 5 unchanged). VP-025 row: ts_sec saturation (.min(u32::MAX)) and large-ts_high Kani vector added to property (M-3). VP-027 row: padding-overrun and bound-by-body → Err(E-INP-008) NOT E-INP-010 added explicitly (C-1 / Decision 20 clarification). VP-031 row: snaplen DROPPED from property domain; formula is now min(original_len, body.len() as u32) (Decision 9 rev 8 / H-3 + M-2 SPB snaplen asymmetry fix). P1 list entries updated for VP-025/027/031. Version bump 2.2→2.3."
  - date: 2026-06-20
    actor: architect
    reason: "Pass-6 adversarial remediation (ADR-009 rev 9): VP property updates only — no VP count changes (total 31 / Kani 14 / proptest 10 / fuzz 2 / integration-unit 5 unchanged). VP-027 row: interface_id discriminant split — empty table → Err(E-INP-009); OOB non-empty table → Err(E-INP-010); slash notation removed; Kani harness must model table-size as symbolic (Decision 22 / F-H4). VP-031 row: formula CORRECTED from min(original_len, body.len() as u32) to min(original_len, body.len() as u32 - 4) = min(original_len, spb_data_available); canonical symbol spb_data_available defined as body.len()-4; rev 8 formula overstated available data by 4 bytes (Decision 22 / F-H2 / F-H3). Version bump 2.3→2.4."
  - date: 2026-06-24
    actor: architect
    reason: "Feature Mode F2 (feature-enip-v0.11.0, issue #316): VP-032 added to Should Prove table (P1; Kani; draft; src/analyzer/enip.rs; 4 harnesses Sub-A/B/C/D; BC-2.17.001/002/003/004/007). P1 count 17→18. Total 31→32. Tooling Selection Kani row updated to include VP-032 (Kani count 14→15). P1 enumeration list extended. Version bump 2.4→2.5."
---

# Verification Architecture

## Provable Properties Catalog

### Must Prove (security boundaries, state machines, arithmetic invariants)

| VP-ID | Property | Invariant | Module | Tool |
|-------|----------|-----------|--------|------|
| VP-001 | FlowKey canonical ordering: (ip_a,port_a) <= (ip_b,port_b) produces identical key for both directions | INV-1 | reassembly/flow.rs | Kani |
| VP-002 | First-wins overlap: existing bytes always win; ConflictingOverlap finding emitted on content mismatch | INV-3 | reassembly/segment.rs | Kani |
| VP-003 | MAX_FINDINGS cap: reassembler never holds more than MAX_FINDINGS+1 findings (the +1 is finalize bypass per INV-6) | INV-6 | reassembly/mod.rs | Kani |
| VP-004 | Content-first dispatch precedence: TLS signature always wins over port; HTTP method prefix wins over port; DispatchTarget::None is NOT inserted into `routes` before the per-flow classification-attempt counter reaches `max_classification_attempts` (default 8); at the cap it is inserted permanently and reclassification stops | INV-2 | dispatcher.rs | Kani |
| VP-005 | SNI 4-way ordered match: given any byte slice, exactly one arm fires; arm 3 (NonAsciiUtf8) fires when valid UTF-8 + non-ASCII + C0 present (INV-5 boundary case) | INV-5 | analyzer/tls.rs | Kani |
| VP-007 | MITRE technique ID format: every ID emitted by analyzers resolves in technique_info; format matches T[0-9]{4}(\.[0-9]{3})? (covers Enterprise techniques, Enterprise sub-techniques, ICS techniques, and ICS sub-techniques including T1692.001/T1692.002 remapped from revoked T0855/T0856 per issue #222) | INV-9 | mitre.rs | Kani |
| VP-008 | decode_packet never panics on arbitrary input bytes: any byte slice returns Ok or Err, never unwinds | (no-panic invariant) | decoder.rs | cargo-fuzz |
| VP-009 | FlowState machine: no transition reaches an undefined state; RST transitions to Closed from any prior state | (state machine) | reassembly/flow.rs | Kani |

### Should Prove (core algorithms, data transformations)

| VP-ID | Property | Invariant | Module | Tool |
|-------|----------|-----------|--------|------|
| VP-006 | HTTP poison monotonicity: `request_poisoned` / `response_poisoned` fields transition only false->true within a flow's lifetime | INV-8 | analyzer/http.rs | proptest |
| VP-010 | buffered_bytes mirrors segment BTreeMap size sum after every insert/flush/overlap operation | INV-6 related | reassembly/segment.rs | proptest |
| VP-011 | flush_contiguous is monotonic: base_offset strictly advances with each call; no byte delivered twice | (correctness) | reassembly/segment.rs | proptest |
| VP-012 | escape_for_terminal: no C0/DEL/C1 byte survives unescaped; all non-ASCII Unicode > U+009F passes through | ADR 0003 | reporter/terminal.rs | proptest |
| VP-013 | JA3 GREASE filter: all values matching the GREASE pattern (0x?A?A) are removed before fingerprint computation | (spec compliance) | analyzer/tls.rs | proptest |
| VP-014 | HttpAnalyzer cross-flow isolation: parse errors and poisoning in flow A do not affect flow B | (isolation) | analyzer/http.rs | proptest |
| VP-015 | TCP sequence wraparound: segment at seq=isn+1=0xFFFF_FFFF (ISN=0xFFFF_FFFE, offset 1) crossing 32-bit boundary reassembles correctly | (arithmetic) | reassembly/segment.rs | Kani |
| VP-022 | Modbus MBAP parse safety and function-code boundary classification: (A) parse_mbap_header never panics and returns None for <8-byte inputs; (B) classify_fc is total over all 256 FC values; (C) exception detection iff fc >= 0x80 | (no-panic + boundary) | analyzer/modbus.rs | Kani |
| VP-023 | DNP3 data-link frame parse safety and FC classification: (A) parse_dnp3_dl_header never panics, None for <10-byte inputs; (B) classify_dnp3_fc total over all 256 FC values, Control/Restart/Write sets correct; (C) validity gate true iff sync==0x0564 and LENGTH>=5; (D) compute_dnp3_frame_len correct over all LENGTH 5..=255, result in [10,292] | (no-panic + boundary + arithmetic) | analyzer/dnp3.rs | Kani |
| VP-024 | ARP frame parse safety and binding-table invariant: (A) extract_arp_frame never panics on any valid ArpPacketSlice input; Some(ArpFrame) for Eth/IPv4, None otherwise; (B) GARP detection total: is_gratuitous_arp(f)==(f.sender_ip==f.target_ip) for all ArpFrame; (C) binding-table last-write-wins determinism and no-duplicate-key; (D) MAX_ARP_BINDINGS cap never exceeded | (no-panic + GARP totality + binding-table invariant) | analyzer/arp.rs + decoder.rs [a] | Kani |
| VP-025 | pcapng timestamp conversion totality: pcapng_timestamp_to_secs_usecs(ts_high, ts_low, if_tsresol) never panics for any (u32, u32, u8) input; ts_usecs always in [0, 999_999]; ts_sec saturated (.min(u32::MAX)) for all inputs; saturating arithmetic for base-10 pow overflow (e>=20) and base-2 shift clamp (e clamped to [0,63]); intermediate u128 product prevents u64 overflow; Kani harness MUST include large-ts_high vector where ticks/ticks_per_sec > u32::MAX to lock the saturation (rev 8 / M-3) | (no-panic + arithmetic totality + range invariant + saturation-locked) | reader.rs (pcapng_pure_core fns) [b] | Kani |
| VP-026 | pcapng SHB parse safety and byte-order detection: SHB byte-order BOM detection correct for LE magic (0x1A2B3C4D) and BE magic (0x4D3C2B1A); no panic for any truncated/malformed SHB byte sequence; SHB < 28 bytes returns Err | (no-panic + byte-order correctness) | reader.rs (pcapng_pure_core fns) [b] | Kani |
| VP-027 | pcapng EPB parse safety and interface_id discriminant: EPB decode never panics; interface_id discriminant-checked — empty table (table.len()==0) → Err(E-INP-009); OOB on non-empty table (interface_id >= table.len() > 0) → Err(E-INP-010) with context string; slash notation '(→ E-INP-009 / E-INP-010)' removed; two distinct cases required (rev 9 / Decision 22 / F-H4); captured_len guard (captured_len <= block_total_length - 32) precedes any data allocation; padding-overrun (20 + captured_len + pad_len(captured_len) > body.len()) → Err(E-INP-008); bound-by-body (captured_len > body.len() - 20) → Err(E-INP-008); these are wirerust body-decode failures NOT E-INP-010 (crate framing); Err returned for all invalid field combinations (rev 8 / C-1 / Decision 20 clarification) | (no-panic + discriminant-bounds-check + guard-before-allocate + padding-overrun-classified) | reader.rs (pcapng_pure_core fns) [b] | Kani |
| VP-028 | pcapng reader no-panic (full path fuzz): PcapSource::from_pcap_reader returns Ok(_) or Err(_) for any arbitrary byte sequence; no panic, no infinite loop; F6 hardening deliverable (cargo-fuzz target: fuzz_pcapng_reader) | (no-panic + termination) | reader.rs | cargo-fuzz |
| VP-029 | pcapng block-walk skip correctness and forward progress: for any sequence of raw blocks (valid, malformed, unknown-type), the block-walk loop always terminates; each Ok(_) iteration advances the cursor by at least 12 bytes (block header minimum); loop breaks on Err(_) without spinning | (termination + forward-progress) | reader.rs | proptest |
| VP-030 | pcapng multi-IDB linktype agreement totality (RESTATED rev 7 / H-3): for sequences of WHITELISTED DataLink values only (the domain where the E-INP-011 conflict check is reachable), the reader either (a) accepts all (all-equal) producing PcapSource.datalink = that DataLink, or (b) returns Err(E-INP-011) immediately on the first differing whitelisted DataLink; no third outcome. Non-whitelisted values short-circuit to E-INP-001 at first IDB (before conflict check) — NOT in VP-030 domain. Comparison unit: DataLink, not raw u16. | (totality + determinism; whitelisted domain) | reader.rs | proptest |
| VP-031 | pcapng SPB captured-len computation correctness (spb_data_available formula): for all (original_len: u32, body: &[u8]) with body.len() >= 4, captured_len == min(original_len, body.len() as u32 - 4) = min(original_len, spb_data_available); the returned slice has exactly captured_len bytes; no out-of-bounds access. Formula CORRECTED from rev 8 (body.len() → body.len()-4); spb_data_available = body.len()-4 = block_total_length-16 is the canonical symbol; the 4-byte original_len header in the body is NOT packet data and must be subtracted. Snaplen DROPPED (Decision 9 rev 8). (resolves M-2 + H-3 + F-H2 + F-H3 / DF-CANONICAL-FRAME-HOLDOUT-001; formula corrected Decision 22 rev 9) | (arithmetic correctness + bounds safety; two-arg min, snaplen-free, body.len()-4 formula) | reader.rs (pcapng_pure_core fns) [b] | proptest |
| VP-032 | EtherNet/IP + CIP frame parse safety and command/service classification: (Sub-A) `parse_enip_header` never panics, None for <24-byte inputs, Some with correct little-endian field layout for all bounded inputs (no attacker-controlled index beyond fixed offsets); (Sub-B) `classify_enip_command` total over all 65,536 u16 values, Unknown arm reachable and non-vacuous; (Sub-C) `is_valid_enip_frame` biconditional: returns true iff h.command in {0x0004, 0x0063, 0x0064, 0x0065, 0x0066, 0x006F, 0x0070, 0x0072, 0x0075} for all u16 cmd values; (Sub-D) `classify_cip_service` total over all 256 u8 values, response-bit mask (0x80 set → Response variant) proven correct, Unknown arm reachable | (no-panic + boundary + totality + biconditional) | analyzer/enip.rs | Kani |

[a] VP-024 umbrella is anchored to `analyzer/arp.rs` (Sub-B/C/D targets). Sub-A Kani harnesses
(`verify_extract_arp_frame_safety`, `verify_extract_arp_frame_eth_ipv4_correctness`,
`verify_extract_arp_frame_none_on_bad_size`) are authored in the `src/decoder.rs` `#[cfg(kani)]`
block because `extract_arp_frame` lives in `src/decoder.rs` (per vp-024-arp-parse-safety.md
§Proof Harness Skeleton and arp-architecture-delta §6 STORY-112). Mirrors verification-coverage-matrix.md footnote [a].

[b] VP-025 / VP-026 / VP-027 Kani targets are pure-core sub-functions within `src/reader.rs`,
NOT the effectful `from_pcap_reader<R: Read>` entry point. Kani requires pure, I/O-free functions.
The three targets are: (VP-025) `pcapng_timestamp_to_secs_usecs(u32, u32, u8) -> (u32, u32)`;
(VP-026) the pure SHB-body decode function (takes `&[u8]` body slice); (VP-027) the pure EPB
fixed-field-decode function (takes `&[u8]` and interface table size). These are colocated in
`src/reader.rs` as private pure-core helpers. VP-028 (cargo-fuzz) correctly targets
`from_pcap_reader` — the effectful entry point appropriate for fuzzing. See ADR-009 rev 5
and VP-INDEX.md footnote [^vp025-027-module-anchor]. VP-025 Kani harness requires either a
precomputed power-of-ten lookup table (Option A, preferred) or `#[kani::unwind(128)]` (Option B)
to be non-vacuous over symbolic `e`; see ADR-009 rev 5 VP-025 Kani Provability Note.

### Test Sufficient (UI logic, non-critical defaults)

| VP-ID | Property | Tool |
|-------|----------|------|
| VP-016 | TerminalReporter MITRE tactic grouping order matches all_tactics_in_report_order | integration test |
| VP-017 | JsonReporter BTreeMap key determinism: repeated calls with same input produce identical JSON | integration test |
| VP-018 | CLI flag parsing: --reassemble/--no-reassemble mutual exclusion (BC-2.12.007, BC-2.12.009) | integration test |
| VP-019 | DNS statistics-only invariant: DnsAnalyzer.analyze() always returns empty Vec | unit test |
| VP-020 | CsvReporter CSV-injection neutralization: cell values starting with =,+,-,@,TAB,CR are prefixed with a single-quote (') | unit test |
| VP-021 | Timestamp provenance threading: Finding.timestamp equals Some(ts) derived from the on_data timestamp arg for all flow-data-path emission sites; segment-limit summary retains None; cross-flow isolation holds | integration test + proptest |


## P0 Verification Properties (required before Phase 5 gate)

- VP-001: FlowKey canonical ordering (INV-1)
- VP-002: First-wins overlap policy (INV-3)
- VP-003: MAX_FINDINGS cap with finalize bypass (INV-6)
- VP-004: Content-first dispatch precedence (INV-2)
- VP-005: SNI 4-way ordered match boundary (INV-5)
- VP-007: MITRE technique ID format completeness (INV-9)
- VP-008: decode_packet no-panic property
- VP-009: FlowState machine validity

## P1 Verification Properties (required before Phase 6 hardening)

- VP-006: HTTP poison monotonicity (INV-8)
- VP-010: buffered_bytes invariant
- VP-011: flush_contiguous monotonicity
- VP-012: escape_for_terminal correctness (ADR 0003)
- VP-013: JA3 GREASE filter
- VP-014: HttpAnalyzer cross-flow isolation
- VP-015: TCP sequence wraparound
- VP-022: Modbus MBAP parse safety and function-code boundary classification [NEW — SS-14]
- VP-023: DNP3 data-link frame parse safety and function-code classification [NEW — SS-15]
- VP-024: ARP frame parse safety and binding-table invariant [NEW — SS-16]
- VP-025: pcapng timestamp conversion totality (no panic, saturating arithmetic, ts_usecs in [0,999999], ts_sec saturated .min(u32::MAX), large-ts_high Kani vector locking saturation) [NEW — SS-01 pcapng, ADR-009 rev 4; amended rev 8 / M-3]
- VP-026: pcapng SHB parse safety and byte-order detection [NEW — SS-01 pcapng, ADR-009 rev 4]
- VP-027: pcapng EPB parse safety, interface_id discriminant (empty table → E-INP-009; OOB non-empty → E-INP-010; two cases, not slash), guard-before-allocate, and padding-overrun/bound-by-body → Err(E-INP-008) [NEW — SS-01 pcapng, ADR-009 rev 4; amended rev 8 / C-1; discriminant split rev 9 / Decision 22 / F-H4]
- VP-028: pcapng reader no-panic, cargo-fuzz (F6 hardening deliverable) [NEW — SS-01 pcapng, ADR-009 rev 4]
- VP-029: pcapng block-walk skip correctness and forward progress [NEW — SS-01 pcapng, ADR-009 rev 4]
- VP-030: pcapng multi-IDB linktype agreement totality — RESTATED (ADR-009 rev 7 / H-3): domain = WHITELISTED DataLink values only; comparison unit = DataLink; non-whitelisted values → E-INP-001 (out of VP-030 scope) [NEW — SS-01 pcapng, ADR-009 rev 4; restated rev 7]
- VP-031: pcapng SPB captured-len computation correctness (spb_data_available formula; min(original_len, body.len() as u32 - 4); formula corrected from rev 8 per Decision 22; snaplen-free; fills SPB framing VP gap per DF-CANONICAL-FRAME-HOLDOUT-001) [NEW — SS-01 pcapng, ADR-009 rev 6; amended rev 8 / Decision 9; formula corrected rev 9 / Decision 22 / F-H2 / F-H3]
- VP-032: EtherNet/IP + CIP frame parse safety and command/service classification (Kani, 4 sub-properties Sub-A..Sub-D; 5 Kani harnesses — Sub-D = totality + request-partition; draft): parse_enip_header no-panic/None-for-short/little-endian fields; classify_enip_command totality over all u16; is_valid_enip_frame biconditional; classify_cip_service totality over all u8 with response-bit proof [NEW — SS-17, ADR-010, feature-enip-v0.11.0 issue #316]


## Tooling Selection

See `tooling-selection.md` for full rationale. Summary:

| Tool | Target Properties | Scope |
|------|-----------------|-------|
| Kani (model checker) | State machine reachability, arithmetic overflow, pointer safety | VP-001, VP-002, VP-003, VP-004, VP-005, VP-007, VP-009, VP-015, VP-022, VP-023, VP-024, VP-025, VP-026, VP-027, VP-032 |
| proptest | Property-based: generate random inputs, check invariants | VP-006, VP-010..014, VP-021, VP-029, VP-030, VP-031 |
| cargo-fuzz (libFuzzer) | No-panic for parser entry points | VP-008, VP-028 |
| cargo-mutants | Mutation coverage for domain logic | SS-06, SS-07, SS-08, SS-10 |


## Proof Harness Skeletons

### VP-001: FlowKey Canonical Ordering (Kani)

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_flowkey_canonical_ordering() {
    let ip_a: u32 = kani::any();
    let port_a: u16 = kani::any();
    let ip_b: u32 = kani::any();
    let port_b: u16 = kani::any();
    use std::net::Ipv4Addr;
    let a = IpAddr::V4(Ipv4Addr::from(ip_a));
    let b = IpAddr::V4(Ipv4Addr::from(ip_b));
    let key_ab = FlowKey::new(a, port_a, b, port_b);
    let key_ba = FlowKey::new(b, port_b, a, port_a);
    assert_eq!(key_ab, key_ba);
}
```

### VP-002: First-Wins Overlap (Kani)

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_first_wins_overlap() {
    // Insert bytes [A,A,A] at offset 0.
    // Insert bytes [B,B,B] at offset 0 where B != A.
    // Assert: insert_segment returns InsertResult::ConflictingOverlap.
    // Assert: buffered content at offset 0..3 is still [A,A,A].
}
```

### VP-005: SNI 4-way Ordered Match (Kani)

```rust
// Real signature (src/analyzer/tls.rs:247):
//   fn extract_sni(extensions: &[TlsExtension<'_>]) -> Option<SniValue>
//
// The 4-way classification is the inline match at tls.rs:252-266:
//   Ok(s) if s.is_ascii() && !contains_c0_or_del(s) => SniValue::Ascii
//   Ok(s) if s.is_ascii()                             => SniValue::AsciiWithControl
//   Ok(s)                                             => SniValue::NonAsciiUtf8
//   Err(_)                                            => SniValue::NonUtf8
//
// Kani proof target: the byte-to-variant mapping. Because `extract_sni`
// takes a parsed extension list (not raw bytes), the proof harness
// exercises the classification match directly via a helper that wraps
// a synthetic SNI extension backed by a kani::any() byte slice.
//
// Illustrative skeleton:
#[cfg(kani)]
#[kani::proof]
fn verify_sni_classification_exhaustive() {
    // Build a synthetic hostname byte slice of bounded length.
    let len: usize = kani::any();
    kani::assume(len <= 32);
    let hostname: Vec<u8> = (0..len).map(|_| kani::any()).collect();
    // Classify the same bytes using the same logic as the inline match.
    let result = match std::str::from_utf8(&hostname) {
        Ok(s) if s.is_ascii() && !s.bytes().any(|b| b < 0x20 || b == 0x7f) => 0u8,
        Ok(s) if s.is_ascii() => 1u8,
        Ok(_) => 2u8,
        Err(_) => 3u8,
    };
    // Exactly one arm fires (result is 0..=3 by construction).
    // INV-5 arm-3-priority: valid UTF-8 + non-ASCII cannot match arm 0 or 1.
    if let Ok(s) = std::str::from_utf8(&hostname) {
        if !s.is_ascii() {
            assert!(result == 2);
        }
    }
}
```

### VP-008: decode_packet No-Panic (cargo-fuzz)

```rust
// fuzz_target in fuzz/fuzz_targets/fuzz_decode_packet.rs:
// Real signature (src/decoder.rs:128):
//   pub fn decode_packet(data: &[u8], datalink: DataLink) -> Result<ParsedPacket>
// NOTE: pre-STORY-111 current signature; STORY-111 changes return type to
//   Result<DecodedFrame> — see VP-008 §Property Statement (vp-008-decode-packet-no-panic.md v2.2).
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    // try all supported link types (DataLink::IPV6 accepted per decoder.rs:134)
    for datalink in [
        DataLink::ETHERNET,
        DataLink::RAW,
        DataLink::IPV4,
        DataLink::IPV6,
        DataLink::LINUX_SLL,
    ] {
        let _ = decode_packet(data, datalink); // must not panic
    }
});
```
