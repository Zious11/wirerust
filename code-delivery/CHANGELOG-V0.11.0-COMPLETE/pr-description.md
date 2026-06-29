## Summary

The v0.11.0 CHANGELOG entry published with the release PR (#337) only documented the EC-X1/EC-X2 carry-buffer direction-split and EC-X2 `saturating_sub` bug fixes. The headline feature — the new EtherNet/IP (ENIP) + CIP protocol analyzer — was entirely absent from the entry, leaving the `## [0.11.0]` section incomplete and misleading for anyone reading the changelog to understand what shipped in this release.

This PR completes the entry by adding the full ENIP/CIP feature documentation with verified PR numbers.

---

## What Changed

**File:** `CHANGELOG.md` only — +111 lines, no source or test files touched.

### Added to `## [0.11.0]` — `### Added` section (new)

- **EtherNet/IP (ENIP) + CIP protocol analyzer** — TCP/44818 flow analysis using the ODVA ENIP + CIP stack, enabled via `--enip` / `--all`. Documents: 24-byte ENIP header parse, CPF item-list walk, StreamDispatcher Rule 7 placement (ADR-010), per-flow `EnipFlowState` with 600-byte carry buffers, CLI flags (`--enip`, `--enip-write-burst-threshold`, `--enip-error-burst-threshold`), and the 7-key `enip_summary` JSON object. Feature references: Feature #316, STORY-130..139, PRs #317–#334, ADR-010.

- **MITRE ATT&CK for ICS detections (ics-attack-19.1):** T0846 Remote System Discovery, T0888 Remote System Information Discovery (two patterns), T0858 Change Operating Mode, T0816 Device Restart/Shutdown, T0836 Modify Parameter, T0814 Denial of Service. New `MitreTactic::IcsExecution` variant (TA0104). Catalog: 25→28 seeded, 17→20 emitted. STORY-133/134/135, PRs #320/#323/#324.

- **Formal verification + QA:** VP-032 Kani harnesses Sub-A through Sub-D (STORY-130/132), `fuzz_enip_cip_parse` cargo-fuzz harness discharging F-P9-002 (PR #332), full-pipeline E2E tests against real ENIP/CIP pcaps — HS-110 through HS-122 (PR #333).

### Added to `## [0.11.0]` — `### Changed` section (new)

- ENIP `enip_summary` wire format canonical key name (`"parse_errors"` not `"total_parse_errors"`), consistent field ordering, null-safety. [PR #331, BC-2.17.021 Invariant 1]

- Green-doc-tense CI gate (`green-doc-tense-gate` job, `bin/check-green-doc-tense`). [PR #321]

### Added to `## [0.11.0]` — `### Fixed` section — ENIP-specific fixes (new bullets prepended before existing EC-X1/EC-X2 bullets)

- ENIP source-IP attribution corrected (direction-based, not port-44818 heuristic). [PR #328, AC-139-002]
- `enip summarize()` now includes still-open flows at call time (RULING-W61-001). [PR #330, BC-2.17.021 Postcondition 1]

### Preserved unchanged

- All existing `### Fixed` bullets for Modbus EC-X1/EC-X2 (PR #336) and DNP3 desync-latch (PR #335/336) are unchanged.
- Footer compare links (`[0.11.0]: ...`, `[Unreleased]: ...`) are unchanged.
- No other version sections (`[0.10.0]`, `[0.9.3]`, etc.) are touched.

---

## Traceability

| Element | Reference |
|---------|-----------|
| Feature | Feature #316 (ENIP analyzer) |
| Stories | STORY-130 through STORY-139 |
| PRs documented | #317–#334 (ENIP feature), #335/#336 (EC-X1/X2/desync fixes) |
| CHANGELOG sections | Added, Changed, Fixed under `## [0.11.0]` |
| Footer links | Intact — not modified |

---

## CI Gate Notes

This repo has a `green-doc-tense` CI gate (`bin/check-green-doc-tense`) that rejects aspirational/future-tense wording ("will", "planned", "future") in doc contexts. The CHANGELOG entry was authored in present/past tense throughout (e.g., "analyzes", "parses", "added", "corrected"). If this gate fails, do not bypass — stop and route a fix.

---

## Pre-Merge Checklist

- [x] Diff is CHANGELOG.md only (+111 lines)
- [x] All version sections other than `[0.11.0]` untouched
- [x] Footer compare links intact
- [x] Existing Fixed bullets preserved
- [x] PR numbers verified against merged PRs in this repo
- [x] green-doc-tense gate: entry authored in present/past tense only
- [ ] CI checks green (pending)
- [ ] Squash-merge into develop
