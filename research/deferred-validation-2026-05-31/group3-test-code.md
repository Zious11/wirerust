# Deferred-Item Validation — Group 3: Test-Code (LOW)

**Date:** 2026-05-31
**Branch:** develop (claim baseline 9954d44; validated against current working tree)
**Scope:** Five LOW test-code deferred items, all touching tracked code under `tests/` or `src/.../#[cfg(test)]`.
**Validator:** research-agent (vsdd-factory)

> Note on line numbers: every claim cited a stale line number (the test files are large and have
> drifted since the claims were filed). Each item below is re-grounded against the **current**
> file/line in the working tree. Line drift alone is not "stale" — the underlying code is what is
> validated.

---

## Summary Table

| Item | Verdict | File | Test-only (no src risk)? |
|------|---------|------|--------------------------|
| W10-D10-sibling | **CONFIRMED** | tests/reassembly_engine_tests.rs:14143 | Yes |
| F-DRIFT-C-001 | **CONFIRMED (partial — narrower than claimed)** | src/analyzer/http.rs:689-690 | Yes |
| F-S058-P11-001 | **CONFIRMED** | tests/tls_analyzer_tests.rs:7515 | Yes |
| F-S058-P11-002 | **CONFIRMED** | tests/tls_analyzer_tests.rs:8598 | Yes |
| W20-NIT-001 | **VALID as ADD, but LOW marginal value** | tests/reporter_json_tests.rs | Yes (addition) |

---

## W10-D10-sibling [LOW, test-quality] — CONFIRMED

**Claim:** `test_story_018_ec008` re-implements the 10,000-flow fill loop inline instead of using
the existing `fill_findings_to_cap` helper.

**Validity:** CONFIRMED. The inline duplication is real and still present.

**Grounding (current lines):**
- Helper: `fill_findings_to_cap` at `tests/reassembly_engine_tests.rs:11544-11578`.
- Duplicating test: `test_story_018_ec008_truncated_at_max_findings_cap_drops_finding` at
  `tests/reassembly_engine_tests.rs:14128`, with its inline `for port in 1u16..=10_000u16` fill
  loop at `14143-14176`.
- The helper is actively reused by 6 other call sites (12306, 12579, 12904, 13239, 15393, plus its
  own def), so the pattern of reuse is already established — this one test is the outlier.

**Current vs. correct — the loops are NOT byte-identical (important caveat for the refactor):**

| Aspect | `fill_findings_to_cap` (helper) | `test_story_018_ec008` (inline) |
|--------|--------------------------------|----------------------------------|
| dst port | `8080` | `80` |
| conflict mechanism | original `b"A"` @ seq 1002 then conflicting `b"Z"` @ seq 1002 (no gap; in-order) | OOO `b"AAAA"` @ seq 1002 with **PSH set + gap at offset 1** then conflicting `b"BBBB"` @ seq 1002 |
| ReassemblyConfig | uses caller-passed config | `max_depth: 10, max_flows: 20_000` |
| client/server const order | `client`/`server` declared together | declared `server` then `client` (cosmetic) |

Both loops produce exactly 10,000 ConflictingOverlap findings (the test asserts
`findings().len() == 10_000` at 14178), so they are **behaviorally equivalent for the cap
precondition**. The inline version deliberately uses an out-of-order segment with a gap
("gap prevents immediate flush", comment at 14160) — this is a real semantic choice, not noise,
but the *resulting finding count* (the only thing EC-008 needs) is identical.

**Recommended fix:** Replace the inline loop (`14130-14182`, i.e. the config + reassembler/handler
setup + fill loop + the precondition assert) with:
```rust
let config = ReassemblyConfig {
    max_depth: 10,
    max_flows: 20_000,
    ..ReassemblyConfig::default()
};
let mut reassembler = fill_findings_to_cap(config);
let mut handler = RecordingHandler::new();
let server = [10, 0, 0, 2];
let client = [10, 0, 0, 1];
assert_eq!(reassembler.findings().len(), 10_000, "EC-008 precondition: ...");
```
`fill_findings_to_cap` already accepts a `config` parameter, so the `max_depth`/`max_flows` intent
is preservable. The helper uses dst port 8080 on ports 1..=10_000; the post-fill "trigger" packet
in this test uses `new_port = 10_001` (14187) which does not collide — consistent with the helper's
existing collision-avoidance comments at other call sites (e.g. 12916, 15407).

