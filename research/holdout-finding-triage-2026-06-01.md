# Holdout Finding Triage — Phase 4 Evaluation (DF-VALIDATION-001)

| Field | Value |
|-------|-------|
| Date | 2026-06-01 |
| Agent | vsdd-factory:research-agent |
| Policy | DF-VALIDATION-001 (validate before any fix/issue) |
| Findings triaged | HS-006 (LOW), HS-016 (LOW), HS-043 (MEDIUM) |
| Cycle | v0.1.0-greenfield-spec |

> Scope note: this is a *validation/triage* pass. No source code, specs, or holdout
> scenarios were modified. Source was read for grounding only (research agent is not the
> asymmetric evaluator). Every claim below is grounded in the cited BC + the actual binary
> behavior in `src/`.

---

## Summary Table

| Finding | Verdict | Severity (revised) | Root-cause class | Recommended route |
|---------|---------|--------------------|------------------|-------------------|
| HS-006 | **confirmed-real** (BC/impl tension, not a pure impl bug) | LOW | BC-clarification (primary) + 1-line impl fix (if BC is held) | Product-owner decision → then implementer (trivial) |
| HS-016 | **mischaracterized** (holdout over-specifies vs the BC) | LOW (informational) | product-owner-decision (BC enhancement, optional) | Product-owner decision; NOT an implementer defect |
| HS-043 | **confirmed-real** (worse than the claim) | **MEDIUM → arguably HIGH-adjacent** | implementation-fix (missing wiring) + product-owner (CLI knob) | Implementer (wire `expire_flows`) + product-owner (decide observability) |

---

## Finding 1 — HS-006: em-dash vs ASCII-hyphen separator in the finding line

### Claim
The terminal finding one-liner uses ASCII hyphen `-` as the verdict→summary separator
instead of the em-dash U+2014 the BC specifies; the em-dash *is* used on the `MITRE: Tn —
name` line, so the token choice is internally inconsistent.

### What the BC says
- **BC-2.09.002** ("Finding Display Renders `[Category] VERDICT (CONFIDENCE) — summary`"):
  the mandated separator is `" — "` (U+2014 em-dash). **Critically, BC-2.09.002 scopes this
  to `Finding`'s `fmt::Display` impl** — its Description says verbatim: *"This Display output
  is used for debugging and logging; **terminal rendering uses the reporter layer**."*
  Architecture Anchor: `src/findings.rs:157-168`. Postcondition 1 / Invariant 1 hardcode the
  em-dash template. The BC says **nothing** about the terminal reporter's separator.

### Actual binary behavior (grounded in source)
Two distinct code paths render a finding line, and they disagree:

1. `impl fmt::Display for Finding` — `src/findings.rs:163`:
   `"[{cat}] {verdict} ({conf}) — {summary}"` → **em-dash U+2014**. ✅ Matches BC-2.09.002.
2. `TerminalReporter::render_finding_prefix` — `src/reporter/terminal.rs:198-201`:
   `"[{}] {} ({}) - {}"` → **ASCII hyphen-minus**. This is the path the holdout's
   `wirerust analyze` terminal output actually exercises.
3. The grouped MITRE line (`render_finding_grouped`, terminal.rs:~231 doc + impl) renders
   `ID — Name` with an **em-dash**, which is the internal inconsistency the holdout flags.

So the holdout's factual observation is **correct**: the terminal finding line uses `-`
while the MITRE line uses `—`. The mismatch is real.

### The nuance that changes the verdict
BC-2.09.002 governs `Finding::Display` (findings.rs), which **already complies** (em-dash).
The terminal reporter is a *separate* layer (ADR 0003) with its **own** reporting BCs
(BC-2.11.007 and siblings, exercised by HS-076/HS-093/HS-099). HS-006's verification approach
reads `wirerust analyze` *terminal* output and asserts the BC-2.09.002 em-dash there — i.e.
it applies a `Finding::Display` contract to a different code path the BC explicitly says it
does not cover. The terminal reporter has no BC mandating a separator character at all (I
read BC-2.09.002/003/004; none of them bind `terminal.rs`).

