# PR Review — #410 `fix(FIX-P4-001): populate direction on all IEC-104 emitted findings`

**Verdict: APPROVE**
Branch: `fix/FIX-P4-001` (HEAD `7edfc5f`) → base `develop` (`547deba`)
Finding fixed: IEC104-FINDING-DIRECTION-001

## Independent verification (fresh-eyes, PR branch checkout)

- `cargo test --test iec104_analyzer_tests` → **211 passed, 0 failed** (200 pre-existing + 11 new)
- `cargo clippy --all-targets -- -D warnings` → clean
- `cargo fmt --check` → clean
- `grep "direction: None"` on new `src/analyzer/iec104.rs` → **zero occurrences**; all 10 emit sites now `Some(direction)`

## Checklist results

**1. Core finding addressed — PASS.** All 10 `Finding` emit sites now carry `direction: Some(direction)` (lines 392, 428, 763, 804, 819, 849, 899, 1047, 1194, 1263). Zero `direction: None` remain. Sites: `process_u_frame` STOPDT-act (T0881) + non-canonical arm (T0814); `detect_iec104_threats` five pushes (T1692.001 x2, T0836, T0827, T0814); `track_ns_desync` (T1692.001); `on_data` carry-overflow + malformed-LEN (T0814 x2).

**2. Signature changes safe — PASS.** `process_u_frame` and `detect_iec104_threats` each gain `direction: Direction`. Only two production callers exist, both in `on_data` (lines 1294, 1303), both updated; `direction` is the `on_data` parameter, in scope at every site. All test callers updated. Whole suite compiles and passes.

**3. Dropped redundant `format!("direction=…")` evidence — CORRECT.** Removed from `track_ns_desync` and `on_data` carry-overflow. Info now in the structured `direction` field. No test asserts on an evidence string containing `direction=`; full suite passes, so the `evidence`-vec change breaks nothing.

**4. Test coverage — PASS (11 tests → 10 sites).** Complete mapping: track_ns_desync (C2S+S2C), process_u_frame STOPDT (C2S+S2C), process_u_frame non-canonical, detect_threats type45/48(both findings)/105/0, on_data carry-overflow, on_data malformed-LEN. `0x13`→STOPDT-act (T0881), `0x03`→default arm (T0814) confirmed against source.

**5. CHANGELOG — ACCURATE & COMPLETE.** Correctly framed additive/backward-compatible (`#[serde(skip_serializing_if = "Option::is_none")]` verified in `src/findings.rs:163`), enumerates all 10 sites, documents both signature changes.

**6. Holdout sweep — CREDIBLE.** `docs/holdout-expectations-sweep-FIX-P4-001.md` marks `COMPLETE`, documents zero IEC-104 holdout scenarios and no exact-JSON IEC-104 assertions; reasoning consistent with the additive change verified here.

**7. Style/quality — CLEAN.** Idiomatic Rust, clippy-clean, well-documented tests with trace IDs. Diff small (~47 source lines), well under the 500-line flag.

## Non-blocking notes

- **[NIT]** Evidence docs (`evidence-report.md`, `AC-P4-001-test-results.txt`) refer to `detect_threats()`; actual name is `detect_iec104_threats`. Doc-only cosmetic mismatch.
- **[NIT]** Six of ten sites are tested single-direction only. Since `direction` is a pure pass-through with no branching, and both directions are proven at `track_ns_desync` and `process_u_frame` STOPDT, coverage is adequate — not a real gap.

No BLOCKING or MINOR findings. Recommend merge.

---

# PR Review — #419 `docs: correct TypeID 45 C_SC_NA_1 direction label in FIX-P4-001 demo evidence`

**Verdict: APPROVE** — no blocking findings.
Branch: `docs/iec104-typeid45-direction-fix` → base `develop`. Follow-up prose
correction to the demo evidence delivered in #410 (this file's review above).

## Scope
2-line prose diff (+2 / -2) across exactly two files, both under
`docs/demo-evidence/FIX-P4-001/`:
- `docs/demo-evidence/FIX-P4-001/evidence-report.md` (line 46)
- `docs/demo-evidence/FIX-P4-001/AC-P4-001-test-results.txt` (line 61)

No `src/`, `Cargo.toml`, or `bin/` files touched. Semantic PR title (`docs:`)
valid. No behavior change, no test change, no new dependencies.

## Checklist results

**1. Correctness against code ground truth — CONFIRMED.**
`src/analyzer/iec104.rs:743-748` — the `match type_id` arm `45..=47 =>` is
commented "TypeIDs 45–47 (C_SC_NA_1, C_DC_NA_1, C_RC_NA_1): switching commands"
and emits `IEC-104 control command` findings (T1692.001). TypeID 45 is handled
purely as a control command and never appears in a monitoring-direction branch.
The complementary arm at `iec104.rs:913` documents "TypeIDs 1–44 (monitoring
direction)…", independently confirming 45 is outside the monitoring range.
IEC 60870-5 classifies C_SC_NA_1 (single command) as a control-direction ASDU.
The prior label "Monitoring direction" was factually wrong; the new
"Control direction (C_SC_NA_1)" is correct. Cited anchor (744-748) is accurate.

**2. Completeness sweep — no missed instances.**
Grepped the entire `docs/demo-evidence/FIX-P4-001/` set (`evidence-report.md`,
`AC-P4-001-test-results.txt`, `demo-json-serialization.rs`) for
"monitoring direction" / TypeID 45 / C_SC_NA. Only two "monitoring direction"
occurrences existed, both referencing TypeID 45 — both fixed by this diff. The
remaining TypeID 45 mention (`evidence-report.md:156`, "Types 0, 45, 48, 105 …
in both directions") is a correct aggregate statement, not a direction label,
correctly untouched. `demo-json-serialization.rs` carries no TypeID 45 direction
label. No mislabel escaped the fix.

**3. PR description accuracy — CONFIRMED.**
Cited line numbers match the diff hunks exactly. "No CHANGELOG entry required"
is correct — AC-158-001 / PG-W71-CHANGELOG excludes `docs/`. "Sibling sweep:
STORY-170 … TypeIDs 1–44 (monitoring direction)" is substantiated by
`iec104.rs:913`. "No test-evidence table" is honest and
PG-W74-PRDESC-ROW-VERIFY-compliant — no fabricated rows; CI green is the sole
gate, appropriate for a prose-only change.

**4. Demo evidence — N/A.** This PR corrects existing demo-evidence prose; no
behavior to record.

**5-8. Commit quality / diff size / missing changes / dependencies — PASS.**
4-line diff, far under the 500-line flag; correction complete across the file
set; no upstream dependencies.

## Findings
None at any severity (BLOCKING / WARNING / NIT). The correction is accurate,
complete, and the PR description matches the diff. Verified: the prose
correction reflects code ground truth at `iec104.rs:743-748`, no other instance
of the same mislabel remains in the FIX-P4-001 file set, and every factual claim
in the PR description checks out. Recommend merge.
