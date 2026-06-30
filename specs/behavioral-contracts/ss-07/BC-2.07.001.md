---
document_type: behavioral-contract
level: L3
version: "2.0"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Wave 16 Pass-2 sibling sweep (F-W16-S052-P2-003) — tighten architecture anchor 379-387 → 384-387 (fn signature excluded; body starts at 384) — 2026-05-28"
  - "v1.4: Wave 16 Pass-4 (F-W16-S052-P4-002) — add VP table rows for invariant 2 (version_counts / ja3_counts bounded at MAX_MAP_ENTRIES) citing discriminating tests at tests/tls_analyzer_tests.rs:2747 and :2811 — 2026-05-28"
  - "v1.5: DF-SIBLING-SWEEP-001 — fix stale test-file line anchors: tls_analyzer_tests.rs:2747 → 4476 (test_BC_2_07_001_inv2_version_counts_bounded_at_max_map_entries), :2811 → 4540 (test_BC_2_07_001_inv2_ja3_counts_bounded_at_max_map_entries); verified against HEAD cfe0112a — 2026-06-01"
  - "v1.6: PG-ARP-F2-007 ss-07 full re-anchor — handle_client_hello range 379-540→389-580 (fn sig 389-394, body 395-580); handshakes_seen++ at 395; version count at 398; JA3 compute at 519; SNI extraction 413-515; weak-cipher 530-556; deprecated 559-579 — 2026-06-13"
  - "v1.7: fix-tls-clienthello-frag F2 scope expansion — 'complete' now includes ClientHello assembled across multiple TLS records via BC-2.07.038 carry-buffer reassembly; Precondition 2 updated to include fragmented-then-assembled path; Invariant 5 added (single-record fast path preserved); EC-007 added (fragmented ClientHello); Related BCs extended (+BC-2.07.038); TLS-CLIENTHELLO-FRAG-001 cross-reference added — 2026-06-29"
  - "v1.8: Pass-1 adversarial reconciliation (SR-008 MED) — Postcondition 8 rewritten to name both drain operations explicitly: (a) record bytes drained from client_buf at the record layer; (b) assembled handshake message exact-consumed (4+body_len) from client_hs_carry at the carry layer; disambiguates which buffer is 'drained' — 2026-06-29"
  - "v1.9: Pass-2 adversarial reconciliation (F-F2-006 MEDIUM — priority inversion documented) — Related BCs: add explicit note that BC-2.07.001 is P0 (single-record fast path) and depends on P1 BC-2.07.038 (fragmented path); the SINGLE-RECORD ClientHello guarantee is P0; the FRAGMENTED-path guarantee is P1; this inversion is a deliberate design choice — P0 cannot wait for a P1 bug to be fixed since the single-record path is independent of the carry layer — 2026-06-29"
  - "v2.0: F5 architecture-anchor re-anchor (F-F5-001) — handle_client_hello 389-580→431-622 (fn sig 431-436, body 437-622); handshakes_seen/version_counts at 437/440; JA3 at 561; SNI extraction 455-558; develop 8b52046; no semantic change — 2026-06-30"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.001: Parse Complete TLS ClientHello: Version, Ciphers, Extensions, SNI, JA3

## Description

When a complete TLS ClientHello is present for a flow — whether delivered in a single
TLS record or assembled from multiple fragmented records via the carry-buffer
reassembly layer (BC-2.07.038) — `TlsAnalyzer` extracts the protocol version, cipher
suite list, and extensions. From extensions it derives the SNI hostname (classified via
`extract_sni`) and computes the JA3 MD5 fingerprint. The JA3 hash is counted in
`ja3_counts`; the SNI is counted in `sni_counts`; the version is counted in
`version_counts`. `handshakes_seen` is incremented once per ClientHello processed.

**Scope expansion (v1.7 — fix-tls-clienthello-frag):** "complete ClientHello" now
encompasses both the single-record path (common case, no behavioral change) and the
multi-record reassembled path (new capability; see BC-2.07.038). The single-record
fast path is preserved: a ClientHello that arrives complete in one 0x16 record is
dispatched via the same carry drain loop and produces identical output (the carry
buffer is populated then immediately consumed in one pass). See TLS-CLIENTHELLO-FRAG-001.

