---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-21
capability: CAP-21
lifecycle_status: active
introduced: feature-s7comm
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/specs/architecture/ARCH-INDEX.md
input-hash: "cf116b5"
---

# BC-2.21.033: Cross-Flow Global Correlation State on `S7commAnalyzer` — Port-102 Sweep Tracking and Per-Destination Expected-Source Baseline

## Description

Two of ADR-014 Decision 5's reused techniques require evidence that spans **multiple TCP
flows**, which `S7commFlowState` (per-flow) cannot represent: **T0846** (Remote System
Discovery) needs "TCP SYN sweep across many hosts on :102, or repeated COTP/Setup `0xF0` to
many addresses" — evidence from one SOURCE touching many DESTINATIONS; **T1692.001**
(Unauthorized Message: Command Message) needs "any S7 command... from a source outside an
allowlist / maintenance window" — evidence from one DESTINATION (a PLC) receiving commands from
an unexpected SOURCE. Neither is representable as per-flow state (a single TCP flow has exactly
one source and one destination). This BC adds two small global (per-`S7commAnalyzer`-instance,
not per-flow) structures, mirroring the precedent `ArpAnalyzer.bindings` already establishes
in this codebase for inherently non-flow-scoped state (BC-2.16.004). This BC has **no emission
surface of its own**; BC-2.21.039 (T0846) and BC-2.21.040 (T1692.001) consume it.

**Scope note on "cross-flow" — deliberately narrow, RESOLVED at F2 INTEGRATE
(2026-09-06).** wirerust is a passive pcap forensics analyzer with no raw-TCP-SYN-scan
visibility at the `S7commAnalyzer` layer (SYN/SYN-ACK observation, if any, belongs to a
lower packet-capture layer this analyzer does not consume). This BC therefore does NOT
attempt true TCP-SYN-sweep detection. **T0846's emission scope for this feature is
explicitly and finally the Setup-Communication (`0xF0`) request sweep — never a
TCP-SYN sweep.** It scopes T0846 to the **Setup Communication (`0xF0`) request** signal
only — the first PDU exchanged on every classic S7comm session — which is a defensible,
directly-observed proxy for "a source is establishing S7comm sessions with many distinct
destinations," without overclaiming SYN-level visibility the analyzer does not have. This
is a narrower scope than the source research's SYN-sweep framing. **INTEGRATE confirms
this is a deliberate, disclosed, and final scope reduction for this feature cycle, not an
oversight or an open question** — a future feature cycle that gains packet-capture-layer
SYN/SYN-ACK visibility (a materially different architectural capability wirerust does not
have today) could add a true SYN-sweep detector as a separate, additive signal; it would
not replace this Setup-Communication-based proxy.

## Preconditions

1. `S7commAnalyzer` is processing classic S7comm (`protocol_id == Some(0x32)`) traffic on
   TCP/102 across one or more flows.

## Postconditions

**State shape (both fields live on `S7commAnalyzer`, not `S7commFlowState`):**

1. `port102_setup_targets: HashMap<IpAddr, HashSet<IpAddr>>` — keyed by SOURCE IP, valued by
   the set of distinct DESTINATION IPs that source has sent a `SetupCommunication` (`0xF0`)
   request to, within the current sweep window (see Postcondition 3 for window semantics).
2. `expected_source_by_destination: HashMap<IpAddr, IpAddr>` — keyed by DESTINATION IP (a PLC),
   valued by the FIRST source IP observed issuing a "command-class" frame
   (`S7ClassicFunction::WriteVar(_)`, `RequestDownload`, `DownloadBlock`, `DownloadEnded`,
   `PlcControl(_)` with any recognized non-`Unrecognized` service, or `PlcStop`) to that
   destination. This mirrors DNP3's established "first-observed-source establishes the
   expected baseline" pattern (BC-2.15.010), adapted from DNP3's single-link-multi-master model
   to S7comm's per-TCP-flow model by keying on the DESTINATION (since each S7comm TCP flow
   already pins one source/destination pair — the interesting cross-flow question is whether
   MULTIPLE DIFFERENT flows send commands to the SAME destination from DIFFERENT sources).

**Update rules:**

