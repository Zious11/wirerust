---
document_type: triage-record
producer: orchestrator burst triage-2026-07-08
date: 2026-07-08
validated_at: develop c4eb1f4
policy: DF-VALIDATION-001
research_verdicts: "10/10 CONFIRMED"
---

# Issue Backlog Triage — 2026-07-08

Triage run `triage-2026-07-08`. All 10 open GitHub issues validated in two passes:
(1) codebase validation against develop `c4eb1f4` by three codebase-analyzer agents;
(2) per-item external research validation per DF-VALIDATION-001 by ten research-agent runs.
All 10 verdicts CONFIRMED. Human approved dispositions: closures + deferrals/issue-updates.
Story drafting and the XS docs PR were NOT approved — recorded as validated backlog only.

---

## EXECUTED

### #101 — CLOSED (not planned)

Superseded by `TD-MAINT-THRESHOLD-CALIB-001` acceptance (PR #382 README Known Limitations).

Research correction: the original framing of #101 as a corpus-availability blocker was wrong.
The correct framing is a priority decision — public labelled ICS corpora exist (SWaT/WADI, HAI,
QUT S7comm, Lemay, 4SICS, dalton). The issue was closed as not planned because the decision to
defer calibration is a product-priority choice, not a dataset-discovery gap.

---

### #4 — CLOSED (completed)

CSV half of the original reporter story has shipped: `src/reporter/csv.rs`,
BC-2.11.020..024, E-8. SQLite half re-filed as #385 (see below).

---

### #385 — CREATED

Title: `feat(reporter): add SQLite reporter`

Research-validated scope:

- Dependency: `rusqlite = { version = "0.40", features = ["bundled"] }` (MSRV 1.77.2;
  no active RUSTSEC advisories at time of triage).
- Schema: flow-centric 8-table design with `flows` as the spine; `findings.flow_id` FKs
  tie back to flows (ADR-0006 attribution tie-in). `PRAGMA foreign_keys=ON`.
- CLI flags: `--sqlite <path>` (fail on existing) + `--sqlite-overwrite`.
- Explicit non-goals: files, tls_sessions, certificates, dns_queries, http_transactions
  (these are explicitly out of scope for this story).
- Size: M (5–8 pts). Capability: CAP-11. New subsystem-spec: SS-11 BCs to be authored.

---

### #67 — COMMENTED (kept open)

Deferral re-validated:

- `linkme 0.3.36`/`inventory` mechanization cost unchanged — deferral stands.
- MITRE ATT&CK v18+v19/v19.1: 0% ID churn on all 26 emitted IDs.
- Test renamed to `test_all_emitted_ids_resolve` (`tests/mitre_tests.rs:456-513`).
- Trigger phrasing tightened to reference real-emitted IDs excluding test placeholders
  (threshold: >20 real-emitted IDs).

Issue updated with re-validation summary and corrected trigger threshold.

---

### #6 — COMMENTED (kept open)

Premise corrected: `rayon` is NOT a runtime dependency. It is a transitive dev-dep
via `criterion`. `rayon 1.11.0`, MSRV 1.80, no advisories.

Pickup prerequisites recorded in the issue:
- `rayon` must become a runtime dep (not just dev-dep) before this story activates.
- `Summary::merge` implementation required.
- Deterministic per-file ordering acceptance criterion.
- ADR-0004 post-fold reads + `CachePadded` false-sharing audit.
- `indicatif` draw-throttling.

Size: M. Deferred to a perf-driven wave.

---

## VALIDATED BACKLOG

Items below are research-validated but NOT yet stories — human deferred story drafting
and the XS docs PR. Recorded here for future wave pickup.

---

### #255 — snake_case JSON enums (story candidate S)

Research refinements:
- `rename_all = "lowercase"` for `Verdict`/`Confidence` (single-word enums).
- `rename_all = "snake_case"` for `ThreatCategory` (multi-word enum) per
  Suricata EVE/ECS/OCSF conventions.
- Add `schema_version` envelope field in the same PR (future-proofs next breaking change).
- Hard cutover 0.11→0.12 idiomatic approach.
- JSON schema is a governed surface OUTSIDE `cargo-semver-checks` scope — must be
  documented in release notes.

---

### #252 — VP-024 multi-file proof_file_hash (story candidate S, ~3 pts, E-11)

Unblocked. Research modifications:
- Mini-Merkle: `sha256(sha256(fileA) || sha256(fileB))` rather than raw byte concat.
  This detects cross-file function moves that raw concat would miss.
- Keep the MD5-first-7 (`input-hash`) and SHA-256 (`proof anchors`) disciplines
  DISTINCT and document the distinction in `CLAUDE.md`.
- Pin `kani_version` as a sibling field at re-lock; latest ~0.65.0 — verify at re-lock.
- Codify exact `kani_proofs` section-scoping (mod block boundaries). This is the
  highest-risk area if left vague.

---

### #63 — TerminalReporter snapshot tests (deferral justified)

`insta 1.48.0` has `strip_ansi_escape_codes`; no advisories.
Snapshot with `use_color: false` (`owo_colors` gating at `terminal.rs:160`).

CORRECTION: the earlier suggestion to "chain after #255" is REFUTED. `Display` vs
`Serialize` paths are fully independent; there is no ordering constraint between #63
and #255.

---

### #361 — per_port_counts ceiling docs (XS docs fix pending)

CORRECTION: the ceiling bound is approximately 2.1 MiB per map / approximately
4.25 MiB combined (hashbrown 7/8 load factor forces 131,072 buckets at 65,536
entries), NOT ~1 MB as originally noted. Ceiling of 65,536 entries per transport
confirmed.

Recommended doc-comment wording captured in the research record. Pair with #67's
stale doc-ref micro-fix (`src/reporter/terminal.rs:348`) in one maintenance docs PR.

---

### #103 — size-symmetry discriminator (deferral reframed)

Deferral is now PRIORITY-GATED on corpus construction (synthesized `fragroute` pcaps
+ testbed keystroke captures + SWaT/HAI/QUT as cross-domain FP baseline), NOT blocked
on dataset discovery.

Resize M→L or split into two stories (corpus-build + implementation).

NEW design constraint: OpenSSH 9.5+ keystroke-timing chaff deliberately increases size
symmetry — this must be named in the story and ideally in the `config.rs:67-69`
doc comment.

Note: the discriminator is genuinely novel. Snort's mechanism (which wirerust ports) is
purely port-based; no production NIDS ships symmetry-based suppression. This novelty
reinforces the need for rigorous corpus validation before implementation.

---

### #3 — C2 beaconing (deferral confirmed; epic + ADR route, ~15–25 pts)

CRITICAL corrections for the future ADR:

(a) RITA-parity metrics (Bowley skewness + MAD jitter) are NOT computable single-pass
O(1)/flow. The ADR must commit to end-of-capture aggregation with memcap'd IAT buffers
(recommended; fits the finalize-report pipeline) or justify quantile sketches explicitly.

(b) MITRE mapping:
- Primary: T1071 + subtechniques (.001/.004) + T1095 (raw-transport beacons — this was
  missing from the original issue).
- T1573 is conditional-only (NOT auto-co-emitted for plain HTTPS).
- T1008 is conditional on observed fallback behavior.

(c) FP gates to seed in the ADR:
- Tranco/Umbrella allowlist (Alexa is deprecated).
- Minimum connection count: 5–20.
- 80–90% size-uniformity gate.
- Internal-to-internal traffic excluded from beaconing detection.

`BC-ABS-002` retires when this epic lands.

---

## NOTES

- `research-67` flagged a possible v19 Defense-Evasion tactic-split staleness for
  T1027/T1036 tactic mappings. NOT filed as a new issue because the MITRE v19 remap
  already shipped in v0.5.0 — treat as pre-resolved unless contrary evidence appears.
