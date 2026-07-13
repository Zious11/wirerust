# AC-162-001 / AC-162-002 — VP-INDEX LMR-003 Amendment + Version Bump

**Story:** STORY-162  
**ACs:** AC-162-001 (LMR-003 template-conformance amendment) and AC-162-002 (version bump 2.39→2.40)  
**Source file:** `.factory/specs/verification-properties/VP-INDEX.md` (factory-artifacts branch)

---

## AC-162-002 Verification: version field

Command:
```
grep "^version:" .factory/specs/verification-properties/VP-INDEX.md
```

Output:
```
version: "2.40"
```

Result: PASS — version is `"2.40"` as required.

---

## AC-162-001 Verification: template-conformance / inputs: / input-hash: in LMR-003

Command:
```
grep -n "template-conformance\|inputs:\|input-hash:" \
  .factory/specs/verification-properties/VP-INDEX.md
```

Selected output (lines containing amendment text — full match set):

```
8:modified: "2026-07-10: [v2.40] LMR-003 template-conformance provenance fields amendment
   (STORY-162 AC-162-001/002, F-S161P1-001) — LMR-003 Locked-Doc-Appendable Provenance Field
   Allowlist extended with two new rows: `inputs:` (permitted meaning: template-conformance
   provenance; MUST be `[]`; modified-log citation required) and `input-hash:` (permitted
   meaning: template-conformance provenance; MUST be `d41d8cd`; modified-log citation required).
   Definition of 'template-conformance provenance fields' added adjacent to allowlist.
   First application: VP-024 v2.5 (STORY-161/162, wave-72) cited as precedent per
   AC-162-001(c). Version 2.39->2.40."
```

Non-empty output confirms amendment text is present.

---

## LMR-003 Amendment — Key Excerpts

### Definition of template-conformance provenance fields (AC-162-001a)

```
**Definition — Template-Conformance Provenance Fields.**
A **template-conformance provenance field** is a frontmatter field appended to a locked
VP document solely to satisfy hook-mandated template conformance validation. Specifically,
the fields `inputs:` and `input-hash:` are template-conformance provenance fields when
added to a locked L4 VP document. They are **non-value-bearing** (the only permitted
values are `inputs: []` — empty list — and `input-hash: d41d8cd` — the MD5 of empty
bytes, which is the canonical hash when `inputs: []`) and **non-integrity** (they do not
anchor proof correctness, harness code, postconditions, property statements, or BC anchors;
the `input-hash:` value for a VP document with `inputs: []` is always `d41d8cd`).
```

### Allowlist extension (Option A — AC-162-001b)

The Locked-Doc-Appendable Provenance Field Allowlist was extended with two rows:

```
| Field name    | Permitted meaning                                                       | Hash/digest? | Notes                                  |
|---------------|-------------------------------------------------------------------------|--------------|----------------------------------------|
| `inputs:`     | Template-conformance provenance (hook-mandated; non-value-bearing;      | No           | First application VP-024 v2.5          |
|               | `inputs: []` only; no new spec-input dependencies; cite exemption       |              | (STORY-161/162, wave-72)               |
|               | in modified-log)                                                        |              |                                        |
| `input-hash:` | Template-conformance provenance (hook-mandated; non-value-bearing;      | Yes (MD5-    | First application VP-024 v2.5          |
|               | `input-hash: d41d8cd` only; must cite this exemption in modified-log)  | first-7) —   | (STORY-161/162, wave-72)               |
|               |                                                                         | advisory     |                                        |
|               |                                                                         | sentinel,    |                                        |
|               |                                                                         | NOT integrity|                                        |
|               |                                                                         | anchor;      |                                        |
|               |                                                                         | permitted    |                                        |
|               |                                                                         | ONLY as      |                                        |
|               |                                                                         | `d41d8cd`    |                                        |
```

### Precedent citation (AC-162-001c)

```
**First application precedent:** VP-024 v2.5 (STORY-161/162, wave-72) — `inputs: []` and
`input-hash: d41d8cd` added as bundled template-conformance hygiene per STORY-162 AC-162-001.
```

---

## Result

| AC | Criterion | Verdict |
|----|-----------|---------|
| AC-162-001 | LMR-003 amendment with definition + allowlist extension + VP-024 v2.5 precedent | PASS |
| AC-162-002 | VP-INDEX version bumped to `"2.40"` | PASS |