**Caveat to flag in the PR:** the helper uses an *in-order* original+conflict pair while the inline
test uses an *out-of-order gapped* pair. If EC-008's intent specifically requires exercising the
OOO/gap code path to reach the cap, the refactor would lose that distinction. Recommendation:
refactor to the helper (DRY win, the assertion proves equivalence), and if the OOO path matters,
keep one dedicated non-helper test for it rather than duplicating the full 10k loop. Given EC-008's
docstring ("the EC variant of AC-005 — exercises the same cap behavior", 14126) the cap behavior,
not the fill mechanism, is the contract — so the helper is appropriate.

**Test-only:** Yes. No `src/` change.

**External research:** Pure codebase fact for the duplication itself. DRY test-helper reuse is a
well-established convention (extract shared fixture builders); no Perplexity query adds anything
beyond confirming the obvious — the only non-trivial judgment (semantic divergence of the two
loops) is a codebase fact established above. No external research needed.

---

## F-DRIFT-C-001 [LOW, cosmetic] — CONFIRMED (but narrower than the claim states)

**Claim:** In `truncate_uri`'s test a doc-comment says "5 'é' = 10 bytes" but the fixture is
"éééé" (4 chars / 8 bytes); logic correct, only the comment is wrong.

**Validity:** CONFIRMED that a comment/fixture mismatch exists — but the claim is **mischaracterized
in two ways** and must be corrected before filing:

1. **The test lives in `src/analyzer/http.rs`** (`#[cfg(test)]` module), not in `tests/`. Function
   `truncate_uri` is defined at `src/analyzer/http.rs:106`; the test
   `test_BC_2_06_010_truncate_uri_multibyte_two_byte_codepoint` is at `687`.
2. **The stale phrase is "a string of 5 'é' characters is 10 bytes", not literally "5 'é' = 10 bytes".**
   The arithmetic in the comment ("5 × 2 = 10") is *internally* correct ('é' = U+00E9 = 0xC3 0xA9 =
   2 bytes, so 5×2 = 10), but it **does not match the fixture**, which is 4 'é' characters.

**Grounding (current lines), `src/analyzer/http.rs`:**
- Line 689-691 (comment):
  `// 'é' encodes as [0xC3, 0xA9] (2 bytes).  A string of 5 'é' characters`
  `// is 10 bytes.  A limit of 3 falls mid-codepoint (after byte 2 of the`
  `// second 'é').  floor_char_boundary(3) must round down to 2.`
- Line 692 (fixture): `let uri = "éééé"; // 4 × 2 = 8 bytes`

So the **inline trailing comment at line 692 is already correct** ("4 × 2 = 8 bytes"). Only the
**block comment at 689-690** still references the wrong count (5 chars / 10 bytes). This is a
residual of a partial earlier fix: the fixture and its inline comment were corrected to "éééé"/8
bytes, but the prose block comment two lines above was not updated. The logic and all four
assertions (a-d, 696-717) are correct and the test passes.

