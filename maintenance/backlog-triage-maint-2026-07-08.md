---
document_type: maintenance-backlog-triage
run: maint-2026-07-08
producer: research-agent
policy: DF-VALIDATION-001
develop_head: b642c0fdabfd6ae9f9ea8d1680b50662c5654e93
develop_short: b642c0f
date: 2026-07-08
scope: 11 deferred findings pending research validation before any GitHub issue is filed (wave-71 items 1-8; wave-70 items 9-11 added under 2026-07-08 scope extension)
scope_extension_2026-07-08: added wave-70 SEC-010 / SEC-011 / SEC-W70-001; also documents wave-71 CR-001 canonical-ID collision (proposal: rename CR-W71-001)
---

# Backlog Triage — maint-2026-07-08

**Producer:** research-agent (DF-VALIDATION-001 triage)
**Policy:** DF-VALIDATION-001 (`.factory/policies.yaml`) — every deferred finding MUST be
research-agent-validated on the current default branch (`develop = b642c0f`) before it
becomes a GitHub issue.
**Ground truth `develop` HEAD at triage:** `b642c0fdabfd6ae9f9ea8d1680b50662c5654e93`.

Each row: locate primary evidence, validate against develop `b642c0f`, verdict, disposition.

---

## 1. SEC-W71-001 — Wave-71 security pass LOW

### Evidence

- STATE.md Deferred-Findings register: "SEC-W71-001 | Wave-71 security pass LOW finding
  (details in cycles/wave-71/wave-gate/ security pass report). Pending DF-VALIDATION-001."
- Wave gate summary (`.factory/cycles/wave-71/wave-gate/gate-summary.md`) dimension (d)
  Security: "CLEAN | SEC-W71-001 LOW pending DF-VALIDATION-001; SEC-W71-002/003
  accepted/no-action."
- No standalone "security pass report" file exists under `cycles/wave-71/`. The one
  wave-71 security artifact on disk is `.factory/code-delivery/STORY-157/security-review.md`,
  which is the only wave-71 story that received a formal application-security review.
- That review enumerates:
  - SEC-001 (LOW, NEW): **CWE-22 Path Traversal** — comment-stripping in
    `bin/compute-input-hash parse_inputs()` allows an entry like `- ../../etc/shadow # RETIRED`
    to resolve outside the repo after the trailing `# ...` suffix is stripped. Pre-PR the
    literal comment text made the path unresolvable; post-PR it resolves.
  - SEC-002 (LOW, pre-existing): CWE-22 — `repo_root / rel_path` silently discards
    `repo_root` if `rel_path` is absolute (Python `PurePath.__truediv__` semantics).
  - SEC-003 (INFO): `exec(compile(...))` in the test harness; already `# noqa: S102`.
  - SEC-004 (INFO): absolute paths (including `$HOME`) leaked in error messages.

By elimination — one LOW is "pending", two LOWs are "accepted/no-action" — the register
maps to: **SEC-W71-001 = SEC-001 (CWE-22 comment-stripping path-traversal enablement)**;
SEC-W71-002/003 = SEC-002 + SEC-003 or SEC-002 + SEC-004 (both accepted as pre-existing /
INFO). This mapping is inferred from severity/finality; the gate summary should state it
explicitly next cycle.

### Validation against develop `b642c0f`

`bin/compute-input-hash` on develop still performs the comment strip without any
`..` / absolute-path guard:

```
bin/compute-input-hash:137-140   # AC-157-010: strip inline comment suffix (` # ...`)
        comment_idx = path.find(" #")
        if comment_idx != -1:
            path = path[:comment_idx].strip()
