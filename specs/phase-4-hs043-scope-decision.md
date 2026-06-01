---
document_type: scope-decision
level: ops
version: "1.0"
status: active
producer: product-owner
timestamp: 2026-06-01T00:00:00Z
phase: 4
finding: HS-043
governing_bc: BC-2.04.013
traces_to: .factory/research/holdout-finding-triage-2026-06-01.md
---

# Scope Decision: HS-043 — Idle-Flow Expiry Not Wired Into Production

## Finding Summary

`expire_flows` (src/reassembly/mod.rs:593-609) exists, is correct, and is well-tested in
isolation. However, it is **never called by the production pipeline**. The CLI loop
(src/main.rs:154-176) calls only `process_packet` + `finalize`. No call site for
`expire_flows` exists anywhere under `src/`. The only callers are under `tests/`.

Consequence: `flows_expired` is structurally always 0 for every CLI invocation. The
advertised idle-flow memory bound (BC-2.04.013 capability anchor: "idle flow expiry is
required to bound memory use in long-running captures") is absent from the shipping binary.
The LRU/max_flows eviction path (BC-2.04.015) still bounds the flow table, so this is not
an immediate out-of-memory risk, but the timeout-based protection layer is dead code in the
production path.

---

## Decision 1 — Is idle-flow expiry a v0.1.0 REQUIREMENT?

**RULING: YES — this is a v0.1.0 requirement, and `expire_flows` MUST be wired into the
production processing path.**

**Basis:** BC-2.04.013 is `lifecycle_status: active`, `introduced: v0.1.0-brownfield`, with
story STORY-019 already delivered. Its Description states:

> "idle flow expiry is required to bound memory use in long-running captures"

The Capability Anchor Justification in the Traceability section repeats:

> "CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md --
>  idle flow expiry is required to bound memory use in long-running captures"

A BC with status `active` in v0.1.0 whose capability anchor explicitly calls out the
memory-bound requirement cannot remain unwired. STORY-019 was marked delivered on the
strength of unit tests that call `expire_flows` directly — those tests did not expose the
wiring gap. The BC is not satisfied until `expire_flows` executes as part of the per-packet
or per-loop production path.

---

## Decision 2 — Should a `--flow-timeout <secs>` CLI flag be added?

**RULING: YES — add `--flow-timeout <secs>` as a `ReassemblyConfig` knob.**

**Rationale:**

1. **Observability.** Without a tunable timeout, black-box verification of HS-043
   (flows_expired >= 1 through the CLI) requires a pcap with >300s idle gaps, which is
   impractical in a test harness. With `--flow-timeout 5`, a synthetic pcap with a 6-second
   idle gap exercises the full code path end-to-end.

2. **Consistency.** The CLI already exposes `--reassembly-depth`, `--reassembly-memcap`,
   `--overlap-threshold`, `--small-segment-threshold`, and related knobs (src/cli.rs:42-110).
   A timeout knob is architecturally coherent with that pattern.

3. **Operator utility.** Long-running captures of high-bandwidth traffic benefit from a
   tighter timeout (e.g., 60s) to aggressively reclaim memory. Captures of slow IoT
   protocols may need 3600s. Hardcoding 300s is operationally inflexible.

4. **LESSON-P1.04 alignment.** The project convention "no unwired flags" cuts both ways:
   adding a flag is the correct fix for a tested-but-unexposed capability.

**Flag specification:**

| Property | Value |
|----------|-------|
| Flag name | `--flow-timeout <secs>` |
| Type | `u64` (seconds) |
| Default | `300` (preserves current behavior) |
| Minimum | `1` |
| Config field | `ReassemblyConfig::flow_timeout_secs` (src/reassembly/config.rs:121) |
| Help text | `Idle flow timeout in seconds. Flows silent for longer than this value are expired and removed from the flow table. Default: 300.` |
| JSON output | `flows_expired` already surfaces in the reassembly summary detail map (mod.rs:720); no additional output change needed. |

---

## Decision 3 — BC-2.04.013 Caller-Note Clarification

**RULING: YES — clarify the BC's caller note so it explicitly requires `expire_flows` to be
called from the per-packet processing path.**

The existing Description in BC-2.04.013 says:

> "The caller is responsible for passing `current_time` (typically the timestamp of the
>  packet being processed)."

