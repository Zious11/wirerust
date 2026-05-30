---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.019: Summary::ingest Derives Service Name from app_protocol_hint

## Description

During `Summary::ingest`, if `packet.app_protocol_hint()` returns `Some(svc)`, the services
counter for that service name is incremented. `app_protocol_hint` infers a service name
purely from the TCP/UDP port tuple (e.g., port 53 -> "DNS", port 80 -> "HTTP", port 443 ->
"TLS/SSL"). Services are NOT classified by content; this is a port-based heuristic. A flow
on a non-standard port (e.g., HTTP on 8080) contributes nothing to the service map.

## Preconditions

1. `Summary::ingest` is called with a `ParsedPacket`.
2. `packet.app_protocol_hint()` may return Some or None.

## Postconditions

1. When `app_protocol_hint()` returns `Some(svc)`: `services[svc]` incremented by 1.
2. When `app_protocol_hint()` returns `None`: `services` map unchanged.
3. The service name is the exact string returned by `app_protocol_hint()`.

## Invariants

1. `services` is a `HashMap<String, u64>`; new service names are inserted with count 1.
2. `app_protocol_hint` is port-based; it does NOT consult the stream dispatcher (LESSON-P3.01).
3. The two protocol-attribution systems (services here vs. dispatcher routes) can disagree
   for non-standard-port flows.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TCP packet on port 80 | services["HTTP"]=1 (or similar) |
| EC-002 | TCP packet on port 9999 | app_protocol_hint()=None; services unchanged |
| EC-003 | UDP packet on port 53 | services["DNS"]=1 |
| EC-004 | Multiple packets same service | services[svc] increments per packet |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Packet with dst_port=443 | services has TLS/SSL or similar | happy-path |
| Packet with dst_port=9999 | services is empty or unchanged | happy-path |
| 2 packets with dst_port=80 | services["HTTP"]=2 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Service counter incremented from app_protocol_hint | unit: test_summary_service_detection |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md -- Summary::ingest's service-counter step (summary.rs:65-67) is part of the Summary accumulation described in CAP-12; summary.rs (C-17) is listed under CAP-12 sources, and the service hint is a per-packet statistic accumulated in the entry-layer packet loop |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence -- note: this service attribution is port-based, NOT content-first; the two can disagree per LESSON-P3.01) |
| Architecture Module | SS-12 (summary.rs, C-17) |
| Stories | STORY-090 |
| Origin BC | BC-SUM-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.12.018 -- composes with (service counting is the second phase of ingest after basic counters)
- BC-2.02.012 -- depends on (app_protocol_hint is defined on ParsedPacket in decoder.rs)

## Architecture Anchors

- `src/summary.rs:65-67` -- app_protocol_hint() call and service counter increment
- `tests/summary_tests.rs` -- test_summary_service_detection

---

### Brownfield-Specific Sections

#### Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/summary.rs:65-67` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

#### Evidence Types Used

- **assertion**: test_summary_service_detection

#### Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (mutates &mut self) |
| **Overall classification** | pure (in-memory state mutation) |

#### Refactoring Notes

No refactoring needed. The port-based/content-based divergence is documented as a known
limitation (LESSON-P3.01) and is not a bug.
