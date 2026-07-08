# PR #382 Fresh-Eyes Review — Known Limitations docs

**Verdict:** APPROVE (with 2 ADVISORY nits, 0 BLOCKING)

**Scope:** `README.md` +27/-0. Docs-only maintenance PR documenting three
uncalibrated detection-threshold defaults. No Rust source, tests, or CI touched.

## Checklist walkthrough

| # | Item | Result |
|---|------|--------|
| 1 | Prose accuracy vs PR description | Consistent — 3 thresholds, dated 2026-07-08, framed as engineering defaults |
| 2 | CLI flag names plausible | Yes — `--overlap-threshold`, `--small-segment-threshold`, `--small-segment-max-bytes`, `--out-of-window-threshold`, `--arp`, `--arp-storm-rate`, `--dnp3`, `--dnp3-direct-operate-threshold` all follow the project's kebab-case convention and match the semantic scope described |
| 3 | Leaked internal IDs / paths | Clean — no `TD-MAINT-THRESHOLD-CALIB-001`, no `.factory/`, no ADR numbers, no internal review IDs surfaced in the public README |
| 4 | Markdown validity | Valid — `##`/`###` heading nesting, bold labels, backticked flags, no broken formatting |
| 5 | Section placement | Sensible — sits between the testing guidance section and the Roadmap; a reader learning about the tool will hit it before considering deployment |
| 6 | Grammar/clarity | See Finding 1 |
| 7 | Calibration risk framing | Honest and proportionate — labels the values as "engineering estimates" / "not derived from any external standard", gives concrete lower-bound guidance (5–20, 3–5), does not overstate risk (no "unsafe" / "broken" language) and does not understate it (explicitly calls out FP/FN risk on "unusual networks") |
| 8 | Blocking issues | None |

## Findings

### Finding 1 — ADVISORY (clarity)

**File:** `README.md`
**Section:** "Reassembly anomaly thresholds"

**Issue:** The sentence *"No NIDS ships enabled, directly-comparable count-based
defaults for these detectors; these values are conservative engineering
estimates."* is hard to parse on first read. The comma after "enabled" makes
it ambiguous whether "enabled" is a verb (past participle modifying "NIDS") or
an adjective attached to "defaults". An operator scanning quickly may misread
this as "No NIDS is currently enabled" rather than "no other NIDS ships
enabled-by-default, directly comparable defaults".

**Suggestion:**

> No comparable NIDS enables count-based defaults for these detectors out of
> the box, so these values are conservative engineering estimates.

or:

> No widely-deployed NIDS ships enabled-by-default, directly-comparable
> count-based defaults for these detectors; the values here are conservative
> engineering estimates.

Non-blocking — meaning is recoverable on a second read.

### Finding 2 — ADVISORY (readability)

**File:** `README.md`
**Section:** "Reassembly anomaly thresholds"

**Issue:** Three distinct parameters (overlap, small-segment run +
max-bytes, out-of-window) are packed into one 4-line sentence with nested
parenthetical flag references. Compared to the tidy structure of the ARP
and DNP3 paragraphs (single knob each), this one is disproportionately
dense and lacks the "OT guidance / lower to X–Y" callout the other two
provide.

**Suggestion:** Optional — split into three sub-bullets or short
sentences, and if there is any operator guidance for reasonable OT tuning
(even "no field data yet, leave at default until observed FP"), match the
pattern of the ARP/DNP3 blocks. Not required for merge.

## What was verified

- Diff is exactly +27/-0 in a single file (`README.md`), no collateral
  changes — matches the "docs-only maintenance PR" description.
- No CI, workflow, `Cargo.toml`, source, or test file touched — no risk of
  behavior change, no need for test coverage.
- No secrets, no internal URLs, no email/PII, no `.factory/` path leakage.
- Section is inserted immediately before `## Roadmap`, preserving the
  existing layout.
- All 8 flag names referenced use consistent `--kebab-case` form and each
  is presented with its default value, which is the pattern an operator
  needs to act on the guidance.
- Framing is defensible: "engineering defaults / not calibrated / may
  produce FP or FN / here is a lower bound to try" is the correct posture
  for uncalibrated detection thresholds shipped to production users — it
  neither downplays nor sensationalizes the risk.

## Recommendation

Approve as-is or after a light copy-edit on the "Reassembly anomaly
thresholds" sentence. Neither finding blocks merge.