```

No sanitization is applied to `path` before `repo_root / path` resolution. `SEC-002` root
cause (`PurePath.__truediv__` swallows an absolute rhs) is also unchanged. Both are still
open on `b642c0f`.

### Exploitability (from STORY-157 security-review)

- Requires factory-artifacts write access (privileged, small trust boundary).
- Output is a 7-char hex MD5 truncation, not file contents.
- Tool is an internal developer CLI, no untrusted callers, no CI privilege-escalation path.
- Even a successful traversal only causes a hash of a file outside `.factory/` to be
  computed and stored. That produces spurious drift signals, not information disclosure.

### CWE classification (external claim)

CWE-22 "Improper Limitation of a Pathname to a Restricted Directory ('Path Traversal')" —
canonical MITRE definition: <https://cwe.mitre.org/data/definitions/22.html>. Matches the
finding exactly (input `../../…` traverses outside the intended base directory after a
sanitizer transform).

### Verdict

**CONFIRMED — real, still open on develop `b642c0f`, correctly classified as LOW / CWE-22.**

### Disposition

**File a single GitHub issue** covering the two developer-tool path-safety hardenings
together (they share a fix). Suggested:

- Title: `bin/compute-input-hash: harden input path resolution against traversal and absolute-path swallowing (CWE-22)`
- Labels: `security`, `low`, `tooling`, `cwe-22`
- Body should:
  - Cite `bin/compute-input-hash:137-140` (comment-strip) and the `repo_root / rel_path`
    call as the two sites.
  - Reference `.factory/code-delivery/STORY-157/security-review.md` §SEC-001 + SEC-002 for
    accepted-LOW rationale.
  - Propose fix: after strip, reject entries containing `..` segments or that resolve to an
    absolute path, or use `Path.resolve(strict=True)` + `is_relative_to(repo_root)`.
- **Also update `cycles/wave-71/wave-gate/gate-summary.md` and `STATE.md`** to make the
  `SEC-W71-001 → SEC-001` mapping explicit, so future audits do not have to re-derive it.

---

## 2. REBIND-COUNT-SATURATING-001 — plain `+=` on `rebind_count`

### Evidence

- STATE.md Deferred-Findings: "REBIND-COUNT-SATURATING-001 | PR #366 security review
  informational note: `rebind_count` in `src/analyzer/arp.rs` uses plain `+=` (not
  `saturating_add`) — pre-existing, non-introduced, realistically unreachable overflow
  (u64 ARP rebind events bounded by capture size; same LOW/informational class as cleared
  SEC-004+SEC-007). Optional hardening…"

### Validation against develop `b642c0f`

Site confirmed at `src/analyzer/arp.rs:856`:

```
855        // Step 1: increment rebind_count (BC-2.16.004 PC1.a).
856        entry.rebind_count += 1;
```

`rebind_count` is declared `u32` at `arp.rs:113` (not `u64` as the finding text says).

Sibling `+=` counter sites in the same file (all also plain, no `saturating_add`):

| Line | Counter | Type | Kind |
|------|---------|------|------|
| 467 | `self.frames_analyzed += 1;` | u64 | per-frame |
| 469 | `self.request_count += 1` | u64 | per-frame |
| 470 | `self.reply_count += 1` | u64 | per-frame |
| 471 | `self.other_opcode_count += 1` | u64 | per-frame |
| 486 | `self.mismatch_findings += 1` | u64 | per-finding |
| 532 | `self.storm_findings += 1` | u64 | per-finding |
| 585 | `self.garp_findings += 1` | u64 | per-finding |
| 586 | `self.spoof_findings += 1` | u64 | per-finding |
| 601 | `self.garp_findings += 1` | u64 | per-finding |
| 661 | `self.spoof_findings += 1` | u64 | per-finding |
| 702 | `self.malformed_frames += 1` | u64 | per-frame |
| 706 | `self.malformed_findings += 1` | u64 | per-finding |
| 856 | `entry.rebind_count += 1` | u32 | per-rebind |
| 1032 | `entry.count_in_window += 1` | u32 | per-window |

`rebind_count` (u32) is the only site with realistic overflow head-room concern — a
`u32` per-entry counter can theoretically saturate at ~4.29e9. All others are `u64`. The
release profile has `overflow-checks = true` (from `Cargo.toml`), so an actual overflow
in debug or release panics; it does not silently wrap.

### Verdict

**CONFIRMED — site exists, still `+=` on develop `b642c0f`; overflow is unreachable in
practice but the discipline gap is real. `rebind_count` u32 (not u64 as stated) is the
one site with genuine overflow head-room; twelve u64 sibling counters have the same
plain `+=` style but astronomically unreachable overflow.**

### Disposition

**Fold into a maintenance micro-PR this sweep** (single-file, low-risk, DF-SIBLING-SWEEP-001
compliant: fix all 14 sites in `arp.rs` in one burst) OR **defer as a nice-to-have**.
Do NOT open a GitHub issue — the finding is informational and the fix is trivially
sized. If not folded this sweep, retain in STATE.md deferred register with the corrected
"u32" note.

Suggested if filed anyway (opt-in): title
`arp.rs: convert plain += counters to saturating_add for defense-in-depth (informational)`;
labels `hardening`, `informational`.

---

## 3. INPUT-HASH-ERROR-STORIES-001 — STORY-001 retired-BC input + STORY-091/121 "missing inputs"

### Evidence

- STORY-001 `inputs:` list (line 14) includes
  `.factory/specs/behavioral-contracts/ss-01/BC-2.01.004.md` with the inline comment
  `# RETIRED 2026-06-19: superseded by BC-2.01.009 (behavioral inversion); file retained
  per append-only-numbering policy`.
- STORY-091 line 10: `inputs: []` (explicit empty list, not missing).
- STORY-121 line 10: `inputs: []` (explicit empty list, not missing).

### Validation against develop `b642c0f`

`bin/compute-input-hash --scan` on `b642c0f`:

```
STORY-001.md     4ae9f11    4ae9f11    MATCH
STORY-091.md     d41d8cd    d41d8cd    MATCH
STORY-121.md     d41d8cd    d41d8cd    MATCH
MATCH=111 STALE=0
```

CLAUDE.md § "Edge Cases":

> "**Empty inputs (`inputs: []` or empty multiline block):** Produces hash `d41d8cd`
> (MD5 of empty bytes). E-11 stories use `inputs: []` because they have no spec inputs;
> the scanner correctly reports MATCH for these stories."

STORY-091 and STORY-121 are both E-11 process-gap stories with `status: draft` and
`behavioral_contracts: []` (pending PO authorship). `inputs: []` is the documented
canonical pattern for that class of story, not a defect.

STORY-001: BC-2.01.004 file is retained on disk per append-only-numbering policy (the
retirement is a metadata change, not a delete). The `inputs:` list still points at the
retained file, so the hash is stable and the scan is MATCH. Adding BC-2.01.009 to the
`inputs:` list is a story-content choice (and a `behavioral_contracts` update), not an
input-hash defect. STORY-001's `behavioral_contracts:` frontmatter already carries the
retirement note on BC-2.01.004 and does NOT list BC-2.01.009 — that is the substantive
inconsistency worth surfacing (behavioral inversion by STORY-123 into BC-2.01.009 is not
back-referenced from STORY-001), but it is a spec-traceability issue, not an
input-hash issue.

### Verdict

**REFUTED (as filed) — no input-hash drift on develop `b642c0f`.**
- STORY-091 / STORY-121: `inputs: []` is by design (E-11 draft convention documented in
  CLAUDE.md); the "missing inputs block" claim is factually incorrect.
- STORY-001: `input-hash` matches; the retired-BC reference is intentional under
  append-only-numbering. There is a residual **spec-traceability** question (should
  STORY-001 back-reference BC-2.01.009 that superseded BC-2.01.004?), but that is a
  separate spec-coherence item, not an input-hash defect.

### Disposition

**Close the backlog row.** Do NOT file a GitHub issue on this finding as stated.
Optionally, open a lower-priority **spec-coherence** ticket for the STORY-001 →
BC-2.01.009 back-reference gap if a spec-coherence sweep independently confirms it worth
fixing. Suggested title if pursued: `STORY-001: add BC-2.01.009 to behavioral_contracts
(post-supersession trace)`; labels `spec`, `traceability`, `low`. Optional.

---

## 4. HS-INDEX-ENIP-WAVE-DRIFT-001 — HS-INDEX ENIP waves "63-68" vs canonical waves 58-61

### Evidence

- `.factory/holdout-scenarios/HS-INDEX.md` EtherNet/IP Feature Holdouts section:
  - "EtherNet/IP (waves 63-68) | 13 seeds (DNP3 convention) | 13 (HS-110..HS-122) |
    CONCRETE — authored v0.11.0-feature-enip"
  - "Stories: STORY-131..STORY-141 (waves 63-68)."
- STORY-INDEX (authoritative story register):
  - STORY-130..STORY-138 → waves 58–61 (E-20 body).
  - STORY-139 → wave 62 (E-20).
  - STORY-140 → wave 63 (E-15, DNP3 fix).
  - STORY-141 → wave 64 (E-14, Modbus fix).
- `.factory/stories/dependency-graph.md` v3.1 changelog: "waves 57 for STORY-129; **58-61
  for E-20**"; wave schedule tables list waves 58–61 for STORY-130..138.

### Validation against develop `b642c0f`

Two independent authorities (STORY-INDEX, dependency-graph) both show E-20 as waves
**58–61** with STORY-130..138 as the E-20 story set. HS-INDEX v2.7+ text carries **two
distinct errors**:

1. **Wrong wave range** for E-20: "waves 63-68" vs ground truth 58-61.
2. **Wrong story enumeration**: "STORY-131..STORY-141" — this mixes 8 E-20 stories
   (STORY-131..138 waves 58–61) with STORY-139 (E-20 wave 62), STORY-140 (E-15 DNP3 fix,
   wave 63), and STORY-141 (E-14 Modbus fix, wave 64). The 13 concrete ENIP holdouts
   HS-110..HS-122 map only to STORY-130..138 (E-20 body); STORY-139/140/141 are fix
   stories in E-14/E-15/E-20 unrelated to the ENIP holdout set.

Ground truth is STORY-INDEX + dependency-graph. HS-INDEX text is stale.

### Verdict

**CONFIRMED — HS-INDEX drift on develop `b642c0f`. Correct values: waves 58-61; stories
STORY-130..STORY-138 (9 stories, not 11).**

### Disposition

**Fold into the maintenance spec-coherence fix PR this sweep** (single-file, single-line
edit; low-risk; DF-SIBLING-SWEEP-001 grep target: search HS-INDEX.md for `63-68` and
`STORY-13[0-9]\.\.STORY-14[01]` to catch any repeated occurrence — there are three
locations by inspection: Feature Holdouts table row, section note "Stories:
STORY-131..STORY-141 (waves 63-68)", and the Feature Holdouts summary table). Bump
HS-INDEX version and add modified-log entry.

If a GitHub issue is preferred instead: title `HS-INDEX: correct E-20 ENIP feature-holdout
wave range (63-68 → 58-61) and story enumeration`; labels `spec`, `drift`, `low`.

---

## 5. EPICS-TOTAL-BCS-DRIFT-001 — epics.md `total_bcs: 337` vs BC-INDEX 345 active; E-5 row omits BC-2.07.038..043

### Evidence

- `.factory/stories/epics.md` frontmatter: `total_bcs: 337`.
- `.factory/stories/epics.md` E-5 row: `BC-2.07.001..037` (37 BCs).
- `.factory/stories/epics.md` v2.1 changelog **self-admits** the drift:
  > "DISCREPANCY NOTE: epics.md pre-E-21 total_bcs 328 was stale by -6 — BC-2.07.038..043
  > (TLS carry-reassembly BCs, fix-tls-clienthello-frag F3 2026-06-29) are absent from
  > E-5 Per-Epic BC row and Coverage Check table; true pre-E-21 total = 334; this v2.1
  > corrects for E-21 only (328+9=337), deferring the E-5 BC row update to a subsequent
  > pass. Residual gap vs BC-INDEX v2.13 (345 active) = 8 (= 6 missing TLS BCs + 2
  > unresolved)."
- `.factory/specs/behavioral-contracts/BC-INDEX.md` v2.20 header: "346 entries (…);
  Active count: 345." (developed to v2.20 as of wave-71 close 2026-07-08).

### Validation against develop `b642c0f`

- `total_bcs: 337` in epics.md → still current on develop `b642c0f`.
- BC-INDEX active count → **345** (v2.20 header).
- Drift = 345 − 337 = 8 BCs, exactly matching the self-admitted note.
- The 6 missing E-5 BCs are BC-2.07.038, .039, .040, .041, .042, .043 (TLS carry-reassembly
  BCs from `fix-tls-clienthello-frag` F3). The additional 2 are described as "unresolved"
  in the v2.1 note — a targeted BC-INDEX diff will identify them (candidates: extension
  BCs authored between epics.md v2.1 and BC-INDEX v2.20, e.g. wave-70 silent-limit
  amendments BC-2.11.035 mitre_attack, or BC-2.16.008 v2.0 / BC-2.16.010 v1.9
  observability counters — but these are AMENDS to existing BCs, not new BC additions;
  the true residual delta needs enumeration).

### Verdict

**CONFIRMED — real drift, still open on develop `b642c0f`, self-admitted in epics.md v2.1
change note.**

### Disposition

**Fold into the maintenance spec-coherence fix PR this sweep** (product-owner-scoped: bump
epics.md E-5 BC list to include BC-2.07.038..043, update Per-Epic BC table, Coverage Check
arithmetic block, and `total_bcs` frontmatter; enumerate and reconcile the residual 2
BCs against BC-INDEX v2.20). This is exactly the pattern DF-SIBLING-SWEEP-001 covers for
BC edits — the fix touches E-5 row + arithmetic + total + coverage confirmed, all in one
file.

If a GitHub issue is preferred: title `epics.md: reconcile total_bcs to BC-INDEX v2.20
(add missing BC-2.07.038..043 to E-5, resolve residual +2 delta)`; labels `spec`, `drift`,
`low`.

---

## 6. DNP3-CLOSEDFLOW-REOPEN-REUSE-001 — `closed_flow_direct_operates` Vec double-lists FlowKey on NAT port reuse

### Evidence

- `src/analyzer/dnp3.rs:339` — `pub closed_flow_direct_operates: Vec<(FlowKey, u32)>`,
  docstring "Per-closed-flow `(FlowKey, direct_operate_count)` entries."
- `src/analyzer/dnp3.rs:378-386` — `on_flow_close(flow_key)` unconditionally pushes
  `(flow_key, flow.direct_operate_count)` after `self.flows.remove(&flow_key)`; no
  dedup, no key-uniqueness check.
- `src/analyzer/dnp3.rs:1842-1851` — `summarize()` merges `closed_flow_direct_operates`
  with live `self.flows` entries into `all_flow_entries: Vec<(FlowKey, u32)>`, sorts by
  FlowKey, and enumerates with `i.to_string()` as the JSON key.

### Validation against develop `b642c0f`

Scenario (NAT port reuse):

1. Flow with `FlowKey K` gets `direct_operate_count = 5`, closes.
   → `closed_flow_direct_operates = [(K, 5)]`, `flows` no longer contains `K`.
2. A new flow with the same 5-tuple (NAT ephemeral-port recycled) opens.
   → `flows` now contains `K` again (with a fresh `Dnp3FlowState`).
3. If the new flow also closes with, say, `direct_operate_count = 3`:
   → `closed_flow_direct_operates = [(K, 5), (K, 3)]`; `flows` no longer contains `K`.
4. `summarize()` builds `all_flow_entries = [(K, 5), (K, 3)]`; sort is stable; enumerate
   yields `{"0": 5, "1": 3}`.

So the FlowKey **is** double-listed in the internal Vec. Whether the observable
`control_operation_counts` map is defective depends on interpretation:

- Under a "per-session" semantic (each open→close is one row), the current behaviour is
  arguably correct — both sessions' counts are preserved, keyed by ordinal index.
- Under a "per-FlowKey" semantic (docstring says "Per-closed-flow", where "flow" =
  FlowKey), duplicate FlowKeys violate the invariant that the merged set is 1:1 with the
  key.

Callers reading `control_operation_counts` see integer-string keys `"0","1",…`, not
FlowKeys. There is no observable direct exposure of the FlowKey duplication in the
public JSON. However, the invariant of "one entry per closed flow keyed by FlowKey" is
observably broken in the internal state.

Formal spec: **BC-2.15.021 governs `on_flow_close`**. Its post-condition 4 says
"`self.closed_flow_direct_operates.push((flow_key, flow.direct_operate_count))`" —
literal push, no dedup. The current implementation is spec-conformant.

### Verdict

**CONFIRMED as observation** — the code does double-list a FlowKey under NAT port
reuse. **REFUTED as a defect** on develop `b642c0f`: BC-2.15.021 postcondition 4 is a
literal push with no uniqueness constraint, and the observable JSON output uses ordinal
indices, so there is no external-facing wrongness. The internal invariant "one entry per
FlowKey" is not stated by the BC and would in fact lose per-session data if enforced by
dedup.

### Disposition

**Defer — no GitHub issue.** Retain as an observation in STATE.md under a
spec-clarification note rather than a defect. If a defect is later discovered
(e.g., a consumer expects unique FlowKeys in `control_operation_counts`), open at that
point with test-driven repro. If the BC intent is actually "per-session", the docstring
"Per-closed-flow" and the field name `closed_flow_direct_operates` should be renamed for
clarity (`closed_session_direct_operates`) — that is a low-priority readability item.

If a GitHub issue is still desired: title `dnp3: document per-session (not per-FlowKey)
semantics of closed_flow_direct_operates Vec (NAT port-reuse case)`; labels `docs`,
`clarification`, `low`.

---

## 7. CR-001 (wave-71) — 1 MINOR + 3 NITs from wave-71 code review

### Evidence

- `.factory/cycles/wave-71/wave-gate/gate-summary.md` dimension (c) Code Review:
  "APPROVE | CR-001 MINOR + 3 NITs; all routed to maintenance/debt; 0 BLOCKING".
- STATE.md deferred-findings register row: "CR-001 (wave-71) | Code review MINOR finding
  + 3 NITs from wave-71 code review; non-blocking; no gate action required. | LOW |
  OPEN — maintenance/tech-debt backlog".

### Validation against develop `b642c0f`

**No dedicated code-review report file was written for the wave-71 code-review dimension.**
The gate summary is the only artifact that mentions "CR-001 MINOR + 3 NITs". There is no
enumeration of *which* MINOR and *which* three NITs, no cited files or line numbers, and
no reproduction steps.

Related but distinct: `.factory/code-delivery/CR-001/pr-description.md` is a
**pre-wave-71** CR-001 refactor PR (unused `take_http_analyzer`), unrelated to this
row.

Per-story reviews (`code-delivery/STORY-15{0,6,7}/pr-review.md`) each contain their own
NITPICK observations, but none is labelled `CR-001 (wave-71)`.

Per DF-VALIDATION-001, a finding "MUST be validated" and "cite sources". A finding that
consists solely of the label "CR-001 MINOR + 3 NITs" with no reproducible content is
**UNVERIFIABLE** on develop as filed.

### Verdict

**UNVERIFIABLE (as filed).** No primary evidence file enumerates the specific MINOR or
the three NITs. The wave-71 code-review step passed APPROVE with no blocking items and
no artifact was persisted. The status "OPEN — maintenance/tech-debt backlog" cannot be
acted on without recovered content.

### Disposition

**Do NOT file a GitHub issue.** Two options:

- **(preferred)** Close the STATE.md row as "no-op — approved with no persisted finding
  detail" and add a **process gap PG-W71-CODEREVIEW-ARTIFACT** to STORY-158's scope: the
  wave-71 gate should have written `.factory/cycles/wave-71/wave-gate/code-review.md`
  enumerating every MINOR/NIT. (Analogous to PG-W71-CHANGELOG and
  PG-W71-CYCLE-ARTIFACT-IDENTITY captured for the same gate.)
- **(alternative)** If the wave-71 code-reviewer sub-agent can be re-consulted from
  cached context/transcripts, backfill the artifact and re-triage.

Either way, DF-VALIDATION-001 blocks filing this row as an issue in its current
under-specified form.

---

## 8. STORY-148-BASIS-RESOLVED-001 — STORY-148 SUPERSEDED

### Evidence

- `.factory/stories/STORY-148.md` frontmatter: `status: superseded`, plus body:
  > "Status changed draft→superseded. PR #362 fully implemented all acceptance criteria:
  > SEC-005 ENIP on_flow_close wiring + SEC-006 DNP3 on_flow_close routing both wired in
  > dispatcher.rs; regression tests at tests/issue_342_flow_leak_regression_tests.rs;
  > issue #342 closed 2026-07-06."
- `.factory/stories/STORY-INDEX.md` v3.18 changelog: "STORY-148 reconciliation: status
  draft→superseded; all scope delivered by PR #362 (D-383, issue #342 closed 2026-07-06);
  SEC-005 ENIP on_flow_close wiring + SEC-006 DNP3 on_flow_close routing both verified on
  develop; no point change (5 pts remain in E-20 total)."
- STORY-INDEX table row: `STORY-148 | … | E-20 | ~ | 5 | superseded | —`.
- Epic-table E-20 row: "STORY-148 on_flow_close wiring + DNP3 flow-map cap (SEC-005/SEC-006,
  maint-2026-07-01) — **superseded by PR #362 (D-383, issue #342 closed 2026-07-06)**".
- STATE.md D-399 reference: (STATE-md history confirms D-399 was the reconciliation
  decision).

### Validation against develop `b642c0f`

Story file, STORY-INDEX, and epics.md all agree on `superseded`. Regression test file
`tests/issue_342_flow_leak_regression_tests.rs` exists (per STORY-148 body citation);
`dispatcher.rs` wiring is on develop; issue #342 is closed on GitHub.

### Verdict

**ALREADY-RESOLVED — CONFIRMED. STORY-148 file + STORY-INDEX + epics.md agree; PR #362
implemented all ACs; issue #342 closed 2026-07-06.**

### Disposition

**Close the backlog row.** No GitHub issue to file. STATE.md deferred register can drop
this entry (or move to a "closed reconciliations" archive). Recommend the maintenance
sweep verify the STATE.md deferred register does not still list this row as OPEN.

---

---

## 9. SEC-010 — Test/bench-only u16 truncation (CWE-197)

### Evidence

- `.factory/tech-debt-register.md` row `SEC-010`: "wave-70 security review, LOW; Test/bench-only u16 truncation (CWE-197); wave-70 adversary context ID: SEC-001; pending DF-VALIDATION-001".
- Primary evidence: `.factory/code-delivery/STORY-149/security-review.md` §SEC-001:
  - Location: `tests/common/tls_fragmented_fixture.rs:19-22` (`wrap_as_tls_record` helper).
  - CWE per that document: **CWE-704** (Incorrect Type Conversion or Cast) with related
    CWE-190 (Integer Overflow). The tech-debt register instead cites **CWE-197** (Numeric
    Truncation Error). CWE-197 is the tighter fit ("high-order bits truncated when a value
    is cast to a smaller integer type"); the STORY-149 review's CWE-704 tag is broader
    but not wrong. Recommend standardising on CWE-197 in future references.
- Wave-70 convergence-state cross-map: `wave-70-story-149/wave-gate/wave-convergence-state.json`
  confirms `wave_context_id "SEC-001" → tech_debt_register_id "SEC-010"` with cwe "CWE-197".

### Validation against develop `b642c0f`

Source of the finding still on develop:

```
tests/common/tls_fragmented_fixture.rs:15  fn wrap_as_tls_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
                                       16      let len = payload.len();
                                       17      let mut record = vec![
                                       18          content_type,
                                       19          0x03,
                                       20          0x03,
                                       21          (len >> 8) as u8,
                                       22          (len & 0xff) as u8,
                                       23      ];
