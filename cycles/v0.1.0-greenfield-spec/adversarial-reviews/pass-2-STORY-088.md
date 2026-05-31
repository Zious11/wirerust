# Adversarial Review — STORY-088 (Implementation) — Pass 2

| Field | Value |
|-------|-------|
| Target | implementation (test formalization, brownfield-formalization mode) |
| Scope | STORY-088 — `tests/main_story_088_tests.rs` (19 tests) + BC-2.12.008..013 + STORY-088.md |
| Cycle | v0.1.0-greenfield-spec |
| Pass | 2 |
| Date | 2026-05-31 |
| Branch | feature/STORY-088-run-analyze-orchestration |
| Worktree HEAD | 698595e |
| Base develop | 45fe526 |
| Verdict | **MEDIUM** (1 NEW MEDIUM mutation gap; 0 Critical/High) |

## Checkout Guard

- Branch = `feature/STORY-088-run-analyze-orchestration` (not develop). OK.
- `#[test]` count = 19 (14 AC + 5 EC). OK.
- Diff scope = only `tests/main_story_088_tests.rs`. src/main.rs reverted clean
  after Pass-1 mutations (`git diff src/main.rs` empty). OK.
- Factory artifacts from main-repo absolute paths. OK.

## Confirmed Invariants Carried From Pass 1

These were live-verified mutation-resistant in Pass 1 and re-confirmed not
re-tested here (efficient carry-forward):
- AC-001 (--all OR-expansion), AC-002 (mitre exclusion), AC-003 (needs_reassembly),
  AC-004 (warning), AC-005 (http/tls skip gate), AC-010/EC-002 (case-sensitive
  ext), AC-011 (non-recursive), AC-012 (bail text), AC-007/EC-004 (NO_COLOR).
- Open from Pass 1 (still open): F-W25-S088-P1-001 (AC-013 vacuous),
  F-W25-S088-P1-002 (AC-014 vacuous), F-W25-S088-P1-003 (AC-006 header-only).

## Pass-2 Focus: assertions NOT deep-mutated in Pass 1

Targeted the sort invariant, the color-present path, and the pcapng-exclusion
mechanism with fresh live mutations.

| # | Mutation (src/main.rs) | AC/EC | Result | Resistant? |
|---|------------------------|-------|--------|-----------|
| A | Remove `files.sort()` (L356) | EC-005, AC-009 | **both PASSED** | **NO — F-W25-S088-P2-001** |
| B | Force `use_color = false` (L43) | AC-008 | FAILED | YES |
| C | Include `.pcapng` in resolve_targets (L351) | AC-009 | FAILED | YES (via reader-error) |

Also confirmed by inspection: `smb3.pcapng` passed to the reader yields
"Failed to parse pcap header / wrong magic number" and non-`.success()`, which
is the mechanism that makes AC-009's **exclusion** claim discriminating
(mutation C confirms it).

## Findings

### F-W25-S088-P2-001 [MEDIUM] — Sort invariant (BC-2.12.011 inv 2) is completely untested; EC-005/AC-009 use identical-copy fixtures

**EC-005** (`test_EC_005_directory_files_returned_sorted`) and **AC-009**
(`test_resolve_targets_directory_pcap_only_sorted`) both claim to verify the
sorted-output postcondition (BC-2.12.011 PC4 + invariant 2: "`files.sort()` is
called before returning"). Both copy the **same** fixture (`http-ooo.pcap`) to
`a.pcap` and `b.pcap` and assert only `stdout` contains "Packets: 32".

Because the two files are byte-identical, the total packet count (32) is
**invariant under ordering**. Live mutation A (delete `files.sort()`) left both
tests GREEN. The sort call is therefore untested — a reordering or removal of
`files.sort()` would not be caught.

EC-005's docstring overclaims: "Negative: if unsorted, order would depend on
filesystem (non-deterministic)." The chosen observable (order-invariant total
count) cannot detect order at all, so the negative case is not actually
exercised.

Severity rationale: MEDIUM. The live code is correct (`files.sort()` present at
L356), and ordering only affects which file's progress bar/packets render first
— cosmetically immaterial when both files are identical. But the BC names the
sort as an explicit invariant and two ACs claim to cover it, so the
traceability is overstated and a real regression (dropping the sort) would
escape.

**Recommendation:** Make the sort observable. Use two **distinguishable**
fixtures whose processing order changes a deterministic, order-sensitive output —
e.g. `a.pcap` = a fixture with N packets, `b.pcap` = a fixture with M≠N packets,
then assert on the first-processed file's per-file marker, OR add a unit-style
assertion on `resolve_targets` ordering if the function is exposed. At minimum,
pick fixtures where reverse order would produce a different, assertable stdout.
If keeping the count-only proxy, drop the "sorted" claim from EC-005/AC-009
prose and BC-traceability to avoid overclaim.

## Trajectory

- Pass 1: 3 MEDIUM. Pass 2: 1 NEW MEDIUM (plus 3 carried-open). Monotonic — no
  new findings exceed the prior pass count; no regression. The Pass-2 finding is
  a genuinely new gap (sort invariant) not surfaced in Pass 1, consistent with
  the "fresh-context finds new things" expectation.

## Verdict

**MEDIUM** — 0 Critical, 0 High, 1 new Medium (F-W25-S088-P2-001) plus 3 carried
from Pass 1. NOT clean. The four open MEDIUMs cluster into two themes:
(1) cosmetic/LOW-confidence-BC assertions written as no-ANSI-stdout checks that
don't discriminate the behavior they name (progress bar F-001/F-002; sort
F-P2-001 uses an order-invariant proxy), and (2) section-header-vs-analysis
proxy for AC-006 (F-003). All are test-strength/traceability gaps, not code
defects. Continue to Pass 3.
