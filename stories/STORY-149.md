---
id: STORY-149
title: "TLS Carry-Path Performance Recovery + Fragmented-Handshake Benchmark Fixture"
epic: E-11
wave: "~"
points: 5
status: draft
depends_on: []
input-hash: TBD
inputs: []
---

# STORY-149 — TLS Carry-Path Performance Recovery + Fragmented-Handshake Benchmark Fixture

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** draft
**Wave:** TBD
**Points:** 5

## Background

The STORY-144/145/146 carry-path additions (fix-tls-clienthello-frag, waves 65–66)
introduced measurable overhead on the `reassembly/tls.pcap` Criterion benchmark: the
criterion crossed the +10% threshold relative to the May-19 baseline, as measured in
the maint-2026-07-01 performance sweep.

Root-cause analysis identified two allocation hotspots in `try_parse_records`
(`src/analyzer/tls.rs`):

- **PERF-001 (HIGH):** The current implementation acquires `flows.get()` / `flows.get_mut()`
  multiple times per 0x16 (handshake) TLS record — re-hashing the `FlowKey` on each
  call. In the hot path this typically means 6–8 repeated HashMap operations against
  the same key per record.
- **PERF-002 (HIGH):** Redundant `Vec` allocations occur per record: carry bytes are
  re-allocated rather than re-used via a local swap pattern.

Secondary allocation smells (LOW severity) also identified:
- **PERF-003:** Hex-encoding in cipher-suite logging allocates a `String` per record.
- **PERF-004:** Cipher-suite `Vec<u16>` is cloned in the `summarize()` path.
- **PERF-005:** Intermediate `String` allocation in JA3 assembly is not pooled across
  records.

Additionally, the existing `tls.pcap` benchmark fixture does not exercise the
multi-record carry-drain path — it delivers complete single-record TLS handshakes. The
carry-drain loop introduced by STORY-144/145 is therefore never exercised by the
existing criterion suite, so regression detection for that path is blind.

## Goal

1. Restructure `try_parse_records` to acquire a single `flows.get_mut()` borrow per
   invocation and work off it throughout, using a local carry-buffer swap (`std::mem::replace`
   or equivalent) to release the borrow before the `&mut self` dispatch call. This
   eliminates repeated FlowKey re-hashing (PERF-001) and the per-record carry Vec
   allocation (PERF-002).
2. Add a Criterion benchmark fixture (at `benches/tls_fragmented.rs` or as a new bench
   group in the existing TLS bench file) that delivers a genuinely fragmented multi-record
   TLS handshake — one that exercises the carry-drain loop. Establish this as the
   regression baseline for future carry-path changes.
3. Verify the combined fix recovers ~5% on the `reassembly/tls.pcap` criterion,
   bringing it back under the WARNING (+10%) threshold relative to the May-19 baseline.
4. Optionally address PERF-003/004/005 if they fall within scope without expanding
   story points (secondary ACs).

## Acceptance Criteria

AC-149-001: `try_parse_records` in `src/analyzer/tls.rs` acquires at most one
  `flows.get_mut()` call per invocation (verified by code inspection; enforced by a
  comment asserting the single-borrow invariant). The carry-buffer swap uses
  `std::mem::replace` or a local Vec swap rather than a fresh allocation per record.

AC-149-002: A Criterion benchmark fixture exists at `benches/tls_fragmented.rs` (or
  as a new bench group in an existing TLS bench file) that delivers a synthetic TLS
  handshake message spanning at least 3 TLS records — i.e., the carry-drain loop
  executes at least twice per synthetic handshake. The fixture is deterministic and
  repeatable.

AC-149-003: Running `cargo bench --bench tls` (or equivalent) against a comparable
  baseline shows the `reassembly/tls.pcap` criterion within +5% of the May-19 baseline
  (i.e., the regression is substantially recovered, within the WARNING threshold).

AC-149-004 (optional): At least one of PERF-003/004/005 is resolved — hex-encoding
  alloc moved out of the hot path, cipher-suite clone replaced with a reference, or
  JA3 assembly buffer reused across calls.

AC-149-005: `cargo test --all-targets` passes without regression; existing VP-039 and
  VP-040 harnesses remain green. `cargo clippy --all-targets -- -D warnings` passes.

## Notes

- Source findings: PERF-001/002 (HIGH) + PERF-003/004/005 (LOW), maint-2026-07-01.
- Primary module: `src/analyzer/tls.rs` (`try_parse_records`).
- The borrow-constraint root cause: `flows.get_mut()` returns a `&mut TlsFlowState`
  borrow on `self.flows`. This borrow conflicts with the `&mut self` call to downstream
  dispatch, requiring a carry swap to drop the borrow before dispatch. STORY-144
  introduced the carry struct but used a naive acquire-per-operation pattern; this story
  consolidates it to a single-borrow pattern.
- Wave assignment is TBD — schedule at v0.12.0 planning.
- Relationship to STORY-150: this story (149) fixes the performance regression first.
  STORY-150 then DRY-refactors the carry-drain duplication. Doing the perf fix first
  avoids attributing any residual regression to the structural refactor.
- Precedent: STORY-147 (mutation-testing defaults, E-11, wave TBD), STORY-143
  (changelog hardening, E-11, wave TBD) — same E-11 pattern of a cycle follow-up
  encoding a lesson into project tooling or infrastructure.
- S-7.02 disposition: this story's creation at draft status captures PERF-001/002 for
  v0.12.0 planning and closes the maint-2026-07-01 perf-sweep open item.
