# Evidence Report: STORY-111

**Story:** STORY-111 — etherparse 0.20 Migration + DecodedFrame/ArpFrame Types + BC-2.02.009 Revision  
**Branch:** `worktree-issue-9-story-111-arp-etherparse-decodedframe`  
**HEAD:** `9cf7bca`  
**Captured:** 2026-06-14  
**Recording method:** VHS v0.11.0 (CLI) + captured log files  
**Note on VHS:** `Wait+Line` is unsupported in VHS 0.11.0 with zsh on macOS. All tapes use `Sleep` for timing. Output is genuine — cargo was pre-warmed in this worktree, so `cargo build --release` and `cargo test` execute quickly.

---

## Product Type Note

STORY-111 is a **migration/scaffolding story** — there is no user-facing CLI/UI behavior change. The appropriate evidence is:
- Build evidence (etherparse 0.20 compiles)
- Test evidence (all ACs pass, zero regressions)
- Lint/fmt evidence (no warnings, no formatting violations)

Real ARP frame extraction (`Ok(DecodedFrame::Arp(...))`) is STORY-112's scope. No ARP "feature demo" exists in STORY-111 by design — the `extract_arp_frame` placeholder returns `None`, which routes to a transitional `Err("ARP extraction not yet implemented")`.

---

## Summary Results

| Gate | Result | Evidence |
|------|--------|----------|
| `cargo build --release` | PASS (7.46s, etherparse v0.20.2 compiled) | `AC-010-etherparse-version.gif/.webm` |
| `cargo test --all-targets` | PASS (53 suites, 0 failures) | `cargo-test-all-targets.log` |
| `cargo clippy --all-targets -- -D warnings` | PASS (0 warnings) | `AC-005-006-clippy-full.gif/.webm` |
| `cargo fmt --check` | PASS (no formatting violations) | `AC-005-006-clippy-full.gif/.webm` |

---

## Acceptance Criteria Coverage

### AC-003: non-IP/non-ARP frame returns "No IP layer found" error

- **Test:** `test_decode_non_ip_non_arp_frame_returns_no_ip_error`
- **Suite:** `tests/decoder_tests.rs`
- **Result:** PASS
- **Recording:** `AC-003-005b-non-panic.gif` / `AC-003-005b-non-panic.webm`
- **Log:** `AC-003-005b-009-key-test-results.log`
- **Notes:** Test renamed from `test_decode_non_ip_frame_returns_error` per v1.3 changelog. ARP subtest clause removed (ARP success path is STORY-112 AC-006).

### AC-005: three-way dispatch arms compile; existing IP-path tests green

- **Test:** `cargo clippy --all-targets -- -D warnings` (no compile-time errors); all `test_decode_*` IP-path tests pass
- **Suite:** `tests/decoder_tests.rs` (16 tests, all pass)
- **Result:** PASS
- **Recording:** `AC-005-006-clippy-full.gif` / `AC-005-006-clippy-full.webm`
- **Notes:** `Some(NetSlice::Arp(arp))` dispatch arm present; `Some(other_net)` → `strict_ip_triple` arm; `None` → `Err("No IP layer found")` arm. All three compile with zero clippy warnings.

### AC-005b: `extract_arp_frame` placeholder is non-panicking; ARP input does not panic

- **Test:** `test_decode_arp_shaped_input_does_not_panic`
- **Suite:** `tests/decoder_tests.rs`
- **Result:** PASS
- **Recording:** `AC-003-005b-non-panic.gif` / `AC-003-005b-non-panic.webm`
- **Log:** `AC-003-005b-009-key-test-results.log`
- **Notes:** `extract_arp_frame` returns `None` (no `todo!()`/`unimplemented!()`). ARP arm routes `None` → transitional `Err("ARP extraction not yet implemented")` — non-panicking. VP-008 no-panic invariant holds.

### AC-006: `strict_ip_triple` ARP unreachable! arm; no runtime panic in any test