**Arithmetic confirmation:** 'é' (U+00E9) is **2 bytes in UTF-8** (0xC3 0xA9) — the claim's
parenthetical is correct. 4 × 2 = 8 bytes (matches fixture). 5 × 2 = 10 bytes (matches the stale
comment's internal math but not the fixture).

**Current vs. correct:**
- Current (689-690): `// [0xC3, 0xA9] (2 bytes).  A string of 5 'é' characters` / `// is 10 bytes.  A limit of 3 falls mid-codepoint (after byte 2 of the`
- Correct: `// [0xC3, 0xA9] (2 bytes).  A string of 4 'é' characters` / `// is 8 bytes.  A limit of 3 falls mid-codepoint (after byte 2 of the`

**Recommended fix:** In `src/analyzer/http.rs`, edit the block comment at lines 689-690 to say
"4 'é' characters" and "is 8 bytes". No code change; the fixture, inline comment, and assertions
are already consistent at 8 bytes.

**Test-only:** Yes — the change is confined to a `#[cfg(test)]` comment in `src/analyzer/http.rs`.
No runtime `src` logic is touched, so zero production risk. (Note for routing: this edits a file
under `src/`, but only a comment inside a test module.)

**External research:** Pure codebase + arithmetic fact ('é' UTF-8 width is fixed). No external
research needed.

---

## F-S058-P11-001 [LOW, deferred] — CONFIRMED

**Claim:** A stale "sync to story after this pass" process-comment exists and should be removed.

**Validity:** CONFIRMED. Present and stale.

**Grounding (current line):** `tests/tls_analyzer_tests.rs:7515`:
```
// Generic-citation ACs (names chosen here, sync to story after this pass):
```
This heads a block (7515-7526) mapping generic-citation ACs (AC-002..AC-015) to chosen test
function names. The "sync to story after this pass" is a transient TODO-style process note directed
at a past authoring pass, not durable test documentation. The named-AC block directly above
(7509-7513) is the "already synced" counterpart, which underscores that this note is a leftover
work-marker.

**Current vs. correct:**
- Current: `// Generic-citation ACs (names chosen here, sync to story after this pass):`
- Correct: `// Generic-citation ACs (names chosen here):`  (drop the trailing process clause)

**Recommended fix:** Remove the `, sync to story after this pass` clause from line 7515 (or, if the
sync genuinely never happened and the story still lacks exact `Test:` citations, escalate that as a
separate doc-sync task — but the *comment* itself is stale and should not remain as a perpetual
in-code TODO). Lowest-risk action: trim to `// Generic-citation ACs (names chosen here):`.

**Test-only:** Yes. Comment-only edit in a `tests/` file.

**External research:** Pure codebase fact. No external research needed.

---

## F-S058-P11-002 [LOW, cosmetic] — CONFIRMED

**Claim:** `test_nonhandshake_types` EC-label header lists EC-002/003/004 but the body covers
EC-001-004 (header/body mismatch).

**Validity:** CONFIRMED. The header omits EC-001, which the body covers.

**Grounding (current lines), `tests/tls_analyzer_tests.rs`:**
- Test fn (current name): `test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently` at `8597`.
- Header label (8598): `// F-S058-P1-003 / AC-013 extension (BC-2.07.033 EC-002/003/004 + STORY EC-006/EC-007):`
- Body type list (8604-8608):
  - `0x14 — ChangeCipherSpec (BC-2.07.033 EC-002)`
  - `0x15 — Alert            (BC-2.07.033 EC-003)`
  - `0x17 — ApplicationData  (BC-2.07.033 EC-001, ...)`  ← **EC-001, absent from header**
  - `0x18 — Heartbeat/unknown (BC-2.07.033 EC-004 / STORY-058 EC-006/007)`

The body enumerates EC-001, EC-002, EC-003, EC-004 (plus STORY EC-006/EC-007 for 0x18). The header
lists only `EC-002/003/004`. EC-001 (ApplicationData / 0x17) is covered in the body but missing
from the header. The header's `+ STORY EC-006/EC-007` matches the 0x18 body annotation, so only the
BC-EC range is wrong.

**Current vs. correct:**
- Current header (8598): `(BC-2.07.033 EC-002/003/004 + STORY EC-006/EC-007):`
- Correct header: `(BC-2.07.033 EC-001/002/003/004 + STORY EC-006/EC-007):`

**Recommended fix:** Add `EC-001/` to the header at line 8598 so it reads `EC-001/002/003/004`. No
code/assertion change — the test already exercises 0x17/EC-001 via the `type_cases` table at 8616.

**Test-only:** Yes. Comment/label-only edit in a `tests/` file.

**External research:** Pure codebase fact. No external research needed.

---

## W20-NIT-001 [LOW, optional ADD] — VALID addition, LOW marginal coverage value

**Claim:** An OPTIONAL future U+0080 C1-boundary test for JsonReporter byte handling
(STORY-076, PR#157). This is an ADDITION, not a fix.

**Validity:** The proposed test is *valid and correct* (it would pass and assert true behavior), but
its **marginal coverage value over the existing tests is LOW**. Recommendation below.

**Current JsonReporter byte-handling coverage (`tests/reporter_json_tests.rs`):**
| Codepoint | Class | Existing test | Asserted behavior |
|-----------|-------|---------------|-------------------|
| 0x00 NUL, 0x07 BEL, 0x1B ESC | C0 (U+0000–U+001F) | `..._del_not_escaped` round-trip @ 304, `..._round_trip` @ 333 | escaped to `\uNNNN`, raw byte absent |
| 0x7F DEL | C0/ASCII boundary (top of ASCII) | `test_BC_2_11_003_del_not_escaped_in_json` @ 304 | passes through raw 0x7F, NOT escaped |
| U+009B CSI | C1 (mid-range, 0xC2 0x9B) | `test_BC_2_11_005_c1_passthrough_raw_utf8` @ 512 | passes through raw 0xC2 0x9B, NOT escaped |
| U+009B + ESC together | C0/C1 asymmetry | `test_BC_2_11_005_c0_escaped_c1_passthrough_in_same_string` @ 552 | C0 escaped, C1 raw, in one string |
| Cyrillic (U+043F…U+0444) | non-control multibyte | `test_BC_2_11_004_...` @ 410+ | raw UTF-8, not escaped |

**Boundary analysis (RFC 8259 + serde_json — externally verified):**
RFC 8259 mandates escaping ONLY of U+0000–U+001F (plus `"` U+0022 and `\` U+005C). It does **not**
require escaping U+007F (DEL) nor the C1 block U+0080–U+009F. serde_json (default) follows this
exactly: it escapes only U+0000–U+001F + `"`/`\`, and emits everything else (including U+0080) as
raw UTF-8. U+0080 encodes as the two raw bytes **0xC2 0x80** in the output and is NOT ``-escaped.
(Sources: RFC 8259 §7; serde_json default escaping behavior — see Research Methods.)

**Does U+0080 add real coverage?**
- U+0080 is the **exact bottom boundary of the C1 block** (first codepoint of the 0x80–0x9F range)
  and the first codepoint immediately above U+007F. The existing C1 test (U+009B) sits in the
  middle of the C1 block; the existing DEL test (U+007F) sits just below it. So U+0080 fills the
  one untested edge between "U+007F raw" and "U+009B raw".
- However, serde_json's escaping decision is a single threshold at U+001F — there is **no special
  handling that changes at U+0080 vs U+009B**. Both are "> U+001F → pass through raw." A U+0080 test
  would exercise the identical code path as the existing U+009B test, differing only in the literal
  byte pair asserted (0xC2 0x80 vs 0xC2 0x9B). It is a true boundary value but tests no distinct
  branch.

**Recommendation: OPTIONAL — defer, low priority.** The existing suite already proves the
contract that governs U+0080 (C0-only escaping; everything ≥ U+0020 raw, demonstrated at both U+007F
and U+009B). Adding U+0080 is defensible as classic boundary-value-analysis (lower edge of the C1
block) and is cheap/low-risk — but it adds essentially zero branch coverage and only marginal
"documented boundary" value. If the team values exhaustive boundary documentation (this suite is
already very boundary-thorough, e.g. it guards both lowercase and uppercase `\u` forms), add it
mirroring `test_BC_2_11_005_c1_passthrough_raw_utf8`:
```rust
let c1_low = "\u{0080}"; // bottom of C1 block, encodes 0xC2 0x80
// assert raw [0xC2, 0x80] present; assert no "\\u0080" / "\\u0080" escape
```
Otherwise skip. This is purely additive and carries **no src risk**.

**Test-only:** Yes — pure test addition, no `src` change.

**External research:** Applied. RFC 8259 control-char escaping scope and serde_json's U+0080
boundary behavior were verified via Perplexity (one query), since the value judgment hinges on an
external spec/library-behavior fact rather than a codebase fact.

---

## Routing notes for the develop PR(s)

- All five are test-only / comment-only (F-DRIFT-C-001 edits a comment inside `src/analyzer/http.rs`'s
  `#[cfg(test)]` module — no production code path). Zero production-logic risk across the group.
- Four are confirmed cosmetic/quality fixes (W10-D10-sibling refactor, F-DRIFT-C-001 comment,
  F-S058-P11-001 comment, F-S058-P11-002 label). W20-NIT-001 is an optional ADD — recommend deferring
  unless exhaustive boundary documentation is desired.
- Per CLAUDE.md `DF-VALIDATION-001`: these are now validated and eligible to be filed as issues.
  Recommend filing the four confirmed items as one batched `test:`/`docs:` cleanup; W20-NIT-001 as a
  separate optional `test:` issue tagged low-priority or `wontfix-candidate`.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Read | 6 | Ground each claim in current file/line (reassembly fill loop + helper, http.rs truncate_uri + tests, tls header/body, json reporter del test) |
| Grep | 6 | Locate helpers, tests, comments, and JSON byte-handling assertions by symbol/string |
| Glob | 1 | Confirm reporter_json_tests.rs path |
| Perplexity search | 1 | Verify RFC 8259 escaping scope + serde_json U+0080 boundary behavior (W20-NIT-001) |
| Training data | 1 area | UTF-8 byte widths of 'é' (U+00E9 = 2 bytes) — standard, cross-checked against in-file fixture comments |

**Total MCP tool calls:** 1 (Perplexity search)
**Training data reliance:** low — every code claim is grounded in a Read of the current tree; the
only training-data fact (UTF-8 width of 'é') is corroborated by the file's own fixture comment and is
arithmetic, not version-sensitive. The single external spec/library question (U+0080 escaping) was
verified via Perplexity rather than recalled.
