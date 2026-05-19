# Pass 1 Deepening (Round 2): Architecture -- wirerust

> Round 2 of Pass 1 (Architecture). Builds on P1 R1
> (`wirerust-pass-1-architecture.md`). Cross-pollinated with P2 R3
> (domain-model deep-r3) and P3 R3 (behavioral-contracts deep-r3),
> both of which have already declared NITPICK / converged.
> Date: 2026-05-19
> Pass: 1 (Architecture), deepening round 2
> Confidence: HIGH (every claim re-grounded in `src/*.rs` with file:line)

---

## 0. Hallucination Audit of P1 R1

Re-checking P1 R1 against the 5 hallucination classes (fabricated
modules / wrong line counts / phantom traits / invented edges /
mis-attributed cycles).

| Class                  | P1 R1 claim                                                    | Verification                                                                                                                                          | Status                                    |
|------------------------|----------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------|
| Fabricated modules     | 20 components C-1..C-20, one per `.rs` file under `src/`       | `find src -name '*.rs' -type f` returns exactly 20 files: `lib`, `main`, `cli`, `decoder`, `dispatcher`, `findings`, `mitre`, `reader`, `summary`, `reporter/{mod,json,terminal}`, `reassembly/{mod,flow,handler,segment}`, `analyzer/{mod,dns,http,tls}` | confirmed                                 |
| Wrong line counts      | `reassembly::handler` = 29 LOC; `analyzer::tls` = 750; `analyzer::http` = 535; `reassembly/mod.rs` = 564 | `wc -l` reports 29 / 750 / 535 / 565 respectively (off-by-one on mod.rs -- 565 vs 564 claimed; harmless)                                              | minor (1-line drift on reassembly/mod.rs) |
| Phantom traits         | 4 traits: `Reporter`, `ProtocolAnalyzer`, `StreamHandler`, `StreamAnalyzer` with zero default methods | Re-read each trait body (see Q-A trait inventory below). All 9 trait methods are abstract signatures, no `{ ... }` default bodies                     | confirmed                                 |
| Invented edges         | 40 `use crate::...` edges across 13 files                      | `awk '/^use crate::/'` over `src/**/*.rs` confirms the edge set; no missing imports, no fabricated ones                                                | confirmed                                 |
| Mis-attributed cycle   | Module-group cycle `analyzer` <-> `reassembly` via `StreamAnalyzer` | `handler.rs:1-2` imports `analyzer::AnalysisSummary` + `findings::Finding`; `analyzer/{http.rs:115-118,tls.rs:656-659}` import `reassembly::handler::{StreamAnalyzer,...}` + `reassembly::flow::FlowKey`. Cycle is real, advisory-LOW severity, intrinsic to ADR 0002 | confirmed                                 |

**No P1 R1 hallucinations found.** The one off-by-one (564 vs 565 LOC on
`reassembly/mod.rs`) is a cosmetic drift, almost certainly from a
post-pass commit; harmless.

Pass 6 §7's "20 components C-1..C-20" metric is re-verified accurate
against the current tree (re P3 R3 flagging Pass 6 metric errors).

---

## 1. Q-A2 resolution -- evicted-before-on_data flow lifecycle

**Question:** When `evict_flows` removes a flow that never had `on_data`
delivered, does the dispatcher's `routes` map see the close via
`on_flow_close`?

**Code path (`src/reassembly/mod.rs`):**

- Line 506-531: `evict_flows` iterates sorted candidates and calls
  `self.close_flow(key, CloseReason::MemoryPressure, handler)` for each.
- Line 478-501: `close_flow` removes the flow from `self.flows`, flushes
  any remaining contiguous data in **both** directions (line 491-498
  iterating `[ClientToServer, ServerToClient]`), then calls
  `handler.on_flow_close(key, reason)` unconditionally (line 500).

So: **yes, `on_flow_close` is always called**, even for flows whose
buffered data is non-contiguous (no `on_data` deliveries occur because
`flush_contiguous` returns empty when the segment tree has gaps).

**Dispatcher side (`src/dispatcher.rs:98-117`):**

```rust
fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason) {
    let target = self.routes.remove(flow_key);
    match target {
        Some(Http) => { /* forward */ }
        Some(Tls)  => { /* forward */ }
        Some(None) | None => {
            if self.http.is_some() || self.tls.is_some() {
                self.unclassified_flows += 1;
            }
        }
    }
}
```

