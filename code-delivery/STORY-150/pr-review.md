# PR Review — STORY-150 (TLS Drain-Loop DRY Refactor)

**PR:** #379
**Branch:** `feature/STORY-150-tls-drain-dry` → `develop`
**Verdict:** **APPROVE**

## Scope Reviewed

- `src/analyzer/tls.rs` — carry-drain-loop DRY refactor + VP-039 table update
- `tests/bc_150_drain_loop_dry_tests.rs` — 10 new tests (2 structural Red Gates + 1 sanity marker + 7 behavior-preservation regression pins)
- `docs/demo-evidence/STORY-150/` — per-AC demo evidence (webm/gif/tape/txt)

Diff stats: **22 files, +1199 / -57**; the production change is confined to a single function body in `src/analyzer/tls.rs` (net -25 lines).

## Checklist Findings

### 1. Diff coherence — PASS
All changes trace to STORY-150 (structural DRY refactor of the TLS carry-drain loop) and its follow-on fix commits (F-150-P1-001..003, F-150-P2-001/002, F-150-P3-002, plus demo evidence). No unrelated changes.

### 2. Description accuracy — PASS
- Single `parse_tls_message_handshake(&msg_bytes)` call site — confirmed at src/analyzer/tls.rs:933.
- Single `let msg_bytes = ...` extraction — confirmed at src/analyzer/tls.rs:928.
- Both were duplicated per-direction before; now unified inside a single `if msg_type == expected_msg_type` block with direction-guarded match arms.

### 3. Direction-guard defense-in-depth (F-150-P1-003) — PASS
- `Ok((_, ClientHello(ch))) if matches!(direction, Direction::ClientToServer)` and symmetric `ServerHello`/`ServerToClient` guards on the Hello dispatch arms.
- Semantic equivalence to pre-refactor per-direction arms preserved exactly: a msg_type/parse-result cross (e.g. msg_type=0x01 gate but parser returns `ServerHello`) falls through to `Ok(_) | Err(_) => self.parse_errors += 1` rather than firing the wrong direction's flag-set + dispatch.
- The guards are functionally redundant against the current `tls-parser` behavior (which reads msg_type from bytes[0] and produces the matching variant) but provide a hard belt against future parser regressions. Correct and cheap.

### 4. Test coverage — PASS (10 tests, correctly targeted)
- **Structural Red Gates (AC-150-001 / TLS-DRAIN-DUP-001):**
  - `test_BC_150_001_..._parse_hs_call_not_duplicated` — asserts exactly one `parse_tls_message_handshake(` call in the function body.
  - `test_BC_150_001_..._msg_bytes_extraction_not_duplicated` — asserts at most one `let msg_bytes` extraction.
  - Both use the same brace-depth `extract_fn_body` helper convention as `bc_149_single_borrow_invariant_tests.rs` — repo-idiomatic.
- **AC-150-003 sanity marker:** `test_BC_150_003_vp039_proof_module_marker_present` — confirms the `mod kani_proofs_vp039` header and `"model step"` table marker survive the refactor. Docstring honestly discloses that full table-content validation is not machine-checkable without a new `// VP-039-LINE: N` annotation convention.
- **Behavior-preservation regression pins (VP-039 / BC-2.07.004 / BC-2.07.028 / BC-2.07.038):**
  - Single-record C2S ClientHello (flag + count + carry empty).
  - Single-record S2C ServerHello (flag + carry empty).
  - Directional isolation both ways (C2S must not touch S2C state; S2C must not touch C2S state).
  - Fragmented C2S/S2C across 3 TLS records exercising header-incomplete guard, cursor advance, and carry restore — the exact paths modeled by Kani `drain_loop_model`.
  - Parse-error symmetry across arms (malformed msg_type=0x01 and msg_type=0x02 produce identical parse_errors increments).
- Fixture helpers `build_client_hello_handshake_bytes` / `build_server_hello_handshake_bytes` are deterministic and parseable; test module wrapped in `mod story_150` per DF-TEST-NAMESPACE-001.

### 5. VP-039 line-correspondence table (AC-150-003) — PASS
Spot-checked table entries against the current source:

| Table entry | Table says | Actual line |
|---|---|---|
| header-incomplete guard | ~900 | 900 |
| msg_type read | ~903 | 903 |
| body_len 3-byte BE decode | ~904 | 904 |
| Decision-4 spoof guard | ~910 | 910 |
| incomplete-body guard | ~920 | 921 |
| dispatch clone slice | ~927 | 928 |
| cursor advance | ~957 | 959 |
| single post-loop drain | ~963 | 964 |

All within the "~" tolerance. Header prose updated to reference STORY-150 / AC-150-001.

### 6. Commit quality — PASS
Semantic PR types used consistently. Commit graph is disciplined: `test:` first (Red Gate), `feat:` next (implementation), then focused `fix:` and `docs:` commits per finding (F-150-P1-003, F-150-P1-001, F-150-P2-001/002, F-150-P3-002), then demo evidence. Every commit references STORY-150 and the specific AC/finding tag.

### 7. Diff size — PASS
Production diff (`src/analyzer/tls.rs`) is 109 changed lines net −25 (unify two ~30-line arms into one ~35-line block + comments + VP-039 table refresh). Bulk of the +1199 total is demo evidence binaries and the 669-line test file.

### 8. Missing changes / dependency status — PASS
No dependencies flagged in the task context; adversarial convergence (8 passes, streak 3/3, BC-5.39.001 satisfied), security review (0 CRITICAL/HIGH/MEDIUM), Kani VP-039 (3/3, 75/12/12), and mutation testing (42/45, 0 new survivors) are all reported clean upstream. CI expected green (fmt/clippy/test/all-targets).

## Correctness / Style / Spec-Fidelity Notes

- Semantic equivalence to pre-refactor code preserved: for every `(msg_type, direction, parse_result)` triple, the post-refactor branch reaches the same terminal action (flag-set + dispatch, `parse_errors += 1`, or silent consume).
- The `expected_msg_type` hoist is a real (small) improvement — computed once outside the loop instead of implicit in an outer `match direction` on every iteration.
- No `unsafe`, no lifetime relaxations, no visibility changes, no new public surface.
- No emojis, no unrelated formatting churn.

## Verdict

**APPROVE** — no blocking findings. Merge when ready.
