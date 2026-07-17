# Holdout-Expectations Sweep — FIX-F5-001

**Policy:** PG-W72-BREAKING-HOLDOUT-SWEEP  
**Fix:** FIX-F5-001 — source_ip + timestamp enrichment on all IEC-104 emit sites  
**Change type:** Additive JSON keys (`source_ip`, `timestamp`) now populated on IEC-104
findings (previously absent via `skip_serializing_if = "Option::is_none"`; now
`Some(ip)` / `Some(ts)` appear in serialized output when the keys carry values)  
**Date:** 2026-07-17  
**holdout-expectations-sweep: COMPLETE**

---

## Scope Assessment

Per `breaking-change-delivery-protocol.md` Scope Trigger 2: `source_ip` and `timestamp`
are optional JSON fields. Previously they were always `None` on IEC-104 findings and thus
absent from serialized output. After this fix they carry real values, causing two keys to
appear in JSON that were previously absent. Conservative reading: in-scope. Sweep executed.

The FIX-P4-001 sweep (2026-07-16) established that no IEC-104 holdout scenarios exist and
no exact-JSON assertions are made on IEC-104 findings. This sweep re-verifies those findings
for `source_ip`/`timestamp` specifically.

---

## Sweep: .factory/holdout-scenarios/

**Search performed:** grep for `source_ip`, `timestamp`, `iec104`, `iec-104`, `IEC-104`,
`IEC104`, `2404`, `T0881`, `T1692.001`, `T0827`, `T0836` in `.factory/holdout-scenarios/`
and `.factory/` tree.

**Result: ZERO IEC-104 holdout scenarios exist.** No changes required.

The only `T0836`/`T0827` references found are EtherNet/IP scenarios (HS-113, HS-119, HS-122)
which are unaffected by this change (EnIP already populates source_ip/timestamp correctly).

---

## Sweep: Assertion mode in existing holdout scenarios

Checked generic scenarios for `source_ip` / `timestamp` exact-equality assertions:

- **HS-007** (JSON serialization / skip-None-fields): verifies that None-value fields are
  absent from serialized output. The additive populated `source_ip`/`timestamp` keys on
  IEC-104 findings satisfy this invariant — they now carry Some values, which is compliant
  with the skip-None-fields serialization rule (only `None` is omitted; `Some` is serialized).
  No conflict.
- **HS-016** and other scenarios: use "findings contains …" / subset assertions. The new
  fields do not invalidate subset containment checks.

---

## Sweep: tests/iec104_analyzer_tests.rs — exact-shape assertions

Searched for `serde_json`, `assert.*json`, `source_ip.*None`, `timestamp.*None` in the
context of IEC-104 on_data assertions (as opposed to test helpers).

**Result: approximately 30 `source_ip: None` / `timestamp: None` occurrences in
`dummy_finding()` — these are test-local stub values for pre-filling `all_findings`
capacity tests. They are not assertions about real IEC-104 analyzer output; they are
just test scaffolding. Unaffected by this fix.**

No test in `tests/iec104_analyzer_tests.rs` asserts `finding.source_ip == None` on any
finding produced by `on_data` with real frames. All such tests use
`.find(|f| f.mitre_techniques.contains(...))` pattern, not exact-shape comparison.

---

## Sweep: tests/ for direct source_ip None assertions on live findings

```
grep -n "source_ip.*None\|\.source_ip == None" tests/iec104_analyzer_tests.rs
```

**Result: zero assertions that `source_ip == None` on a finding returned by `on_data`.**
The only `source_ip: None` occurrences are inside `dummy_finding()` constructor at lines
5846-5847 and in the Iec104FlowState desync test helper at ~3446-3447. Neither is an
assertion on actual analyzer output.

---

## Sweep: demo-evidence fixtures

Searched `docs/demo-evidence/` and `.factory/` for IEC-104 demo-evidence fixtures with
exact finding JSON.

**Result: no IEC-104 demo-evidence fixtures found. No action required.**

---

## Verdict

| Location | Stale? | Action required |
|----------|--------|-----------------|
| `.factory/holdout-scenarios/` | No IEC-104 scenarios exist | None |
| `tests/iec104_analyzer_tests.rs` | No `source_ip == None` assertions on live output | None |
| `docs/` demo-evidence | No IEC-104 demo-evidence fixtures | None |
| HS-007 skip-None-fields invariant | Satisfied — Some values are serialized correctly | None |

**No repairs needed. The additive `source_ip`/`timestamp` fields on IEC-104 findings are
backward-compatible with all existing assertions and holdout expectations.**

holdout-expectations-sweep: COMPLETE
