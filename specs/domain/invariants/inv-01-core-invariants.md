---
artifact: L2-inv-01
traces_to: ../domain-spec.md
title: Core Domain Invariants
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
version: "1.4"
modified:
  - "v1.1: Pass-12 corpus-cleanup F-C-P12-001: INV-9 enforcement anchor re-anchored from stale :122-156 (let info=match@:123, _=>return None@:153, }@:156) to current :128-182 (technique_info fn@:128, let info=match id@:129, _ => return None@:179, closing }@:182). — 2026-06-13"
  - "v1.2: ARP-F2 Pass-14 remediation: INV-9 rule prose updated 'Finding.mitre_technique' → 'Finding.mitre_techniques' (STORY-100 AC-008 renamed scalar→Vec; STALE fix per pass-14 discrimination rule). — 2026-06-13"
  - "v1.3: Pass-19 C-03 — INV-2 dispatcher.rs anchors corrected: classify fn :90-117→:184-242; cache-lookup/retry-budget block :133-154→:269-290 (block starts at :269 with 'let target = if let Some(&cached) = self.routes.get(flow_key)'). — 2026-06-13"
  - "v1.4: P19 straggler anchor sweep — INV-2 inline HTTP/ arm cite :104→:199; INV-5 SniValue enum :200→:201, match block :251-265→:252-266; INV-6 MAX_FINDINGS const :54→:56, mod.rs guard sites :461/:495/:524→:479/:515/:546, lifecycle guard sites :101/:121→:111/:141; INV-8 request poison :408-409→:427-428, response poison :467-468→:488-489. Verified against src/reassembly/mod.rs, lifecycle.rs, analyzer/http.rs, analyzer/tls.rs. — 2026-06-13"
---

# Core Domain Invariants

These are the load-bearing invariants of wirerust's domain -- rules that must hold for the
system's forensic correctness guarantees to be valid. Each is grounded in source code and
the ingestion corpus.

Source: pass-8-deep-synthesis.md section 2 ("5 most architecturally-significant invariants"),
pass-2-domain-model.md VO catalog, pass-2-R3.md, pass-3-R4.md.


## INV-1: FlowKey Canonical Ordering

**Rule:** For a TCP connection between (ip_a, port_a) and (ip_b, port_b), `FlowKey::new`
stores the endpoint with the SMALLER (ip, port) tuple as (lower_ip, lower_port). The
comparison is tuple-pair comparison, not independent per-field.

**Why load-bearing:** A->B and B->A must produce the identical key. If the implementation
sorted IPs and ports independently, connections sharing one field but having different values
for the other would map to the same FlowKey, merging unrelated flows.

**Enforcement:** `src/reassembly/flow.rs:48`
```rust
if (ip_a, port_a) <= (ip_b, port_b) { ... }
```

**Tests:** `test_flow_key_canonicalization`, `test_flow_key_same_ip_different_ports`
(tests/reassembly_flow_tests.rs:7,23).

**Corpus refs:** VO-1, C-7, BC-RAS-001.


## INV-2: Content-First Dispatch Precedence

**Rule:** Protocol identification is determined by inspecting the first bytes of reassembled
TCP content before falling back to port numbers. Precedence order:
1. `data.len() >= 5 AND data[0] == 0x16 AND data[1] == 0x03` (two-byte prefix check,
   5-byte buffer minimum to guard against short data) => TLS
2. HTTP method token prefix (`GET `, `POST `, `PUT `, `DELETE `, `HEAD `, `OPTIONS `,
   `PATCH `, `CONNECT `, `TRACE `, or `HTTP/` via `starts_with`) => HTTP
   (the `HTTP/` arm matches response-first streams; see `src/dispatcher.rs:199`)
3. Port-based fallback (80/443/8080/8443) => HTTP or TLS
4. Otherwise => `DispatchTarget::None` for this call

**None caching behavior (verified against `src/dispatcher.rs:269-290`):**
`DispatchTarget::None` is NOT immediately cached. Each `None` result increments a
per-flow `classification_attempts` counter. Once the counter reaches
`max_classification_attempts` (default 8), the dispatcher permanently inserts
`DispatchTarget::None` into `routes` and removes the attempt counter. From that point
the flow is short-circuited as `None` on every subsequent chunk without re-running
`classify`. Successful classifications (Http or Tls) are cached immediately and are
immutable for the flow's lifetime.

**Why load-bearing:** This is the dispatch identity per ADR 0001. Changing to port-first
would break the sniffing of protocol-aware attacks where attackers run non-standard ports.