```

`payload.len()` is `usize`; the code splits it into two `u8` bytes for the TLS record
length field without any `debug_assert!(payload.len() <= u16::MAX as usize)` guard. If a
future test or bench were to pass a payload > 65 535 bytes, the record header would silently
misrepresent length. Current callers stay well under u16::MAX.

**Production-code check:** `grep -rn " as u16" src/` on develop `b642c0f` returns **zero
matches** in `src/`. The truncation pattern is exclusively in test/bench fixture code
(`tests/common/tls_fragmented_fixture.rs` and the various `tests/bc_*_tests.rs` fixtures).
Not exploitable via production code paths. Confirmed test/bench-only.

### Verdict

**CONFIRMED — pattern still present on develop `b642c0f`, and confirmed test/bench-only
(zero `as u16` sites in `src/`).** Threat model is limited to fixture-authoring mistakes,
not runtime exploitation.

### Disposition

**Fold into a maintenance fix PR this sweep — trivial** — add the proposed
`debug_assert!(payload.len() <= u16::MAX as usize, …)` from the STORY-149 security review
(document already carries the exact patch). One-line addition to
`tests/common/tls_fragmented_fixture.rs`. DF-SIBLING-SWEEP-001 target: grep the test tree
for other unguarded `.len() as u16` splits in fixture builders (results shown by earlier
scan: `tests/bc_150_drain_loop_dry_tests.rs:133`, `tests/bc_f6_mutation_gap_tests.rs:91`,
`tests/bc_2_02_story00{2,5}_tests.rs` — assess whether each also warrants a debug_assert
guard for parity, or accept them because the callers are bounded by construction).

If a GitHub issue is preferred instead: title
`tests: guard test/bench u16 truncation with debug_assert (CWE-197)`; labels `test`,
`hardening`, `low`, `cwe-197`. Also reconcile the CWE tag: STORY-149 security-review says
CWE-704; tech-debt register + wave-70 convergence state say CWE-197. Recommend canonical
tag **CWE-197** going forward.

---

## 10. SEC-011 — Borrow-budget comment gap; partial fix at 5b41eca

### Evidence

- Tech-debt register row `SEC-011`: "borrow-budget comment gap in test/bench code…
  BORROW BUDGET annotations were subsequently added at commit 5b41eca which partially
  addresses the gap. Wave-70 adversary context ID: SEC-002."
- Primary evidence (STORY-149 security-review.md §SEC-002):
  > "The borrow-budget CI tests grep for `.get(` / `.get_mut(` but not for
  > `self.flows[key]` (HashMap index-operator). A future contributor using Index syntax
  > would evade the CI count, silently reintroducing PERF-001 overhead."
- Commit `5b41eca` inspected (`git show 5b41eca --stat`):
  > "docs(STORY-149): add BORROW BUDGET annotations at process_handshake_carry sites
  > (F-S149P5-001)"
  9 insertions, `src/analyzer/tls.rs` only. Adds three `// BORROW BUDGET (STORY-149):
  site N of ≤3` inline comments plus a doc-comment paragraph.

### Validation against develop `b642c0f`

Two related-but-distinct gaps existed at wave-70 close:

- **Gap A (F-S149P5-001, "annotation gap"):** every acquisition site in
  `process_handshake_carry` must carry a `BORROW BUDGET` inline marker; the inspection
  test at `tests/bc_149_single_borrow_invariant_tests.rs:194-220` enforces the count.
  **Resolved at 5b41eca** — three markers added; the enforcement test is passing on
  develop `b642c0f`.
- **Gap B (STORY-149 security-review §SEC-002, "anti-gameability enumeration gap"):**
  the anti-gameability test at
  `tests/bc_149_single_borrow_invariant_tests.rs:234-273` enumerates forbidden aliasing
  patterns (`= &mut self.flows`, `= &self.flows`, `self.flows.entry(`,
  `self.flows.iter_mut(`) but **does NOT enumerate `self.flows[` (HashMap Index
  operator)**. A future contributor writing `self.flows[key].client_hello_seen = true;`
  would bypass both the acquisition-site count and the alias/entry/iter checks while
  still forcing a HashMap re-hash. This gap is **UNCHANGED on develop `b642c0f`** —
  5b41eca did not touch that assert list.

Currently zero `self.flows[` usages exist in `src/analyzer/tls.rs`, so no live regression.

The tech-debt register wording ("BORROW BUDGET annotations…partially addresses the gap")
conflates Gap A with Gap B — 5b41eca fully resolved Gap A but did not touch Gap B. The
"partially addresses" phrasing understates that Gap B is a distinct enforcement hole and
was not moved by 5b41eca.

### Verdict

**CONFIRMED (residual gap present).** Gap A is CLOSED at 5b41eca. **Gap B — the
anti-gameability enumeration missing `self.flows[` — is UNCHANGED on develop `b642c0f`.**
No current regression (zero live index-operator usages), but the enforcement hole persists.

### Disposition

**Fold into a maintenance fix PR this sweep — trivial** — add one line to the
anti-gameability test:

```
assert!(
    !body.contains("self.flows["),
    "AC-149-001 anti-gameability: `{fn_name}` must not use HashMap index syntax \
     self.flows[key] — index operator forces a re-hash but evades the \
     flows.get_mut/flows.get acquisition-site grep (STORY-149 / F-S149P1-001)."
);
```

Insert into the `for (fn_name, body) in [ … ]` loop at
`tests/bc_149_single_borrow_invariant_tests.rs:240-272` alongside the four existing
`assert!(!body.contains("…"))` calls. DF-SIBLING-SWEEP-001 grep: search
`process_handshake_carry` and `try_parse_records` bodies for any `self.flows[` occurrence
before merging (currently zero; the fix is preventative).

Also **update the tech-debt register wording** to disambiguate Gap A (CLOSED at 5b41eca)
from Gap B (residual enforcement hole).

If a GitHub issue is preferred instead: title
`tests(tls borrow-budget): extend anti-gameability enumeration to cover self.flows[key] index syntax`;
labels `test`, `hardening`, `low`.

---

## 11. SEC-W70-001 — Unbounded `TlsAnalyzer::all_findings` Vec (CWE-770, pre-existing)

### Evidence

- Tech-debt register row `SEC-W70-001`: "wave-70 security review, LOW — pre-existing;
  Unbounded `TlsAnalyzer::all_findings` Vec (CWE-770); no cap; predates wave 70; bounded
  in practice by capture file size."
- Wave-70 convergence-state: `wave_context_id "SEC-W70-001" → tech_debt_register_id
  "SEC-W70-001"`, cwe `CWE-770`.

### Validation against develop `b642c0f`

`src/analyzer/tls.rs:401` — `all_findings: Vec<Finding>`; still unbounded on develop.

There are 7 unconditional `self.all_findings.push(Finding {…})` sites (lines 539, 575,
596, 649, 671, 736, 757), none gated by a cap. The struct also carries two `#[doc(hidden)]
pub` test seams at lines 1193-1208:

```
1188  /// Exposes `self.all_findings.len()` so integration tests can verify
1189  /// that `TlsAnalyzer` does NOT apply the `MAX_FINDINGS` cap used by
1190  /// `TcpReassembler` (BC-2.04.024 invariant 4 / AC-007b — analyzer non-cap).
1191  /// The analyzer pushes to `all_findings` unconditionally — there is no local cap.
```

So the absence of a cap is **explicitly spec-designed** via BC-2.04.024 Invariant 4 /
AC-007b — the reassembly layer (`TcpReassembler::MAX_FINDINGS = 10 000`) is the
architectural cap point; individual analyzers do not each carry their own cap. The design
is documented, the test seams enforce it as an invariant, and the "no cap" state is
tested to remain true.

### House precedent (wave-71 BC-2.16.016 for ARP)

Wave-71 STORY-156 added **BC-2.16.016 v1.2** which documents an analogous case:

> "`ArpAnalyzer::process_arp` returns a `Vec<Finding>` with NO upper bound on the number
> of findings it may contain. Unlike the stream-reassembly analyzers (HTTP, TLS, Modbus,
> DNP3) which bound their findings output via the reassembly layer `MAX_FINDINGS = 10,000`
> cap, `process_arp` operates at the Ethernet link layer and bypasses the reassembly path
> entirely (BC-2.16.015 Invariant 2)… This absence of a findings cap is intentional
> design."

Notably BC-2.16.016 explicitly says the TLS/HTTP/Modbus/DNP3 stream-analyzers **DO** bound
their output via the reassembly layer cap — which contradicts the BC-2.04.024 invariant
4 / AC-007b assertion (via test seams) that `TlsAnalyzer::all_findings` is unbounded at
the analyzer level. This is a **spec-vs-spec tension**, not a code defect:

- BC-2.04.024 invariant 4: `TlsAnalyzer` does not apply an analyzer-local cap; the cap
  lives in `TcpReassembler`.
- BC-2.16.016 rationale: stream analyzers "bound their findings output via the reassembly
  layer".

Both are internally coherent if read carefully: the reassembly-layer cap applies to what
the reassembler *emits* (10 000-finding cap for reassembler-owned findings), and
per-analyzer findings vecs accumulate independently on the analyzer side. The BC-2.16.016
narrative overstates the architectural cap coverage of TLS/HTTP/Modbus/DNP3 output — none
of those analyzers has an analyzer-local output cap either; they push to their own
`all_findings` Vec unbounded, exactly like `TlsAnalyzer::all_findings` does.

### Exploitability

- Threat model per tech-debt register: "Bounded in practice by capture file size; realistic
  threat model limited to offline analysis."
- wirerust is a passive, offline pcap analyzer (per STATE.md: "passive analyzer; no
  external service calls" — DTU skip rationale). Adversarial input is bounded by the
  analyst-controlled pcap file being examined.
- Growth rate: one Finding entry per TLS handshake/alert-detection event. A 1 GB pcap
  might yield tens of thousands of TLS findings; a Finding is roughly a
  `Verdict + Confidence + Category + Evidence-string`, on the order of hundreds of bytes.
  Even at 100 000 findings × 500 bytes ≈ 50 MB — non-catastrophic on any modern host.
- Not remotely reachable. No CVSS applies in the usual sense (attack vector: local file
  analyst-controlled).

### Verdict

**CONFIRMED as observation, spec-compliant on develop `b642c0f`. NOT a defect** per
BC-2.04.024 invariant 4 / AC-007b (analyzer non-cap is by-design). Wave-71's BC-2.16.016
established the house precedent that "unbounded per-analyzer findings Vec is intentional
for capture-file-bounded offline analysis." SEC-W70-001 is the same class of finding for
`TlsAnalyzer` — should receive the same disposition: **documented as intentional design,
not fixed with a cap.**

### Disposition

**Do NOT file a GitHub issue.** Recommended actions in order of increasing effort:

1. **(minimal)** Update the tech-debt register row to add cross-reference: "Spec-designed
   per BC-2.04.024 invariant 4 / AC-007b (analyzer non-cap enforced by test seams
   `all_findings_len_for_testing` / `push_finding_for_testing`). Same class as ARP
   BC-2.16.016 wave-71 disposition — unbounded is intentional; capture-file size is the
   real bound in the offline-analysis threat model." Mark **CLOSED — no action; documented
   intentional design.**
2. **(recommended)** Author a symmetric BC for TLS analogous to BC-2.16.016 — e.g.
   BC-2.07.NNN "TlsAnalyzer::all_findings Vec is Unbounded — No MAX_FINDINGS Cap by
   Design." Same structure as BC-2.16.016 v1.2, referencing BC-2.04.024 as the
   architectural anchor. This would give the finding a durable spec home (the way
   wave-71 gave ARP a durable spec home for the same design choice).
3. **(deferred)** Reconcile BC-2.16.016's narrative that stream analyzers "bound their
   findings output via the reassembly layer" — that statement overstates the coverage.
   Either narrow it ("the *reassembler-emitted* findings for stream flows are bounded by
   `TcpReassembler::MAX_FINDINGS = 10,000`; the per-analyzer `all_findings` Vecs are
   independent and unbounded, same as ARP") or leave alone and note the drift in
   BC-INDEX for future spec-coherence sweep.

If a GitHub issue is desired at all (not recommended): title
`spec: document TlsAnalyzer::all_findings unbounded-cap as intentional design (parallel to ARP BC-2.16.016)`;
labels `spec`, `docs`, `informational`.

---

## Canonical-ID Collision — Wave-71 "CR-001" vs Tech-Debt Register "CR-001"

Item 7 in this triage names the wave-71 code-review finding **CR-001 (wave-71)**. The
tech-debt register (`.factory/tech-debt-register.md`) already contains a distinct row:

```
| CR-001 | [Phase-5 secondary review, MEDIUM] dispatcher `pub` analyzer fields …
          | P2 | Phase-5 secondary code-review | CLOSED — merged PR #177 (02e9c00) 2026-06-01 |
```

Also listed under "Resolution History": `CR-001 | 2026-06-01 | PR #177 | dispatcher pub
fields encapsulated`.

These are two entirely different findings sharing the label `CR-001`. If either is copied
verbatim into a downstream registry or GitHub issue, they will collide.

**Proposed rename:** the wave-71 finding should be referenced as **`CR-W71-001`** in all
downstream artifacts (STATE.md, tech-debt register, backlog, gate-summary references).
Naming convention matches the wave-scoped SEC prefix (`SEC-W70-001`, `SEC-W71-001`) already
in use in this repo.

**Concrete edits recommended (out of scope for this triage; team-lead to schedule):**

- `.factory/STATE.md` deferred-findings register: rename "CR-001 (wave-71)" →
  "CR-W71-001 (wave-71)".
- `.factory/cycles/wave-71/wave-gate/gate-summary.md` dimension (c) row: rename
  "CR-001 MINOR + 3 NITs" → "CR-W71-001 MINOR + 3 NITs".
- `.factory/tech-debt-register.md` Open Items table: when the wave-71 code-review
  artefact is recovered (see item 7 disposition) and rows are added, use `CR-W71-001` +
  `CR-W71-002/003/004` (or `NIT-W71-001..003`) — do NOT reuse `CR-001`, `CR-002`, etc.
  (those pre-exist).

The register writer should NOT merge these into the closed `CR-001` (PR #177) row.

---

## Verdict Summary Table

| ID | Verdict | Disposition |
|----|---------|-------------|
| SEC-W71-001 | **CONFIRMED** (CWE-22, still open on `b642c0f`) | **File GitHub issue** — combined path-traversal + absolute-path hardening for `bin/compute-input-hash`; also patch gate-summary + STATE.md to make the `SEC-W71-001 → SEC-001` mapping explicit |
| REBIND-COUNT-SATURATING-001 | **CONFIRMED** (site + 13 sibling `+=` sites) | **Fold into fix PR this sweep** (arp.rs saturating_add sweep, DF-SIBLING-SWEEP-001) OR **defer**; do NOT file GitHub issue |
| INPUT-HASH-ERROR-STORIES-001 | **REFUTED** (091/121 by-design; 001 hash MATCH) | **Close backlog row**; optional spec-traceability micro-item for STORY-001 → BC-2.01.009 back-ref |
| HS-INDEX-ENIP-WAVE-DRIFT-001 | **CONFIRMED** (waves 58-61, stories 130-138) | **Fold into spec-coherence fix PR this sweep** (HS-INDEX one-file edit, three occurrences) |
| EPICS-TOTAL-BCS-DRIFT-001 | **CONFIRMED** (delta 8; 6 named + 2 unresolved) | **Fold into spec-coherence fix PR this sweep** (epics.md E-5 row + arithmetic + `total_bcs` → 345 after resolving residual 2) |
| DNP3-CLOSEDFLOW-REOPEN-REUSE-001 | **CONFIRMED observation / REFUTED as defect** | **Defer** — spec-conformant per BC-2.15.021 PC-4; optional docstring rename; no GitHub issue |
| CR-001 (wave-71) → **rename `CR-W71-001`** | **UNVERIFIABLE** (no primary evidence file) | **Close as under-specified**; add PG-W71-CODEREVIEW-ARTIFACT to STORY-158; do NOT file GitHub issue; **rename to CR-W71-001 to resolve collision with closed pre-existing CR-001 (PR #177)** |
| STORY-148-BASIS-RESOLVED-001 | **ALREADY-RESOLVED** | **Close backlog row**; drop from STATE.md deferred register |
| SEC-010 | **CONFIRMED** (test/bench-only; zero `as u16` in `src/`) | **Fold into fix PR this sweep** — add `debug_assert!(payload.len() <= u16::MAX as usize)` to `wrap_as_tls_record`; sibling-sweep other `.len() as u16` fixture builders; reconcile CWE tag → CWE-197 |
| SEC-011 | **CONFIRMED residual** (Gap A CLOSED at 5b41eca; Gap B enumeration hole persists) | **Fold into fix PR this sweep** — add `!body.contains("self.flows[")` to anti-gameability enumeration in `tests/bc_149_single_borrow_invariant_tests.rs:240-272`; correct register wording |
| SEC-W70-001 | **CONFIRMED observation / NOT a defect** (spec-designed per BC-2.04.024 inv-4 / AC-007b; wave-71 BC-2.16.016 established house precedent for the same class) | **Close backlog row — documented intentional design**; recommend authoring symmetric BC-2.07.NNN "TlsAnalyzer all_findings unbounded by design" mirroring BC-2.16.016 v1.2; no GitHub issue |

## External Citations

- MITRE CWE-22 "Improper Limitation of a Pathname to a Restricted Directory":
  <https://cwe.mitre.org/data/definitions/22.html> (SEC-W71-001).
- MITRE CWE-197 "Numeric Truncation Error":
  <https://cwe.mitre.org/data/definitions/197.html> (SEC-010 recommended canonical tag;
  supersedes CWE-704/CWE-190 mixed tagging in STORY-149 security-review.md).
- MITRE CWE-770 "Allocation of Resources Without Limits or Throttling":
  <https://cwe.mitre.org/data/definitions/770.html> (SEC-W70-001 — CWE tag from wave-70
  convergence state and tech-debt register; retained here for reference even though
  disposition is CLOSED-BY-DESIGN).
- MITRE CWE-693 "Protection Mechanism Failure":
  <https://cwe.mitre.org/data/definitions/693.html> (SEC-011 Gap B — CWE tag from
  STORY-149 security-review §SEC-002; anti-gameability enforcement hole).

All other verdicts rely on in-repo evidence at `develop = b642c0f` cited inline.
