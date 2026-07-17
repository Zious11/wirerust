# Holdout-Expectations Sweep — FIX-P4-001

**Policy:** PG-W72-BREAKING-HOLDOUT-SWEEP  
**Fix:** IEC104-FINDING-DIRECTION-001 — populate `direction` on all IEC-104 emitted Findings  
**Change type:** Additive JSON key (`direction`) now emitted on IEC-104 findings (previously absent via `skip_serializing_if = "Option::is_none"`)  
**Date:** 2026-07-16  
**holdout-expectations-sweep: COMPLETE**

---

## Scope Assessment

Per `breaking-change-delivery-protocol.md` Scope Trigger 2: the additive optional `direction`
key is an observable JSON output schema change (a key previously absent now appears on IEC-104
findings). Conservative reading: in-scope. Sweep executed.

Per `story-174-scope-validation-followup.md` Q1, this sweep was predicted to be near-empty
because: (a) no IEC-104 holdout scenarios exist; (b) holdout assertions are contains/subset,
not exact-match. Both predictions confirmed below.

---

## Sweep: .factory/holdout-scenarios/

**Search performed:** grep for `iec104`, `iec-104`, `IEC-104`, `IEC104`, `2404`, `T0881`,
`T1692`, `T0827` in `.factory/holdout-scenarios/` (all `.md` files).

**Result: ZERO IEC-104 holdout scenarios exist.**

The only `T0836`/`T0827` references found are EtherNet/IP scenarios (HS-113, HS-119, HS-122)
which are unaffected by this change.

---

## Sweep: Assertion mode in existing holdout scenarios

Checked HS-007 (JSON serialization / skip-None-fields) and HS-016 (real-world corpus):

- **HS-007** describes that `direction` is absent when `None` (a general serialization
  invariant). It does not assert IEC-104-specific finding shapes. The additive `direction` key
  on IEC-104 findings does NOT break HS-007: the scenario verifies that `None` fields are
  omitted, which remains true for all non-IEC-104 analyzers.
- **HS-016** uses `direction=None` in a generic Finding description; not IEC-104 specific.

All other holdout scenarios use "Assert findings contains …" / "contains …" patterns (subset
assertions), which survive additive fields.

---

## Sweep: tests/ for exact-JSON equality assertions on IEC-104 findings

Searched `tests/iec104_analyzer_tests.rs` for `serde_json`, `assert.*json`, `json.*direction`,
`direction.*json`, `to_string.*direction`. **Result: zero exact-JSON equality assertions
on IEC-104 findings.** No tests would break from the new `direction` key appearing in JSON.

---

## Sweep: demo-evidence fixtures

Searched the tree for IEC-104 demo-evidence fixtures that might assert exact finding JSON shape.
**Result: no IEC-104 demo-evidence fixtures found in docs/ or .factory/.**

---

## Verdict

| Location | Stale? | Action required |
|----------|--------|-----------------|
| `.factory/holdout-scenarios/` | No IEC-104 scenarios exist | None |
| `tests/iec104_analyzer_tests.rs` | No exact-JSON direction assertions | None |
| `docs/` demo-evidence | No IEC-104 demo-evidence fixtures | None |

**No repairs needed. All expectations are backward-compatible with the additive `direction`
key.**

holdout-expectations-sweep: COMPLETE
