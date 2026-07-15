# Fresh-Eyes PR Review — PR #406 (STORY-172)

**IEC-104 carry buffers + frame-walk loop + flow lifecycle (wave-81)**

## Overall Verdict: APPROVE

No blocking findings. All 13 CI checks pass (Test, Clippy, Format, Fuzz build,
CHANGELOG gate, Action pin gate, Semantic PR, Audit, Deny, and gates). The
production code (295 lines) is correct, well-documented, and matches the PR
description, ADR-013, and the CHANGELOG. Tests are substantive and assert real
effects, not just no-panic.

---

## Verification Against the Review Focus Areas

### 1. Frame-walk loop correctness — VERIFIED CORRECT
All four advance cases are handled, and every loop iteration provably advances
`pos` or breaks, so termination is guaranteed:
- **bad start byte** (`!= 0x68`) → `pos += 1`, no finding, no carry clear
- **malformed LEN** (`0x68` + LEN outside `[4,253]`) → `pos += 2` + EMIT-WITH-DEDUP T0814
- **valid frame** → `pos += frame_len` (`len+2`). Crucially, `pos += frame_len` sits
  *outside* the `if let Some(header) = parse_apci_header(frame)` block, so even a
  `None` parse result still advances the cursor — no infinite loop.
- **insufficient** (`buf.len() - pos < frame_len`, or a lone `0x68` with `<2` bytes)
  → stash remaining bytes to the directional carry and break.

Memory safety: `frame[6..]` is only reached when `frame_len >= 6` (guaranteed since
`len >= 4`), yielding a valid (possibly empty) ASDU slice. `frame = &buf[pos..pos+frame_len]`
is guarded by the availability check.

### 2. Carry-overflow guard — VERIFIED WALK-FIRST
The final code checks `carry.len() > MAX_IEC104_CARRY_BYTES` on the **directional
carry alone**, before `drain()`, and the delivery is always appended and walked
regardless. It uses `>` (not `>=`), and the EC-001 test pins the 254-byte residual →
no T0814 boundary. This correctly reflects the F-172-001 remediation and matches the
ADR-013 Decision 2 / CHANGELOG prose (no aggregate `carry+delivery` pre-check; the
delivery is never discarded before frame extraction — anti-evasion invariant).

### 3. Per-direction dedup — VERIFIED INDEPENDENT
All four flags exist and are used correctly: overflow uses
`carry_overflow_reported_c2s/s2c`; malformed-LEN uses `malformed_len_reported_c2s/s2c`.
`test_BC_2_19_026_malformed_len_first_s2c_after_c2s` explicitly asserts the S2C
malformed flag fires independently while a pre-set C2S flag stays untouched.

### 4. on_flow_close — VERIFIED NO-PANIC
`self.flows.remove(&flow_key)` returns `Option` and is a no-op on unknown keys. The
fuzz harness double-closes the same key to exercise this path.

### 5. Test quality — STRONG
Dispatch-effect tests (e.g. `_stopdt_act_after_startdt_emits_t0881`) assert finding
count, MITRE technique string, category, and verdict — driven end-to-end through
`on_data`, not the pure-core functions directly. This is exactly the coverage
F-172-002 was meant to add.

### 6. No regression risk
The production change is purely additive (two struct fields already existed as stubs;
`on_data`/`on_flow_close` bodies replace `todo!()`). CI Test job (2584 tests) passes;
story_167..171 modules are untouched.

---

## Findings (all non-blocking)

### F1 — LOW / informational (not a defect introduced by this PR)
| Field | Value |
|-------|-------|
| Severity | suggestion (LOW) |
| Category | quality |
| Location | `src/analyzer/iec104.rs` — `on_data` Finding construction (and module-wide) |
| Finding | All emitted findings set `timestamp: None`, `direction: None`, `source_ip: None`, and `on_data` explicitly discards its timestamp (`let _ = ts;`) even though `ts` and `direction` are in scope. This drops forensic context useful for triage. |
| Not-a-regression | Established module-wide convention — every Finding across STORY-167..171 (lines 361, 397, 719, 760, 775, 805, 855, 1006) uses the same `None` pattern. |
| Suggestion | Wire `direction`/`timestamp` through the whole IEC-104 module in a dedicated follow-up story rather than piecemeal here. |

### F2 — NIT / informational
| Field | Value |
|-------|-------|
| Severity | nit |
| Category | coverage |
| Location | `src/analyzer/iec104.rs` — carry-overflow guard |
| Finding | The guard (`carry.len() > 255`) is unreachable from conformant frame-walk traffic: the walk only ever stashes a residual `<= 254` bytes and drains fully each call, so `carry.len()` never exceeds 254 in normal operation. |
| Disposition | Intentional defense-in-depth (SEC-001-S168), correctly documented in the doc-comment and ADR, and covered by `vector_iii` via direct state injection. No action needed. |

---

## Checklist Notes
- **Diff Coherence:** All changes relate to STORY-172. No unrelated changes.
- **Description Accuracy:** PR body, ADR-013 Decision 2, and CHANGELOG all accurately
  describe the final walk-first implementation.
- **Test Coverage:** Changed lines have direct, effect-asserting coverage (26 new tests).
- **Demo Evidence:** 9 artifacts under `docs/demo-evidence/STORY-172/` (evidence-report.md
  + 8 per-AC `.md` files). These are `.md` test-harness evidence, not `.gif`/`.webm`. For a
  pure in-memory state machine with no CLI/UI/visual surface, test-harness evidence is the
  appropriate form and is consistent with prior IEC-104 stories — NOT flagged as blocking
  (the `.txt`/missing blocking rule does not apply to a non-visual library change).
- **Commit Quality:** Conventional format, story ID present, clear messages, TDD sequence
  (stubs → red tests → impl → test hardening → doc refresh → evidence → ADR).
- **Diff Size:** 3108 additions total, but production code is only 295 lines; the rest is
  tests (1200), ADR-013 (531), and demo evidence. Reasonable.
- **Missing Changes:** None — all 8 ACs traced to tests and evidence.
- **Dependency Status:** Upstream STORY-170/171 merged.

---

_Fresh-eyes review performed on the diff, PR description, and test evidence only._
