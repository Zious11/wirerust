# AC-169-006 — `parse_asdu` Purity: No Side Effects, No Finding Emission, No State Mutation

**Story:** STORY-169: IEC-104 ASDU Header Extraction: parse_asdu / Asdu with Broken-Out DUI Fields  
**AC:** AC-169-006  
**Traces to:** BC-2.19.015 invariant 2; ADR-013 Decision 8  
**Wave:** 78

---

## Acceptance Criterion

- `parse_asdu` emits no findings, mutates no shared state, and performs no I/O
- The extraction layer returns an `Option<Asdu>` data value
- The calling effectful layer (STORY-170) emits findings (e.g., T0814 on `None`)
- Calling `parse_asdu` twice with identical input produces identical output (determinism invariant)

---

## Structural Evidence

### Signature Verification

Command:
```
grep -n "pub fn parse_asdu\|pub struct Asdu\|pub type_id\|pub sq\|pub count\|pub cot_cause\|pub cot_pn\|pub cot_test\|pub cot_originator\|pub casdu\|pub first_ioa" src/analyzer/iec104.rs
```

Output:
```
468:pub struct Asdu {
473:    pub type_id: u8,
477:    pub sq: bool,
481:    pub count: u8,
485:    pub cot_cause: u8,
489:    pub cot_pn: bool,
494:    pub cot_test: bool,
497:    pub cot_originator: u8,
501:    pub casdu: u16,
506:    pub first_ioa: Option<u32>,
554:pub fn parse_asdu(asdu_body: &[u8]) -> Option<Asdu> {
```

`parse_asdu` takes `&[u8]` (immutable slice reference) and returns `Option<Asdu>`.
No `&mut` parameters. No `Iec104FlowState` argument. No `Finding` output. No `Vec` side channel.

### Purity Classification (ADR-013 Decision 8)

| Property | Evidence |
|----------|----------|
| No I/O | Signature: `fn parse_asdu(asdu_body: &[u8]) -> Option<Asdu>` — no I/O traits, no file handles |
| No state mutation | No `&mut` parameter; `Iec104FlowState` not referenced |
| No finding emission | Return type is `Option<Asdu>`, not `(Option<Asdu>, Option<Finding>)` or similar |
| Deterministic | Determinism invariant test passes (see below) |
| Total (no panic) | No-panic invariant test passes (see below) |

---

## Determinism Invariant Test

The purity claim is directly exercised by `test_BC_2_19_015_invariant_parse_asdu_pure_deterministic`
(part of the BC-2.19.015 test group, filed under AC-169-001). The test calls `parse_asdu` twice
with the same input and asserts both results are equal — demonstrating no hidden state is mutated
between calls.

The test is shown passing in the AC-001 evidence (`5/5 PASS` for `story_169::test_BC_2_19_015`):

```
test story_169::test_BC_2_19_015_invariant_parse_asdu_pure_deterministic ... ok
```

---

## Pure/Effectful Boundary (ADR-013 §Decision 8)

```
parse_asdu (pure-core)
  Input:  &[u8]   (immutable ASDU body slice)
  Output: Option<Asdu>  (data value; no side effects)
  ┌──────────────────────────────────────────────────────────┐
  │  Returns None:  body.len() < 6                           │
  │  Returns Some:  9 broken-out fields extracted            │
  │  No findings, no mutations, no I/O                       │
  └──────────────────────────────────────────────────────────┘

STORY-170 on_data (effectful-shell)  [NOT this story]
  Calls parse_asdu(...)
  On None:    emits T0814 "Denial of Service" finding
  On Some:    dispatches on type_id for TypeID-based detection
```

The effectful boundary is preserved: `parse_asdu` returns data; `on_data` (STORY-170) acts on it.

---

## Verdict

AC-169-006: **PASS** — `parse_asdu` is a pure free function with signature `fn(&[u8]) -> Option<Asdu>`;
no `&mut` parameters, no `Iec104FlowState` argument, no finding emission, no I/O.
Determinism invariant test (`test_BC_2_19_015_invariant_parse_asdu_pure_deterministic`) passes.
ADR-013 Decision 8 pure/effectful boundary maintained.
