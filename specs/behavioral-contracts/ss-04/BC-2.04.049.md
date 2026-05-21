---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/flow.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
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

# BC-2.04.049: FlowKey::Display Uses U+2192 Arrow (Not ASCII ->)

## Description

`FlowKey`'s `Display` implementation formats as
`"lower_ip:lower_port -> upper_ip:upper_port"` using the Unicode RIGHT ARROW character
U+2192 (`->` in the source code is actually `\u{2192}`). Per pass-3 R4 analysis, this is a
hidden output-encoding contract: consumers that parse finding summaries with a regex
expecting ASCII `->` will silently fail to match, because the actual separator is the
3-byte UTF-8 encoding of U+2192 (0xE2 0x86 0x92).

## Preconditions

1. A `FlowKey` has been constructed with valid IP addresses and ports.

## Postconditions

1. `format!("{}", flow_key)` produces a string of the form
   `"A.B.C.D:P1 -> E.F.G.H:P2"` where the arrow is U+2192 (Unicode RIGHT ARROW,
   rendered as `->` in the source but stored as `->` Unicode character).
2. The lower (canonically-ordered) endpoint appears on the left; the upper on the right.
3. The IP addresses are formatted by `IpAddr::fmt` (standard Rust display: dotted-decimal
   for IPv4, colon-separated for IPv6).
4. The ports are decimal integers.

## Invariants

1. The arrow character is ALWAYS U+2192 (`\u{2192}`, UTF-8: E2 86 92), never ASCII `->`.
2. The canonical ordering (lower_ip:lower_port on the left) is guaranteed by `FlowKey::new`
   (INV-1).
3. Finding summaries that embed the flow key display (e.g., "Conflicting TCP segment overlap
   on flow A.B.C.D:P1 -> E.F.G.H:P2") contain U+2192, not ASCII `->`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | IPv4 flow key | "1.2.3.4:80 -> 5.6.7.8:443" with U+2192 |
| EC-002 | IPv6 flow key | "::1:80 -> ::2:443" with U+2192 (IpAddr::fmt does NOT add RFC-3986 brackets; IPv6 addresses render bracket-free in this format) |
| EC-003 | Same IP, different ports | "1.2.3.4:22 -> 1.2.3.4:50000" |
| EC-004 | Grep pipeline using ASCII -> to match finding summaries | Will NOT match; U+2192 required |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| FlowKey{lower=1.2.3.4:80, upper=5.6.7.8:443} | "1.2.3.4:80 -> 5.6.7.8:443" (U+2192) | happy-path |
| Regex b"->".contains(format!("{}", key)) | NO MATCH (wrong encoding) | edge-case |
| Regex "\u{2192}".contains(format!("{}", key)) | MATCH | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Display output contains U+2192, not ASCII -> | unit: dedicated test asserting exact UTF-8 bytes |
| — | Display output format matches "ip:port ARROW ip:port" pattern | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- FlowKey display format is the human-readable output contract for TCP flow identification |
| L2 Domain Invariants | INV-1 (FlowKey canonical ordering -- the Display reflects the canonical left-right ordering) |
| Architecture Module | SS-04 (reassembly/flow.rs:66-74, C-7) |
| Stories | STORY-011 |
| Origin BC | BC-RAS-049 (pass-3 ingestion corpus, MEDIUM confidence) |

## Related BCs

- BC-2.04.003 -- depends on (FlowKey canonical ordering is what determines left/right in the display)
- BC-2.04.018 -- related to (ConflictingOverlap finding summary embeds FlowKey display)

## Architecture Anchors

- `src/reassembly/flow.rs:66-74` -- FlowKey Display implementation with U+2192

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/flow.rs:66-74` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **documentation**: pass-3-R4 analysis identifying U+2192 vs ASCII -> as a hidden output-encoding contract
- **guard clause**: `write!(f, "{}:{} \u{2192} {}:{}", ...)` at flow.rs:70 (the -> in source is the Unicode arrow character; the `write!` macro spans flow.rs:68-72)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | writes to formatter (pure Display) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure |
