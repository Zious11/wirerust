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

# BC-2.18.002: `protocols` Subcommand JSON Mode Outputs Structured Protocol Array

## Description

When `wirerust protocols` is invoked with the global `--json` flag, the output is a
JSON object containing a `"protocols"` array. Each element represents one `KnownProtocol`
entry (filtered per `--all`, `--supported`, or `--unsupported` flag). The JSON schema is
stable and machine-parseable, enabling downstream tooling to consume the coverage catalog
programmatically. The same port-102 collision and L2/multicast caveats from BC-2.18.001 are
represented in the structured data via the `port_detectable` and `ethertype` fields rather
than as free-text footnotes.

## Related BCs

- BC-2.18.001 — parallel (terminal output mode of the same catalog data)
- BC-2.18.003 — depends on (`supported_protocols()` / `unsupported_protocols()` back the filter sets)
- BC-2.12.022 — composes with (CLI dispatch routes `wirerust protocols --json` to `run_protocols(json=true)`)

## Preconditions

1. `wirerust protocols` is invoked with the global `--json` flag.
2. An optional filter flag is present: `--all` (default), `--supported`, or `--unsupported`.
3. `KNOWN_PROTOCOLS` is a non-empty static array.

## Postconditions

1. Output is a single JSON object on stdout.
2. The object contains a `"protocols"` array whose elements correspond 1-to-1 with the filtered catalog set (same semantics as BC-2.18.001 Postconditions 1–3).
3. Each element is a JSON object with these fields:
   - `"name"`: string — protocol name (e.g., `"Modbus/TCP"`, `"BACnet/IP"`, `"IEC 61850 GOOSE"`)
   - `"category"`: string — one of `"ICS"`, `"IT"` (exactly two variants; L2-ness is expressed via `transport=LinkLayer` and `port_detectable=false`, NOT via a third category variant)
   - `"transport"`: string — one of `"TCP"`, `"UDP"`, `"LinkLayer"`
   - `"canonical_ports"`: array of integers — port numbers (empty array `[]` for link-layer entries with no TCP/UDP port)
   - `"ethertype"`: integer or null — EtherType value (e.g., `35000` for 0x88B8 GOOSE) for link-layer entries; `null` for TCP/UDP entries
   - `"port_detectable"`: boolean — `true` for TCP/UDP entries; `false` for link-layer/multicast entries (those with `transport=LinkLayer`)
   - `"supported"`: boolean — `true` if the protocol has an active wirerust dissector; `false` otherwise
4. The `"protocols"` array elements appear in catalog-declaration order (matching the order of entries in `KNOWN_PROTOCOLS`). No additional sort is applied.
5. Exit code is 0.
6. Output is valid JSON (parseable by `jq`).

## Invariants

1. The `"protocols"` array length equals the number of entries in the filter set (same as BC-2.18.001 row count for the same filter).
2. `"port_detectable": false` implies `"canonical_ports": []` (a non-detectable entry has no canonical TCP/UDP ports). The converse does NOT hold in general: ARP has `"canonical_ports": []` AND `"port_detectable": false` AND `"ethertype": null` (ARP is handled via `DecodedFrame::Arp` outside the dispatcher, not by port detection). Therefore the original "iff" is weakened: `port_detectable: false` ⇒ `canonical_ports: []`, but `canonical_ports: []` does NOT imply `ethertype` non-null (ARP is the counterexample).
3. No two elements have the same `"name"` value (each KNOWN_PROTOCOLS entry has a unique name).
4. `"supported": true` entries all have non-empty `"canonical_ports"` arrays (except ARP, which has `"canonical_ports": []` and is a special-case L2-handled entry; see BC-2.18.003).
5. The JSON schema does not change based on the filter flag; only the number of elements changes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--json --supported` | `"protocols"` array contains only 7 supported entries; all have `"supported": true` |
| EC-002 | `--json --unsupported` | `"protocols"` array contains ~23 unsupported entries; all have `"supported": false`; link-layer entries (`transport=LinkLayer`) have `"port_detectable": false`; entries with an EtherType have `"ethertype"` non-null (ARP is the exception: `transport=LinkLayer`, `ethertype=null`) |
| EC-003 | GOOSE entry in JSON | `{"name": "IEC 61850 GOOSE", "category": "ICS", "transport": "LinkLayer", "canonical_ports": [], "ethertype": 35000, "port_detectable": false, "supported": false}` (0x88B8 = 35000 decimal; ICS category; NOT a separate "L2" category) |
| EC-004 | BACnet/IP entry in JSON | `{"name": "BACnet/IP", "category": "ICS", "transport": "UDP", "canonical_ports": [47808], "ethertype": null, "port_detectable": true, "supported": false}` |
| EC-005 | Modbus/TCP entry in JSON | `{"name": "Modbus/TCP", "category": "ICS", "transport": "TCP", "canonical_ports": [502], "ethertype": null, "port_detectable": true, "supported": true}` |
| EC-006 | TLS entry with two ports | `{"canonical_ports": [443, 8443], ...}` (array with two elements) |
| EC-007 | ARP entry | `{"name": "ARP", "category": "IT", "transport": "LinkLayer", "canonical_ports": [], "ethertype": null, "port_detectable": false, "supported": true}` — special case: L2-handled but supported; port_detectable is false (not a TCP/UDP port protocol) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `wirerust protocols --json` | Valid JSON with `"protocols"` array of ~30 elements | happy-path |
| `wirerust protocols --json --supported` | `"protocols"` array length == 7 | filter |
| GOOSE JSON element | `ethertype` is non-null integer; `port_detectable` is false; `canonical_ports` is `[]` | L2-schema |
| BACnet/IP JSON element | `transport` is `"UDP"`, `canonical_ports` is `[47808]`, `port_detectable` is true | udp-schema |
| Modbus/TCP JSON element | `transport` is `"TCP"`, `canonical_ports` is `[502]`, `supported` is true | supported-schema |
| `jq '.protocols | length'` on `--json --all` output | ~30 (equals KNOWN_PROTOCOLS.len()) | parseable |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-041 | Catalog oracle cross-check (same backing sets as BC-2.18.001 filter logic; oracle: `entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p)) \|\| entry.name=="ARP"`) | proptest: `proptest_vp041_oracle_cross_check` |
| — | JSON output is valid (jq parseable); `"protocols"` array length equals filter set size | unit: `test_BC_2_18_002_json_schema_valid` |
| — | `port_detectable: false` entries have `canonical_ports: []` | unit: `test_BC_2_18_002_l2_entries_no_ports` |
| — | `supported: true` entries match `supported_protocols()` set | unit: `test_BC_2_18_002_supported_flag_matches_function` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 |
| Capability Anchor Justification | CAP-18 ("Protocol Coverage Catalog") per domain/capabilities/cap-18-protocol-coverage-catalog.md §CAP-18 — this BC defines the machine-readable JSON output schema for the `protocols` subcommand, enabling programmatic consumption of the Protocol Coverage Catalog |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-18 (src/protocols.rs C-26); SS-12 (src/main.rs `run_protocols()`) |
| ADR | ADR-012 Decision 1 (hand-curated static array), Decision 3 (port-102 + L2/multicast caveats encoded via `port_detectable`/`ethertype` fields), Decision 7 (category tagging) |
| Stories | TBD (F3 story decomposition) |

## Architecture Anchors

- `src/protocols.rs` — `KNOWN_PROTOCOLS`, `KnownProtocol` struct, `all_protocols()` / `supported_protocols()` / `unsupported_protocols()` (pure-core catalog access)
- `src/main.rs` — `run_protocols(filter, json)` — when `json=true`, renders JSON using `serde_json` or equivalent; applies same filter logic as terminal path

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

- VP-041 — `proptest_vp041_oracle_cross_check` — backs partition and disjoint invariants that underpin the filter-flag output sets (same invariants apply to JSON output count)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | stdout write (effectful shell layer); catalog lookup is pure core |
| **Global state access** | read-only (`KNOWN_PROTOCOLS` is `&'static`) |
| **Deterministic** | yes |
| **Thread safety** | yes (read-only static data) |
| **Overall classification** | pure (catalog functions); effectful (JSON rendering and stdout in run_protocols) |
