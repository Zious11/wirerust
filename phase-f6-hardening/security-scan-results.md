# Phase F6 — Security Scan Results (Feature #100)

**Feature:** issue-100-pcap-timestamps
**develop HEAD:** `256a490`
**Date:** 2026-06-09
**Scope:** full tree for `cargo audit` / `cargo deny`; 6 changed source files for static review

---

## Summary

| Scanner | Result | Findings (CRITICAL/HIGH) |
|---------|--------|--------------------------|
| `cargo audit` | PASS (1 known-allowed warning) | 0 |
| `cargo deny check` | PASS (advisories, bans, licenses, sources all ok) | 0 |
| semgrep | **SKIPPED — tool not installed** | n/a (manual review performed instead) |
| Manual adversarial review (delta) | PASS | 0 |

**No CRITICAL or HIGH severity findings. F6 security gate: PASS.** No `security-reviewer`
escalation required.

---

## cargo audit (full tree)

```
Loaded 1123 security advisories
Scanning Cargo.lock for vulnerabilities (193 crate dependencies)

Crate:    rand
Version:  0.8.5
Warning:  unsound (RUSTSEC-2026-0097)
Title:    Rand is unsound with a custom logger using `rand::rng()`
Dependency tree:
  rand 0.8.5 └── phf_generator └── phf_codegen └── tls-parser └── wirerust

warning: 1 allowed warning found
```

- **RUSTSEC-2026-0097** is the **known-accepted** advisory called out in the F6 brief. It is a
  transitive dependency: `rand 0.8.5` is pulled in by `phf_generator`/`phf_codegen`, which is a
  **build-time code-generation** dependency of `tls-parser`. It does not appear in the runtime
  call graph of wirerust, and wirerust does not use `rand::rng()` with a custom logger. It is
  surfaced as a **warning (allowed)**, not an error — `cargo audit` does not fail the build.
- This advisory is **unrelated to Feature #100** (the timestamp delta adds no new dependency;
  it uses `chrono`, already in the tree).
- **0 unaccepted advisories.**

## cargo deny check (full tree)

```
advisories ok, bans ok, licenses ok, sources ok
```

All four `cargo deny` gates pass. The `advisories ok` result confirms RUSTSEC-2026-0097 is
correctly accounted for in the deny configuration (otherwise this gate would fail). License
compliance, dependency bans, and source allowlisting all pass. **0 findings.**

## semgrep

`semgrep` is **not installed** in this environment (`command not found: semgrep`). Per the F6
brief, this is documented as a skip and substituted with a targeted manual adversarial review
of the 6 changed source files (below). The CI-side `action-pin-gate` (workflow SHA-pinning)
is enforced separately in CI and is out of this local F6 scope.

## Manual adversarial review of the delta

The timestamp is **attacker-controlled pcap data surfaced into `Finding` output**. Reviewed the
6 changed files (`src/reassembly/{mod,lifecycle,handler}.rs`, `src/dispatcher.rs`,
`src/analyzer/{http,tls}.rs`) against the relevant security properties:

| Check | Finding | Result |
|-------|---------|--------|
| Panic on adversarial timestamp | Only operation is `DateTime::from_timestamp(v as i64, 0)`, total over all `u32` (see kani-results.md). No `unwrap()`/`expect()` on the timestamp conversion in production code — all 21 sites bind the `Option` directly into the `Finding.timestamp` field. | SAFE |
| Integer overflow | Only arithmetic is the widening cast `u32 as i64` (cannot overflow). Release profile sets `overflow-checks = true`; no narrowing or wrapping ops on the timestamp. | SAFE |
| Internal-detail / info leakage | `Finding.timestamp` carries only the capture timestamp as ISO-8601; no memory addresses, paths, or internal state are surfaced. Provenance is explicitly framed as capture-relative (non-authoritative) per BC-2.09.007 invariant 3 / NIST SP 800-86. | SAFE |
| Unsafe code introduced | None. `grep` of the delta shows no `unsafe` blocks added. | SAFE |
| Cross-flow leakage (data isolation) | Per-flow `last_ts` stored keyed by `FlowKey` (consistent with existing per-flow state); proptest `prop_cross_flow_timestamp_isolation` confirms flow A's timestamp never appears in flow B's findings. | SAFE |
| Denial-of-service via timestamp | The timestamp adds no allocation, no loop bound, and no recursion. It is a fixed-size `u32` stored once per flow. No amplification. | SAFE |
| Secret/credential handling | Not applicable — timestamps are not secrets. | n/a |

## Verdict

**F6 security gate: PASS.** Zero CRITICAL/HIGH findings. The only advisory is the
pre-existing, known-accepted, build-time-only RUSTSEC-2026-0097, which `cargo deny` already
accommodates and which is unrelated to the timestamp delta. semgrep was unavailable and is
documented as skipped, with a manual adversarial review substituted that found no issues in the
attacker-controlled timestamp path.