**Enforcement:** `src/dispatcher.rs:184-242` (`classify` function); `src/dispatcher.rs:269-290`
(cache lookup + retry-budget logic in `on_data`; block starts at line 269 with
`let target = if let Some(&cached) = self.routes.get(flow_key)`).
**Corpus refs:** ADR 0001, VO-E-22, BC-DSP-001..006.


## INV-3: First-Wins Overlap Policy

**Rule:** When TCP segment reassembly encounters overlapping bytes:
- If the new segment covers only gap positions: gap bytes are added (PartialOverlap).
- If the new segment covers already-buffered bytes with IDENTICAL content: silent duplicate (Duplicate).
- If the new segment covers already-buffered bytes with DIFFERENT content: the buffered bytes
  win; the conflicting new bytes are rejected; an Anomaly/Likely/High finding is emitted with
  MITRE T1036 (ConflictingOverlap).

**Why load-bearing:** This is the forensic-truth principle. The first bytes received are
treated as ground truth. ConflictingOverlap findings are the primary signal for TCP evasion
attacks (segment-splicing / IDS bypass attempts).

**Enforcement:** `src/reassembly/segment.rs:FlowDirection::insert_segment` returning
`InsertResult::ConflictingOverlap`; engine match at `src/reassembly/mod.rs:401-434`
(`insert_payload_segment` result dispatch, including the `ConflictingOverlap` arm at
`:408-411` that calls `generate_conflicting_overlap_finding`).
**Corpus refs:** VO-1, BC-RAS-036/037/018, InsertResult (E-13).


## INV-4: Raw-Data / Display-Layer Separation (ADR 0003)

**Rule:** `Finding.summary` and `Finding.evidence` fields carry raw post-`from_utf8_lossy`
bytes. No escape function is applied at construction. `escape_for_terminal` is called ONLY
by `TerminalReporter`. `JsonReporter` delegates to serde_json (RFC 8259).

**Why load-bearing:** Attackers embed C0/DEL/non-UTF-8 bytes in URIs, SNI hostnames, and
TCP payloads. If escape logic runs at construction time, the escaped form reaches JSON
consumers -- forensic data is permanently lost. ADR 0003 was established after PR #49
used `{:?}` Debug formatting at construction time.

**Enforcement:** Convention only; documented in the module header doc-comment at
`src/findings.rs:10-14` and the `Finding::Display` doc at `src/findings.rs:148-156`.
The compiler does not enforce this. Any analyzer that calls `escape_for_terminal` at
construction time silently violates the invariant without a compile error.

**Corpus refs:** ADR 0003, VO-9, BC-FND-005, BC-RPT-001..012.


## INV-5: SNI 4-Way Classification Ordered Match

**Rule:** An SNI byte sequence is classified by the `extract_sni` function into one of four
`SniValue` variants using ordered match arms evaluated top-down:

1. `from_utf8 OK` AND `is_ascii()` AND NOT `contains_c0_or_del()` => `Ascii` (silent, no finding)
2. `from_utf8 OK` AND `is_ascii()` AND `contains_c0_or_del()` => `AsciiWithControl` (T1027)
3. `from_utf8 OK` AND NOT `is_ascii()` => `NonAsciiUtf8` (T1027)
4. `from_utf8 Err` => `NonUtf8` (T1027)

The `is_ascii()` predicate is the controlling gate between arms 2 and 3. For SNI bytes that
are valid UTF-8 but contain BOTH non-ASCII chars AND C0/DEL control bytes, arm 3 fires (NOT
arm 2). The summary for arm 3 says "non-ASCII characters"; the control-byte signal is
recoverable from hex evidence only (BC-TLS-037; pass-2-R3 Target 2).

**Why load-bearing:** The four variants produce different finding summaries and different
downstream SIEM text. Knowing the precedence is necessary to correctly interpret which finding
a given SNI will produce.

**Enforcement:** `src/analyzer/tls.rs:201` (`SniValue` enum definition);
`src/analyzer/tls.rs:252-266` (the match block inside `extract_sni`).
**Corpus refs:** VO-E-35, BC-TLS-014..020, BC-TLS-037.


## INV-6: MAX_FINDINGS Cap with Cap-Bypass for Finalize

**Rule:** The reassembly engine's `findings: Vec<Finding>` is capped at `MAX_FINDINGS = 10,000`
(reassembly/mod.rs:56). Guard checks push or count-as-dropped via `self.findings.len() >= MAX_FINDINGS`
at five sites (mod.rs:479,515,546 and lifecycle.rs:111,141).

**Exception:** `TcpReassembler::finalize()` pushes the segment-limit summary finding
UNCONDITIONALLY, bypassing the cap guard (BC-RAS-054). This finding can make
`self.findings.len() == MAX_FINDINGS + 1`.

