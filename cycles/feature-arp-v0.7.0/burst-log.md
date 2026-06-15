# Burst Log — feature-arp-v0.7.0

Narrative record of each burst in the ARP security analyzer feature cycle.

---

## Burst: F4 Wave-Level Adversarial Convergence Pass 1 + D-074 Remediation (2026-06-15)

### Context

ARP feature (E-16) CODE-COMPLETE on develop (d038711 — STORY-115 PR #241 merged).
F4 wave-level adversarial convergence started: 3 fresh-context passes over the full
ARP delta (vs baseline 31d1231 pre-ARP).

### Agents Dispatched

1. **adversary (fresh-context)** — F4 wave-level Pass 1 over full ARP delta.
2. **consistency-validator** — full-corpus consistency audit post-F4-delivery.
3. **research-agent** — validate D-074 threshold-0 rejection convention (HIGH confidence required).
4. **spec-writer (PO)** — BC bumps per D-074 adjudication.
5. **story-writer** — STORY-114 v1.1→v1.2 + STORY-115 v1.1→v1.2 per D-074 back-propagation.
6. **implementer** — PR #242 D-074 fix (src/main.rs guards + new test file).
7. **pr-reviewer** — code review + security review of PR #242.
8. **state-manager (this burst)** — input-hash recompute + STATE.md update + commit.

### Adversarial Pass 1 Results

- **Adversary:** 1 genuine MEDIUM finding **F-ARP-F4P1-001** + LOW observations.
  - F-ARP-F4P1-001 (MEDIUM): `--arp-storm-rate 0` and `--arp-spoof-threshold 0` not
    rejected at CLI. ARP comparisons are inclusive (`>=`) so 0 is a degenerate
    always-fire condition — should be fail-fast rejected (mirrors Modbus precedent).
  - NOT CLEAN. Convergence counter reset to 0/3.
- **Consistency-validator:** CONSISTENT — zero gaps.

### Decision D-074 (2026-06-15)

Reject `--arp-storm-rate 0` and `--arp-spoof-threshold 0` at CLI with fail-fast
`anyhow::bail!`. Rationale: ARP comparisons are inclusive (`>=`) → 0 is degenerate
always-true; mirrors modbus reject-0; reconciles with dnp3 (strict `>`, accepts 0)
under the invariant "accept 0 only where comparison is strict; reject where inclusive."
Research-agent validated HIGH confidence
(`.factory/research/arp-threshold-zero-convention.md`).

### Spec Files Touched

| File | Before | After | Change |
|------|--------|-------|--------|
| BC-2.16.008.md | v1.7 | v1.8 | EC-006 added (reject --arp-storm-rate 0) |
| BC-2.16.012.md | v1.2 | v1.3 | PC2 + EC-004 added (reject --arp-spoof-threshold 0) |
| BC-2.16.013.md | v1.2 | v1.3 | PC2 + EC-004 added (reject --arp-storm-rate 0) |

### Story Files Touched

| File | Before | After | Change |
|------|--------|-------|--------|
| STORY-114.md | v1.1 | v1.2 | AC-006 + EC-014 (D-074 back-propagation) |
| STORY-115.md | v1.1 | v1.2 | EC-011 update + AC-011 (D-074 back-propagation) |

### Code Delivered

- **PR #242** merged to develop (merge commit **fee71ee**; impl commit 3c1cecb).
- `src/main.rs` +10 lines: two `anyhow::bail!` guards (arp_spoof_threshold == 0;
  arp_storm_rate == 0).
- `tests/bc_2_16_d074_arp_threshold_zero_tests.rs` — 4 new tests RED→GREEN:
  - `test_cli_arp_spoof_threshold_0_rejected`
  - `test_cli_arp_storm_rate_0_rejected`
  - `test_cli_arp_spoof_threshold_1_accepted`
  - `test_cli_arp_storm_rate_1_accepted`
- Code review: APPROVE 0 findings. Security review: PASS. 9 CI checks green.
- Branch + worktree cleaned post-merge.

### Maintenance PR

- **PR #237** (dependabot chrono 0.4.44→0.4.45) merged to develop (d50b652).

### Input-Hash Recompute

STORY-114 and STORY-115 BC inputs changed (BC-2.16.012 v1.2→v1.3,
BC-2.16.013 v1.2→v1.3, BC-2.16.008 v1.7→v1.8). Recomputed:

| Story | Old Hash | New Hash | Status |
|-------|----------|----------|--------|
| STORY-114 | 5705a10 | 1325d69 | MATCH after recompute |
| STORY-115 | 2e0eca2 | bb1d83a | MATCH after recompute |

Full scan post-recompute: STORY-111=d05149f MATCH, STORY-112=8a4d566 MATCH,
STORY-113=7c61bae MATCH, STORY-114=1325d69 MATCH, STORY-115=bb1d83a MATCH.
ALL 5 ARP STORIES MATCH.

### Develop HEAD

fee71ee (PR #242 merged, post chrono-bump d50b652 also present).
`fee71ee == origin/develop` (verified 2026-06-15).

### Convergence Status

arp_f4_wave_adversary_convergence_counter = **0/3** (Pass 1 NOT clean — F-ARP-F4P1-001
MEDIUM remediated via PR #242 + D-074; consistency CONSISTENT; next = fresh Pass 1
restart).

---
