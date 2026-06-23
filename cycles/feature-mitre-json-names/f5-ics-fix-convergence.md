---
title: "F5 ICS Tactic-Catalog Fix — Per-Fix Adversarial Convergence Report"
branch: fix/ics-tactic-ids
finding: "F-1 HIGH — ICS techniques emitting Enterprise tactic IDs"
remediation_commit: 719816e
passes: 3
verdict: CONVERGED
date: 2026-06-23
---

# F5 ICS Tactic-Catalog Fix — Per-Fix Adversarial Convergence Report

## Summary

Three clean fresh-context adversarial passes were conducted on branch `fix/ics-tactic-ids`
after the F-1 HIGH remediation commit (719816e). All passes returned zero HIGH or CRITICAL
findings. The fix is eligible for fix-PR to develop.

## Background

F5 scoped-adversarial review (D-209) found **F-1 HIGH**: ICS techniques under
`mitre_domain=ics-attack` were emitting Enterprise tactic IDs (e.g., Discovery TA0007 instead
of ICS Discovery TA0102). Research-validated against MITRE ATT&CK ICS v19.1. Three additional
errors found: T0830 (Adversary-in-the-Middle) was mapped to Lateral Movement when the
authoritative tactic is Collection/TA0100; T0831 (Manipulation of Control) was mapped to
Impair Process Control when the authoritative tactic is Impact/TA0105; T0885 was missing
IcsCommandAndControl.

Human-authorized comprehensive catalog fix produced commit 719816e: 3 new MitreTactic
variants, 5 techniques remapped.

---

## Pass 1 — Commit 74a48ea (demo re-recorded after 719816e)

**Result: CLEAN — 0 HIGH / 0 CRITICAL**

### Findings

| ID | Severity | Summary |
|----|----------|---------|
| L-1 | LOW | Stale comment on T0830 in `src/mitre.rs` still referenced an earlier (incorrect) tactic description. |

### Process Gap

**Drift guard does not pin TA-id values.** The existing test for ICS techniques only asserted
that ICS techniques do not emit Enterprise tactic IDs (absence check), but did not assert the
exact TA-id each technique should emit (presence + value check). This means a future edit that
swaps two ICS TA-ids would not be caught by the test suite.

### Resolution

Both issues addressed in commit **cf22de9**:
- Stale comment on T0830 corrected.
- New test `test_ics_techniques_resolve_authoritative_tactic_ids` added: asserts all 12
  ICS-domain techniques against exact expected TA-id pairs (T0840→TA0100, T0830→TA0100,
  T0831→TA0105, T0846→TA0102, T0888→TA0102, T0880→TA0104, T0855→TA0104, T0885→TA0101,
  T0836→TA0108, T0814→TA0108, T0879→TA0110, T0826→TA0110).

---

## Pass 2 — Commit cf22de9 (hardening: comment fix + authoritative TA-id pin test)

**Result: CLEAN — 0 HIGH / 0 CRITICAL**

### Pass-1 Process Gap

Confirmed closed. `test_ics_techniques_resolve_authoritative_tactic_ids` is present and
asserts 12 exact id→TA-id pairs. The drift guard now pins values, not just absence.

### Observations (non-blocking)

| ID | Severity | Summary |
|----|----------|---------|
| OBS-2-1 | LOW | Pre-existing comment-grouping quirk in `src/mitre.rs`: the ICS technique arm comments do not follow a uniform grouping convention used elsewhere, but this is cosmetic and pre-dates this fix. |
| OBS-2-2 | LOW | No ARP pcap fixture in `tests/fixtures/` for a live demonstration of the T0830→Collection(ICS)/TA0100 remap. The remap is unit-test-verified but not visually demonstrated via a packet-level end-to-end run. |

Both observations are non-blocking. OBS-2-2 is recorded as a backlog deferral (see below).

---

## Pass 3 — Commit cf22de9 (same head — fresh-context re-verification)

**Result: CLEAN — 0 HIGH / 0 CRITICAL / 0 MEDIUM (by novelty class)**

### Authoritative Tactic Re-verification

All 20 MitreTactic TA-ids re-verified against MITRE ATT&CK ICS v19.1 authoritative source:

| MitreTactic Variant | TA-id | Authoritative Name |
|---------------------|-------|--------------------|
| IcsInitialAccess | TA0108 | Initial Access |
| IcsExecution | TA0104 | Execution |
| IcsPersistence | TA0110 | Persistence |
| IcsPrivilegeEscalation | TA0111 | Privilege Escalation |
| IcsEvasion | TA0103 | Evasion |
| IcsDiscovery | TA0102 | Discovery |
| IcsLateralMovement | TA0109 | Lateral Movement |
| IcsCollection | TA0100 | Collection |
| IcsCommandAndControl | TA0101 | Command and Control |
| IcsInhibitResponseFunction | TA0107 | Inhibit Response Function |
| IcsImpairProcessControl | TA0106 | Impair Process Control |
| IcsImpact | TA0105 | Impact |
| Discovery | TA0007 | Discovery (Enterprise) |
| LateralMovement | TA0008 | Lateral Movement (Enterprise) |
| Exfiltration | TA0010 | Exfiltration (Enterprise) |
| CommandAndControl | TA0011 | Command and Control (Enterprise) |
| Persistence | TA0003 | Persistence (Enterprise) |
| PrivilegeEscalation | TA0004 | Privilege Escalation (Enterprise) |
| Impact | TA0040 | Impact (Enterprise) |
| ImpairProcessControl | TA0106 | Impair Process Control (ICS — pre-existing alias) |

All 20 verified correct. No regressions introduced.

### Finding Summary

| Severity | Count | Notes |
|----------|-------|-------|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 0 | — |
| LOW | 0 | No novel findings at this pass |
| INFORMATIONAL | 0 | — |

---

## Convergence Trajectory

```
Pass 1 (74a48ea): 1 LOW + 1 process-gap → addressed in cf22de9
Pass 2 (cf22de9): process-gap CLOSED; 2 LOW observations (non-blocking)
Pass 3 (cf22de9): 0 findings (novel); all 20 TA-ids re-verified
```

**CONVERGED** — Three consecutive passes, zero HIGH/CRITICAL, all gates green.

---

## Final Branch State

| Item | Value |
|------|-------|
| Branch | fix/ics-tactic-ids |
| HEAD | cf22de9 |
| Commits | 719816e (fix), 74a48ea (demo), cf22de9 (hardening) |
| New MitreTactic variants | 3 (IcsDiscovery TA0102, IcsCollection TA0100, IcsCommandAndControl TA0101) |
| Techniques remapped | 5 (T0846, T0888 → IcsDiscovery; T0885 → IcsCommandAndControl; T0830 → IcsCollection; T0831 → IcsImpact) |
| BCs bumped | 5 (BC-2.10.002 v1.6, BC-2.10.003 v1.5, BC-2.10.007 v1.9, BC-2.11.035 v1.1, BC-2.16.004 v1.8) |
| Test suite | Full suite green; clippy -D warnings clean; fmt clean |
| Authoritative-pin test | `test_ics_techniques_resolve_authoritative_tactic_ids` (12 exact id→TA-id pairs) |

---

## Backlog Deferrals

### DRIFT-ARP-DEMO-FIXTURE-001 [LOW]

No ARP pcap fixture in `tests/fixtures/`, so the T0830→Collection(ICS)/TA0100 remap is
unit-test-verified but not visually demonstrated via a packet-level end-to-end run. Deferred —
add an ARP fixture and demo in a future cycle. Reason: LOW severity; correctness is fully
covered by `test_ics_techniques_resolve_authoritative_tactic_ids`.

### DRIFT-MITRE-SUBSET-COUNT-TESTS-001 [LOW]

The `mitre/multitag` test suite contains dual-count subset tests (21/13 seeded/emitted vs the
STORY-114-superseding 25/17 counts). Pre-existing cruft; the tests cover technique-name and
emission, not tactic-id values. Deferred — consolidate in a future maintenance sweep. Reason:
LOW severity, no correctness impact; authoritative TA-id pin test already closes the
value-correctness gap.

---

## Verdict

**CONVERGED.** Branch `fix/ics-tactic-ids` @ `cf22de9` is eligible for fix-PR to develop.