3. On each `SetupCommunication` request observed (`S7ClassicFunction::SetupCommunication`,
   `0xF0`, from `src_ip` to `dst_ip`): `port102_setup_targets[src_ip]` gains `dst_ip` (set
   insert; a repeat `dst_ip` for the same `src_ip` is a no-op on the set). A sweep window of
   `S7_SWEEP_WINDOW_SECS = 300` (wirerust engineering default, no external standard — mirrors
   ARP's `ARP_FLAP_WINDOW_SECS` disclosure pattern, BC-2.16.004 Invariant 2) is tracked
   per-source via a `first_setup_ts: HashMap<IpAddr, u32>` companion map; when the window
   elapses since a source's first tracked `SetupCommunication` timestamp, that source's entry
   in `port102_setup_targets` resets (empty set, fresh window) on its next `SetupCommunication`.
4. On each command-class frame observed (per Postcondition 2's definition) targeting `dst_ip`
   from `src_ip`: if `expected_source_by_destination` has no entry for `dst_ip`, it is set to
   `src_ip` (this establishes the baseline — no finding is emitted for the baseline-establishing
   frame itself). If an entry already exists and equals `src_ip`, no change. If an entry
   already exists and differs from `src_ip`, the entry is left UNCHANGED (the ORIGINAL baseline
   source is never overwritten by a later, different source) — BC-2.21.040 (T1692.001) is what
   reacts to this mismatch; this BC only maintains the map.

## Invariants

1. **Global, not per-flow**: both maps live on `S7commAnalyzer` and persist across flow
   open/close — a TCP flow closing (BC-2.21.003) does NOT reset `port102_setup_targets` or
   `expected_source_by_destination`, since the evidence these maps carry is inherently
   about cross-flow behavior that a single flow's lifecycle should not erase.
2. **Baseline is first-write-wins, never overwritten**: `expected_source_by_destination`
   mirrors DNP3's convention (BC-2.15.010) exactly — the first source establishes trust, and
   an "unexpected" source never displaces it (unlike ARP's `bindings` table, BC-2.16.005,
   which is last-write-wins; the two "first source" tables in this codebase deliberately use
   opposite update policies for their respective threat models — ARP tracks the CURRENT
   IP→MAC truth, S7comm's baseline tracks the ORIGINAL authorized engineering station).
3. **Engineering-default thresholds, explicitly disclosed**: `S7_SWEEP_WINDOW_SECS = 300` has
   no external standard backing it (mirrors BC-2.16.004's disclosure for ARP's thresholds) —
   it is a wirerust engineering choice, overridable in a future CLI-flag extension (not in this
   feature's scope).
4. **No allowlist/maintenance-window CLI mechanism in this feature**: the source research's
   phrase "outside an allowlist / maintenance window" describes the IDEAL mechanism; this
   feature implements the weaker but wire-observable "first-seen-source" proxy instead
   (mirroring DNP3's own documented limitation, BC-2.15.010 EC-011: "the product has no
   configured-allowlist mechanism... the future `--dnp3-expected-master` allowlist flag (DRIFT)
   will be the escape hatch"). A future `--s7-expected-engineering-station` flag is the
   analogous escape hatch for S7comm, out of scope for this feature.
5. **This BC is state-only**: no `Finding` is emitted here — BC-2.21.039 and BC-2.21.040 own
   emission.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Source A sends `SetupCommunication` to 5 distinct destinations within the sweep window | `port102_setup_targets[A]` has 5 entries |
| EC-002 | Source A sends `SetupCommunication` to the SAME destination twice | `port102_setup_targets[A]` has 1 entry (set semantics) |
| EC-003 | Source A's sweep window elapses with no new `SetupCommunication`, then a new one arrives | `port102_setup_targets[A]` resets to a single-entry set for the new destination |
| EC-004 | First command-class frame to PLC X from source A | `expected_source_by_destination[X] = A`; no finding (baseline established) |
| EC-005 | Second command-class frame to PLC X from source A (same source) | No change — `expected_source_by_destination[X]` remains `A` |
| EC-006 | Command-class frame to PLC X from source B (different from established baseline A) | `expected_source_by_destination[X]` remains `A` (never overwritten); BC-2.21.040 reacts |
| EC-007 | A `ReadVar` (not command-class) frame from a new source to PLC X, no prior baseline | No baseline established — `ReadVar` is excluded from the command-class definition (Postcondition 2) |

## Canonical Test Vectors

| Event sequence | `port102_setup_targets` / `expected_source_by_destination` state | Category |
|---|---|---|
| A→{X,Y,Z} SetupComm (3 destinations) | `port102_setup_targets[A] = {X,Y,Z}` | happy-path: sweep accumulation |
| A→X WriteVar, then A→X WriteVar again | `expected_source_by_destination[X] = A` (unchanged on repeat) | happy-path: baseline stability |
| A→X WriteVar, then B→X PlcStop | `expected_source_by_destination[X] = A` (B does not overwrite) | happy-path: baseline protection |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| `expected_source_by_destination` is monotonically first-write-wins: for any destination, once set, the value never changes for the lifetime of the `S7commAnalyzer` instance | proptest P1 — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — this BC is the cross-flow correlation substrate CAP-21's T0846 and T1692.001 emission call-sites (ADR-014 Decision 5) require, since neither is representable in per-flow `S7commFlowState` |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned) — `S7commAnalyzer` global fields |
| ADR | ADR-014 Decision 5 (T0846 "multi-host TCP/102 sweep evidence," T1692.001 "source outside an allowlist") |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none directly — state-only substrate BC for BC-2.21.039 T0846 and BC-2.21.040 T1692.001) |

## Related BCs

- BC-2.21.010 — depends on (`SetupCommunication` classification, `port102_setup_targets` input)
- BC-2.21.003 — composes with (flow close does NOT affect this BC's global state, in contrast)
- BC-2.16.004 — composes with (the `ArpAnalyzer.bindings` precedent this BC's global-state pattern follows, for the opposite — last-write-wins — update policy)
- BC-2.21.039, BC-2.21.040 — composes with (both consume this BC's state for emission)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `S7commAnalyzer.port102_setup_targets: HashMap<IpAddr, HashSet<IpAddr>>`, `S7commAnalyzer.first_setup_ts: HashMap<IpAddr, u32>`
- `src/analyzer/s7comm.rs` (planned) — `S7commAnalyzer.expected_source_by_destination: HashMap<IpAddr, IpAddr>`
- `src/analyzer/s7comm.rs` (planned) — `const S7_SWEEP_WINDOW_SECS: u32 = 300;` (wirerust engineering default)
- `src/analyzer/arp.rs` — `ArpAnalyzer.bindings` (precedent for global, non-flow-scoped analyzer state)
- `.factory/research/s7comm-mitre-ics-tagging.md` §Per-technique validation table (T0846, T1692.001 rows)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(Deferred — proptest P1, anticipated VP-048 range.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates `S7commAnalyzer`-instance-level (cross-flow) state |
| **Deterministic** | yes — same sequence of frames across all flows produces same state |
| **Thread safety** | `S7commAnalyzer` is single-threaded (consistent with wirerust's single-threaded pipeline) |
| **Overall classification** | stateful orchestration |