**Observability:** `ReassemblyStats.dropped_findings: u64` (added P1.01 / #73) is incremented
each time a finding is suppressed by the cap. Accessible via
`summarize().detail["dropped_findings"]`. Cap hits are now observable; the prior
observability gap (NFR-RES-022) is closed.

**Note:** The `HttpAnalyzer.all_findings` and `TlsAnalyzer.all_findings` vecs are NOT subject
to this cap. Only the reassembly engine enforces MAX_FINDINGS.

**Why load-bearing:** Prevents unbounded memory growth from adversarial input designed to
flood the finding buffer. The cap is the primary resource-bounding mechanism for the engine.

**Enforcement:** `MAX_FINDINGS` const at `src/reassembly/mod.rs:56`; guard sites at
`mod.rs:479,515,546` (three per-direction anomaly threshold checks in
`check_anomaly_thresholds`) AND `reassembly/lifecycle.rs:111,141` (conflicting-overlap
and truncated findings in `generate_conflicting_overlap_finding` /
`generate_truncated_finding`). Five guard sites across two files.
**Corpus refs:** ADR 0002, NFR-RES-001, NFR-RES-022, BC-RAS-054.


## INV-7: Finalize-Once Latch

**Rule:** `TcpReassembler::finalize()` must be called exactly once per reassembler instance.
A `finalized: bool` latch at mod.rs:615-618 makes subsequent calls no-ops.

**Safety net (P0.03 / #72):** `impl Drop for TcpReassembler` emits a one-shot `eprintln!`
if the reassembler is dropped without `finalize()` having been called, naming how many flows
and bytes were discarded. The `run_analyze` IIFE pattern in main.rs ensures `finalize()` is
reached before any `Err` escapes the function body, closing the prior escape-via-`?`
gap (Smell #9 / domain-debt D-01, now retired).

**Why load-bearing:** Finalize is the only cleanup path that emits the segment-limit summary
finding and closes all remaining open flows. Skipping it loses forensic data silently.

**Enforcement:** `src/reassembly/mod.rs:615-618` (latch: `if self.finalized { return; }`
guard + `self.finalized = true` assignment); `impl Drop` at `mod.rs:851-865`
(tripwire: `FINALIZE_SKIPPED_WARNED` one-shot guard); main.rs IIFE pattern
(caller guarantee).
**Corpus refs:** BC-RAS-054, LESSON-P0.03.


## INV-8: HTTP Poisoning is Monotonic False-to-True

**Rule:** `HttpFlowState.request_poisoned` and `HttpFlowState.response_poisoned` are set to
`true` once when the respective direction's `error_count >= POISON_THRESHOLD (3)`. They never
reset to `false` within a flow's lifetime (pass-2-R3 Target 3 confirmed zero `= false`
assignments in http.rs).

**Related:** `request_error_count` / `response_error_count` are NON-monotonic -- they reset
to 0 on a successful parse. The threshold is therefore consecutive errors, not cumulative.

**Why load-bearing:** A poisoned direction silently absorbs all subsequent bytes. If the
latch could reset, a successfully-parsed segment after poisoning would re-open the parsing
path unexpectedly. The monotonic design ensures a flow with proven repeated-parse-failures
stays quarantined for its lifetime.

**Enforcement:** `src/analyzer/http.rs:427-428` (request direction poison transition:
`request_error_count >= POISON_THRESHOLD` => `request_poisoned = true`);
`src/analyzer/http.rs:488-489` (response direction poison transition:
`response_error_count >= POISON_THRESHOLD` => `response_poisoned = true`);
absence of `= false` assignments (confirmed pass-2 R3 Target 3).
**Corpus refs:** BC-HTTP-010..016, VO-E-32.


## INV-9: MITRE Technique ID Format

**Rule:** All MITRE technique IDs emitted in `Finding.mitre_techniques` follow the pattern
`TXXXX` (4-digit parent) or `TXXXX.NNN` (3-digit sub-technique suffix). IDs not present in
`technique_info`'s static match return `None` from `technique_name()` and
`technique_tactic()`.

**Why load-bearing:** Terminal reporter uses `technique_tactic()` to group findings. An
unrecognized ID produces an ungrouped finding with a `<id> (unknown)` label. JSON output
passes the raw ID string through without validation.

**Enforcement:** `src/mitre.rs:128-182` (`technique_info` function; fn declaration at `:128`, `let info = match id`
block starts at `:129`, wildcard arm `_ => return None` at `:179`, closing `}` at `:182`).
**Corpus refs:** VO-6, BC-MIT-005..008, CAP-10.
