# Adversarial Spec Review — feature-enip-v0.11.0 (SS-17), Pass 3

**VERDICT: FAIL — 3 CRITICAL, 4 HIGH, 4 MEDIUM. Novelty: HIGH.**

Dominant pattern: propagation-lag of correct Pass-1/Pass-2 fixes to secondary
docs (PRD §2.17 table, verification-*.md architecture anchors, BC-INDEX
changelog/comments, sibling CLI BC-2.17.020). Core content (endianness
algorithm, CIP service codes, EMITTED 17→20, frame-walk soundness, Kani
non-vacuity, canonical holdout, constants) CONFIRMED CORRECT.

## Findings (all REMEDIATED in the Pass-3 sweep)

| ID | Severity | Finding | Resolution |
|----|----------|---------|------------|
| F3-000 | CRITICAL | BC-2.17.020 stale default 20→50 (write-burst threshold) | FIXED: BC-2.17.020 updated to default=50 |
| F3-001 | CRITICAL | PRD §2.17 table header still read "BE" instead of "LE" | FIXED: PRD §2.17 title corrected to LE |
| F3-002 | CRITICAL | verification-coverage-matrix.md endianness column still read "BE" | FIXED: coverage matrix updated BE→LE |
| F3-003 | HIGH | PRD --all/--enip contradiction (--all described as excluding --enip) | FIXED: --all includes --enip per ADR-010 and BC-2.17.020 |
| F3-004 | HIGH | PRD §2.17 ForwardClose entry title stale | FIXED: PRD ForwardClose title corrected |
| F3-005 | HIGH | BC-INDEX changelog row cited 16→15 SS-17 variants (stale from pre-BC-2.17.025 era) | FIXED: BC-INDEX changelog updated to correct variant count |
| F3-006 | HIGH | BC-INDEX pdu_count comment described wrong field semantics | FIXED: BC-INDEX pdu_count comment corrected |
| F3-007 | MEDIUM | verification-architecture.md harness count cited "4 harnesses" without noting 5 sub-properties | FIXED: updated to "4 sub-properties/5 harnesses" |
| F3-008 | MEDIUM | BC-INDEX SS-17 header count 24→25 (already fixed cb01468, residue in a comment block) | FIXED: residue removed |
| F3-009 | MEDIUM | Service name-labels misaligned between BC table and VP descriptive entry | FIXED: BC↔VP service labels aligned |
| F3-010 | MEDIUM | BC-2.17.008 title scope note referenced 0x00B2 ambiguously | FIXED: BC-2.17.008 title scope clarified |

## Orchestrator Exhaustive-Grep Bonus Finds (Pass-3 sweep)

Three additional BE residues found by orchestrator grep that the adversary
missed — all fixed in the same sweep commit:

| Location | Line | Residue | Resolution |
|----------|------|---------|------------|
| VP-INDEX:119 | 119 | "big-endian" in VP-032 summary row | FIXED |
| verification-architecture.md:110 | 110 | "BE" in anchor comment | FIXED |
| cap-17-enip-cip-analysis.md:20 | 20 | "big-endian" in capability overview | FIXED |

## Status

**ALL 11 adversary findings + 3 orchestrator bonus finds REMEDIATED in the
Pass-3 exhaustive propagation sweep. Pass 4 running. Convergence counter: 0/3.**

Pass-3 sweep commit: (see factory-artifacts git log after this commit).
