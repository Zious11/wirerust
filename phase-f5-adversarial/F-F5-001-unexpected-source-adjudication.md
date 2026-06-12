---
document_type: adjudication
finding_id: F-F5-001
status: resolved
produced_by: architect
date: 2026-06-11
feature: issue-008-dnp3-analyzer
bc_ref: BC-2.15.010 Invariant 5
holdout_ref: HS-W37-002
adr_ref: ADR-007 Decision 5
---

# F-F5-001: Unexpected-Source Detection — Design Directive

## Gap Summary

BC-2.15.010 Invariant 5 (v1.2) mandates that a Control-class FC from a
non-allowlisted source address emits T1692.001 at count=1, independently of the
10/60s volumetric burst threshold. The current implementation contains only the
volumetric burst check (`count > threshold`). The `master_addrs_seen` field is
populated by STORY-107 but is never read by any detection branch. No
expected/allowlisted set exists on `Dnp3Analyzer`. Holdout HS-W37-002 (P0,
must-pass) tests exactly this behavior and explicitly warns it will fail if the
unexpected-source check is gated behind the same `count > threshold` condition.

---

## 1. MECHANISM (chosen)

**DECISION: First-seen-master learning — the first master address observed on a
flow becomes the "expected" set; any subsequent distinct master address from
which a Control-class FC arrives is "unexpected".**

### Justification

HS-W37-002 specifies: "A Control-class FC arrives from source address 0x0099
which is NOT in the expected/allowlisted master-address set for this flow."
It provides no CLI-flag setup instructions and no pre-population step. This
implies the test constructs a fresh flow, delivers one Control FC from a first
master address to establish the flow context, and then delivers one Control FC
from 0x0099. The "expected set" is implicitly the first master address seen on
the flow — not a CLI-configured allowlist.

BC-2.15.010 Invariant 5 uses the language "non-allowlisted SOURCE address" and
"comparing `src` against an allowlist or expected master-address set." The phrase
"or expected master-address set" is the operative phrase. The [F2-GATE] note
confirms the default is `direct_operate_threshold = 10` (already pinned) but
there is no parallel [F2-GATE] note for an allowlist CLI flag. No BC-2.15.0xx
exists that specifies a `--dnp3-expected-master` flag. This absence is decisive:
the BC corpus does not commission a CLI flag for this feature.

The `master_addrs_seen` field was introduced in ADR-007 Decision 4 and
STORY-107 precisely for this purpose — its description in the struct comment
reads "Source link addresses seen claiming DIR=1 (master direction)." It is
populated on every valid master-direction frame. The natural interpretation is:
the first master address observed on a flow is the expected master; any additional
distinct master address triggering a Control FC is unexpected.

**Chosen mechanism:** On the first Control-class FC seen on a flow (i.e., when
`flow.master_addrs_seen` already has at least one entry from prior master frames
on the same flow, OR we treat the first Control FC itself as the establishing
event), check whether the source address of the current Control FC is the same as
the *first* entry in `master_addrs_seen`. If the source is not present in
`master_addrs_seen` at all at the time of the Control FC, it is unexpected.

**Precise rule:** at the moment a Control-class FC is processed:
- If `flow.master_addrs_seen.is_empty()`: this is the first master address seen
  on the flow. It is NOT unexpected (it establishes the expected set). Push it
  via the existing logic (which already runs before the detection branch).
- If `!flow.master_addrs_seen.is_empty()` AND `src` is NOT in
  `flow.master_addrs_seen`: the source is unexpected. Emit T1692.001 immediately
  at count=1, regardless of `direct_operate_count` vs `direct_operate_threshold`.

Note: the `master_addrs_seen` population already happens at line ~446 BEFORE the
detection branch fires at line ~514. The ordering is correct as-is:

```
// (existing) populate master_addrs_seen for master-direction frames
if is_master_frame(header.control)
    && !flow.master_addrs_seen.contains(&header.source)
    && flow.master_addrs_seen.len() < MAX_MASTER_ADDRS
{
    flow.master_addrs_seen.push(header.source);   // pushes FIRST, THEN detection runs
}

// then: detect_control_class_burst_split is called
```

Because the population already runs first: when the unexpected-source check
runs inside `detect_control_class_burst_split`, `master_addrs_seen` will already
contain the current frame's source if it was the first master frame (just added),
OR it already contained it from a prior frame (not unexpected), OR it was not
added because `MAX_MASTER_ADDRS` is full.

Wait — this creates a subtle ordering problem: a genuinely unexpected source
would be added to `master_addrs_seen` by the population step before the
detection step can see it was absent. The fix: the unexpected-source check must
happen BEFORE the population step pushes the new address into `master_addrs_seen`.

**Revised ordering (critical):**

```
// Step A: check unexpected-source BEFORE populating master_addrs_seen
if is_master_frame(header.control)
    && classify_dnp3_fc(app_fc) == Dnp3FcClass::Control
    && !flow.master_addrs_seen.is_empty()       // expected set is established
    && !flow.master_addrs_seen.contains(&src)   // src is unknown
{
    // Unexpected-source detection fires here (see §2 for emission details)
    detect_unexpected_source(flow, findings, app_fc, dest, src, ts, flow_key);
}

// Step B: (existing) populate master_addrs_seen
if is_master_frame(header.control)
    && !flow.master_addrs_seen.contains(&header.source)
    && flow.master_addrs_seen.len() < MAX_MASTER_ADDRS
{
    flow.master_addrs_seen.push(header.source);
}
```

The unexpected-source check sees `master_addrs_seen` before the new address is
appended, so it correctly identifies the source as absent/unexpected.

**The first master address to send a Control FC establishes the expected set.**
Specifically: if the very first frame processed on a flow is a Control-class FC
from src=0x0001, then `master_addrs_seen` is empty at Step A (condition
`!flow.master_addrs_seen.is_empty()` is false), so no unexpected-source finding
fires. Step B then adds 0x0001. When a subsequent Control FC arrives from
src=0x0099, Step A sees `master_addrs_seen = [0x0001]`, src=0x0099 is not
present, so the unexpected-source finding fires.

This is consistent with HS-W37-002, which expects that an address not in the
expected set fires at count=1 without a threshold.

**No CLI flag required.** The mechanism is self-learning per-flow with no
configuration surface. This is a deliberate product decision: requiring operators
to enumerate all expected DNP3 master addresses per segment via CLI would be
impractical and create a barrier to deployment. See §4 (CONFIG) for the full
analysis.

---

## 2. EMISSION SEMANTICS

### Trigger condition

Emit ONE T1692.001 finding when, at the moment a Control-class FC is being
processed, the following conditions all hold:

1. `is_master_frame(header.control)` is true (DIR=1, master direction).
2. `classify_dnp3_fc(app_fc) == Dnp3FcClass::Control`.
3. `!flow.master_addrs_seen.is_empty()` (expected set is established; first
   master address has been seen on this flow).
4. `!flow.master_addrs_seen.contains(&src)` (current source is NOT in expected
   set).
5. `!flow.unexpected_source_emitted` (one-shot guard — see below).
6. `findings.len() < MAX_FINDINGS` (DoS cap per BC-2.15.022).

The check fires at count=1 — meaning the very first Control FC from an
unexpected source triggers it, regardless of `direct_operate_count`.

### New per-flow guard field

Add one new field to `Dnp3FlowState`:

```rust
/// One-shot guard: T1692.001 unexpected-source finding already emitted for
/// this flow. Set true on first unexpected-source emission; NEVER reset
/// (flow-lifetime guard — same lifetime policy as unsolicited_anomaly_emitted).
/// This is a flow-lifetime guard, NOT a per-window guard: if the attacker
/// changes source address mid-flow, the first occurrence emits once and
/// subsequent occurrences are suppressed. This prevents flooding when an
/// attacker rotates through many source addresses.
pub unexpected_source_emitted: bool,
```

The guard is flow-lifetime (not window-scoped). Rationale: the signal that a
non-authorized master address has sent a Control command is a one-time alert
event for the flow. Multiple unexpected sources on the same flow should not
produce repeated findings (that would reconstitute a flood vector). After the
first emission, the analyst is alerted; further analysis is manual.

### Finding fields (exact)

