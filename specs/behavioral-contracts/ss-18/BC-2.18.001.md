---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-01T18:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-18
capability: CAP-18
lifecycle_status: active
introduced: feature-protocol-coverage-F2
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.18.001: `protocols` Subcommand Terminal Catalog Output Lists All KNOWN_PROTOCOLS Entries

## Description

`wirerust protocols` (with optional `--all`, `--supported`, or `--unsupported` filter flag)
prints a terminal table of catalog entries from `KNOWN_PROTOCOLS`. Each row displays the
protocol name, category (`ICS` or `IT`), transport (`TCP`, `UDP`, or `LinkLayer`), canonical
port(s) or "—" for link-layer protocols, and supported status. Link-layer entries (those
with `transport=LinkLayer`, i.e., `port_detectable: false`) are marked with `[L2]` in the
transport column as a display convention. The table includes a fixed footnote warning about
the port-102 four-way collision and a note that link-layer/multicast protocols (`transport=LinkLayer`)
are never detectable in the dynamic gap report. `ProtocolCategory` has exactly two variants:
`ICS` and `IT`. L2-ness is indicated by `transport=LinkLayer ∧ port_detectable:false`, not
by a separate category variant.

## Related BCs

- BC-2.18.002 — parallel (JSON mode output of the same catalog data)
- BC-2.18.003 — depends on (`supported_protocols()` / `unsupported_protocols()` provide the sets used by the filter flags)
- BC-2.18.004 — depends on (partition invariant ensures the filtered sets together equal KNOWN_PROTOCOLS)
- BC-2.12.022 — composes with (CLI dispatch routes `wirerust protocols` to `run_protocols()` which calls this output logic)

## Preconditions

1. `wirerust protocols` is invoked with zero or one filter flag: `--all` (default), `--supported`, or `--unsupported`.
2. `KNOWN_PROTOCOLS` is a non-empty static array (compile-time constant).
3. The `--json` global flag is NOT set (terminal output path; see BC-2.18.002 for JSON mode).

## Postconditions

1. For `--all` (or no filter flag): all entries in `KNOWN_PROTOCOLS` are printed.
2. For `--supported`: only entries where `supported == true` (i.e., `supported_protocols()` result) are printed.
3. For `--unsupported`: only entries where `supported == false` (i.e., `unsupported_protocols()` result) are printed.
4. Each printed row contains: name, category (`ICS` or `IT`), transport indicator (`TCP`, `UDP`, or `[L2]` for `LinkLayer` entries), canonical port(s) (comma-separated u16 values, or `—` for entries with `canonical_ports: &[]`), and a supported indicator (`yes` / `no`). `ProtocolCategory` has only two variants (`ICS`, `IT`); L2-ness is expressed via the transport column, not via a third category.
5. EtherType is printed for link-layer entries (`transport=LinkLayer`) (e.g., `0x88B8 (35000)` for GOOSE); the EtherType column is `—` for non-link-layer (TCP/UDP) entries.
6. The output includes a fixed port-102 collision footnote: `"NOTE: TCP/102 hosts S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 — gap reports on port 102 cannot be attributed to a single protocol."` (exact text may differ in implementation; the semantic requirement is that the four-way collision is identified and named).
7. The output includes a fixed link-layer/multicast note for entries with `port_detectable: false` (i.e., `transport=LinkLayer`): those entries are listed with `port_detectable: false` indicated (e.g., marker in a `[L2]` transport column or footnote), making clear they will never appear in the `CoverageGapsSummary` dynamic gap report.
8. Output rows appear in catalog-declaration order (the order of entries in `KNOWN_PROTOCOLS`). No additional sort is applied at render time.
9. Exit code is 0.

## Invariants

