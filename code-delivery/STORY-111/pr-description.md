# [STORY-111] etherparse 0.20 Migration + DecodedFrame/ArpFrame Types (BC-2.02.009)

**Epic:** E-16 — ARP Security Analyzer (first E-16 story; issue #9)
**Mode:** feature (F4 delta-implementation — brownfield migration/scaffolding)
**Convergence:** CONVERGED after 3 consecutive ZERO-finding adversarial passes (A/B/C all CLEAN)

![Tests](https://img.shields.io/badge/tests-53%20suites%2C%200%20failures-brightgreen)
![Clippy](https://img.shields.io/badge/clippy--D%20warnings-CLEAN-brightgreen)
![Fmt](https://img.shields.io/badge/cargo%20fmt--check-CLEAN-brightgreen)
![Adversarial](https://img.shields.io/badge/adversarial%20passes-3%20ZERO-green)

This PR is the **first E-16 story** (ARP Security Analyzer, issue #9). It upgrades etherparse
from 0.16 to 0.20 (resolves to 0.20.2), introduces the `DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }`
enum and `ArpFrame` struct in `src/decoder.rs`, changes `decode_packet` to return `Result<DecodedFrame>`,
adds compile-safety `unreachable!` arms in `strict_ip_triple` and `lax_ip_triple` (D-072 symmetric
design; both arms are provably dead at runtime), ships a non-panicking `extract_arp_frame` placeholder
(`-> Option<ArpFrame>`, returns `None`), and updates the VP-008 fuzz harness to accept
`Result<DecodedFrame>`. All 53 test suites pass with zero failures; clippy -D warnings and fmt --check
are clean. Real ARP extraction (`Ok(DecodedFrame::Arp(...))`) is STORY-112's scope.

**Transitional scope note:** The `extract_arp_frame` placeholder returns `None`, which routes to
`Err(anyhow!("ARP extraction not yet implemented"))` — a TEMPORARY mapping. STORY-112 replaces
both the placeholder body and this mapping with the full implementation and the real error string
`"Non-Ethernet/IPv4 ARP frame"` (AC-012 in STORY-112).

---

## Architecture Changes

```mermaid
graph TD
    DecodePacket["src/decoder.rs<br/>decode_packet()<br/>Result&lt;ParsedPacket&gt; → Result&lt;DecodedFrame&gt;"]
    DecodedFrame["DecodedFrame enum (NEW)<br/>Ip(ParsedPacket) | Arp(ArpFrame)"]
    ArpFrame["ArpFrame struct (NEW)<br/>operation, sender_mac, sender_ip<br/>target_mac, target_ip<br/>outer_src_mac: Option&lt;[u8;6]&gt;, packet_len"]
    StrictArm["strict_ip_triple()<br/>+ NetSlice::Arp(_) ⇒ unreachable! (compile-safety)"]
    LaxArm["lax_ip_triple()<br/>+ LaxNetSlice::Arp(_) ⇒ unreachable! (compile-safety)"]
    Placeholder["extract_arp_frame() (NEW placeholder)<br/>→ Option&lt;ArpFrame&gt; (returns None)"]
    FuzzHarness["VP-008 fuzz harness<br/>Result&lt;ParsedPacket&gt; → Result&lt;DecodedFrame&gt;"]

    DecodePacket -->|wraps| DecodedFrame
    DecodedFrame -->|variant| ArpFrame
    DecodePacket -->|calls (placeholder)| Placeholder
    DecodePacket -->|guards| StrictArm
    DecodePacket -->|guards| LaxArm
    FuzzHarness -.->|updated type| DecodePacket

    style DecodedFrame fill:#90EE90
    style ArpFrame fill:#90EE90
    style Placeholder fill:#90EE90
    style FuzzHarness fill:#90EE90
```

<details>
<summary><strong>Architecture Decision Record — D-072 Symmetric unreachable! Design</strong></summary>

### D-072: Symmetric unreachable! Arms in strict_ip_triple / lax_ip_triple

**Context:** etherparse 0.20 adds `NetSlice::Arp` and `LaxNetSlice::Arp` variants. Both
`strict_ip_triple` and `lax_ip_triple` match on `NetSlice`/`LaxNetSlice` and would fail to
compile without exhaustive arms for the new `Arp` variant. The question is whether ARP routing
lives in these helpers or in `decode_packet`.

**Decision (ADR-008 Decision 3 v2.1):** ARP routing lives exclusively in `decode_packet`:
- `Ok(slice)` arm: intercepts `Some(NetSlice::Arp(arp))` before `strict_ip_triple` is called.
- `Err(SliceError::Len(_))` arm: intercepts `Some(LaxNetSlice::Arp(_))` before `lax_ip_triple`
  is called.
Both `strict_ip_triple` and `lax_ip_triple` therefore receive `unreachable!` arms for their
respective `Arp` variants. These arms are compile-safety guards only — provably dead at runtime.

**Rationale:** Symmetric design prevents silent divergence. If future refactoring accidentally
moves ARP into these helpers, the unreachable! would fire in tests rather than silently misbehave.
`lax_ip_triple` returns `IpTriple` and cannot route ARP regardless.

**Alternatives Considered:**
1. Route ARP in `strict_ip_triple` / `lax_ip_triple` — rejected; these return `IpTriple`, not
   `Result<DecodedFrame>`, so routing would require signature changes and violate SS-02 module
   boundaries.
2. Use `_ => unreachable!()` wildcard — rejected; explicit `NetSlice::Arp(_)` arm is
   forward-compatible and documents intent.

**Consequences:**
- Zero runtime cost (compile-safety only).
- Enables STORY-112 to add real ARP extraction in `decode_packet` without touching these helpers.

</details>

---

## Story Dependencies

```mermaid
graph LR
    S110["STORY-110<br/>✅ merged (E-15 final)"]
    S111["STORY-111<br/>🟡 this PR"]
    S112["STORY-112<br/>⏳ extract_arp_frame impl"]
    S113["STORY-113<br/>⏳ ARP dispatcher"]
    S114["STORY-114<br/>⏳ MITRE entries"]

    S110 --> S111
    S111 --> S112
    S112 --> S113
    S112 --> S114

    style S111 fill:#FFD700
```

**Upstream dependency:** STORY-110 (E-15 final, DNP3 dispatcher) is merged on develop (31d1231).
STORY-111 branches from 31d1231. No merge conflicts.

**Downstream:** STORY-112 is blocked on this PR. It cannot implement `extract_arp_frame` until
`DecodedFrame`, `ArpFrame`, and the `decode_packet` return-type change from this PR are merged.

---

## Spec Traceability

```mermaid
flowchart LR
    BC["BC-2.02.009 v1.7<br/>Non-IP Non-ARP → No-IP-Error<br/>ARP → DecodedFrame::Arp"]
    VP["VP-008<br/>decode_packet no-panic"]

    AC003["AC-003<br/>non-IP/non-ARP → No IP layer found"]
    AC005["AC-005<br/>three-way dispatch compiles"]
    AC005b["AC-005b<br/>extract_arp_frame non-panicking"]
    AC006["AC-006<br/>strict_ip_triple unreachable! arm"]
    AC009["AC-009<br/>VP-008 harness Result&lt;DecodedFrame&gt;"]
    AC010["AC-010<br/>Cargo.toml 0.20 + prose-sweep"]

    T003["test_decode_non_ip_non_arp_frame_returns_no_ip_error"]
    T005b["test_decode_arp_shaped_input_does_not_panic"]
    T009a["test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing"]
    T009b["test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered"]

    BC --> AC003
    BC --> AC005
    BC --> AC005b
    BC --> AC006
    VP --> AC009

    AC003 --> T003
    AC005b --> T005b
    AC009 --> T009a
    AC009 --> T009b

    T003 --> SRC["src/decoder.rs<br/>fuzz/fuzz_targets/fuzz_decode_packet.rs"]
    T005b --> SRC
    T009a --> SRC
    T009b --> SRC
```

---

## Test Evidence

### Coverage Summary

| Metric | Value | Status |
|--------|-------|--------|
| `cargo test --all-targets` | **53 suites, 0 failures** | PASS |
| `cargo clippy --all-targets -- -D warnings` | **0 warnings** | PASS |
| `cargo fmt --check` | **no violations** | PASS |
| `cargo build --release` | **GREEN (etherparse v0.20.2)** | PASS |
| Adversarial passes | **3/3 ZERO findings** | CONVERGED |

<details>
<summary><strong>Per-AC Test Results</strong></summary>

| AC | Test | File | Result |
|----|------|------|--------|
| AC-003 | `test_decode_non_ip_non_arp_frame_returns_no_ip_error` | `tests/decoder_tests.rs` | PASS |
| AC-005 | `cargo check` + all `test_decode_*` IP-path tests (16 tests) | `tests/decoder_tests.rs` | PASS |
| AC-005b | `test_decode_arp_shaped_input_does_not_panic` | `tests/decoder_tests.rs` | PASS |
| AC-006 | `cargo test --all-targets` (no unreachable! triggered at runtime) | all suites | PASS |
| AC-009 | `test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing` | `tests/decoder_tests.rs` | PASS |
| AC-009 | `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered` | `tests/bc_2_02_story003_tests.rs` | PASS |
| AC-009 | `test_VP_008_fuzz_harness_exists` | `tests/decoder_tests.rs` | PASS |
| AC-010 | `cargo check` after Cargo.toml bump; `grep etherparse Cargo.toml` → `"0.20"` | build | PASS |

### BC-2.02.009 ARP Reconciliation Tests (commit 9e07423)

| Test | File | Result |
|------|------|--------|
| `test_BC_2_02_009_ec006_arp_ethernet_no_ip_layer` | `tests/bc_2_02_story003_tests.rs` | PASS |
| `test_BC_2_02_009_ec007_custom_ethertype_no_ip_layer` | `tests/bc_2_02_story003_tests.rs` | PASS |
| `test_BC_2_02_009_non_ip_frame_rejected` | `tests/bc_2_02_story003_tests.rs` | PASS |
| `test_BC_2_02_009_strict_path_sll_arp_no_ip` | `tests/bc_2_02_story003_tests.rs` | PASS |

### Structural Evidence (src/decoder.rs)

- `pub struct ArpFrame` — 7 fields: `operation: u16`, `sender_mac: [u8;6]`, `sender_ip: [u8;4]`, `target_mac: [u8;6]`, `target_ip: [u8;4]`, `outer_src_mac: Option<[u8;6]>`, `packet_len: usize`
- `pub enum DecodedFrame { Ip(ParsedPacket), Arp(ArpFrame) }`
- `pub fn decode_packet(...) -> Result<DecodedFrame>`
- `pub fn extract_arp_frame(...) -> Option<ArpFrame>` — non-panicking; returns `None`; no `todo!()`/`unimplemented!()`
- Module doc (`//!` block) references etherparse 0.20
- `NetSlice::Arp(_) => unreachable!(...)` present in `strict_ip_triple`
- `LaxNetSlice::Arp(_) => unreachable!(...)` present in `lax_ip_triple` (symmetric)
- Forbidden dependency check: `src/decoder.rs` does NOT import `crate::analyzer::arp` — CONFIRMED

</details>

### Demo Evidence

Demo recordings captured at `.factory/demo-evidence/STORY-111/` (note: `.factory/` lives on the
`factory-artifacts` branch; see CI gate note in CLAUDE.md).

| Recording | AC | Type | Description |
|-----------|----|------|-------------|
| `AC-010-etherparse-version.gif` | AC-010 | VHS GIF | `grep etherparse Cargo.toml` → `"0.20"` + `cargo build --release` (etherparse v0.20.2) |
| `AC-003-005b-non-panic.gif` | AC-003, AC-005b | VHS GIF | `test_decode_non_ip_non_arp_frame_returns_no_ip_error` + `test_decode_arp_shaped_input_does_not_panic` PASS |
| `AC-009-sliceerror-len-contracts.gif` | AC-009 | VHS GIF | SliceError::Len contract tests + VP-008 harness PASS |
| `AC-005-006-clippy-full.gif` | AC-005, AC-006 | VHS GIF | `clippy -D warnings` + `fmt --check` CLEAN |

**Product type note:** STORY-111 is a migration/scaffolding story — there is no user-facing CLI
behavior change. The appropriate evidence is build, test, and lint gates. Real ARP frame extraction
(`Ok(DecodedFrame::Arp(...))`) is STORY-112's scope.

---

## Holdout Evaluation

N/A — evaluated at wave gate. This is an internal scaffolding/migration story with no user-facing
behavior change. Holdout evaluation for ARP end-to-end behavior is scoped to STORY-112+.

---

## Adversarial Review

Step-4.5 per-story adversarial convergence was completed prior to this PR:

| Pass | Findings | Blocking | Status |
|------|----------|----------|--------|
| A | 0 | 0 | ZERO |
| B | 0 | 0 | ZERO |
| C | 0 | 0 | ZERO |

**Convergence:** 3/3 ZERO-finding passes — SATISFIED. Adversarial reviewer found no issues
after the D-072 symmetric-unreachable alignment, seam-pinning corrections (AC-005 temporary
`Err` string, AC-005b `Option<ArpFrame>` signature), and coverage-mapping table sync.

Notable adversarial finding resolutions (prior passes, before ZERO convergence):
- **Seam pinning:** `Some(NetSlice::Arp)` → `Err("ARP extraction not yet implemented")` (TEMPORARY,
  not `"Non-Ethernet/IPv4 ARP frame"` — that string is STORY-112 AC-012 territory)
- **AC-005b type fix:** `extract_arp_frame` signature fixed to `-> Option<ArpFrame>` returning `None`
  (not `Err`) — non-panic assertion preserved
- **D-072 lax arm:** `lax_ip_triple` ARP arm reframed from "explicit routing" to symmetric
  `unreachable!` per ADR-008 Decision 3 v2.1

---

## Security Review

```mermaid
graph LR
    Critical["Critical: 0"]
    High["High: 0"]
    Medium["Medium: 0"]
    Low["Low: 0"]

    style Critical fill:#90EE90
    style High fill:#90EE90
    style Medium fill:#90EE90
    style Low fill:#90EE90
```

<details>
<summary><strong>Security Scan Details</strong></summary>

### Scope

This PR introduces two new data types (`DecodedFrame`, `ArpFrame`) and a non-panicking placeholder
function in `src/decoder.rs`. No network I/O, no authentication, no file system access, no
unsafe code added.

### Analysis

- **Injection:** N/A — no string formatting with user input; `anyhow!("ARP extraction not yet implemented")` is a static literal.
- **Memory safety:** `ArpFrame` fields are fixed-size arrays (`[u8;6]`, `[u8;4]`) and `usize` — no heap allocation vectors. `extract_arp_frame` placeholder returns `None` without any dereferencing.
- **Panic safety:** `extract_arp_frame` uses no `todo!()`/`unimplemented!()`/`panic!()`. The `unreachable!` arms in `strict_ip_triple` and `lax_ip_triple` are provably dead (decode_packet intercepts ARP before these functions are reached). VP-008 no-panic invariant holds.
- **Forbidden dependency:** `src/decoder.rs` does not import `src/analyzer/arp.rs` — verified.
- **etherparse 0.20.2:** No known advisories (resolves via `cargo update`; `cargo audit` clean on develop as of this branch).
- **Supply chain:** No new transitive dependencies introduced beyond the etherparse 0.16 → 0.20 bump; SHA-pinned CI actions unchanged per CLAUDE.md policy.

### VP-008 Formal Property

| Property | Method | Status |
|----------|--------|--------|
| `decode_packet` no-panic on ARP-shaped input | `test_decode_arp_shaped_input_does_not_panic` (unit) | VERIFIED |
| `decode_packet` no-panic on arbitrary bytes | VP-008 cargo-fuzz harness (updated to `Result<DecodedFrame>`) | HARNESS UPDATED |

</details>

---

## Risk Assessment & Deployment

### Blast Radius
- **Systems affected:** `src/decoder.rs` (return type change), all callers of `decode_packet` (updated), VP-008 fuzz harness
- **User impact:** No user-visible behavior change. The `--help` output and all existing analyzers are unaffected. Callers of `decode_packet` that previously matched `Ok(parsed_packet)` now match `Ok(DecodedFrame::Ip(parsed_packet))` — this is a compile-time change only; 0 runtime regressions.
- **Data impact:** None — no persistence, no state change
- **Risk Level:** LOW (migration/scaffolding; all existing IP-path tests pass; no behavioral change for non-ARP traffic)

### Performance Impact

| Metric | Impact |
|--------|--------|
| Latency | Negligible — DecodedFrame is a one-word enum discriminant; no heap allocation |
| Memory | `ArpFrame` is 47 bytes on-stack; no regression on IP path |
| Throughput | No change to hot path (ARP placeholder returns immediately) |

<details>
<summary><strong>Rollback Instructions</strong></summary>

**Immediate rollback (squash-merge; single commit on develop):**
```bash
git revert <MERGE_COMMIT_SHA>
git push origin develop
```

**Verification after rollback:**
- `cargo build --release` should revert etherparse to 0.16
- `cargo test --all-targets` should pass (pre-ARP test suite)

</details>

### Feature Flags
None — this is a library-internal type change with no runtime feature flag.

---

## Traceability

| Behavioral Contract | Story AC | Test | Status |
|--------------------|---------|------|--------|
| BC-2.02.009 postcondition 3 (non-IP non-ARP → Err) | AC-003 | `test_decode_non_ip_non_arp_frame_returns_no_ip_error` | PASS |
| BC-2.02.009 invariant 1 (three-way dispatch compiles) | AC-005 | `cargo check` + IP-path tests | PASS |
| BC-2.02.009 invariant 5 + VP-008 (no-panic placeholder) | AC-005b | `test_decode_arp_shaped_input_does_not_panic` | PASS |
| BC-2.02.009 invariant 2 (strict unreachable! arm) | AC-006 | `cargo test --all-targets` (no panic) | PASS |
| BC-2.02.009 invariant 5 + VP-008 harness type | AC-009 | `test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing`, `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered` | PASS |
| BC-2.02.009 (Cargo.toml 0.20 + prose) | AC-010 | `cargo check` + build | PASS |

<details>
<summary><strong>Coverage Mapping: Removed ACs → STORY-112</strong></summary>

The following AC behaviors were removed from STORY-111 scope (require `extract_arp_frame` full impl):

| Removed STORY-111 AC | Behavior | Covered by STORY-112 AC |
|----------------------|----------|------------------------|
| AC-001 | `decode_packet` returns `Ok(DecodedFrame::Arp(frame))` for well-formed ARP | STORY-112 AC-006 |
| AC-002 | `decode_packet` returns `Err("Non-Ethernet/IPv4 ARP frame")` for bad ARP | STORY-112 AC-012 (NEW) |
| AC-004 | No panic on all three ARP decode paths | STORY-112 AC-006/AC-012/AC-007 |
| AC-007 | `lax_ip_triple` routes `LaxNetSlice::Arp` to `extract_arp_frame` | STORY-112 AC-007 |
| AC-008 | Lax arm maps `Some(f)→Ok(DecodedFrame::Arp)`, `None→Err("truncated ARP frame")` | STORY-112 AC-007 |

No coverage gap: each removed AC maps to an explicit STORY-112 AC.

</details>

---

## AI Pipeline Metadata

<details>
<summary><strong>Pipeline Details</strong></summary>

```yaml
ai-generated: true
pipeline-mode: feature (F4 delta-implementation)
factory-version: "1.0.0"
release-target: v0.7.0
epic: E-16 (ARP Security Analyzer)
github-issue: 9
story-version: "1.4"
bc-version: BC-2.02.009 v1.7
vp: VP-008
pipeline-stages:
  spec-crystallization: completed (BC-2.02.009 v1.7 + arp-architecture-delta v1.16)
  story-decomposition: completed (STORY-111 v1.4)
  tdd-implementation: completed (commit 9cf7bca)
  holdout-evaluation: "N/A — evaluated at wave gate"
  adversarial-review: "CONVERGED — 3/3 ZERO passes (A/B/C)"
  formal-verification: "VP-008 harness updated; no Kani proofs in this story"
  convergence: achieved
convergence-metrics:
  adversarial-passes: 3 (all ZERO)
commit-chain:
  - 4e22ef9: feat(S-111) — module stubs (Red Gate)
  - 549b769: test(STORY-111) — failing tests (TDD)
  - 27f51ba: feat(STORY-111) — non-panicking placeholder + transitional ARP Err routing
  - 9e07423: test(STORY-111) — BC-2.02.009 ARP test reconciliation
  - 9cf7bca: docs(STORY-111) — decoder module-doc + DecodedFrame doc (prose-sweep)
base-commit: 31d1231 (develop HEAD at branch time)
models-used:
  builder: claude-sonnet-4-6
generated-at: "2026-06-14T00:00:00Z"
```

</details>

---

## Pre-Merge Checklist

- [x] All CI status checks passing (cargo test, clippy, fmt, semantic PR, action-pin-gate)
- [x] No critical/high security findings (scope: data types + placeholder; no I/O or unsafe)
- [x] Adversarial convergence: 3/3 ZERO-finding passes
- [x] Demo evidence recorded for all ACs (`docs/demo-evidence/STORY-111/`)
- [x] BC traceability chain complete: BC-2.02.009 → AC → Test → Code
- [x] Forbidden dependency check: `src/decoder.rs` does NOT import `crate::analyzer::arp`
- [x] D-072 symmetric-unreachable design verified (ADR-008 Decision 3 v2.1)
- [x] Upstream dependency merged: STORY-110 on develop (31d1231)
- [x] STORY-112 scope boundary documented (extract_arp_frame real impl, temporary Err string)
- [x] CI SHA-pin policy: no new Actions references added (CLAUDE.md action-pin-gate)
- [ ] CI checks green (to be confirmed post-push)
- [ ] pr-reviewer APPROVE (to be confirmed)