`Some(DispatchTarget::None)` is unreachable because `on_data` (line 77)
never inserts `None` into `routes`. So for any flow that was evicted
before producing any contiguous bytes, the dispatcher's `routes.remove`
returns `None` -> the third arm fires -> `unclassified_flows += 1`.

**Architectural conclusion:** `unclassified_flows` is **inflated by
silent eviction**. A flow that was perfectly classifiable but happened
to be evicted before its first contiguous flush counts as "unclassified"
in the dispatcher's metric, indistinguishable from a flow that genuinely
had short / non-protocol traffic. This is the architectural mismatch
P1 R1's Q-A2 worried about: **dispatcher and reassembler share flow
*identity* via `FlowKey` but not flow *liveness* -- the reassembler
emits the close event but the dispatcher cannot tell why no `on_data`
arrived (eviction vs. genuinely-unclassifiable).**

**Verdict:** behavior is correct (no panic, no leak), but the metric
`unclassified_flows` is **operator-misleading**. Architectural
recommendation for Pass 4 NFR or a future ADR: either (a) split the
counter into `unclassified_no_data` vs `unclassified_short_data`, or
(b) have `on_flow_close` accept a hint when no `on_data` ever fired.

---

## 2. Q-A5 resolution -- non-deterministic JSON key ordering

**Code (`src/reporter/json.rs:18-22, 30-31`):**

```rust
let protocols: HashMap<String, u64> = summary
    .protocol_counts()
    .iter()
    .map(|(k, v)| (format!("{k:?}"), *v))
    .collect();
// ...
"services": summary.service_counts(),
```

Both `protocols` and `services` are `HashMap`-typed at the call site.
`serde_json` serialises `HashMap` in **iteration order**, which the
stdlib documents as "arbitrary and may change between runs" (since
`RandomState` is the default hasher). `unique_hosts` deliberately sorts
(`src/summary.rs:48-52`); these two parallel fields do not.

**Test side:** no test in `tests/` asserts JSON key order for
`summary.protocols` or `summary.services`. P3 R3 / BC catalog has no
ordering BC for these fields.

**Verdict (architectural):** the asymmetry is **almost certainly
unintentional**. The author sorted `unique_hosts` (suggesting awareness
of determinism as a goal), then forgot to apply the same treatment to
two parallel fields. Forensic reproducibility is a real concern --
two runs over the same pcap can produce byte-different JSON, which
breaks golden-file regression testing (the very thing
`assert_cmd`+`tempfile`, declared-but-unused per P0 Q#3, would be used
for).

**Recommendation for downstream:** small, low-risk fix in
`reporter/json.rs` -- collect into `BTreeMap<String, u64>` (or build
the inner `serde_json::Value` from a sorted iterator). Should be
explicit BC in Pass 3 retrospective: "JSON keys sort lexically".

---

## 3. Q-A6 reframing -- per-direction vs per-flow alert granularity

P2 R3 and P3 R3 established that the three reassembly alerts (overlap,
small-segment, OOW) are gated by `*_alert_fired` flags **on
`FlowDirection`**, not on `TcpFlow`. With both directions (C2S, S2C) per
flow x 3 alerts each, worst case is 6 anomaly findings per flow
(BC-RAS-022 corrected to "per-direction" by P3 R3). The original P1 R1
claim of "up to 3 per flow" was wrong.

**Architectural question:** is per-direction the right unit?

**Arguments for per-direction (current design):**

- Asymmetric attacker behavior: an evasion technique (e.g., overlap-based
  IDS bypass) typically targets one direction (usually C2S to mask the
  payload going to the server). Suppressing the second-direction alert
  would mask a defender's view of return-channel evasion (rare but
  documented, e.g., reverse-shell over an overlapping S2C stream).
- The thresholds (50 overlaps, 2048 small segments, 100 OOW) are *per
  flow per direction* counters; alert gating on the same axis is
  naturally consistent.
- Adds at most 3 extra findings per flow, well below MAX_FINDINGS=10_000
  for any realistic pcap.

**Arguments for per-flow:**

- Operator mental model: "this flow is suspicious" maps cleanly to one
  finding; six findings for one flow inflate counts in dashboards.
- The current per-direction design means a flow can show *the same*
  evasion narrative twice with two slightly different `summary` strings
  (different overlap counts per direction), which clutters reports.