1. Every entry in `KNOWN_PROTOCOLS` is either in the `--supported` output or the `--unsupported` output, but never both (partition invariant per BC-2.18.004).
2. `--all` output row count equals `KNOWN_PROTOCOLS.len()`.
3. The port-102 collision footnote MUST appear whenever any port-102 entry (S7comm, S7comm-plus, IEC 61850 MMS, or ICCP/TASE.2) is in the printed set.
4. The output function is a pure-core pass through `all_protocols()`, `supported_protocols()`, or `unsupported_protocols()` (no I/O beyond terminal rendering).
5. No `Finding` is emitted; no analyzer state is modified.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--supported` filter only | Only 7 supported entries printed (Modbus/TCP, DNP3, EtherNet/IP+CIP, TLS, ARP, DNS, HTTP); no link-layer entries (with `port_detectable: false`) in this set |
| EC-002 | `--unsupported` filter only | All non-supported entries printed including 5 link-layer/multicast entries (`transport=LinkLayer`); port-102 footnote present (S7comm/MMS/etc. are all unsupported) |
| EC-003 | `--all` or no flag | All ~30 entries printed; both supported and unsupported; port-102 footnote present; link-layer entries have `[L2]` transport indicator |
| EC-004 | Link-layer entry (e.g., IEC 61850 GOOSE) in `--unsupported` output | Displayed with category=`ICS`, transport `[L2]`, ports `—`, EtherType `0x88B8 (35000)`; `port_detectable: false` indicated. GOOSE is ICS-category with LinkLayer transport — NOT a third category. |
| EC-005 | BACnet/IP (UDP/47808) in `--unsupported` output | Displayed with transport `UDP`, port `47808`; `port_detectable: true` (it IS dynamically detectable via UDP counter) |
| EC-006 | Port-102 entries (S7comm/S7comm-plus/MMS/ICCP) each appear as distinct rows with port `102` | Four separate rows; footnote about collision present |
| EC-007 | HART-IP (UDP/5094, TCP also documented upstream) in catalog | Listed with `transport=UDP`, `canonical_ports=[5094]`; TCP availability noted in the entry's description string only — the `transport` field carries exactly ONE value per entry |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `wirerust protocols --all` | All ~30 rows; port-102 footnote present; link-layer entries marked with `[L2]` transport indicator | happy-path |
| `wirerust protocols --supported` | 7 rows (Modbus/TCP 502, DNP3 20000, EtherNet/IP+CIP 44818, TLS 443/8443, ARP —, DNS 53, HTTP 80/8080); exit 0 | happy-path |
| `wirerust protocols --unsupported` | All non-supported entries (~23); 5 link-layer entries present (`transport=LinkLayer`); port-102 footnote; exit 0 | happy-path |
| `wirerust protocols` (no flag) | Identical to `--all` | default-behavior |
| GOOSE in `--unsupported` output | Row shows category=ICS, transport=[L2], ports=—, ethertype=0x88B8 (35000), supported=no | link-layer-entry |
| BACnet/IP in `--unsupported` output | Row shows transport=UDP, port=47808, supported=no | udp-entry |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-041 | Catalog oracle cross-check: for each entry in KNOWN_PROTOCOLS, `entry ∈ supported_protocols() ⟺ entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) \|\| entry.name=="ARP"` (covers partition + disjoint invariants backing `--supported`/`--unsupported` filter sets) | proptest: `proptest_vp041_oracle_cross_check` |
| — | `--all` row count == KNOWN_PROTOCOLS.len() | unit: `test_BC_2_18_001_all_row_count` |
| — | `--supported` output matches `supported_protocols()` exactly | unit: `test_BC_2_18_001_supported_filter` |
| — | Port-102 footnote present when port-102 entries are in output | unit: `test_BC_2_18_001_port102_footnote` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 |
| Capability Anchor Justification | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 — this BC defines the terminal output shape of the `protocols` subcommand's static catalog surface, which is the primary user-visible expression of the Protocol Coverage Catalog capability |
| L2 Domain Invariants | None directly (pure-core output; no domain invariants from the brownfield spec apply) |
| Architecture Module | SS-18 (src/protocols.rs C-26; run_protocols() in src/main.rs) |
| ADR | ADR-012 Decision 1 (hand-curated static array), Decision 3 (port-102 + L2/multicast caveats), Decision 4 (ICS+IT scope), Decision 7 (category tagging) |
| Stories | TBD (F3 story decomposition) |

## Architecture Anchors

- `src/protocols.rs` — `KNOWN_PROTOCOLS: &[KnownProtocol]` static array (C-26); `all_protocols()`, `supported_protocols()`, `unsupported_protocols()` pure-core functions
- `src/protocols.rs` — `KnownProtocol` struct: `name`, `category`, `transport`, `canonical_ports`, `ethertype`, `port_detectable`, `description`
- `src/main.rs` — `run_protocols(filter, json)` effectful CLI dispatch function; consumes pure-core catalog functions and renders terminal table

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

- VP-041 — `proptest_vp041_oracle_cross_check` — backs partition and disjoint invariants that underpin the filter-flag output sets

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | terminal/stdout write (effectful shell layer only; catalog lookup is pure core) |
| **Global state access** | read-only (`KNOWN_PROTOCOLS` is a `&'static` constant) |
| **Deterministic** | yes (same filter always produces same output) |
| **Thread safety** | yes (read-only static data) |
| **Overall classification** | pure (catalog functions); effectful (terminal rendering in run_protocols) |
