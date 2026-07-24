# AC-180-007 — Silent-Range Code Comment Narrowed to {52–57, 65–99}

**Story:** STORY-180: IEC-104 Timed Control Command Detection: TypeIDs 58–64  
**AC:** AC-180-007  
**Traces to:** BC-2.19.022 v1.1 architecture anchor; BC-2.19.029 invariant 6 note; BC-2.19.030 invariant 6 note  
**Wave:** 85

---

## Acceptance Criterion

The code comment at `detect_iec104_threats` (previously lines 912–914, now lines 1013–1017
after the new arms were inserted) MUST:
- Name the silent range as `{52–57, 65–99}` (not `52–99`)
- State that TypeIDs 58–64 were removed from the silently-logged set
- Credit BC-2.19.029 (58–60) and BC-2.19.030 (61–64) as the handlers

---

## Source-Level Verification

Command:
```
grep -n "52.*57\|65.*99\|were here prior\|BC-2.19.029\|BC-2.19.030" src/analyzer/iec104.rs | tail -10
```

Output:
```
709:/// | 58–60 (C_SC/DC/RC_TA)       | T1692.001               | Possible | BC-2.19.029 |
710:/// | 61–64 (C_SE_TA/C_BO_TA)     | T1692.001 + T0836       | Possible | BC-2.19.030 |
714:/// | {52–57, 65–99, …} (unhandled) | none (silently logged)  | —        | BC-2.19.022 |
838:        // (BC-2.19.029 postconditions 1–2; invariants 1–2; AC-180-001/002).
875:        // (BC-2.19.030 postconditions 1–2; invariants 1–2; AC-180-003).
1014:        // TypeIDs 1–44 (monitoring direction), {52–57, 65–99}, 102 (C_RD_NA_1), 104, 106–127.
1015:        // TypeIDs 58–64 were here prior to wave-85-spec-evolution; they are now handled by
1016:        // BC-2.19.029 (58–60) and BC-2.19.030 (61–64).
1017:        // No finding emitted — silently logged (BC-2.19.022 v1.1 invariant 1; AC-170-005).
```

---

## Comment Text (src/analyzer/iec104.rs, lines 1013–1017)

```rust
// Defined-but-unhandled TypeIDs in [1, 127] not covered by the arms above:
// TypeIDs 1–44 (monitoring direction), {52–57, 65–99}, 102 (C_RD_NA_1), 104, 106–127.
// TypeIDs 58–64 were here prior to wave-85-spec-evolution; they are now handled by
// BC-2.19.029 (58–60) and BC-2.19.030 (61–64).
// No finding emitted — silently logged (BC-2.19.022 v1.1 invariant 1; AC-170-005).
```

---

## Dispatch Table Docstring Update (src/analyzer/iec104.rs, line 714)

The dispatch table docstring was also updated:
```
| {52–57, 65–99, …} (unhandled) | none (silently logged)  | —        | BC-2.19.022 |
```
Previously read `52–99`; now reflects `{52–57, 65–99}` after the removal of 58–64.

---

## Verdict

AC-180-007: **PASS** — The silent-range comment at lines 1013–1017 names `{52–57, 65–99}`,
states that TypeIDs 58–64 were removed from the catch-all, and credits BC-2.19.029 and
BC-2.19.030 as the handlers. The dispatch-table docstring at line 714 also reflects
the narrowed range. Both are confirmed by source grep above.