## Preconditions

1. `TlsAnalyzer::on_data` has been called with bytes for the client direction.
2. The carry buffer for the client direction (`client_hs_carry`) contains a complete
   ClientHello handshake message — either because a single 0x16 record payload was
   sufficient (single-record fast path), or because bytes from multiple records have
   been accumulated by BC-2.07.038 reassembly (fragmented path). In both cases
   `handle_client_hello` receives the same fully assembled bytes.
3. `payload_len <= MAX_RECORD_PAYLOAD` (18,432 bytes) for every contributing record;
   oversized individual records are rejected before the carry buffer (see BC-2.07.004).
4. The flow has not yet been marked `done()` (both hellos already seen).

## Postconditions

1. `handshakes_seen` is incremented by 1.
2. The ClientHello `version` field value (u16) is inserted/incremented in
   `version_counts` (bounded by `MAX_MAP_ENTRIES = 50,000`).
3. A JA3 MD5 hex string (32 lowercase hex chars) is computed via `compute_ja3` and
   inserted/incremented in `ja3_counts` (bounded by `MAX_MAP_ENTRIES`).
4. If the ClientHello extensions include a non-empty SNI list, the first hostname is
   classified and its string key is inserted/incremented in `sni_counts`.
5. If the classified SNI is not `SniValue::Ascii`, a Finding is pushed to
   `all_findings` (see BC-2.07.014..019).
6. If the ClientHello cipher list contains any weak cipher (NULL/ANON/EXPORT), a
   Finding is pushed to `all_findings` (see BC-2.07.009).
7. If the version is <= 0x0300 (SSL 3.0), a Finding is pushed to `all_findings`
   (see BC-2.07.011).
8. Two distinct drain operations occur when a complete ClientHello is dispatched:
   (a) the TLS record bytes are drained from `client_buf` at the record layer as the
   full TLS record payload is consumed by `try_parse_records`; and (b) exactly
   `4 + body_len` bytes of the assembled ClientHello are exact-consumed from
   `client_hs_carry` at the carry layer (the exact-consume step in BC-2.07.038
   Postcondition 4 and Invariant 2). Both drains are required and occur in this order.

## Invariants

1. `handshakes_seen` increments exactly once per ClientHello, regardless of how
   many SNI entries or weak ciphers are present — and regardless of whether the
   ClientHello arrived in one record or was reassembled from fragments.
2. All counter maps are bounded at `MAX_MAP_ENTRIES`; new keys are silently dropped
   when the map is full.
3. Raw SNI bytes are not escaped at this layer (ADR 0003 / INV-4).
4. GREASE cipher/extension/group values are filtered before JA3 computation (INV-2
   of JA3 spec; see BC-2.07.006).
5. The single-record fast path is preserved: the carry drain loop dispatches a
   complete single-record ClientHello in one pass with no observable behavioral
   difference from the pre-fix path. The carry buffer is populated (append) then
   immediately consumed (exact-consume) within the same `try_parse_records` call.
   No regression to existing single-record test coverage.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ClientHello with no extensions (ch.ext = None) | JA3 computed with empty strings for ext/curves/pf; no SNI counting |
