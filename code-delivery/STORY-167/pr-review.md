# PR #401 — STORY-167 IEC-104 APCI Core Parser — Fresh-Eyes Review

## VERDICT: APPROVE

No blocking or major findings. The implementation is bounds-safe, pure, well-documented,
correctly scoped, and green against all reproduced CI gates.

---

## Verification performed (reproduced in worktree)

| Gate | Result |
|------|--------|
| `cargo fmt --check` | PASS (exit 0) |
| `cargo clippy --all-targets` (RUSTFLAGS=-Dwarnings, forced fresh) | PASS (no warnings/errors) |
| `cargo test --test iec104_analyzer_tests` | 30 passed; 0 failed; 0 ignored |
| Demo-evidence absolute-path scrub | Clean (no `/Users/` or `/home/` leaks) |

## Checklist (8/8)

1. **Diff coherence** — All changes (iec104.rs, mod.rs, CHANGELOG, tests, demo evidence)
   relate to STORY-167. No unrelated churn.
2. **Description accuracy** — PR body matches the diff. Claimed 30-test count (5+3+3+2+7+10)
   matches actual `#[test]` functions and CI output.
3. **Test coverage** — All six BCs (BC-2.19.001–006) covered with canonical vectors, boundary
   values (LEN 3/4/253/254/255), and cross-function consistency invariants (both directions).
   No-panic invariant explicitly tested.
4. **Demo evidence** — 7 AC `.md` files + `evidence-report.md` present. Pure-core library
   function with no CLI/web/visual surface, so VHS/GIF recordings are inapplicable; markdown
   test-output capture is the appropriate evidence form. Accepted (see MINOR).
5. **Commit quality** — Conventional format, STORY-167 tagged, clear messages.
6. **Diff size** — 1650 additions, but source is only 211 lines; bulk is tests (656) and
   evidence markdown. Reasonable (see MINOR).
7. **Missing changes** — Scope correctly excludes `on_data` shell (STORY-172) and `classify()`
   dispatch wiring (STORY-173); PR is explicit. Nothing missing for this story's scope.
8. **Dependency status** — `depends_on: []`, first story in E-22. No upstream PR gate.

## Correctness notes (source review)

`parse_apci_header` — every slice index (`data[0..=5]`) is dominated by the `data.len() < 6`
early return, so it is panic-free on any input. Guards are ordered length → start → LEN-low →
LEN-high, each returning `None`. The `[4,253]` bound guarantees `len + 2 ≤ 255` (no u8 overflow),
matching the documented arithmetic-safety invariant. CF1–CF4 doc comments (I/S/U discrimination,
STARTDT-act `0x07` decode) are accurate to IEC 60870-5-104. `is_valid_iec104_frame` short-circuits
correctly with `len >= 2` before indexing. Kani harness is correctly gated behind `#[cfg(kani)]`
and does not affect the normal build.

---

## Findings

### MINOR (accept with disposition note)

- **[MINOR / demo-evidence]** Demo evidence is markdown test-capture rather than `.gif`/`.webm`.
  Disposition: accepted — pure-core parser with no visual/interactive surface, so screen
  recordings are inapplicable. evidence-report.md documents all 7 ACs with reproducible commands
  and captured output. No action required.

- **[MINOR / size]** Diff is 1650 additions (exceeds the 500-line flag), but production source is
  only 211 lines; remainder is tests and evidence markdown. Disposition: accepted — no meaningful
  review-burden concern.

### NIT (optional)

- **[NIT / iec104.rs:132]** `parse_apci_header` sets `start: 0x68` as a literal rather than
  `data[0]`. Provably equivalent (guard at line 117 rejects anything but `0x68`), but a reader
  expecting "all fields extracted verbatim" may pause. Consider `start: data[0]` for uniformity.
  Not a defect.

- **[NIT / iec104.rs:81-84]** `Iec104ParseError::Incomplete` is defined but never constructed —
  `parse_apci_header` returns `Option`, not `Result`. Documented as a STORY-168 skeleton and `pub`
  (no `dead_code` warning; clippy clean). Acceptable forward seam.

- **[NIT / iec104.rs:200-201]** The Kani harness calls `parse_apci_header(&data)` twice (once as
  `let _ =`, then again in the `if let Some`). Harmless and `#[cfg(kani)]`-only, but the first call
  is redundant. Could collapse to a single bind.

None of these block merge.

## Recommendation: APPROVE