- **Test:** `cargo test --all-targets` (all 53 suites, 0 failures — no unreachable! triggered at runtime)
- **Result:** PASS
- **Recording:** `AC-005-006-clippy-full.gif` / `AC-005-006-clippy-full.webm`
- **Log:** `cargo-test-all-targets.log`
- **Notes:** `NetSlice::Arp(_) => unreachable!("ARP frames are routed before strict_ip_triple")` is present as compile-safety arm. No test reaches it at runtime (ARP frames are intercepted in `decode_packet`'s `Ok(slice)` arm first).

### AC-009: SliceError::Len contract tests green; VP-008 harness return type updated

- **Tests:**
  - `test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing` — PASS
  - `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered` — PASS
  - `test_VP_008_fuzz_harness_exists` — PASS
- **Suite:** `tests/decoder_tests.rs` + `tests/bc_2_02_story003_tests.rs`
- **Result:** PASS
- **Recording:** `AC-009-sliceerror-len-contracts.gif` / `AC-009-sliceerror-len-contracts.webm`
- **Log:** `AC-003-005b-009-key-test-results.log`
- **Notes:** VP-008 fuzz harness (`fuzz/fuzz_targets/fuzz_decode_packet.rs`) updated to `Result<DecodedFrame>` return type; handles both `Ok(DecodedFrame::Ip(_))` and `Ok(DecodedFrame::Arp(_))` as non-panic outcomes. `SliceError::Len` is unchanged in etherparse 0.20 — both contract tests confirm lax recovery still works.

### AC-010: `Cargo.toml` updated to etherparse 0.20; prose-sweep in decoder.rs

- **Test:** `grep etherparse Cargo.toml` → `etherparse = "0.20"`; `cargo build --release` green
- **Result:** PASS
- **Recording:** `AC-010-etherparse-version.gif` / `AC-010-etherparse-version.webm`
- **Log:** `AC-010-cargo-build-release.log`
- **Notes:** `etherparse v0.20.2` is the resolved version (per `cargo build --release` output). Both `//!` module doc and `SliceError` import comment block in `src/decoder.rs` updated to reference etherparse 0.20. Version-pin comment in `Cargo.toml` updated alongside the version pin.

---

## BC-2.02.009 Reconciled ARP Tests (4 tests, all green)

These tests in `tests/bc_2_02_story003_tests.rs` were reconciled to BC-2.02.009 v1.6 in commit `9e07423`:

| Test | Result |
|------|--------|
| `test_BC_2_02_009_ec006_arp_ethernet_no_ip_layer` | PASS |
| `test_BC_2_02_009_ec007_custom_ethertype_no_ip_layer` | PASS |
| `test_BC_2_02_009_non_ip_frame_rejected` | PASS |
| `test_BC_2_02_009_strict_path_sll_arp_no_ip` | PASS |

---

## Key Structural Evidence (src/decoder.rs)

Confirmed present (via `grep` on decoder.rs):
- `pub struct ArpFrame` with 7 fields (operation, sender_mac, sender_ip, target_mac, target_ip, outer_src_mac: `Option<[u8;6]>`, packet_len)
- `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }`
- `pub fn decode_packet(...) -> Result<DecodedFrame>`
- `pub fn extract_arp_frame(...) -> Option<ArpFrame>` (non-panicking; returns `None`)
- Module doc references etherparse 0.20 (`//!` block, line 52–57)
- `NetSlice::Arp(_) => unreachable!(...)` in `strict_ip_triple`
- `LaxNetSlice::Arp(_) => unreachable!(...)` in `lax_ip_triple` (symmetric compile-safety guard)

Forbidden dependency check: `src/decoder.rs` does NOT import `crate::analyzer::arp` — confirmed by zero grep hits.

---

## Test Suite Totals

```
53 test suites
0 failures
All cargo gates: build, test, clippy, fmt — GREEN
```

---

## Recordings Index

| File | AC | Type | Description |
|------|----|------|-------------|
| `AC-010-etherparse-version.gif` | AC-010 | VHS GIF | `grep etherparse Cargo.toml` + `cargo build --release` tail |
| `AC-010-etherparse-version.webm` | AC-010 | VHS WEBM | Same (archival) |
| `AC-010-etherparse-version.tape` | AC-010 | VHS script | Source tape |
| `AC-003-005b-non-panic.gif` | AC-003, AC-005b | VHS GIF | AC-003 + AC-005b tests green |
| `AC-003-005b-non-panic.webm` | AC-003, AC-005b | VHS WEBM | Same (archival) |
| `AC-003-005b-non-panic.tape` | AC-003, AC-005b | VHS script | Source tape |
| `AC-009-sliceerror-len-contracts.gif` | AC-009 | VHS GIF | SliceError::Len + VP-008 tests green |
| `AC-009-sliceerror-len-contracts.webm` | AC-009 | VHS WEBM | Same (archival) |
| `AC-009-sliceerror-len-contracts.tape` | AC-009 | VHS script | Source tape |
| `AC-005-006-clippy-full.gif` | AC-005, AC-006 | VHS GIF | `clippy -D warnings` + `fmt --check` clean |
| `AC-005-006-clippy-full.webm` | AC-005, AC-006 | VHS WEBM | Same (archival) |
| `AC-005-006-clippy-full.tape` | AC-005, AC-006 | VHS script | Source tape |
| `cargo-test-all-targets.log` | All | Log | Full `cargo test --all-targets` output (53 suites) |
| `AC-003-005b-009-key-test-results.log` | AC-003, AC-005b, AC-009 | Log | Extracted per-AC test lines |
| `AC-010-cargo-build-release.log` | AC-010 | Log | Build output summary |
| `AC-005-006-009-clippy-fmt.log` | AC-005, AC-006, AC-009 | Log | Clippy + fmt captured output |
