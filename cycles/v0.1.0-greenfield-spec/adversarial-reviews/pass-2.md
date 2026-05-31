# Adversarial Review — STORY-086 (Implementation) — Pass 2

| Field | Value |
|-------|-------|
| Target | implementation (test formalization) |
| Scope | STORY-086 — `tests/cli_story_086_tests.rs` |
| Pass | 2 |
| Date | 2026-05-31 |
| Focus | Tautology / false-positive / mutation-resistance (vectors Pass 1 under-weighted) |
| Verdict | CLEAN (1 Low; no Critical/High/Medium) |

## Method

Re-attacked the test file specifically for assertions that would pass even if the
source were wrong: wildcard match arms hiding variant drift, `is_err()`-style
non-discriminating error checks, and skipped-field (`..`) bindings that leave a
contracted invariant unasserted. Ran both CLI test files together (14 + 15 = 29
tests, all green) to confirm no cross-file regression and no name collision.

## Mutation-Resistance Probes (all PASS)

- **Variant-discrimination.** AC-001 (line 87) and AC-005 (line 239) use *exhaustive*
  match arms — `Commands::Summary { .. } => panic!` and `Commands::Analyze { .. } => panic!`
  — not a `_` wildcard. A drift in the `Commands` enum (added/renamed variant) would
  fail compilation, not silently pass. Stronger than required.
- **Error-kind discrimination.** AC-003 asserts `MissingRequiredArgument`; AC-007,
  EC-003, EC-004 assert `UnknownArgument`. None use bare `is_err()`. A no-target
  parse cannot masquerade as an unknown-arg rejection and vice-versa — the tests
  would catch a clap-config mutation that swapped `required = true` for a different
  constraint.
- **Default-state discrimination.** AC-001 explicitly asserts `!dns && !http && !tls
  && !mitre && !all`; a mutation flipping any clap default to `true` (e.g.
  `default_value_t = true`) is caught.
- **Type-shape discrimination.** AC-006 / AC-009 use `let _: bool = …` which fails to
  compile if a field became `Option<bool>`. Mutation-resistant at the type level.
- **No-collision.** 29 tests across two files run clean; `mod story_086` wrapper holds.

## Findings

### F-P2-001 (LOW / Informational) — AC-002 `--http --tls` sub-block omits the `mitre = false` assertion

In `test_analyze_individual_protocol_flags`, the second sub-block
(`["wirerust","analyze","--http","--tls","cap.pcap"]`, lines 127-142) destructures
with `..`, skipping `mitre`. It asserts `dns/http/tls/all` but not `mitre = false`.
AC-002's contract clause is "absent flags remain false," and `mitre` is absent in
this argv. The specific "mitre stays false when http/tls are set" combination is
therefore not asserted *in this block*.

Mitigating coverage: AC-004 (`test_mitre_flag_does_not_imply_analyzers`) and EC-002
(`test_EC_002_mitre_alone`) both explicitly assert `mitre = false` when mitre is not
passed, and the first sub-block of AC-002 (`--dns` only) asserts `!mitre`. So the
invariant "absent mitre ⇒ false" is covered three other ways. This is a local
assertion-completeness gap, not a behavioral coverage hole. Severity LOW; a one-line
add (`assert!(!mitre, ...)`) would make the block self-complete. No acceptance impact.

## Novelty Assessment

Decaying. Pass 1 surfaced 3 Low (BC-surface coverage + a doc-comment citation).
Pass 2 surfaced 1 Low (a local assertion-completeness gap), found via a different
attack vector (mutation-resistance rather than BC-coverage). Finding count
2 < 3 monotonic-decreasing. No Critical/High/Medium in either pass. The
mutation-resistance probes all passed — the tests genuinely discriminate.

## Recommendation

CLEAN. F-P2-001 is non-blocking. Continue to Pass 3 (minimum 3 clean passes).
Invariants from Pass 1 re-confirmed; add to carry-forward: tests are
mutation-resistant on variant shape, error kind, default state, and field type.