### External-convention check (Perplexity)
A CLI/security-tooling convention search was decisive in the **opposite** direction from a
naive "fix the impl to em-dash" reaction: ASCII hyphen-minus is the recommended default
separator for human-readable, pipe-friendly security-CLI output (Snort/Suricata/Zeek/grep
are heavily piped through `cut -d'-'`/`awk`; non-ASCII em-dash breaks byte-oriented parsing
and renders as tofu on minimal/serial terminals). Em-dash is acceptable only for an explicit
"pretty" mode. This means the **terminal reporter's hyphen is arguably the more correct
choice for terminal output**, and the inconsistency is better resolved by aligning the
*MITRE line down to hyphen* than by pushing em-dash into the main finding line.

### Verdict: confirmed-real (BC/impl tension), severity LOW
The observation is factually accurate (real inconsistency), but it is **not a clean
implementation bug** — the implementation satisfies the BC that actually governs it. The
defect is a **specification gap**: the terminal reporter's separator is unspecified, and the
two layers + the two terminal sub-lines (finding vs MITRE) drifted.

### Root-cause class: BC-clarification (primary); trivial impl fix only if PO holds em-dash

### Recommended route + concrete fix
1. **Product-owner decision (route this first).** Decide the terminal-reporter separator
   policy and record it in a reporting BC (BC-2.11.x). Two coherent options:
   - **(Recommended) Hyphen everywhere in terminal output.** Keep `render_finding_prefix`'s
     `-`, change the grouped MITRE line from `—` to `-` for internal consistency, and add a
     one-line clause to a BC-2.11.x reporting contract stating the terminal layer uses ASCII
     `-`. Rationale: matches the piped-CLI convention (Perplexity), keeps `Finding::Display`'s
     em-dash for logging/debug intact, resolves the inconsistency, and is terminal-safe.
   - **(Alternative) Em-dash everywhere in terminal output.** Change `render_finding_prefix`
     to `"[{}] {} ({}) — {}"` (terminal.rs:199) so it matches `Finding::Display` and the
     MITRE line. Costs portability/grep-friendliness per the convention research.
2. **Implementer fix (trivial, only after PO picks an option):** a single-character edit in
   `src/reporter/terminal.rs:199` (and/or the grouped MITRE line) — exact format string is
   `"[{}] {} ({}) - {}"` at terminal.rs:198-201. No logic change.

### Citations
- BC: `.factory/specs/behavioral-contracts/ss-09/BC-2.09.002.md` (Description ¶2; PC1; Inv1;
  Anchor findings.rs:157-168).
