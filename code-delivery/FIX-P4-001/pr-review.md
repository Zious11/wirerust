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