```
category:         ThreatCategory::Execution
verdict:          Verdict::Likely
confidence:       Confidence::High
summary:          "DNP3 unauthorized control command from unexpected source: \
                   src={src:#06X} is not in expected master set \
                   {master_set} on dest={dest:#06X}"
evidence:         ["FC=0x{app_fc:02X} dest={dest:#06X} src={src:#06X}",
                   "expected_masters={master_set}"]
mitre_techniques: ["T1692.001"]
source_ip:        Some(resolve_master_ip(flow_key))
timestamp:        Some(pcap-relative)
```

Where `{master_set}` is the formatted list of `flow.master_addrs_seen` entries,
e.g. `"[0x0001]"`.

**Confidence is High** (not Medium). Rationale from BC-2.15.010 Invariant 5: an
unauthorized source issuing even ONE control command is the high-value signal —
this is the primary gate. The burst-threshold finding (which fires at count > 10
from a known source) uses Medium because it could be legitimate maintenance. An
entirely unknown source address using a Control FC has no benign explanation in
a correctly-configured DNP3 segment. High confidence is appropriate.

**Summary string:** the summary is DISTINCT from the burst-threshold summary
("DNP3 unauthorized control command burst: {count} control FCs in {elapsed}s
window (threshold {threshold})"). The unexpected-source summary must not echo
the threshold or window elapsed because this check is threshold-independent.
The implementer must ensure the two summaries are textually different so tests
can discriminate between them.

### Interaction with burst guard

The unexpected-source check and the burst check are INDEPENDENT. They share the
same `direct_operate_count` counter (the unexpected source's FC is still counted)
and the same `direct_operate_emitted` one-shot guard. They each have their own
separate one-shot guard:

- `flow.unexpected_source_emitted` — guards the unexpected-source finding.
- `flow.direct_operate_emitted` — guards the burst-threshold finding.

Both can fire on the same flow if: (a) an unexpected source sends 1 Control FC
(triggers unexpected-source finding, sets `unexpected_source_emitted=true`), and
then (b) the count later crosses the burst threshold from the same or any source
(triggers burst finding, sets `direct_operate_emitted=true`). Both findings
would appear in `all_findings`. This is correct: they represent different threat
signals (unauthorized identity vs flood volume).

### Interaction with MAX_MASTER_ADDRS

If `master_addrs_seen` is already at MAX_MASTER_ADDRS (64 entries), a new
unexpected source arriving from a 65th address will NOT be added to
`master_addrs_seen` (existing cap logic). The unexpected-source check will still
fire (the source is not in `master_addrs_seen`) and the finding will be emitted
(if `unexpected_source_emitted` is false). This is correct: an attacker
saturating the master-address table cannot use that saturation to suppress
the unexpected-source detection.

### Interaction with window expiry

`unexpected_source_emitted` is a flow-lifetime guard. The 300s correlation window
expiry (which resets the six windowed correlation fields) does NOT reset it.
The 60s burst window reset (`direct_operate_emitted = false`) does NOT reset it.
These are different concerns.

---

## 3. INTEGRATION POINT

### Location

Add a new associated function `detect_unexpected_source_split` in
`src/analyzer/dnp3.rs`, following the same borrow-split pattern as the existing
`detect_control_class_burst_split` et al.

Signature:

```rust
#[allow(clippy::too_many_arguments)]
fn detect_unexpected_source_split(
    flow: &mut Dnp3FlowState,
    findings: &mut Vec<Finding>,
    app_fc: u8,
    dest: u16,
    src: u16,
    now_ts: u32,
    flow_key: &FlowKey,
)
```

### Call site

In `on_data`, the unexpected-source check must be inserted BEFORE the
`master_addrs_seen` population step and BEFORE the `match classify_dnp3_fc(app_fc)`
dispatch. The call site in `on_data` (after the FIR=1 gate and before the
classification match) becomes:

```rust
// --- Unexpected-source check (BC-2.15.010 Invariant 5) ---
// MUST precede master_addrs_seen population so we can test
// whether src was absent BEFORE it is added to the set.
// Only fires for Control-class FCs from master-direction frames.
if is_master_frame(header.control)
    && classify_dnp3_fc(app_fc) == Dnp3FcClass::Control
    && !flow.master_addrs_seen.is_empty()
    && !flow.master_addrs_seen.contains(&src)
{
    Self::detect_unexpected_source_split(
        flow,
        &mut self.all_findings,
        app_fc,
        dest,
        src,
        ts,
        &flow_key,
    );
}

// --- (existing) master_addrs_seen population ---
if is_master_frame(header.control)
    && !flow.master_addrs_seen.contains(&header.source)
    && flow.master_addrs_seen.len() < MAX_MASTER_ADDRS
{
    flow.master_addrs_seen.push(header.source);
}
```

IMPORTANT: the existing `master_addrs_seen` population block currently runs at
~line 446, which is OUTSIDE the `if frame_len >= 13 && has_user_data(...)` gate
(it runs for every valid gate-passed frame, not just FIR=1 user-data frames).
The unexpected-source check, by contrast, should only fire for FIR=1 Control-class
FCs where we have extracted an app_fc. Therefore the unexpected-source call goes
INSIDE the `if frame_len >= 13 && has_user_data(...) { if transport_is_fir(...) {`
block, but logically BEFORE the master_addrs_seen push. This means we need to
reorder the master_addrs_seen population: move the push into the FIR=1 block as
well, or duplicate the population logic inside the FIR=1 block for the specific
case of Control FCs.

**Simplest correct approach:** keep the existing master_addrs_seen push at its
current location (it covers all master-direction frames for the address-set
tracking BC-2.15.016 PC5-6), but ALSO perform the unexpected-source check
INSIDE the FIR=1 + Control branch by checking whether `src` was in
`master_addrs_seen` BEFORE this frame's population would have added it. Since
the population at ~line 446 runs before the FIR=1 gate at ~line 458, by the
time we reach the Control branch, `master_addrs_seen` already contains `src`
if this was its first frame (not unexpected). This means a first-ever Control FC
from a new master will NOT trigger the unexpected-source check — which is the
desired behavior.

To summarize: with the existing ordering (population BEFORE FIR=1 gate), the
logic is naturally correct:

- First Control FC from src=0x0001: master_addrs_seen was empty, population
  pushes 0x0001 before the FIR=1 gate runs. Inside the Control branch,
  `master_addrs_seen.contains(&0x0001)` is true. No unexpected-source finding.
- Second Control FC from src=0x0001: already in set. No unexpected-source finding.
- First Control FC from src=0x0099: `master_addrs_seen = [0x0001]`, population
  adds 0x0099 (if < MAX_MASTER_ADDRS). Inside Control branch,
  `master_addrs_seen.contains(&0x0099)` is now TRUE (just added). No finding.
  WRONG.

This ordering is broken. The population step destroys the "was absent" signal
before the check runs. The fix is mandatory:

**REQUIRED CHANGE to on_data ordering:**

Move `detect_unexpected_source_split` BEFORE the existing master_addrs_seen
population block (currently at ~line 446). This means the check must be done
inside the `if frame_len >= 13 && has_user_data(...)` FIR=1 block but with a
pre-check that looks at `master_addrs_seen` before the population at line ~446
has run for this frame's source. The practical solution:

1. Relocate the unexpected-source check to before the master_addrs_seen push
   by restructuring the code block order. Specifically:
   - FIRST: if FIR=1 and Control-class FC and master-direction and src not in
     master_addrs_seen and master_addrs_seen not empty → call
     `detect_unexpected_source_split`.
   - THEN: push src into master_addrs_seen (existing population step).
   - THEN: proceed with burst detection, pending-request insertion, etc.

The cleanest implementation moves the master_addrs_seen population step to
INSIDE the FIR=1 user-data gate, interleaved with the detection logic, rather
than at the top of the valid-frame block. The population for non-FIR-1 or
non-user-data master frames can stay at the outer scope to preserve STORY-107's
full address-tracking semantics.

**Concrete restructured on_data snippet (pseudocode):**

```
// 1. (keep at outer scope) population for all master-direction frames:
if is_master_frame(h.control)
    && !flow.master_addrs_seen.contains(&h.source)
    && flow.master_addrs_seen.len() < MAX_MASTER_ADDRS
{
    // NOTE: only push here for NON-Control FCs or non-FIR-1 frames.
    // For FIR=1 Control FCs the push is deferred to after the
    // unexpected-source check (see inside FIR=1 block below).
    // ... but this split is complex.
}
```

Alternative: the simplest correct approach is a snapshot check:

```rust
// Inside the FIR=1 + user-data gate, BEFORE the classify dispatch:
let src_was_known = flow.master_addrs_seen.contains(&src);
let expected_set_established = !flow.master_addrs_seen.is_empty();

// (existing) master_addrs_seen population runs here (unchanged):
if is_master_frame(header.control)
    && !flow.master_addrs_seen.contains(&header.source)
    && flow.master_addrs_seen.len() < MAX_MASTER_ADDRS
{
    flow.master_addrs_seen.push(header.source);
}

// Then: unexpected-source check using the PRE-PUSH snapshot:
match classify_dnp3_fc(app_fc) {
    Dnp3FcClass::Control => {
        if is_master_frame(header.control)
            && expected_set_established
            && !src_was_known
        {
            Self::detect_unexpected_source_split(
                flow, &mut self.all_findings,
                app_fc, dest, src, ts, &flow_key,
            );
        }
        // ... existing broadcast, pending_requests, burst detection
    }
    // ...
}
```

This is the RECOMMENDED approach. Take a bool snapshot of whether `src` was
already known BEFORE the population runs, then use the snapshot in the Control
branch. No restructuring of the population step is required. The snapshot
variables are stack-local and cheap.

### Ordering relative to broadcast and burst detection

Inside the `Dnp3FcClass::Control` arm, the emission ordering must be:

1. Broadcast anomaly check (existing, BC-2.15.018) — fires for broadcast DEST.
2. Unexpected-source check (NEW, BC-2.15.010 Inv 5) — fires for unknown SRC.
3. Pending-request insertion (existing, BC-2.15.014).
4. Burst detection (existing, BC-2.15.010 volumetric).

This ordering ensures that if both broadcast and unexpected-source conditions
hold simultaneously, the broadcast finding appears first in `all_findings` for
that frame. The unexpected-source finding appears second. The burst finding
(if count > threshold in the same frame) appears third. This matches BC-2.15.013
co-emission ordering (most-specific direct finding first).

---

## 4. CONFIG / CLI SURFACE

**No new CLI flag is required and no new BC is needed to start implementation.**

Rationale:
- BC-2.15.010 Invariant 5 specifies "comparing `src` against an allowlist or
  expected master-address set" — the "or expected master-address set" path
  (first-seen learning) is fully specified and requires no external configuration.
- There is no BC-2.15.0xx in the corpus for a `--dnp3-expected-master` flag.
- HS-W37-002 sets up no CLI flag; it expects the behavior from flow state alone.
- ADR-007 Decision 6 covers only `--dnp3-direct-operate-threshold`.
- ADR-007 Open Items for F3/Human Decision do not mention an expected-master flag.

The `master_addrs_seen` field (STORY-107, BC-2.15.016 PC5-6) is the data
structure that implements the expected-master set. It was designed for exactly
this purpose, per ADR-007 Decision 4.

**No product-owner involvement is required before implementation begins.**

Optional future enhancement (not blocking): a `--dnp3-expected-master <addr>`
repeatable flag that pre-seeds `master_addrs_seen` in `Dnp3Analyzer::new()`,
allowing operators in well-known topologies to avoid the first-seen learning
period. This would require a new BC (e.g., BC-2.15.025) and a PO sign-off.
It is out of scope for this remediation cycle.

---

## 5. BC OPEN-QUESTION STATUS

**BC-2.15.010 Invariant 5 is sufficiently specified to implement without PO
involvement.** The [F2-GATE] marker in BC-2.15.010 only refers to the
`direct_operate_threshold` default value (which is already pinned at 10 per
holdout frontmatter `direct_operate_threshold_default: 10`). There is no [F2-GATE]
on the unexpected-source mechanism itself.

The phrase "allowlist or expected master-address set" is the only ambiguity, and
it is resolved by this adjudication in favor of the first-seen-learning mechanism
(matching HS-W37-002's setup, matching `master_addrs_seen` design intent, and
matching the absence of a CLI-flag BC).

**One BC update is required post-implementation:** BC-2.15.010 should be amended
to add:
- EC-009: "First Control FC from a new master address (master_addrs_seen empty)
  → no unexpected-source finding; address added to expected set."
- EC-010: "Control FC from a second distinct master address (master_addrs_seen
  has ≥1 entry, src not present) → one T1692.001 unexpected-source finding at
  count=1; `unexpected_source_emitted = true`."

The implementer may author this BC amendment as part of the story; PO review
is sufficient (not a blocking gate).

---

## 6. TEST / HOLDOUT REQUIREMENTS

### Unit tests to add (STORY-108 scope)

The following unit tests must be added to the existing test module in
`src/analyzer/dnp3.rs`:

**test_unexpected_source_fires_at_count_1**
- Setup: fresh `Dnp3Analyzer`, threshold=10. Deliver one Control FC from
  src=0x0001 (establishes expected set). Deliver one Control FC from
  src=0x0099.
- Assert: `all_findings.len() == 1`. Finding has `mitre_techniques=["T1692.001"]`,
  `confidence == Confidence::High`, summary contains "unexpected source".
- Assert: `flow.direct_operate_count == 2` (both FCs counted by burst counter).
- Assert: `flow.direct_operate_emitted == false` (burst guard NOT set; count=2,
  threshold=10, 2>10 is false).

**test_unexpected_source_independent_of_threshold**
- Setup: `direct_operate_threshold=10`. Deliver 9 Control FCs from src=0x0001
  (count=9, within window, no burst finding yet). Deliver 1 Control FC from
  src=0x0099 (count=10).
- Assert: exactly ONE finding in `all_findings`: the unexpected-source T1692.001
  (NOT the burst-threshold T1692.001; `10 > 10` is false).
- Assert: summary of finding contains "unexpected source", NOT "burst".

**test_unexpected_source_one_shot_guard**
- Setup: deliver Control FCs from src=0x0001 (establish expected set), then
  deliver 3 Control FCs from src=0x0099.
- Assert: exactly ONE unexpected-source finding (first FC from 0x0099 triggers,
  subsequent two are suppressed by `unexpected_source_emitted = true`).

**test_first_master_is_expected**
- Setup: fresh flow. Deliver first Control FC from src=0x0001 (no prior frames).
- Assert: NO finding. `flow.master_addrs_seen == [0x0001]`.
  `flow.unexpected_source_emitted == false`.

**test_unexpected_source_and_burst_both_fire**
- Setup: threshold=3. Deliver 1 Control FC from src=0x0001 (establishes set).
  Deliver 4 Control FCs from src=0x0099 (total direct_operate_count=5).
- Assert: `all_findings` contains two T1692.001 findings:
  - finding[i] has summary containing "unexpected source" (fired at count=2,
    i.e., the first FC from 0x0099).
  - finding[j] has summary containing "threshold 3" (fired when count reached 4,
    exceeding threshold=3).
  Both guards are set: `unexpected_source_emitted=true`,
  `direct_operate_emitted=true`.

### Holdout confirmation

**HS-W37-002 will be satisfied** by this design. The holdout scenario delivers
"a Control-class FC from source address 0x0099 which is NOT in the
expected/allowlisted master-address set for this flow" at count=1. The
implementation above fires the unexpected-source check on the first Control FC
from an address not in `master_addrs_seen`, at count=1, regardless of threshold.
The holdout's note "If the implementation gates the unexpected-source check
behind the same `direct_operate_count > threshold` condition, this test will
fail" is directly addressed: the new `detect_unexpected_source_split` branch is
NOT gated by the threshold condition.

HS-W37-001 (burst threshold at boundary) is unaffected by this change — the
burst detection path is unchanged.

---

## 7. VP IMPACT

**No VP change required.** This is detection logic in the effectful shell, not
pure-core parse/classify logic. VP-023 covers the pure-core functions
(`parse_dnp3_dl_header`, `classify_dnp3_fc`, `is_valid_dnp3_frame_header`,
`compute_dnp3_frame_len`). The unexpected-source check reads from
`master_addrs_seen` (Vec membership test) and pushes a finding — these are
effectful shell operations already classified as "test sufficient" in BC-2.15.010
purity classification. VP-023 Sub-B verifies `classify_dnp3_fc` totality and is
unchanged.

---

## 8. REMEDIATION SEQUENCE

| Step | Action | Owner | Blocking? |
|------|--------|-------|-----------|
| 1 | Add `unexpected_source_emitted: bool` field to `Dnp3FlowState` with doc comment. | Implementer | Yes — all subsequent steps depend on this |
| 2 | Add `src_was_known` + `expected_set_established` snapshot variables in `on_data` inside the FIR=1 gate, capturing state BEFORE the master_addrs_seen push runs (see §3). | Implementer | Yes |
| 3 | Add `detect_unexpected_source_split` associated function (see §2 for full emission spec). The function signature mirrors `detect_control_class_burst_split`. | Implementer | Yes |
| 4 | Insert call to `detect_unexpected_source_split` in the `Dnp3FcClass::Control` arm of `on_data`, AFTER the broadcast check, BEFORE pending-request insertion, using the snapshots from Step 2. | Implementer | Yes |
| 5 | Add the five unit tests listed in §6. All must pass before PR opens. | Implementer | Yes (gate for PR) |
| 6 | Run `cargo test --all-targets` and `cargo clippy --all-targets -- -D warnings`. Both must be clean. | Implementer | Yes |
| 7 | Amend BC-2.15.010 to add EC-009 and EC-010 (see §5). PO reviews. | Implementer drafts / PO confirms | No (can happen post-merge as follow-up, but should happen in the same story if feasible) |
| 8 | Confirm HS-W37-002 passes in the Phase 4 holdout evaluation run. | Evaluator | Yes (Phase 4 gate) |

No product-owner decision is required before Steps 1–6 begin. The mechanism is
fully adjudicated. Step 7 is a spec housekeeping amendment that can be done
concurrently with or immediately after implementation.

---

## Appendix: Summary of Existing Code Confirmed-Not-Implementing This

| Location | What it does | Gap |
|----------|-------------|-----|
| `src/analyzer/dnp3.rs` line ~446–451 | Populates `master_addrs_seen` for master-direction frames | Population only; never read in detection branches |
| `src/analyzer/dnp3.rs` `detect_control_class_burst_split` (~615–673) | Only checks `count > threshold` | Does not consult `master_addrs_seen` at all |
| `src/analyzer/dnp3.rs` `Dnp3Analyzer` struct (~267–276) | Has `direct_operate_threshold` | No `expected_master_addrs` field or similar |
| `src/cli.rs` | Has `--dnp3-direct-operate-threshold` | No `--dnp3-expected-master` flag |

---

---

# REVISION 2 (post-slice-A)

**Revision date:** 2026-06-12
**Produced by:** architect (adversarial adjudication)
**Supersedes:** Revision 1 (§§1–8 above) on all topics where this revision conflicts.
**Slice-A findings resolved:** F-A-001 [BLOCKER], F-A-002 [BLOCKER], F-A-003 [MAJOR],
F-A-004 [MAJOR], F-A-005 [MAJOR], plus MINOR items on summary string, MAX_MASTER_ADDRS
full case, rotation case, and is_non_dnp3 skip.

The original directive (Revision 1) is NOT sound to build. This section replaces it
completely for implementation purposes. Read this section in full before writing a
single line of code.

---

## R2-1. DIR-BIT BUG (F-A-001) — BLOCKER — Pre-existing in STORY-107

### Finding

`is_master_frame` at `src/analyzer/dnp3.rs:1481–1484` reads:

```rust
// BC-2.15.016 postcondition 5 (PC5): DIR bit is bit 4 (mask 0x10). DIR=1 → master.
control & 0x10 != 0
```

This is WRONG. In DNP3 (IEEE 1815) the link-layer control byte bit layout is:

```
Bit 7: DIR  — direction (1 = master → outstation)
Bit 6: PRM  — primary (1 = primary station)
Bit 5: FCB  — frame count bit
Bit 4: FCV  — frame count valid (master→outstation) or DFC (outstation→master)
Bits 3–0: FC — link function code
```

DIR is bit 7, mask 0x80. Bit 4 (mask 0x10) is FCV/DFC, not DIR.

### Impact

The canonical master frame used in every BC byte vector and holdout scenario is
`CTRL=0xC4`:

```
0xC4 = 1100 0100
  bit7=1 (DIR=1, master direction)  ← correct signal
  bit6=1 (PRM=1)
  bit5=0 (FCB=0)
  bit4=0 (FCV=0)                    ← NOT the DIR bit
  nibble=0100 (FC=UNCONFIRMED_USER_DATA)
```

Under the BUGGY mask: `0xC4 & 0x10 = 0x00` → `is_master_frame(0xC4) = false`.
This means `master_addrs_seen` is NEVER populated by canonical master frames.
The entire F-001 unexpected-source feature would be inert on real and holdout traffic.

The F5 RED tests DODGED this bug by using a non-canonical `CTRL=0xD4`:
`0xD4 & 0x10 = 0x10 != 0` (FCV bit, not DIR bit). This made the tests pass the buggy
check accidentally while being semantically wrong about which bit is DIR.

### Decision

**CONFIRMED: DNP3 link-layer DIR bit is bit 7 (mask 0x80). The fix is mandatory.**

```rust
// CORRECTED (BC-2.15.016 PC5): DIR bit is bit 7 (mask 0x80). DIR=1 → master.
control & 0x80 != 0
```

Verification: `0xC4 & 0x80 = 0x80 != 0` → `is_master_frame(0xC4) = true`. Canonical
master frames now correctly populate `master_addrs_seen`. The feature is no longer inert.

### BC-2.15.016 PC5 Correction Required (Product-Owner action)

The current BC-2.15.016 Postcondition 5 text and the `is_master_frame` doc-comment both
say "DIR bit is bit 4 (mask 0x10)". This is incorrect and must be corrected by the
Product Owner:

**Current text (WRONG):**
```
5. When a frame with DIR=1 (master-direction, `is_master_frame(control)`) is observed,
   `src` is appended to `flow.master_addrs_seen` if not already present.
```
(with the comment in source reading "DIR bit is bit 4 (mask 0x10)")

**Corrected text (product-owner must apply):**
```
5. When a frame with DIR=1 (master-direction, `is_master_frame(control)`) is observed,
   `src` is appended to `flow.master_addrs_seen` if not already present.
   IMPLEMENTATION NOTE: DIR is bit 7 of the link-control byte (mask 0x80),
   per IEEE 1815 DNP3 link-layer framing. `is_master_frame(control)` tests
   `control & 0x80 != 0`. Mask 0x10 is FCV/DFC (bit 4), NOT DIR.
```

The Architecture Anchors section of BC-2.15.016 should also note the corrected mask.

### STORY-107 Test Impact

The following STORY-107 test in `tests/dnp3_flow_state_tests.rs` asserts the BUGGY
behavior and MUST be corrected:

**`test_BC_2_15_016_is_master_frame_dir_bit`** (line ~841):

This test currently asserts:
- `is_master_frame(0x10) == true` — bit 4 only set; under corrected mask `0x10 & 0x80 = 0` → FALSE. Test must be updated.
- `is_master_frame(0xD4) == true` — under corrected mask `0xD4 & 0x80 = 0x80` → TRUE. Assertion is accidentally correct but for wrong reasons; the comment must be corrected.
- `is_master_frame(0xFF) == true` — under corrected mask `0xFF & 0x80 != 0` → TRUE. Correct.
- `is_master_frame(0x00) == false` — correct under both masks.
- `is_master_frame(0x04) == false` — under corrected mask `0x04 & 0x80 = 0` → FALSE. Correct.
- `is_master_frame(0xEF) == false` — the test comment says "0xEF & 0x10 == 0"; under corrected mask `0xEF & 0x80 = 0x80 != 0` → TRUE (not false!). The 0xEF assertion INVERTS under the corrected mask. This assertion must be removed or replaced.

**Required corrections to `test_BC_2_15_016_is_master_frame_dir_bit`:**

1. Replace `is_master_frame(0x10)` assertion with a DIR=1 canonical frame: use `0xC4`
   (DIR=1, PRM=1, FCV=0, FC=4). Assert `is_master_frame(0xC4) == true` with comment
   "control=0xC4 (canonical master frame: DIR=1 bit7 set) must return true".
2. Keep `is_master_frame(0xD4) == true` but correct the comment: "control=0xD4
   (DIR=1 bit7 set; also PRM+FCV bits set) must return true".
3. Keep `is_master_frame(0xFF) == true`.
4. Keep `is_master_frame(0x00) == false` and `is_master_frame(0x04) == false`.
5. REMOVE or replace `is_master_frame(0xEF) == false`. Replace with a genuine
   DIR=0 frame with bit 7 clear but other high bits set, e.g. `0x44`
   (bit7=0, bit6=1: PRM=1, DIR=0 → outstation direction):
   `0x44 & 0x80 = 0` → false. Assert `is_master_frame(0x44) == false` with
   comment "control=0x44 (DIR=0, PRM=1, outstation direction) must return false".
6. Update the test doc-comment: remove all references to "bit 4" and "0x10";
   replace with "bit 7" and "0x80".

**`build_master_frame` helper** (`tests/dnp3_flow_state_tests.rs` line ~90–95) uses
`CTRL=0xD4` with comment "DIR+PRM bits: 0xD4" and "0xD4 = 1101 0100: DIR(1) PRM(1)...".
Under the corrected mask, `0xD4 & 0x80 = 0x80` — this frame is still a master frame.
The behavior of the test is UNCHANGED (master_addrs_seen is still populated). However
the comment is WRONG: it says "DIR(1) PRM(1) FCB(0) FCV(0)" but bit 4 of 0xD4 is 1
(FCV=1, not 0). The comment must be corrected:

```rust
/// Build a master-direction frame: control has DIR bit set (bit 7, mask 0x80).
/// Uses nibble 0x04 (UNCONFIRMED_USER_DATA) with DIR+PRM+FCV bits: 0xD4.
fn build_master_frame(dest: u16, src: u16) -> Vec<u8> {
    // 0xD4 = 1101 0100: DIR(1, bit7) PRM(1, bit6) FCB(0, bit5) FCV(1, bit4) FC(0100=UNCONF_USER_DATA)
    // DIR=1 because 0xD4 & 0x80 = 0x80 != 0. Note: FCV is bit4, not DIR.
    build_frame(5, dest, src, 0xD4)
}
```

The `test_master_addrs_cap_at_64` test uses `build_master_frame` → `CTRL=0xD4`. Under
the corrected mask this test CONTINUES TO WORK correctly because `0xD4 & 0x80 = 0x80`
(DIR=1 is still satisfied). No behavioral change; comment fix only.

**F5 RED test `build_control_frame`** in `tests/dnp3_f5_remediation_tests.rs` (line ~80)
uses `CTRL=0xD4` and has an extensive comment attributing the correct behavior to the
BUGGY mask (`control & 0x10 != 0`). Under the corrected mask this frame STILL satisfies
`is_master_frame` (`0xD4 & 0x80 = 0x80`), so all F5 RED tests continue to function
correctly. However, the comment explaining the CTRL choice must be corrected:

The line:
```
///   `is_master_frame` checks bit 4 (mask 0x10) per BC-2.15.016 PC5:
///     `control & 0x10 != 0`  (see src/analyzer/dnp3.rs `is_master_frame`).
///   0xD4 & 0x10 = 0x10 ≠ 0 → is_master_frame=true.
```
must be replaced with:
```
///   `is_master_frame` checks bit 7 (mask 0x80) per BC-2.15.016 PC5 (corrected):
///     `control & 0x80 != 0`  (see src/analyzer/dnp3.rs `is_master_frame`).
///   0xD4 & 0x80 = 0x80 ≠ 0 → is_master_frame=true.
```
The NOTE about existing tests using CTRL=0xC4 must also be corrected: CTRL=0xC4 with the
CORRECTED mask now DOES satisfy `is_master_frame` (`0xC4 & 0x80 = 0x80 != 0`), so the
burst-threshold tests would also see master_addrs_seen populated. The NOTE should be
updated to: "CTRL=0xC4 with the corrected mask also satisfies is_master_frame; 0xD4 is
used here to maintain consistency with existing STORY-107 tests."

**NEW canonical test helper required (R2-§6):** the test suite must add a
`build_canonical_master_control_frame` helper using `CTRL=0xC4` (the byte vector from
BC-2.15.010's Canonical Test Vectors section). See R2-§6 for the spec.

### Summary of is_master_frame change

| Mask | `is_master_frame(0xC4)` | `is_master_frame(0xD4)` | `is_master_frame(0x10)` |
|------|------------------------|------------------------|------------------------|
| 0x10 (BUGGY) | false (WRONG) | true | true |
| 0x80 (CORRECT) | true (CORRECT) | true | false |

The CRITICAL difference: canonical `CTRL=0xC4` master frames now correctly return `true`.

---

## R2-2. SOURCE/DIRECTION SEMANTICS AND REDUNDANT-MASTER FALSE POSITIVE (F-A-002) — BLOCKER

### Source Address Semantics

**DECISION: `src` used in the unexpected-source check is the DNP3 link-layer SOURCE
field (`header.source`, type `u16`), NOT an IP address.**

This is the same `header.source` field that populates `master_addrs_seen`. It is
decoded as a 16-bit little-endian value from bytes [6..8] of the link-layer frame
(after the START, LENGTH, CTRL, and DEST fields). The `source_ip` field in the emitted
Finding is a SEPARATE concept — it is resolved from the IP-layer FlowKey via
`Self::resolve_master_ip(flow_key)` and represents the TCP/IP transport endpoint, not
the DNP3 application address. These two values are orthogonal: `src` (DNP3 link-layer
address) identifies the DNP3 station; `source_ip` (IPv4/IPv6) identifies the host. The
unexpected-source check operates on DNP3 link-layer addresses only.

### Direction Restriction

The unexpected-source check applies ONLY to frames where `is_master_frame(header.control)`
is true (i.e., DIR=1, master direction, bit 7 set — using the corrected 0x80 mask).
This ensures:
- Outstation responses (DIR=0) do NOT trigger the check.
- Outstation source addresses are NOT added to `master_addrs_seen`.
- Only master-direction Control FCs (which can issue commands) are subject to source
  authorization. This is consistent with the corrected `is_master_frame` semantics and
  with BC-2.15.016 PC5's intent.

### Redundant-Master False Positive Decision

**DECISION: The v1 first-seen-master learning mechanism is ACCEPTED with a documented
limitation. Redundant-master sites WILL experience false positives.**

Rationale: BC-2.15.010 Invariant 5 says "non-allowlisted SOURCE address" without
qualifying the multi-master case. From a security posture perspective, an additional
master address issuing Control FCs that was not present at flow establishment IS
anomalous — even if it may be a legitimate redundant SCADA master in some topologies.
The acceptable-FP rate for this feature is low (one finding per flow, due to the
one-shot guard), and the product does not have a configured-allowlist mechanism in v1.

The escape hatch for operators at redundant-master sites is the future
`--dnp3-expected-master` flag (DRIFT item, not currently in scope). Until that flag
exists, operators at known-redundant-master sites can acknowledge the alert manually.

**This decision is documented as EC-011 in BC-2.15.010 (product-owner action required).**

EC-011 text for the product owner to insert:

```
| EC-011 | Redundant-SCADA-master topology: two legitimate master addresses (e.g.,
  0x0001 primary + 0x0002 backup) both issue Control FCs on the same flow.
  After 0x0001 establishes the expected set, a Control FC from 0x0002 triggers
  the unexpected-source finding (T1692.001, Confidence=High) at count=1.
  This is a conscious false-positive: the product has no configured-allowlist
  mechanism in v1 to suppress this. The one-shot flow-lifetime guard
  (unexpected_source_emitted=true) limits FP volume to one finding per flow.
  Operators at redundant-master sites should acknowledge this finding class;
  the future --dnp3-expected-master allowlist flag (DRIFT) will be the escape
  hatch. This edge case is an accepted limitation documented in the v1 design. |
```

---

## R2-3. SINGLE NORMATIVE SNAPSHOT ORDERING (F-A-003) — MAJOR

### The Ordering Problem

Revision 1 gave three contradictory orderings in §§1, 2, and 3. This revision
collapses to ONE normative version. All other orderings in Revision 1 are void.

### The Code Structure (confirmed from source)

In `src/analyzer/dnp3.rs`, the relevant code currently executes in this order within
`on_data`:

```
Line 446–451: master_addrs_seen push (outer scope — runs for all valid gate-passed frames)
Line 458+:    if frame_len >= 13 && has_user_data(header.control) {
Line 460:         if transport_is_fir(transport_octet) {
Line 461:             let app_fc = flow.carry[12];
Line 488:             match classify_dnp3_fc(app_fc) {
Line 489:                 Dnp3FcClass::Control => { ... }
```

The `master_addrs_seen` push at line 446–451 runs BEFORE the FIR=1 gate. This means
by the time the `Dnp3FcClass::Control` arm executes, the push has ALREADY run for
this frame's source. A naive "check inside the Control arm" would see the source
already added — making all first-ever sources appear as "known" and the check inert.

### The Single Correct Approach: Pre-Push Snapshot

**NORMATIVE ORDERING — this is the ONLY correct implementation:**

Immediately after `parse_dnp3_dl_header` succeeds AND after the frame passes the
validity gate (before line 446, inside the `if frame_len >= 13` gate that wraps
the current line-441 comment), capture two boolean snapshots:

```rust
// --- Snapshot for unexpected-source check (BC-2.15.010 Invariant 5) ---
// MUST be captured BEFORE the master_addrs_seen push below.
// These snapshots reflect the pre-push state of master_addrs_seen for THIS frame.
let src_was_known = flow.master_addrs_seen.contains(&header.source);
let expected_set_established = !flow.master_addrs_seen.is_empty();
```

**EXACT INSERTION POINT:** These two lines must appear between the frame validity
confirmation (after the `continue` on malformed) and the `master_addrs_seen` push at
line 446. Specifically: after line 442 (`flow.frame_count += 1`) and BEFORE line 446
(`if is_master_frame(header.control)`). In the current line numbering:

```
line 442: flow.frame_count += 1;
          ↑ ← INSERT snapshot here (after frame_count increment) ↑
line 444: // BC-2.15.016 PC5–6: master-direction (DIR=1) frame → record ...
line 446: if is_master_frame(header.control)
line 447:     && !flow.master_addrs_seen.contains(&header.source)
line 448:     && flow.master_addrs_seen.len() < MAX_MASTER_ADDRS
line 449: {
line 450:     flow.master_addrs_seen.push(header.source);
line 451: }
```

The `master_addrs_seen` push at lines 446–451 remains UNCHANGED in its location.
After it runs, the snapshots correctly reflect whether `src` was in the set
BEFORE this frame was processed.

### Use of Snapshots in the Control Arm

Inside `Dnp3FcClass::Control`, the unexpected-source check uses the PRE-PUSH snapshots:

```rust
Dnp3FcClass::Control => {
    // (existing) broadcast anomaly check
    if is_broadcast_destination(dest) {
        Self::detect_broadcast_anomaly(...);
    }

    // NEW: unexpected-source check (BC-2.15.010 Invariant 5)
    // Uses pre-push snapshots: src_was_known and expected_set_established.
    // is_master_frame uses corrected 0x80 mask.
    if is_master_frame(header.control)
        && expected_set_established
        && !src_was_known
    {
        Self::detect_unexpected_source_split(
            flow,
            &mut self.all_findings,
            app_fc,
            dest,
            src,
            ts,
            &flow_key,
        );
    }

    // (existing) pending_requests insertion
    if app_fc != 0x06 { ... }

    // (existing) burst detection
    Self::detect_control_class_burst_split(...);
}
```

**No other ordering is permitted.** The Revision 1 "Step A before population /
Step B population" alternative from §1 and the "simplest correct approach" alternative
from §3 that duplicates logic inside the FIR=1 block are both superseded by this
snapshot approach. The snapshot approach requires zero restructuring of the
`master_addrs_seen` push and is the minimal-diff implementation.

### Why `src_was_known` uses `header.source` not `src`

`src` is declared at line 470 (`let src = header.source;`) INSIDE the
`if frame_len >= 13 && has_user_data(header.control)` gate, which is at line 458.
The snapshot must be captured BEFORE line 446, which is OUTSIDE that inner gate.
Therefore the snapshot uses `header.source` directly (same value, available in
outer scope). When `src` is declared at line 470, the snapshot is already computed.

---

## R2-4. FALL-THROUGH INVARIANT (F-A-004) — MAJOR

### Statement

**INVARIANT: The unexpected-source emission in `detect_unexpected_source_split` MUST
NOT early-return, `continue`, or short-circuit out of the `Dnp3FcClass::Control` arm.**

After `detect_unexpected_source_split` returns (whether it emitted a finding or not),
execution MUST continue to:

1. The pending-request insertion: `if app_fc != 0x06 { Self::insert_pending_request(...) }`
2. The burst detection: `Self::detect_control_class_burst_split(...)`

This is a hard invariant. Consequences of violating it:

- Any test that asserts `flow.direct_operate_count == 2` after one establishing frame
  and one unexpected-source frame would fail (count would be 1, not 2).
- The `test_unexpected_source_and_burst_both_fire` test would become unsatisfiable:
  the burst check must also run when an unexpected source sends multiple Control FCs,
  so that when the count eventually exceeds the threshold, the burst finding fires.
- Pending-request tracking (BC-2.15.014) would be skipped for unexpected-source frames,
  violating BC-2.15.014 which requires ALL Control-class FCs (not just from expected
  sources) to be tracked.

**Implementation rule:** `detect_unexpected_source_split` is a detection side-effect
function that pushes a finding and sets a guard. It does NOT alter control flow.
The call site in the `Dnp3FcClass::Control` arm is a plain `if` statement (no `?`,
no `return`, no `break`). All subsequent statements in the arm execute unconditionally.

---

## R2-5. HS-W37-002 AMENDMENT (F-A-005) — MAJOR

### The Problem

HS-W37-002 as currently written (wave-35-39-holdout.md line 323) says:

```
Scenario: A Control-class FC arrives from source address 0x0099 which is NOT in the
expected/allowlisted master-address set for this flow. Count=1 (first occurrence).
Default threshold=10.
```

This description contains NO establishing frame. Under the first-seen-learning mechanism,
if the ONLY Control FC on the flow is from src=0x0099, then:
- `expected_set_established = !flow.master_addrs_seen.is_empty()` → FALSE (set is empty)
- The unexpected-source check condition is NOT met
- 0x0099 is added to `master_addrs_seen` (establishes the set)
- No finding is emitted

A literal single-frame reading of HS-W37-002 would make the holdout FAIL even with a
correct implementation.

### REQUIRED AMENDMENT

The Product Owner MUST amend HS-W37-002 to pin the exact two-frame sequence. The amended
text must replace the current Scenario paragraph as follows:

---

**Amended HS-W37-002 Scenario (replace current Scenario paragraph):**

```
Scenario: Two-frame sequence on a fresh flow. Default threshold=10.

Frame 1 (establishing frame): A Control-class FC (FC=0x05, DIRECT_OPERATE) arrives
from src=0x0001, dest=0x0003. This is the first master-direction Control FC on the
flow. Expected set is empty at the start of frame 1 processing. After frame 1:
master_addrs_seen=[0x0001], direct_operate_count=1. No finding emitted.

Frame 2 (unexpected-source frame): A Control-class FC (FC=0x05, DIRECT_OPERATE)
arrives from src=0x0099, dest=0x0003. At the start of frame 2 processing:
master_addrs_seen=[0x0001] (established), src=0x0099 is NOT present.
After frame 2: one T1692.001 finding emitted with confidence=High, summary containing
"unexpected source" and "0x0099". direct_operate_count=2 (both FCs counted).
direct_operate_emitted=false (count=2, threshold=10, 2 > 10 = false).
unexpected_source_emitted=true.
```

**Assertion (replace current Assertion paragraph):**

```
1. After frame 1: all_findings.len() == 0. master_addrs_seen == [0x0001].
2. After frame 2: all_findings.len() == 1.
   finding[0].mitre_techniques == vec!["T1692.001"].
   finding[0].confidence == Confidence::High.
   finding[0].summary contains "unexpected source".
   finding[0].summary contains "0x0099".
   finding[0].summary contains "0x0003".
   flow.direct_operate_count == 2.
   flow.direct_operate_emitted == false.
   flow.unexpected_source_emitted == true.
```

---

The NOTE for evaluators should be updated to reference the corrected is_master_frame
mask: "Frames must use CTRL=0xC4 or any control byte with bit 7 set (DIR=1) for
is_master_frame to return true with the corrected 0x80 mask."

---

## R2-6. COMPLETE REVISED TEST LIST AND HELPER SPECS

### Canonical 0xC4 Test Helper Requirement

Every test that verifies unexpected-source behavior MUST use a helper function that
builds a frame with `CTRL=0xC4` OR explicitly documents that `CTRL=0xD4` is used and
explains why it also satisfies `is_master_frame` with the 0x80 mask. A dedicated
canonical helper is required to make the test suite self-documenting:

```rust
/// Build a canonical DNP3 master Control-class frame matching the BC-2.15.010
/// Canonical Test Vector byte sequence.
///
/// CTRL=0xC4: DIR=1 (bit7, mask 0x80), PRM=1 (bit6), FCB=0 (bit5), FCV=0 (bit4),
///            FC=0x04 (UNCONFIRMED_USER_DATA, lower nibble).
/// 0xC4 & 0x80 = 0x80 != 0 → is_master_frame=true (corrected mask).
/// 0xC4 & 0x0F = 0x04 → has_user_data=true.
///
/// This matches the holdout HS-W37-002 annotated frame:
///   "CTRL=0xC4 (DIR=1, PRM=1, FCV=0, link-FC=4)"
fn build_canonical_master_control_frame(app_fc: u8, dest: u16, src: u16) -> Vec<u8> {
    // LENGTH=8 → frame_len=15 (5 header + 8 data + 2 block CRC)
    let mut frame = vec![0u8; 15];
    frame[0] = 0x05; // START1
    frame[1] = 0x64; // START2
    frame[2] = 8;    // LENGTH
    frame[3] = 0xC4; // CTRL: DIR=1(bit7), PRM=1(bit6), FCV=0(bit4), FC=4(UNCONF_USER_DATA)
    let [dl, dh] = dest.to_le_bytes();
    frame[4] = dl; frame[5] = dh;
    let [sl, sh] = src.to_le_bytes();
    frame[6] = sl; frame[7] = sh;
    // bytes 8–9: header CRC placeholder (0x00)
    frame[10] = 0xC0; // transport: FIR=1 (bit6), FIN=1 (bit7)
    frame[11] = 0x00; // app control
    frame[12] = app_fc; // application function code
    // bytes 13–14: data-block CRC placeholder (0x00)
    frame
}
```

This helper MUST be present in `tests/dnp3_f5_remediation_tests.rs`. Tests may use
either `build_canonical_master_control_frame` (preferred) or the existing
`build_control_frame` (CTRL=0xD4), but the canonical helper must be present as a
correctness anchor even if not used in every test.

### Pinned Summary String

One test (`test_unexpected_source_fires_at_count_1`) asserts the summary in full.
The canonical summary string format (from §2 of Revision 1, unchanged) is:

```
"DNP3 unauthorized control command from unexpected source: src={src:#06X} is not in expected master set {master_set} on dest={dest:#06X}"
```

Where `{master_set}` is the formatted list of `flow.master_addrs_seen` entries at
the time of emission, e.g. `"[0x0001]"`. For the two-frame test scenario (frame 1
from 0x0001 establishes set, frame 2 from 0x0099 is unexpected), the snapshots
capture `master_addrs_seen = [0x0001]` BEFORE the push; `src=0x0099`; `dest=0x0003`.
The expected summary is:

```
"DNP3 unauthorized control command from unexpected source: src=0x0099 is not in expected master set [0x0001] on dest=0x0003"
```

The test asserts `f.summary == <above string>` (exact equality) OR the existing
`f.summary.contains("unexpected source")` + `contains("0x0099")` + `contains("0x0003")`
sub-string checks are sufficient for all tests. The exact-equality assertion is preferred
for `test_unexpected_source_fires_at_count_1` as it prevents silent format drift.

The `{master_set}` formatting: implement as `format!("{:?}", &flow.master_addrs_seen)`
which renders as `[0x0001]` for a single-entry vec if addresses are formatted via
`{:#06X}` custom Display, OR use a manual join. The implementation must produce a
deterministic, parseable representation. Recommended implementation in
`detect_unexpected_source_split`:

```rust
let master_set: Vec<String> = flow.master_addrs_seen
    .iter()
    .map(|a| format!("{a:#06X}"))
    .collect();
let master_set_str = format!("[{}]", master_set.join(", "));
```

This produces `[0x0001]` for a single entry and `[0x0001, 0x0002]` for two entries.

### Revised Test List (all tests required in STORY-108 scope)

The following replaces the test list in Revision 1 §6. All tests use
`build_canonical_master_control_frame` (CTRL=0xC4) unless noted.

---

**test_canonical_master_frame_helper_satisfies_is_master_frame** [NEW — correctness anchor]
- Call `is_master_frame(0xC4)`. Assert true.
- Call `is_master_frame(0xD4)`. Assert true.
- Call `is_master_frame(0x00)`. Assert false.
- Call `is_master_frame(0x04)`. Assert false (UNCONF_USER_DATA with DIR=0).
- This test documents the corrected 0x80 mask behavior and must be the first test in
  the unexpected-source test module. It catches any regression if someone reverts the
  mask fix.

---

**test_unexpected_source_fires_at_count_1** [UPDATED from Revision 1]
- Setup: fresh `Dnp3Analyzer`, threshold=10. Deliver `build_canonical_master_control_frame(0x05, 0x0003, 0x0001)` → establishes expected set.
  Deliver `build_canonical_master_control_frame(0x05, 0x0003, 0x0099)` → unexpected.
- Assert: `all_findings.len() == 1`.
- Assert finding[0]: `mitre_techniques == vec!["T1692.001"]`, `confidence == Confidence::High`,
  `verdict == Verdict::Likely`, `category == ThreatCategory::Execution`.
- Assert finding[0].summary (EXACT): `"DNP3 unauthorized control command from unexpected source: src=0x0099 is not in expected master set [0x0001] on dest=0x0003"`.
- Assert: `flow.direct_operate_count == 2` (both FCs counted).
- Assert: `flow.direct_operate_emitted == false` (2 > 10 = false).
- Assert: `flow.unexpected_source_emitted == true`.

---

**test_unexpected_source_independent_of_threshold** [UNCHANGED from Revision 1, CTRL updated]
- Setup: threshold=10. Deliver 9 Control FCs from src=0x0001. Deliver 1 from src=0x0099.
- Assert: exactly ONE finding; summary contains "unexpected source"; summary does NOT
  contain "burst"; `flow.direct_operate_count == 10`; `flow.direct_operate_emitted == false`.

---

**test_unexpected_source_one_shot_guard** [UNCHANGED from Revision 1, CTRL updated]
- Setup: 1 Control FC from 0x0001 (establishes set). 3 Control FCs from 0x0099.
- Assert: exactly ONE unexpected-source finding (first FC from 0x0099 triggers; subsequent
  two suppressed by `unexpected_source_emitted = true`).

---

**test_first_master_is_expected** [UNCHANGED from Revision 1, CTRL updated]
- Setup: fresh flow. Deliver first Control FC from src=0x0001.
- Assert: NO finding. `flow.master_addrs_seen == [0x0001]`.
  `flow.unexpected_source_emitted == false`. `flow.direct_operate_count == 1`.

---

**test_unexpected_source_and_burst_both_fire** [UNCHANGED from Revision 1, CTRL updated]
- Setup: threshold=3. Deliver 1 Control FC from 0x0001. Deliver 4 Control FCs from 0x0099.
  Total `direct_operate_count = 5` after all 5 FCs.
- Assert: `all_findings.len() == 2`.
  - One finding with summary containing "unexpected source" (fired at count=2, first FC from 0x0099).
  - One finding with summary containing "threshold 3" (fired when count exceeded 3).
  - Both guards set: `unexpected_source_emitted == true`, `direct_operate_emitted == true`.
- Note on count: count=1 from frame1 (0x0001), count=2 from frame2 (0x0099 first FC,
  unexpected-source fires), count=3 from frame3, count=4 from frame4 (4>3 → burst fires),
  count=5 from frame5 (guard already set, no second burst finding).

---

**test_unexpected_source_max_master_addrs_full** [NEW — MINOR item from Slice A]
- Setup: Fill `master_addrs_seen` to `MAX_MASTER_ADDRS` (64) entries by delivering 64
  master-direction non-Control frames from src=1..=64 (these establish the set without
  triggering Control-class detection). Then deliver a Control FC from src=0x0099 (not in set).
- Assert: ONE unexpected-source finding emitted (the cap-full case does NOT suppress detection;
  attacker cannot use table saturation to evade unexpected-source check).
- Assert: `flow.master_addrs_seen.len() == 64` (0x0099 was NOT added due to cap).
- This test verifies the invariant from Revision 1 §2 "Interaction with MAX_MASTER_ADDRS".

---

**test_unexpected_source_second_distinct_unexpected_source** [NEW — MINOR item from Slice A, rotation case]
- Setup: 1 Control FC from 0x0001 (establishes set). 1 Control FC from 0x0099 (unexpected, guard fires). 1 Control FC from 0x0100 (a SECOND unexpected source address).
- Assert: exactly ONE unexpected-source finding total (the one-shot flow-lifetime guard
  `unexpected_source_emitted` suppresses the second distinct unexpected address).
- Assert: `flow.master_addrs_seen` contains 0x0001, 0x0099, 0x0100 (all were pushed —
  the cap is 64; source-address push and finding emission are independent).
- This verifies that an attacker rotating source addresses cannot flood findings.

---

**test_unexpected_source_skipped_on_non_dnp3_flow** [NEW — MINOR item from Slice A]
- Setup: fresh flow. Set `flow.is_non_dnp3 = true` directly via test mutation. Deliver a
  Control FC from src=0x0099 (after setting up master_addrs_seen=[0x0001] via pre-test
  state manipulation OR via a pre-bail establishing frame).
- Assert: NO finding emitted. `flow.is_non_dnp3 == true` means `on_data` returns
  immediately (BC-2.15.009 / BC-2.15.016 EC-006); the unexpected-source check is
  never reached.
- Implementation note: the `is_non_dnp3` bail is at the very top of `on_data` (per
  BC-2.15.009); the test confirms the skip by verifying no finding is pushed even with
  a Control FC payload on a bailed flow.

---

## R2-7. PRODUCT-OWNER ACTION ITEMS (BLOCKING BEFORE IMPLEMENTATION)

The following spec changes are owned by the PRODUCT OWNER and must be applied BEFORE
the implementer begins coding. They correct foundational errors in the specification
that the implementer would otherwise code against incorrectly.

| # | Document | Change Required | Blocking? |
|---|----------|-----------------|-----------|
| PO-1 | `BC-2.15.016.md` Postcondition 5 | Correct "DIR bit is bit 4 (mask 0x10)" to "DIR bit is bit 7 (mask 0x80), per IEEE 1815 DNP3 link-layer framing. `is_master_frame(control)` tests `control & 0x80 != 0`. Mask 0x10 is FCV/DFC (bit 4), NOT DIR." | YES |
| PO-2 | `BC-2.15.016.md` Architecture Anchors | Add note: "`is_master_frame` uses mask 0x80 (bit 7 = DIR bit per IEEE 1815)." | YES |
| PO-3 | `BC-2.15.010.md` Edge Cases | Add EC-011 (redundant-master limitation) per text in R2-2 above. | YES |
| PO-4 | `wave-35-39-holdout.md` HS-W37-002 | Replace Scenario and Assertion per R2-5 above (pin two-frame sequence; add exact assertion list). | YES |

All four actions should be applied as a single atomic edit to the spec documents,
confirmed by the product owner, before the implementer proceeds to R2-8.

---

## R2-8. IMPLEMENTER ACTION ITEMS (REVISED REMEDIATION SEQUENCE)

| Step | Action | Blocking? |
|------|--------|-----------|
| 1 | Fix `is_master_frame`: change `control & 0x10 != 0` to `control & 0x80 != 0`. Update the doc-comment to say "DIR bit is bit 7 (mask 0x80) per IEEE 1815". | YES |
| 2 | Fix `tests/dnp3_flow_state_tests.rs` `test_BC_2_15_016_is_master_frame_dir_bit`: (a) replace `is_master_frame(0x10)` with `is_master_frame(0xC4)` and update message; (b) remove/replace `is_master_frame(0xEF) == false` with `is_master_frame(0x44) == false`; (c) update all comments to say "bit 7" / "mask 0x80". | YES |
| 3 | Fix `tests/dnp3_flow_state_tests.rs` `build_master_frame` doc-comment: correct "DIR+PRM bits: 0xD4" → "DIR=bit7, PRM=bit6, FCV=bit4 (set); 0xD4 & 0x80 = 0x80 → is_master_frame=true". | YES |
| 4 | Fix `tests/dnp3_f5_remediation_tests.rs` `build_control_frame` doc-comment: correct references to "bit 4 (mask 0x10)" → "bit 7 (mask 0x80)"; update the NOTE about CTRL=0xC4. | YES |
| 5 | Verify STORY-107 `test_master_addrs_cap_at_64` still passes under the corrected mask. It uses `build_master_frame` → `CTRL=0xD4`; `0xD4 & 0x80 = 0x80` → still true. No behavioral change; run `cargo test test_master_addrs_cap_at_64` to confirm green. | YES |
| 6 | Add `Dnp3FlowState::unexpected_source_emitted: bool` field if not already present (the stub was added per red-gate-log). | YES |
| 7 | Capture `src_was_known` and `expected_set_established` snapshot booleans at the EXACT INSERTION POINT specified in R2-3: after `flow.frame_count += 1` (line 442) and BEFORE the `master_addrs_seen` push (line 446). Both snapshots reference `header.source`. | YES |
| 8 | Add `detect_unexpected_source_split` associated function per signature in Revision 1 §3 (signature unchanged). Emission semantics per Revision 1 §2 (unchanged) with: corrected `is_master_frame` (0x80 mask), `expected_set_established` snapshot (not live `!flow.master_addrs_seen.is_empty()`), `src_was_known` snapshot (not live `.contains`), one-shot guard `unexpected_source_emitted`. | YES |
| 9 | Insert call site in `Dnp3FcClass::Control` arm per R2-3 ordering (after broadcast check, before pending-requests, using snapshot bools). Confirm NO early-return/short-circuit (R2-4 fall-through invariant). | YES |
| 10 | Add `build_canonical_master_control_frame` helper (CTRL=0xC4) to test file per R2-6. | YES |
| 11 | Add all 8 tests listed in R2-6. Run each until green. Confirm `test_first_master_is_expected` remains green throughout. | YES |
| 12 | Run full test suite: `cargo test --all-targets`. All must pass. | YES |
| 13 | Run clippy: `cargo clippy --all-targets -- -D warnings`. Clean. | YES |
| 14 | Confirm HS-W37-002 (as amended) passes in Phase 4 holdout evaluation. | YES (Phase 4 gate) |

---

## R2-9. SUMMARY OF CHANGES FROM REVISION 1

| Topic | Revision 1 | Revision 2 |
|-------|-----------|-----------|
| `is_master_frame` mask | 0x10 (bit 4, FCV) | **0x80 (bit 7, DIR) — CORRECTED** |
| Canonical master frame | 0xC4 was NOT a master frame (0xC4 & 0x10 = 0) | **0xC4 IS a master frame (0xC4 & 0x80 = 0x80)** |
| Test helper | `build_control_frame` with CTRL=0xD4 only | **Must also have `build_canonical_master_control_frame` with CTRL=0xC4** |
| Snapshot insertion point | Three contradictory orderings described | **Single: after frame_count++ at line 442, before push at line 446** |
| Snapshot variable scope | Ambiguous (some Revision 1 variants put snapshots inside FIR=1 block) | **Outer scope — valid before has_user_data/FIR=1 gate** |
| Fall-through invariant | Not explicitly stated | **Explicit: detect_unexpected_source_split must not alter control flow** |
| HS-W37-002 | Under-specified (single-frame reading fails) | **Amended: exact two-frame sequence; exact assertion list** |
| Redundant-master FP | Not addressed | **Documented as EC-011; accepted known limitation** |
| `src` identity | Not clarified vs source_ip | **`src` = DNP3 link-layer u16; source_ip = IP-layer endpoint from FlowKey** |
| Test count | 5 tests | **8 tests (3 new: canonical-helper sanity, MAX_MASTER_ADDRS full, rotation, is_non_dnp3 skip)** |
| BC-2.15.016 PC5 | States "bit 4 (0x10)" | **PO must correct to "bit 7 (0x80)"** |
| BC-2.15.010 EC-011 | Not present | **PO must add redundant-master limitation edge case** |
| PO involvement | None required | **PO must apply 4 spec corrections (PO-1..PO-4) before implementation** |