- Impl (compliant em-dash): `src/findings.rs:163`.
- Impl (terminal hyphen — the holdout's actual path): `src/reporter/terminal.rs:198-201`.
- Holdout: `.factory/holdout-scenarios/HS-006-finding-display-format-and-verdict-tokens.md`
  (Verification Approach bullet 4; Rubric "Data integrity").
- Convention: Perplexity search (ASCII-hyphen preferred for piped security-CLI output;
  clig.dev; Unix CLI culture).

---

## Finding 2 — HS-016: raw conflicting bytes vs fixed description in overlap evidence

### Claim
The conflicting-overlap (T1036) finding's `evidence` is a fixed description string
("Retransmitted segment contains different data") rather than the raw conflicting bytes that
BC-2.04.037 expects; TLS SNI findings DO emit raw hex evidence, so the raw-byte contract is
honored unevenly.

### What the BCs actually say
I read all three BCs the holdout traces to:

- **BC-2.04.018** (the finding-emission contract). Postcondition 2 enumerates the finding's
  fields exactly: category=Anomaly, verdict=Likely, confidence=High, mitre=T1036,
  `summary: contains the FlowKey display string`, `direction: None`. **It does NOT mention
  the `evidence` field at all** — neither requiring raw bytes nor forbidding a description.
- **BC-2.04.037** (the `insert_segment` classification contract). Its postconditions are
  about the **return value and buffer state** (`ConflictingOverlap` returned; `segments`/
  `buffered_bytes` unchanged; `overlap_count++`; original bytes preserved). It is explicitly
  scoped: PC5 + Invariant 3 say the finding "is emitted by the engine match arm ... **outside
  the scope of `insert_segment` itself**." **BC-2.04.037 never specifies the `evidence`
  contents.** The holdout's BC-linkage table even describes BC-2.04.037 as testing "original
  bytes preserved" (the buffer), not the evidence field.
- **BC-2.09.005** (the raw-data contract, ADR 0003/INV-4). This says: *when* `summary`/
  `evidence` carry attacker-controlled bytes, they are stored post-`from_utf8_lossy` **without
  additional escaping**, and `escape_for_terminal` is never called at construction. It is a
  **"do not escape what you do put there"** contract — it does **not** mandate that the
  conflicting-overlap finding *must populate* evidence with raw conflicting bytes.

### Actual binary behavior (grounded in source)
- Overlap finding: `src/reassembly/lifecycle.rs:105-115`
  (`generate_conflicting_overlap_finding`): `summary = "Conflicting TCP segment overlap on
  flow {key}"` (FlowKey display — **satisfies BC-2.04.018 PC2**), `evidence = vec![
  "Retransmitted segment contains different data".to_string()]` (a **fixed description**, no
  raw bytes), `mitre_technique = T1036`, `direction = None`. All BC-2.04.018 PC2 fields match.
- TLS SNI findings: `src/analyzer/tls.rs:442/462/482` emit `evidence: vec![format!("hex:
  {hex}")]` — raw hex. So the holdout's factual contrast is **accurate**: TLS does emit raw
  bytes in evidence; the overlap path emits a static string.

### Verdict: mischaracterized — severity LOW (informational), NOT an implementer defect
The factual observation (TLS emits raw-byte evidence; the overlap finding emits a static
description) is **true**, but the holdout's framing — that BC-2.04.037 *expects* raw
conflicting bytes in the overlap evidence — is **not supported by any of the three cited
BCs**. None of BC-2.04.018, BC-2.04.037, or BC-2.09.005 requires the overlap `evidence` to
contain the raw conflicting bytes. The current implementation is **BC-compliant**:
- BC-2.04.018 PC2: every enumerated field matches (incl. summary = FlowKey, direction = None).
- BC-2.04.037: the buffer/return-value postconditions are about `insert_segment`, which is a
  separate function; the finding-emission path is out of its scope by its own text.
- BC-2.09.005: not violated — the description string is not attacker-controlled bytes that got
  escaped; it's a hardcoded constant, which BC-2.09.005 neither prohibits nor governs.

So the rubric's "Data integrity (0.3): evidence contains raw byte sequences, not a sanitized
description" is a holdout-author **expectation that exceeds the contract**. This is a
"holdout over-specifies relative to the BC" case, mirrored by the HS-016 author themselves
noting the BC tests buffer preservation, not the evidence field.

### Root-cause class: product-owner-decision (optional BC enhancement)
There is a legitimate **forensic-value argument** for the holdout author's expectation: a
T1036 evasion finding is more useful if its evidence shows the conflicting bytes (e.g.
`existing: 41 41 41 | conflicting: 42 42 42`), exactly as TLS SNI does. But that is a
**product enhancement to the contract**, not a fix to a violated one.

### Recommended route + concrete fix
1. **Product-owner decision.** Decide whether forensic parity with TLS (raw bytes in
   overlap evidence) is desired. Two outcomes:
   - **If yes (recommended for forensic consistency):** amend **BC-2.04.018** (add a PC/clause
     mandating `evidence` carry the raw conflicting/original byte excerpts, e.g. hex of the
     overlapping range), and optionally cross-reference BC-2.09.005. Then route to implementer
     to thread the conflicting + original byte slices from the `insert_segment`
     ConflictingOverlap path into `generate_conflicting_overlap_finding`
     (`src/reassembly/lifecycle.rs:96-116`). Note: this requires the segment layer to surface
     the differing bytes to the engine match arm (`src/reassembly/mod.rs:379-382`), which
     currently passes only `key` + `src_ip` — a real (small) data-plumbing change, not a
     one-liner. Respect the MAX_FINDINGS cap and ADR-0003 raw-byte (no-escape) rule.
   - **If no:** record a one-line clarification in BC-2.04.018 that the overlap evidence is a
     fixed descriptive string by design, and update/relax the HS-016 rubric's "Data integrity"
     weight so it does not assert a contract that does not exist. (Holdout authorship is PO
     scope; the research agent does not edit holdouts.)
2. **No implementer defect to file** under DF-VALIDATION-001 — the current code satisfies the
   contracts as written.

### Citations
- BCs: `.factory/specs/behavioral-contracts/ss-04/BC-2.04.018.md` (PC2 — no evidence clause),
  `.factory/specs/behavioral-contracts/ss-04/BC-2.04.037.md` (PC1-5, Inv3 — `insert_segment`
  scope only), `.factory/specs/behavioral-contracts/ss-09/BC-2.09.005.md` (no-escape contract,
  not a populate-with-raw-bytes mandate).
- Impl (overlap finding, static evidence): `src/reassembly/lifecycle.rs:105-115`.
- Impl (TLS raw-hex evidence, the contrast): `src/analyzer/tls.rs:442,462,482`.
- Holdout: `.factory/holdout-scenarios/HS-016-real-world-corpus-evasion-pcap.md`
  (Step 3; Rubric "Data integrity" 0.3; Failure Guidance).

---

## Finding 3 — HS-043: `flows_expired` (idle-flow timeout) is not externally observable

### Claim
`flows_expired` is not externally observable — there is no `--flow-timeout`/
`flow_timeout_secs` CLI knob and the default is large enough that `flows_expired` never
increments through the CLI, so the holdout's `flows_expired >= 1` cannot be exercised
black-box.

### What the BCs say
- **BC-2.04.013** ("expire_flows Closes Idle Flows Past flow_timeout_secs"): fully specifies
  the *logic* — `expire_flows(current_time, handler)` closes flows where `state==Closed OR
  (current_time > last_seen AND current_time - last_seen > timeout)`, closes with
  `CloseReason::Timeout`, increments `stats.flows_expired`, underflow-safe, strict `>`.
  Architecture Anchor: `src/reassembly/mod.rs:536-552`. **Crucially, BC-2.04.013 specifies
  the function's behavior but says nothing about *who calls it* in the production pipeline,
  nor that the timeout is CLI-configurable.** The default `flow_timeout_secs = 300` is in
  `ReassemblyConfig::default()`.
- **BC-2.04.029** (close_flow missing-key one-shot warning): defensive guard; "Should not
  occur in normal operation" per the holdout. Not the crux.

### Actual binary behavior (grounded in source) — three sub-questions

**(a) Does the implementation HAVE idle-flow-expiry logic + a `flows_expired` counter?**
**YES.** `TcpReassembler::expire_flows` exists at `src/reassembly/mod.rs:536-552`, correctly
implements the BC-2.04.013 guard (`flow.state == FlowState::Closed || (current_time >
flow.last_seen && (current_time - flow.last_seen) > timeout)`), closes with
`CloseReason::Timeout`, and increments `self.stats.flows_expired` (mod.rs:549). The counter
field is `src/reassembly/stats.rs:16` and is surfaced in the reassembly summary detail map
(`src/reassembly/mod.rs:663`: `detail.insert("flows_expired".into(), ...)`). It is extensively
unit/integration-tested (`tests/reassembly_engine_tests.rs` — `test_BC_2_04_013_*`, boundary
EC-007/EC-008, underflow guard, Closed-state branch). The logic is correct and well-covered.

**(b) Is the timeout configurable via CLI?** **NO.** I read all of `src/cli.rs`. The global
flags are `--no-color`, `--output-format`, `--json`, `--csv`, `--reassemble`,
`--no-reassemble`, `--reassembly-depth`, `--reassembly-memcap`, `--overlap-threshold`,
`--small-segment-threshold`, `--small-segment-max-bytes`, `--small-segment-ignore-ports`,
`--out-of-window-threshold`. **There is no `--flow-timeout` / `--flow-timeout-secs` flag.**
`flow_timeout_secs` is only ever the hardcoded `300` from `ReassemblyConfig::default()`
(`src/reassembly/config.rs:121`). (Note the project's own LESSON-P1.04 "no unwired flags"
convention in cli.rs — adding a knob is consistent with that philosophy, in reverse: a
tested-but-unexposed capability.)

**(c) Worse than the claim — `expire_flows` is NEVER CALLED in production at all.**
This is the load-bearing finding. The CLI per-packet loop in `src/main.rs:154-176` calls only
`reasm.process_packet(&parsed, raw.timestamp_secs, &mut dispatcher)` per packet, then
`reasm.finalize(&mut dispatcher)` once at end (main.rs:189). I confirmed via grep that **the
only call sites of `expire_flows` in the entire repo are in `tests/`** — never in `src/`.
I also read `process_packet` (`src/reassembly/mod.rs:134-174`) end-to-end: it does **not**
internally call `expire_flows` either. And `finalize` closes remaining flows with
`CloseReason::Finalize` (not `Timeout`) and does **not** touch `flows_expired`.

Therefore: **`flows_expired` is structurally guaranteed to be 0 for every CLI invocation,
regardless of pcap timestamps or idle gaps.** The holdout's mental model ("Flow B is expired
when the next packet arrives after 300s of silence") is **incorrect about the implementation**
— but its *conclusion* (`flows_expired >= 1` is not black-box reachable) is not just correct,
it understates the problem: even with a `--flow-timeout 1` knob, nothing would call
`expire_flows`, so the counter would still never move via the CLI.

### Verdict: confirmed-real — severity MEDIUM (the holdout rated it MEDIUM; I concur, leaning higher)
This is a genuine **dead-code / missing-wiring defect**, not a holdout over-specification.
The idle-flow-expiry capability — a documented memory-safety bound for long captures
(BC-2.04.013 capability anchor: *"idle flow expiry is required to bound memory use in
long-running captures"*) — is **implemented, tested, and then never invoked by the binary.**
The 100K `max_flows` LRU eviction (BC-2.04.015) still bounds memory, so this is not an
immediate OOM, which is why MEDIUM (not HIGH) is defensible — but the advertised
idle-timeout protection is effectively absent from the shipping product.

### Root-cause class: implementation-fix (missing wiring) + product-owner-decision (CLI knob / observability)
Two separable issues:
1. **Wiring gap (implementation-fix):** `expire_flows` is never called in the production
   pipeline. This is a clear defect regardless of the holdout.
2. **Observability/config gap (product-owner-decision):** no CLI knob exposes
   `flow_timeout_secs`, and `flows_expired`, while present in the reassembly summary detail
   map, cannot be driven non-zero black-box even if `expire_flows` were wired, unless a pcap
   with >300s idle gaps is used or the timeout is tunable.

### Recommended route + concrete fix
1. **Implementer (primary defect):** wire `expire_flows` into the per-packet loop. The
   natural call site is inside `process_packet` (or in the `main.rs` loop immediately before/
   after `process_packet`), passing the current packet's `timestamp_secs` so that an arriving
   packet triggers expiry of any flow idle beyond `flow_timeout_secs`. This is exactly the
   semantics BC-2.04.013 describes ("The caller is responsible for passing `current_time`
   (typically the timestamp of the packet being processed)") and matches the holdout's mental
   model. Anchor for the fix: call `self.expire_flows(timestamp, handler)` near
   `src/reassembly/mod.rs:140` (start of `process_packet`) OR in `src/main.rs:162-164`. A
   per-packet sweep is O(flows); if that is a perf concern, gate it to fire every N packets or
   every M seconds of wall-stream-time — that is an implementation detail to validate against
   the reassembly perf design (`docs/superpowers/specs/2026-04-06-reassembly-perf-design.md`).
   This wiring is the load-bearing change and must be accompanied by an integration test that
   drives `flows_expired > 0` through the public `process_packet` path (not a direct
   `expire_flows` call), since the existing tests only exercise `expire_flows` directly and
   therefore did not catch the wiring gap.
2. **Product-owner decision (observability):** decide whether to (a) add a `--flow-timeout`
   (seconds) CLI flag wiring `ReassemblyConfig::flow_timeout_secs` — consistent with the
   existing `--reassembly-*` / `--*-threshold` knobs and the LESSON-P1.04 convention — so the
   capability is operator-tunable and the holdout's `flows_expired >= 1` is exercisable
   black-box with a small timeout; and/or (b) confirm `flows_expired` appears in the JSON
   reassembly summary (it does, via mod.rs:663) so the holdout can assert on it. If neither is
   desired, the alternative is for the PO to relax HS-043 to not require black-box
   observability of an internal default — but given finding (c), the underlying wiring defect
   should be fixed regardless of the holdout's observability ask.

> DF-VALIDATION-001 disposition: HS-043 yields a **validated implementation-fix finding**
> (the un-wired `expire_flows`). This one is eligible to be filed as a GitHub issue after PO
> sign-off on the wiring approach + the observability decision. The holdout's specific
> mechanism description should be corrected in the issue (it's "never called," not "default
> too large").

### Citations
- BC: `.factory/specs/behavioral-contracts/ss-04/BC-2.04.013.md` (Description; PC1-2; Anchor
  mod.rs:536-552; capability-anchor memory-bound justification),
  `.factory/specs/behavioral-contracts/ss-04/BC-2.04.029.md` (defensive guard, non-crux).
- Impl (logic present, correct): `src/reassembly/mod.rs:536-552`; counter
  `src/reassembly/stats.rs:16`; summary surface `src/reassembly/mod.rs:663`; default 300
  `src/reassembly/config.rs:121,33`.
- Impl (no CLI knob): `src/cli.rs:42-110` (full global-flag set; no `--flow-timeout`).
- Impl (never wired): `src/main.rs:154-176` (loop calls only `process_packet`), `:189`
  (`finalize`); `process_packet` body `src/reassembly/mod.rs:134-174` (no internal
  `expire_flows`); grep confirms `expire_flows` call sites exist only under `tests/`.
- Holdout: `.factory/holdout-scenarios/HS-043-timeout-idle-cleanup.md` (Scenario steps 4/6;
  Verification "flows_expired is 1"; Failure Guidance).

---

## Cross-Finding Notes

- **HS-006 and HS-016 are both "scope-mismatch" findings**: the holdout applies a contract
  (BC-2.09.002 for HS-006; an inferred raw-bytes expectation for HS-016) to a code path or
  field the BC does not actually bind. Neither is a clean implementer bug; both are
  product-owner clarification/enhancement decisions. HS-006 additionally surfaces a real,
  fix-worthy internal inconsistency (finding line `-` vs MITRE line `—`).
- **HS-043 is the only finding that is a true, fileable implementation defect** under
  DF-VALIDATION-001: a tested capability (`expire_flows`) is never invoked by the binary.
  This is the highest-value outcome of the triage and the one that warrants an issue (post
  PO sign-off on the wiring + observability approach).
- Recurring theme: tests that exercise engine methods **directly** (e.g. `expire_flows`)
  rather than through the public `process_packet`/CLI surface can mask wiring gaps. Worth a
  note to the test strategy that lifecycle methods get at least one through-the-front-door
  integration test.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity search | 1 | CLI/security-tool separator convention (em-dash vs ASCII hyphen) for HS-006 |
| Read | 14 | 3 holdout scenarios; 6 BC files (2.09.002, 2.04.037, 2.04.018, 2.09.005, 2.04.013, 2.04.029); HS-INDEX; findings.rs; terminal.rs; lifecycle.rs; reassembly/mod.rs (expire_flows + process_packet); config.rs; cli.rs; main.rs; tls.rs |
| Glob | 3 | locate holdout scenarios, src tree, BC files |
| Grep | 4 | `expire_flows`/`flows_expired`/`flow_timeout` call-site enumeration; TLS evidence sites; main.rs wiring; process_packet body |
| Context7 | 0 | n/a — no external library version question in scope |
| Tavily | 0 | n/a — single external convention question; Perplexity sufficient and corroborated by clig.dev/Wikipedia/Cisco/HN sources it cited |
| WebFetch | 0 | n/a |
| WebSearch | 0 | n/a |
| Training data | 1 area | Rust syntax/`fmt::Display` semantics (low-risk, structural); all behavioral claims grounded in read source + cited BCs |

**Total external MCP/tool calls:** 1 Perplexity + 25 local Read/Glob/Grep = 26.
**Training data reliance:** low — every behavioral/version-independent claim is grounded in
the actual `src/` lines and the cited BC files; the only external-knowledge input
(separator convention) was sourced via Perplexity with named citations.