| EC-002 | ClientHello with extensions that fail `parse_tls_extensions` | `parse_errors++`; JA3 computed with empty ext fields; SNI not extracted |
| EC-003 | ClientHello with SNI extension but empty ServerNameList | No SNI count; no finding; handshake still counted (see BC-2.07.022) |
| EC-004 | ClientHello with all GREASE ciphers | JA3 cipher field is empty string after filtering; no weak-cipher finding |
| EC-005 | ClientHello version = 0x0303 (TLS 1.2) | Version counted; no deprecated-protocol finding |
| EC-006 | ClientHello when `ja3_counts` is at MAX_MAP_ENTRIES with a new JA3 hash | New hash silently dropped; count unchanged |
| EC-007 | ClientHello fragmented across two 0x16 records (RFC 5246 §6.2.1) — new in v1.7 | Bytes accumulated via BC-2.07.038 carry reassembly; `handle_client_hello` called with fully assembled bytes after second record arrives; SNI and JA3 populated; `parse_errors=0` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Full ClientHello with "example.com" SNI, TLS 1.2 version, strong ciphers | handshakes_seen=1; sni_counts["example.com"]=1; ja3_counts has one entry; no findings | happy-path |
| ClientHello with GREASE cipher 0x0a0a in cipher list alongside one real cipher | JA3 string excludes 0x0a0a; same JA3 as without GREASE cipher | happy-path |
| ClientHello with null extensions field | ja3 computed with 4 empty comma-separated fields after version; sni_counts empty | edge-case |
| ClientHello with version 0x0300 (SSL 3.0) | findings has one deprecated-protocol finding | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | handshakes_seen increments exactly once per ClientHello | unit: test_parse_client_hello |
| — | JA3 is 32 lowercase hex chars | proptest: compute_ja3_has_five_fields_and_hex_hash |
| — | GREASE values do not change JA3 hash | proptest: compute_ja3_is_grease_invariant |
| — | Invariant 2: version_counts bounded at MAX_MAP_ENTRIES; new keys silently dropped when full | unit: test_BC_2_07_001_inv2_version_counts_bounded_at_max_map_entries (tests/tls_analyzer_tests.rs:4476) |
| — | Invariant 2: ja3_counts bounded at MAX_MAP_ENTRIES; new keys silently dropped when full | unit: test_BC_2_07_001_inv2_ja3_counts_bounded_at_max_map_entries (tests/tls_analyzer_tests.rs:4540) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- this BC is the primary ClientHello processing entry point for all TLS analysis |
| L2 Domain Invariants | INV-5 (SNI 4-way classification), INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:431-622, C-13) |
| Stories | STORY-052, STORY-144 |
| Origin BC | BC-TLS-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.07.002 -- related to (ServerHello counterpart)
- BC-2.07.003 -- related to (short-circuit after both hellos seen)
- BC-2.07.006 -- composes with (GREASE filtering in JA3)
- BC-2.07.007 -- composes with (JA3 string format)
- BC-2.07.009 -- composes with (weak cipher detection)
- BC-2.07.011 -- composes with (deprecated version detection)
- BC-2.07.038 -- depends on (carry-buffer reassembly that delivers the assembled ClientHello bytes to handle_client_hello; this BC's Precondition 2 expanded to include the reassembled path)

> **Priority-Inversion Note (F-F2-006, deliberate design):** BC-2.07.001 is **P0** but depends on BC-2.07.038 which is **P1**. This inversion is intentional and documented. The **single-record ClientHello path** (the common case, no fragmentation) is the P0 guarantee — it does not require the carry layer to exist; a single-record ClientHello passes through the carry drain loop even if the carry-buffer implementation is minimal. The **fragmented ClientHello path** (requiring carry accumulation across multiple records) is the P1 guarantee (BC-2.07.038). P0 and P1 are independently deliverable because a broken or absent carry layer does not break the single-record path — it only breaks fragmented-ClientHello reassembly. Reviewers must not raise this inversion as an unintentional inconsistency; it is a deliberate scope-split between the common P0 path and the evasion-resistance P1 enhancement.

## Architecture Anchors

- `src/analyzer/tls.rs:431-622` -- `handle_client_hello` implementation (fn sig 431-436, body 437-622)
- `src/analyzer/tls.rs:437-440` -- `handshakes_seen` increment (437) and `version_counts` update (440)
- `src/analyzer/tls.rs:561` -- JA3 computation and count
- `src/analyzer/tls.rs:455-558` -- SNI extraction and finding emission
- `tests/tls_analyzer_tests.rs` -- test_parse_client_hello

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:431-622` (`handle_client_hello`) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: TlsClientHelloContents struct fields from tls_parser crate
- **guard clause**: `ch.ext` None/Some branch; `parse_tls_extensions` Ok/Err branch
- **assertion**: test_parse_client_hello exercises the full path

## Story Anchor

STORY-144 (TLS Carry Buffer + ClientHello Fragmentation Reassembly — amended: "complete" now includes multi-record reassembled path; single-record fast path unchanged; wave 65)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates handshakes_seen, version_counts, ja3_counts, sni_counts, all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
