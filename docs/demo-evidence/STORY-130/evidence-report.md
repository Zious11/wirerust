# Demo Evidence Report — STORY-130

**Story:** EtherNet/IP Pure-Core Parse: ENIP Header, Command Classification, Frame Validity, and Kani VP-032  
**Story ID:** STORY-130  
**Wave:** 58  
**Product type:** Pure-core library (no CLI surface — CLI flags arrive in STORY-131)  
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test enip_analyzer_tests`)  
**Recorded:** 2026-06-25  
**Test result at recording time:** 21 passed / 0 failed / 0 ignored

---

## AC Coverage Map

| AC | Title | Test filter used | Artifact (GIF) | Artifact (WebM) | Tape |
|----|-------|-----------------|---------------|----------------|------|
| AC-130-001 | `parse_enip_header` accept path — canonical 24-byte LE vector | `test_parse_enip_header` | `AC-001-002-parse-header-accept-reject.gif` | `AC-001-002-parse-header-accept-reject.webm` | `AC-001-002-parse-header-accept-reject.tape` |
| AC-130-002 | `parse_enip_header` reject path — empty and 23-byte input, no panic | `test_parse_enip_header` | `AC-001-002-parse-header-accept-reject.gif` | `AC-001-002-parse-header-accept-reject.webm` | `AC-001-002-parse-header-accept-reject.tape` |
| AC-130-003 | `classify_enip_command` — all 9 named variants + Unknown arm | `test_classify` | `AC-003-005-classify-command.gif` | `AC-003-005-classify-command.webm` | `AC-003-005-classify-command.tape` |
| AC-130-004 | `is_valid_enip_frame` biconditional gate — command-only | `test_is_valid` | `AC-004-validity-gate.gif` | `AC-004-validity-gate.webm` | `AC-004-validity-gate.tape` |
| AC-130-005 | `classify_enip_command` Unknown arm reachable at 0x0000, 0xFFFF, gap 0x0067 | `test_classify` | `AC-003-005-classify-command.gif` | `AC-003-005-classify-command.webm` | `AC-003-005-classify-command.tape` |
| AC-130-006 | VP-032 Kani Sub-A/B/C harnesses | _cargo kani only — not run in cargo test_ | `AC-001-006-all-tests.gif` (note below) | `AC-001-006-all-tests.webm` | `AC-001-006-all-tests.tape` |

> **AC-130-006 note:** Kani harnesses in `#[cfg(kani)] mod kani_proofs` are not exercised by
> `cargo test`. The full-suite recording (`AC-001-006-all-tests.*`) demonstrates the 21/21 green
> result covering AC-001 through AC-005. Kani proof verification requires `cargo kani` and is
> tracked as a separate CI step outside the scope of demo recording.

---

## Recordings Detail

### AC-001-002-parse-header-accept-reject

Demonstrates both the accept path and the reject path of `parse_enip_header`.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests parse_header::test_parse_enip_header`
- Five tests exercise: canonical 24-byte vector (session_handle=0x01020304), exactly 24
  zero bytes, 23-byte input, empty slice, 23-byte 0xFF slice
- All five tests pass green
- Confirms BC-2.17.002 postconditions 2–7 (LE field decode) and BC-2.17.001 postconditions 1–3
  (None for < 24 bytes)

**Tests in recording:**
- `test_parse_enip_header_valid`
- `test_parse_enip_header_too_short`
- `test_parse_enip_header_exactly_24`
- `test_parse_enip_header_no_panic_empty`
- `test_parse_enip_header_no_panic_23_bytes`

---

### AC-003-005-classify-command

Demonstrates `classify_enip_command` — all 9 named ODVA command codes and the Unknown arm.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests parse_header::test_classify`
- Four tests: all 9 named mappings (0x0004, 0x0063..0x0075), Unknown at 0x0001, Unknown at
  0x0000, Unknown at 0xFFFF, Unknown at gap value 0x0067
- All five tests pass green
- Confirms BC-2.17.004 postcondition 2 (named mappings), postcondition 4 (Unknown reachable),
  VP-032 Sub-B non-vacuity requirement (DF-KANI-NONVACUITY-001)

**Tests in recording:**
- `test_classify_enip_command_known`
- `test_classify_enip_command_unknown`
- `test_classify_enip_command_unknown_zero`
- `test_classify_enip_command_unknown_ffff`
- `test_classify_enip_command_unknown_gap`

---

### AC-004-validity-gate

Demonstrates `is_valid_enip_frame` biconditional: true iff command in known-command set;
other header fields do not affect the result.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests parse_header::test_is_valid`
- Seven tests: all 9 known commands return true, 0x0000 returns false, boundary (Cancel=true,
  0x0076=false, 0xFFFF=false), all-fields-zeroed (command=0x0000 → false), command-only gate
  (ListIdentity+zeroed fields → true), gap 0x0071 → false, 0x0062 → false
- All seven tests pass green
- Confirms BC-2.17.003 postconditions 1–4 (biconditional, reject unknown, command-only,
  all u16 values) and invariant 1; VP-032 Sub-C

**Tests in recording:**
- `test_is_valid_enip_frame_known_commands_true`
- `test_is_valid_enip_frame_unknown_command_false`
- `test_is_valid_enip_frame_boundary_commands`
- `test_is_valid_enip_frame_all_fields_zeroed`
- `test_is_valid_enip_frame_command_only_gate`
- `test_is_valid_enip_frame_gap_value`
- `test_is_valid_enip_frame_below_list_identity`

---

### AC-001-006-all-tests (complete suite)

Full `enip_analyzer_tests` suite — 21/21 green — covering all unit-test-verifiable ACs.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests` (no filter — all 21 tests)
- All 21 tests in `mod parse_header` pass
- Confirms complete coverage of AC-130-001 through AC-130-005 in a single view

---

## Artifacts

```
.factory/demo-evidence/STORY-130/
  AC-001-002-parse-header-accept-reject.tape    (VHS script)
  AC-001-002-parse-header-accept-reject.gif     (132 KB)
  AC-001-002-parse-header-accept-reject.webm    (101 KB)
  AC-003-005-classify-command.tape              (VHS script)
  AC-003-005-classify-command.gif               (132 KB)
  AC-003-005-classify-command.webm              (131 KB)
  AC-004-validity-gate.tape                     (VHS script)
  AC-004-validity-gate.gif                      (144 KB)
  AC-004-validity-gate.webm                     (156 KB)
  AC-001-006-all-tests.tape                     (VHS script)
  AC-001-006-all-tests.gif                      (186 KB)
  AC-001-006-all-tests.webm                     (110 KB)
  evidence-report.md                            (this file)
```

## Coverage Summary

| AC | Evidenced | Path demonstrated |
|----|-----------|-------------------|
| AC-130-001 | Yes | accept path (canonical 24-byte SendRRData vector, all six fields) |
| AC-130-002 | Yes | reject path (empty, 23-byte) |
| AC-130-003 | Yes | all 9 named mappings + Unknown at 0x0001 |
| AC-130-004 | Yes | biconditional (all 9 true, boundaries, command-only gate) |
| AC-130-005 | Yes | Unknown at 0x0000, 0xFFFF, gap 0x0067 |
| AC-130-006 | Partial | Unit tests 21/21 green; Kani Sub-A/B/C require `cargo kani` — not in demo scope |