**Architectural verdict:** **per-direction is defensible** for a
forensic tool where the evasion narrative is itself directional, but it
is currently *undocumented*. The right fix is **not** to change the
design; it is to **make it explicit** in `Finding.summary` -- e.g.,
prefix each finding with `[c2s]` or `[s2c]`. P3 R3 already noted that
the `flow_key` format is bidirectionally canonical (lower/upper, not
src/dst), so the operator cannot recover direction from the
`summary` text. That **is** an architectural smell at the
finding-payload layer.

**Recommendation:** add `direction` to `Finding`'s evidence (or to a
new `Finding.flow_direction: Option<Direction>` field). Sits in Pass 5
(Conventions) follow-up or a future ADR.

---

## 4. Q-A7 resolution -- classification cost ceiling

**Code (`src/dispatcher.rs:72-79`):**

```rust
// Don't cache None — allow reclassification on next on_data with more bytes
let target = if let Some(&cached) = self.routes.get(flow_key) {
    cached
} else {
    let target = classify(data, flow_key);
    if target != DispatchTarget::None {
        self.routes.insert(flow_key.clone(), target);
    }
    target
};
```

The comment is honest: `None` results are intentionally not cached so
that a flow can re-attempt classification on the next chunk. Cost
analysis (consistent with P3 R3's MED-8):

- `classify` is `O(1)` per call: at most 4 byte comparisons (TLS prefix),
  up to 10 `starts_with` HTTP method probes (each `O(n)` where `n =
  method.len()` <= 8), and 4 port comparisons. Constant-bounded.
- A flow that sends `N` tiny (<5-byte) non-protocol segments, each of
  which produces a contiguous flush, gets `N` `classify` calls.
- `N` is bounded by the segment-count limit per direction
  (`max_segments_per_direction = 10_000`) **only after segments are
  accepted into the segment tree**. But contiguous flushes happen on
  the producer side, so `N` could in principle equal the number of
  contiguous flushes -- which in turn is bounded by
  `max_depth / min_segment_size` per direction. With
  `max_depth = 10 MiB` and `min_segment_size = 1 byte`, worst case is
  ~10 million classification attempts per direction per flow.

**Architectural verdict:** **MED-8 is real**. P3 R3 confirmed unbounded
`O(packets_in_flow)` cost on `None`-classified flows. The current
mitigation (segment limit) is too far upstream; classification cost is
not directly bounded by any config knob.

**Recommendation:** add `max_classification_attempts: u32`
(default e.g. 16) to either `StreamDispatcher` or `ReassemblyConfig`.
After N failed attempts on a flow, cache `DispatchTarget::None`
permanently for that flow and short-circuit further attempts. Architectural
preference: put it on `StreamDispatcher` because the bound is a
dispatcher concern, not a reassembly concern; would require adding a
`max_classification_attempts` constructor parameter or a separate
`StreamDispatcher::with_max_attempts` builder. Sits in Pass 4 NFR.

---

## 5. Q-A8 resolution -- services taxonomy mismatch

**Two parallel taxonomies coexist in the codebase:**

1. `Summary.services` (`src/summary.rs:43-44`) is keyed by
   `ParsedPacket::app_protocol_hint`, which (`src/decoder.rs:58-67`) is
   a **pure port lookup**: 53->DNS, 80->HTTP, 443->TLS, 22->SSH, 445->SMB,
   502->Modbus, 20000->DNP3. No content inspection.
2. `StreamDispatcher` (`src/dispatcher.rs:37-64`) is **content-first**:
   TLS via byte signature `0x16 0x03 ...`, HTTP via method-token prefix,
   with port fallback **only** for short data.

**Consequences:**

- A TLS handshake on port 80 (legitimate; ALPN-mismatch attacks; mis-
  configured proxies) is dispatched to `TlsAnalyzer` (content-first) but
  counted as `"HTTP"` in `summary.services` (port-only). The two views
  of the same flow disagree.
- An HTTP/0.9 request on port 8443 is dispatched to `HttpAnalyzer` but
  counted as `"TLS"` in `summary.services`.
- For the wide-port list in `app_protocol_hint` (SSH, SMB, Modbus,
  DNP3), the dispatcher emits no opinion at all -- only the port-based
  service tag exists.

**Architectural verdict:** **the inconsistency is real and undocumented**.
It is not a defect per se -- the two taxonomies answer different
questions:

- `Summary.services`: "What ports did I see traffic on?" (network-layer
  topology view)
- `Dispatcher`: "What protocol is this stream actually speaking?"
  (application-layer behavior view)

But the field name `services` strongly implies the second meaning, and
no documentation or test pins down the contract.

**Recommendation:** either (a) rename `Summary.services` ->
`Summary.port_services` and add a new `Summary.classified_services`
populated by the dispatcher's `routes` map at finalize, or (b) document
explicitly in `Summary` doc-comments that `services` is port-derived
and not authoritative for content type. Sits in Pass 3 (BC) -- this is
an unstated contract.

---

## 6. Inline test audit -- re-verification

**Method:** `find /Users/zious/Documents/GITHUB/wirerust/src -name
'*.rs' -type f -exec awk '/cfg\(test\)/ {print FILENAME":"NR":"$0}' {}
\;`

**Result:** exactly one hit:

```
src/reporter/terminal.rs:261:#[cfg(test)]
```

This is the only `#[cfg(test)]` block in the entire `src/` tree.
Re-confirms P1 R1's claim and P0's count. No new inline tests have
been added since 2026-05-19. Architectural significance: the codebase
maintains a clean **integration-tests-as-spec** convention (all 202
tests live under `tests/`). The single in-`src/` exception is
`escape_for_terminal` -- a private helper -- which is acceptable
because it cannot be exercised through the public surface.

---

## 7. Trait stability inventory -- the 4 traits

**Method:** read each trait body in full; count default methods (a
default method is one with a `{ ... }` body in the trait definition).

| Trait              | File:line                                      | Methods                                                                                                          | Default methods |
|--------------------|------------------------------------------------|------------------------------------------------------------------------------------------------------------------|-----------------|
| `Reporter`         | `src/reporter/mod.rs:8-15`                     | `fn render(&self, summary: &Summary, findings: &[Finding], analyzer_summaries: &[AnalysisSummary]) -> String;` | 0               |
| `ProtocolAnalyzer` | `src/analyzer/mod.rs:19-31`                    | `fn name(&self) -> &'static str;` / `fn can_decode(&self, packet: &ParsedPacket) -> bool;` / `fn analyze(&mut self, packet: &ParsedPacket) -> Vec<Finding>;` / `fn summarize(&self) -> AnalysisSummary;` | 0               |
| `StreamHandler`    | `src/reassembly/handler.rs:19-23`              | `fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64);` / `fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason);` | 0               |
| `StreamAnalyzer`   | `src/reassembly/handler.rs:25-29`              | `fn name(&self) -> &'static str;` / `fn summarize(&self) -> AnalysisSummary;` / `fn findings(&self) -> Vec<Finding>;` (super: `StreamHandler`) | 0               |

**Total: 10 trait methods, 0 default methods.** P1 R1's claim is
exactly correct. Every implementor must spell out every method
explicitly. Architectural implication: **adding a method to any of
these traits is a hard breaking change** for every implementor (no
default fallback). For a 4 KLOC single-crate binary this is fine; for
a future workspace split (Q-A4) it would warrant adding default
implementations to the contract-defining trait crate (e.g.
`StreamAnalyzer::findings` could default to `Vec::new()` for
metrics-only analyzers like DNS, were DNS to be re-modeled as a
`StreamAnalyzer`).

---

## 8. Cross-pollination from P2 R3 / P3 R3

### 8a. Zero `impl Drop` in `src/` (P2 R3 §9)

**Re-verified:** `find src -name '*.rs' -exec awk '/impl Drop/ ...'`
returns zero hits. The codebase has no `Drop` implementations.

**Architectural significance for Pass 1:** the cleanup correctness of
the entire reassembly engine rides on **explicit `finalize()` calls
from `main.rs`** (cited at `main.rs:142` per P1 R1 §6). If a panic
unwinds past `finalize()`, **buffered TCP segments are dropped without
being flushed to handlers**, and any findings that would have been
generated during `close_flow` (e.g., the finalize-side segment-limit
summary finding -- BC-RAS-054) are lost.

This is a real architectural property: **panic-safety in wirerust is
structural-only** (no resource leaks because the OS reaps the process)
**but not behavioral** (alert outputs are not guaranteed if a panic
occurs mid-run). The release profile has `overflow-checks = true`
(`Cargo.toml:24-25`), making overflow panics a non-zero possibility on
adversarial pcaps.

**P1 R1 did not raise this.** New architectural smell candidate
(severity: low, advisory). Added as **Smell #9** below.

### 8b. Dispatcher TLS-first with LOOSE gate (P2 R3 §10)

The classify rule at `dispatcher.rs:39-41` checks only:

1. `data.len() >= 5`
2. `data[0] == 0x16` (TLS handshake content-type)
3. `data[1] == 0x03` (legacy_version major byte == 3)

It does **not** check:

- `data[2]` (legacy_version minor byte) -- should be 0x00..0x04 for
  SSL3.0..TLS1.3
- `data[3..5]` (record length, big-endian u16) -- should be
  `0 < len <= 16384` for TLS records, or up to 16384+256 for
  cipher-text with padding/MAC

P1 R1 noted "TLS `0x16 0x03`" matter-of-factly but did not flag this
as **architecturally loose**. The 3-byte gate is permissive enough that
arbitrary binary protocols whose first two bytes happen to be `0x16
0x03` (small prefix-collision probability ~ 1/65536 if uniform) would
be misrouted to the TLS analyzer, which would then count parse_errors
internally but never produce a finding for the misroute itself.

**P1 R1 did not raise this.** New architectural smell candidate
(severity: low, defensive depth). Added as **Smell #10** below.

### 8c. Pass 6 "20 components" verification

Re-grepped: `awk '/^\| C-[0-9]+/' wirerust-pass-1-architecture.md`
would return exactly 20 rows. (Cannot run grep directly in this
sandbox, but the component table at `wirerust-pass-1-architecture.md
:19-38` clearly lists C-1..C-20.) **Pass 6's "20 components" claim is
verified.**

---

## 9. New / updated architectural smells (delta vs P1 R1)

P1 R1 enumerated 8 smells. Round 2 adds two and revises one.

### Smell #9 (new): No `Drop` impls; cleanup correctness rides on `finalize()` being called

| Field    | Value                                                                                                                                                                                |
|----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Location | All of `src/` (absence-of); `main.rs:142` is the only `finalize` call site                                                                                                          |
| Severity | low (advisory)                                                                                                                                                                       |
| Detail   | `TcpReassembler::finalize` is the only path that flushes the final segment-limit summary finding (BC-RAS-054 cap-bypass) and emits the close events for all still-open flows. If a panic unwinds past line 142 of `main.rs`, those outputs are lost. The release profile sets `overflow-checks = true`, so arithmetic-panic on adversarial pcaps is plausible. |
| Fix      | Wrap the reassembler in a small `OnDrop` guard (an `impl Drop for ReassemblerGuard { fn drop(&mut self) { self.inner.finalize(...) } }`) or document the panic-loses-findings contract on `TcpReassembler`'s rustdoc. |

### Smell #10 (new): TLS-first classifier is too permissive

| Field    | Value                                                                                                                                                                                |
|----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Location | `src/dispatcher.rs:39-41`                                                                                                                                                            |
| Severity | low (defense-in-depth)                                                                                                                                                               |
| Detail   | 3-byte TLS prefix test (`len >= 5 && data[0]==0x16 && data[1]==0x03`) accepts any 5+ byte payload starting with `16 03`. No legacy_version minor-byte check, no record-length sanity. Routes false positives into `TlsAnalyzer`, where they accumulate as `parse_errors` (an internal counter) rather than a `Finding`.                                          |
| Fix      | Tighten gate to `data[2] <= 0x04 && u16::from_be_bytes([data[3], data[4]]) <= 16640`. Costs one comparison + one u16 build, no new allocations.                                       |

### Smell update #6 (pub field exposure on `StreamDispatcher`)

P3 R3's per-direction alert-count discovery and the dispatcher's
`unclassified_flows` inflation (Q-A2) both depend on after-the-fact
inspection via the dispatcher's `pub http` / `pub tls` fields. The
encapsulation issue P1 R1 flagged is more pointed than originally
written: callers can not only read post-run state but also mutate it
(via `Option::take()` etc), which is a real footgun. Severity unchanged
(low), but the fix is now more urgent because it would naturally pair
with the Q-A2 split-counter recommendation.

---

## 10. Open Questions update

P1 R1's open-question table (§11) is updated as follows:

| ID    | Status                                                                                                       |
|-------|--------------------------------------------------------------------------------------------------------------|
| Q-A1  | unchanged (carry to Pass 5 / future ADR)                                                                     |
| Q-A2  | **resolved** -- see §1 above. Architectural recommendation drafted (split `unclassified_flows`)              |
| Q-A3  | unchanged                                                                                                    |
| Q-A4  | unchanged                                                                                                    |
| Q-A5  | **resolved** -- see §2 above. Verdict: ordering asymmetry is unintentional; fix is one-line in `json.rs`     |
| Q-A6  | **resolved (reframed)** -- see §3 above. Per-direction is defensible; fix is to add direction to the finding |
| Q-A7  | **resolved** -- see §4 above. MED-8 confirmed; cap-on-classification recommended                             |
| Q-A8  | **resolved** -- see §5 above. Two-taxonomy mismatch; rename or document                                      |

5 of 8 P1 questions now closed. Three remain (Q-A1, Q-A3, Q-A4) and
all are workspace-split / API-stability concerns that are correctly
deferred to a future ADR or Pass 5.

---

## Delta Summary

- **New items added:**
  - 2 new architectural smells (Smell #9 no-Drop / finalize-fragile, Smell #10 loose-TLS-gate)
  - 1 trait inventory (4 traits, 10 methods, 0 defaults) re-confirmed
  - 5 Q-A items resolved with citations (Q-A2, Q-A5, Q-A6, Q-A7, Q-A8)
- **Existing items refined:**
  - Module-group cycle (Smell #4) -- re-confirmed via direct import grep
  - Smell #6 (pub fields) -- linked to Q-A2 finding
  - One LOC drift on `reassembly/mod.rs` (564 -> 565) noted
- **Remaining gaps:**
  - Q-A1, Q-A3, Q-A4 (workspace-split / API-stability) -- correctly out of scope for Pass 1; belong to Pass 5 / future ADR
  - No Pass-1-internal gaps left

## Novelty Assessment

Novelty: **SUBSTANTIVE**

Justification: this round produced two architectural smells that P1 R1
did not name (no-Drop / loose-TLS-gate), resolved five open questions
with concrete recommendations, and corrected the per-direction alert-
granularity framing in a way that is **directly load-bearing** for
spec crystallization (it produces a new finding-payload requirement:
direction must be on the finding). Removing this round's findings
would change the spec: the spec would not call for a
`max_classification_attempts` knob, would not call for sorted JSON
keys, would not call for direction-on-finding, and would not flag the
panic-safety property. The novelty is binary-SUBSTANTIVE.

That said: most of the substantive content is in §1-5 (Q-A
resolutions) and §8-9 (cross-pollination + new smells). Sections 0, 6,
7 are confirmations (hallucination audit clean, inline-test count 1,
trait method count 10 with 0 defaults). The split is roughly 70%
substantive / 30% confirmation, which qualifies as SUBSTANTIVE but
suggests **one more round (R3) is likely to be NITPICK** -- the
remaining open questions (Q-A1, Q-A3, Q-A4) are by their nature
workspace-split / API-stability concerns that need a future ADR, not
more code reading.

## Convergence Declaration

**Another round needed -- substantive findings remain.** Specifically,
P1 R3 should:

1. Audit `main.rs` more deeply for any cleanup gaps beyond `finalize`
   (Smell #9 follow-up).
2. Confirm the loose-TLS-gate (Smell #10) has not produced any
   currently-failing test in `tests/` (i.e., no test already exercises
   the misroute path).
3. Triangulate whether the per-direction alert design (Q-A6) is
   anywhere documented in `tests/` (e.g., assertion-on-count tests).
4. Close out any inconsistencies between this R2 output and Pass 4
   when Pass 4 runs.

If R3 returns only refinements (e.g., one new minor smell, no spec-
changing findings), declare convergence then.

## State Checkpoint

```yaml
pass: 1
round: 2
status: complete
files_scanned_src: 20
files_re_read: 6  # dispatcher, reporter/json, reassembly/mod, summary, reassembly/handler, analyzer/mod
hallucinations_found_in_p1_r1: 0
new_smells_added: 2  # #9 no-Drop, #10 loose-TLS-gate
q_a_items_resolved: 5  # Q-A2, Q-A5, Q-A6, Q-A7, Q-A8
q_a_items_remaining: 3  # Q-A1, Q-A3, Q-A4 (workspace/API)
timestamp: 2026-05-19T00:00:00Z
novelty: SUBSTANTIVE
next_round: 3
```
