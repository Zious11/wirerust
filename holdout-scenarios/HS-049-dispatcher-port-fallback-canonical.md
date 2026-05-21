---
document_type: holdout-scenario
level: ops
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-031.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.003.md
input-hash: "[md5-pending]"
traces_to: .factory/stories/STORY-031.md
id: "HS-049"
category: "behavioral-subtleties"
must_pass: "true"
priority: "must-pass"
epic_id: "E-3"
behavioral_contracts:
  - BC-2.05.003
lifecycle_status: active
introduced: v0.1.0-greenfield-spec
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# Holdout Scenario: Port Fallback Uses Canonical Port Ordering for Non-Standard Source Ports

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

## Scenario

A TCP connection to an HTTPS server arrives where the client uses a high ephemeral
source port (e.g., src=54321, dst=443). The content is short enough that TLS content
detection doesn't fire on the first chunk. Port-based fallback must correctly recognize
port 443 from the canonically-ordered port pair, not just from the destination port.

1. A pcap contains a flow from `192.168.1.100:54321` to `10.0.0.1:443`. The first
   data chunk is a single byte (insufficient for TLS content check), and the byte
   is not an HTTP method prefix.
2. The user runs: `wirerust analyze <short-first-chunk-pcap> --output-format json`
3. Port fallback fires: `lower_port() = 443` is in the TLS port list, so the flow
   is classified as TLS.
4. Subsequent chunks (the full TLS ClientHello) are forwarded to the TLS analyzer.
5. TLS-level findings or statistics appear in the output.

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.05.003 | postcondition 1: port 443 (TLS) fallback routes to TLS | Flow correctly identified as TLS via port |
| BC-2.05.003 | invariant 1-2: TLS ports checked before HTTP ports; lower_port/upper_port canonical ordering | Port 443 found via canonical ordering regardless of source vs destination |

## Verification Approach

```bash
wirerust analyze <short-first-chunk-443.pcap> --output-format json
```

Verify:
- The flow is classified as TLS (TLS statistics or JA3 fingerprints present).
- No HTTP analysis runs for this flow.
- The classification persists (immutable cache) even after a full TLS ClientHello
  arrives on the next chunk.

For the mirrored case — a flow from `443:high-port` with the server initiating —
verify the same canonical port ordering applies and port 443 is still recognized.

## Evaluation Rubric

- **Functional correctness** (weight: 0.5): Port 443 correctly triggers TLS fallback
  regardless of which endpoint is source vs destination.
- **Edge case handling** (weight: 0.2): Port 8443 also correctly triggers TLS fallback
  (another TLS port); port 8080 triggers HTTP fallback.
- **Error quality** (weight: 0.1): No crash from short first-chunk flows.
- **Performance** (weight: 0.1): Port fallback is O(1); no quadratic lookup.
- **Data integrity** (weight: 0.1): Correct analyzer is called for the flow after
  classification is cached.

## Edge Conditions

- Port 8443 on the upper port: canonical ordering finds it; TLS classified.
- Port 80 and port 443 in the same flow key: TLS port wins (TLS ports checked first).
- Unknown port (e.g., 9999): no fallback match; result is DispatchTarget::None.
- First chunk of length exactly 4 (one byte short for TLS check): TLS content
  check skipped; HTTP check runs; if no HTTP prefix, port fallback runs.

## Failure Guidance

"HOLDOUT LOW: HS-049 (satisfaction: 0.XX) — port-based fallback classification
failed when the well-known port (443/8443/80/8080) was the source port rather
than the destination port; canonical port ordering was not applied correctly."
