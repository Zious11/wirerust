---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/summary.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.021: Summary Serializes with total_packets/total_bytes/skipped_packets Fields

## Description

`Summary` derives `serde::Serialize`, making it directly serializable by `JsonReporter`.
The public fields `total_packets: u64`, `total_bytes: u64`, and `skipped_packets: u64`
serialize under their field names. The private `hosts: HashSet<IpAddr>`,
`protocols: HashMap<Protocol, u64>`, and `services: HashMap<String, u64>` fields do NOT
serialize directly via the derive; instead `JsonReporter` accesses them via the public
accessor methods (`unique_hosts()`, `protocol_counts()`, `service_counts()`) and constructs
the JSON output manually using `serde_json::json!`.

## Preconditions

1. `Summary` is passed to `JsonReporter::render`.
2. `Summary` has been populated via `ingest` calls.

## Postconditions

1. JSON `"summary"` object contains `"total_packets"` as u64 integer.
2. JSON `"summary"` object contains `"total_bytes"` as u64 integer.
3. JSON `"summary"` object contains `"skipped_packets"` as u64 integer.
4. JSON `"summary"` object contains `"unique_hosts"` as array of IP strings (via accessor).
5. JSON `"summary"` object contains `"protocols"` as object with Protocol-name keys (via accessor).
6. JSON `"summary"` object contains `"services"` as object with service-name keys (via accessor).
7. Protocol keys are serialized via `{k:?}` Debug format (e.g., "Tcp", "Udp").

## Invariants

1. `#[derive(Serialize)]` is on the `Summary` struct, but `JsonReporter` uses `serde_json::json!`
   for the complete summary object shape rather than directly serializing the struct. This means
   private fields are accessed via their public methods.
2. The `protocols` HashMap keys use Debug formatting (`{k:?}`) to produce string keys for
   JSON because `Protocol` does not implement `Serialize` as a string type.
3. The `services` HashMap keys are already `String`; they serialize directly as string keys.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty summary (zero packets) | All counts 0; hosts=[], protocols={}, services={} |
| EC-002 | Protocol enum variant "Icmp" | JSON key is "Icmp" (Debug format) |
| EC-003 | Service with special chars in name | Serialized as JSON string key |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Summary after 2 packets, 0 skipped | JSON has total_packets=2, skipped_packets=0 | happy-path |
| Summary with Protocol::Tcp x3 | "protocols": {"Tcp": 3} | happy-path |
| Summary with services["HTTP"]=2 | "services": {"HTTP": 2} | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Summary fields serialize correctly | unit: test_json_reporter_includes_skipped_packets, test_full_pipeline |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per capabilities.md §CAP-12 -- Summary (summary.rs / C-17) is listed under CAP-12 sources; its public serializable fields (total_packets, total_bytes, skipped_packets) are owned by the Summary accumulation step that CAP-12 orchestrates; the serde derive is on the Summary struct in C-17, not in any reporter |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (summary.rs, C-17; reporter/json.rs, C-19) |
| Stories | S-TBD |
| Origin BC | BC-SUM-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.11.001 -- depends on (JsonReporter's summary output shape is built from Summary here)
- BC-2.12.018 -- depends on (ingest populates the fields that serialize here)
- BC-2.12.020 -- depends on (unique_hosts() accessor provides the serialized host list)

## Architecture Anchors

- `src/summary.rs:18-19` -- #[derive(Serialize)] on Summary
- `src/reporter/json.rs:47-57` -- serde_json::json! block constructing the summary object

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/summary.rs:18-19` and `src/reporter/json.rs:47-57` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **type constraint**: #[derive(Serialize)] on Summary
- **assertion**: test_json_reporter_includes_skipped_packets, test_full_pipeline (integration)

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes (BTreeMap used for protocol/service ordering in JsonReporter) |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |

#### Refactoring Notes

No refactoring needed.
