---
artifact: verification-property
vp_id: VP-028
title: "pcapng Reader No-Panic (Full Path Fuzz)"
status: verified
phase: P1
tool: cargo-fuzz
subsystem: SS-01
module: "reader.rs"
producer: architect
timestamp: 2026-06-19T00:00:00Z
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-009-pcapng-reader-design.md
feature_cycle: feature-pcapng-reader
source_bc: BC-2.01.017
bcs:
  - BC-2.01.017
verification_lock: true
verified_at_commit: "1ca30a3"
verified_prs: "#293, #294"
---

# VP-028: pcapng Reader No-Panic (Full Path Fuzz)

## Property Statement

The effectful entry point `from_pcap_reader<R: Read>` in `src/reader.rs` **never panics**
on arbitrary byte inputs. This is the full-path no-panic property: the entire pcapng
read-and-parse pipeline from raw bytes through block-walk, SHB/IDB/EPB/SPB dispatch,
and field extraction must remain panic-free regardless of what a fuzzer delivers.

This property is complementary to the pure-core Kani proofs (VP-025/VP-026/VP-027):
VP-028 covers the effectful integration path and finds panics that can only manifest
when multiple layers interact with attacker-controlled bytes.

## Verified BCs

| BC-ID | Description | Sub-property |
|-------|-------------|-------------|
| BC-2.01.017 | pcapng reader no-panic: from_pcap_reader survives arbitrary byte inputs without panicking | Full-path fuzz |

## Tool Rationale

cargo-fuzz (libFuzzer) is the correct tool here because:
- `from_pcap_reader<R: Read>` performs I/O, allocates, and uses a generic `Read` impl
  — outside Kani's pure-core model.
- Fuzzing exercises the full integration path including length-driven dispatch, error
  propagation, and iterator state, finding panics across code paths that Kani cannot
  enumerate symbolically in a bounded budget.
- VP-008 (`decode_packet` no-panic) establishes the precedent for using cargo-fuzz
  on the effectful I/O-carrying entry points.

## Fuzz Target

```
fuzz/fuzz_targets/fuzz_pcapng_reader.rs
```

Fuzz run statistics at F6 lock:
- Executions: **2,340,242**
- Duration: **121 seconds**
- Crashes: **0**
- Corpus: pcap and pcapng fixtures from the test suite

## Feasibility Assessment

**Assessment: FEASIBLE (completed — 2.34M execs, 0 crashes at F6 lock).**

libFuzzer is well-suited to binary file format parsers. The fuzz target wraps the full
pcapng reader path with a cursor over the fuzzer-supplied bytes; all panics are caught
as crashes by libFuzzer. 2.34M executions at 0 crashes provides high confidence in
panic-freedom on the exercised corpus.

## Lifecycle

| Phase | Action | Status |
|-------|--------|--------|
| F2 (spec evolution) | VP-028 designed as F6 hardening deliverable, added to VP-INDEX (ADR-009 rev 4) | draft |
| F4 (TDD implementation) | Fuzz target `fuzz_pcapng_reader.rs` authored | draft |
| F6 (formal hardening) | 2,340,242 execs / 121s / 0 crashes confirmed; verification_lock set | draft → verified |

Lock: `status: verified`, `verification_lock: true` set by state-manager after F6 confirmation
@ develop 1ca30a3 (PRs #293 + #294).