This is correct about the argument semantics but silent on **who the caller is** and
**when the call must happen**. This gap allowed STORY-019 to be delivered with tests that
call `expire_flows` directly (as the "caller"), satisfying the letter of the BC while
missing the production wiring requirement entirely.

The BC has been updated (see version bump below) to add an explicit caller obligation
stating that the production processing path — specifically the per-packet loop — is the
required call site, and that direct test-only invocations do not satisfy the production
wiring requirement.

---

## Acceptance Specification for the Implementer

### What must be wired

`expire_flows` must be called from the per-packet production path with the packet's
`timestamp_secs` as `current_time`. The preferred call site is at the top of
`process_packet` (src/reassembly/mod.rs, near line 140, before or after updating
`flow.last_seen` for the arriving packet):

```rust
// At the start of process_packet, or in main.rs immediately before/after
// the process_packet call:
self.expire_flows(timestamp_secs, handler);
```

Alternatively, the wiring may live in `src/main.rs` in the per-packet loop body
(lines 154-176), immediately before `reasm.process_packet(...)`. Either site is
acceptable provided the call receives the current packet's timestamp.

Performance note: `expire_flows` iterates all flows (O(n)). For high-throughput captures
this may be costly per-packet. An acceptable optimization is to gate the call — e.g., every
100 packets, or whenever `current_time` has advanced by at least 1 second since the last
sweep — but the gate must not skip the call entirely, and it must preserve the invariant
that a flow idle for exactly `flow_timeout_secs` is NOT expired while a flow idle for
`flow_timeout_secs + 1` is. The gating strategy must be reviewed against the reassembly
perf design (docs/superpowers/specs/2026-04-06-reassembly-perf-design.md).

If `--flow-timeout` is added: wire `ReassemblyConfig::flow_timeout_secs` from the parsed
CLI arg before constructing `TcpReassembler`.

### What the test must prove

1. **Through-the-front-door integration test** (not a direct `expire_flows` call):
   - Build a `TcpReassembler` with `flow_timeout_secs = 5` (or via `--flow-timeout 5` CLI).
   - Feed it packets establishing Flow B at t=0.
   - Feed it a packet for Flow A at t=6 (> 5s after Flow B's last packet).
   - After `process_packet` returns, assert `stats.flows_expired >= 1`.
   - Assert Flow B is no longer in the flow table (or that the flow count decreased).
   - Assert Flow A is still tracked (it is within the timeout window at t=6).
   - This test exercises the entire wiring from CLI arg → config → `process_packet` →
     `expire_flows` without calling `expire_flows` directly.

2. **Boundary condition** (strict `>`):
   - Flow idle for exactly `flow_timeout_secs` — assert NOT expired.
   - Flow idle for `flow_timeout_secs + 1` — assert expired.

3. **CLI black-box test** (if `--flow-timeout` flag is added):
   - Run `wirerust analyze <synthetic-pcap> --flow-timeout 5 --output-format json`
   - Assert `flows_expired >= 1` in the JSON output.
   - The pcap must have at least one flow idle for > 5s.

### CLI surface (if flag added)

- `--flow-timeout 0` must be rejected with a clear error message (minimum 1s).
- `--flow-timeout` must appear in `--help` output alongside the other `--reassembly-*` knobs.

---

## Version Bump Record

| Artifact | Old Version | New Version | Change |
|----------|-------------|-------------|--------|
| BC-2.04.013 | 1.4 | 1.5 | Added explicit caller obligation (PC0) clarifying production wiring requirement |
| HS-043 | unchanged | — | No rubric change; the scenario is correct as written; the fix is to the production code and BC |

---

## Risk Note

The existing unit tests in `tests/reassembly_engine_tests.rs` (test_BC_2_04_013_*) are
**not regression-sufficient** for the wiring requirement — they will continue to pass even
if `expire_flows` is never called in production. The new integration test described in the
acceptance spec above is the load-bearing regression guard. It must be added as part of
the fix story, not deferred.

Recurring theme (from triage report): lifecycle methods with direct-call test coverage can
mask wiring gaps. After this fix, it is worth a test-strategy note that any future engine
lifecycle method (expire, flush, purge, etc.) must have at least one integration test that
exercises it through the public `process_packet` or CLI surface.
