---
document_type: performance-review
cycle: feature-pcapng-reader
phase: F2
author: performance-engineer
date: 2026-06-19
status: complete
---

# F2 Performance Review: pcapng Reader Specification

**Scope:** Specification-level performance assessment of the pcapng reader feature
(ADR-009, BC-2.01.009..018, STORY-123..127). No implementation exists yet; this
review identifies gaps in the spec's treatment of large-capture performance and
recommends concrete NFR/AC additions before F3 implementation begins.

**Stressor reference:** `maccdc2012_00000.pcap` (~1 GB, mixed HTTP/TLS/DNS/SMB,
classic pcap today — its pcapng equivalent is the canonical ~1 GB scale target
once pcapng support lands).

---

## Executive Summary

| Severity | Count | Must address before F3 |
|----------|-------|------------------------|
| CRITICAL | 0 | — |
| HIGH     | 3 | F-PERF-001, F-PERF-002, F-PERF-003 |
| MEDIUM   | 2 | F-PERF-004, F-PERF-005 |
| LOW      | 1 | F-PERF-006 |

**Must-address before F3 (implementation):** F-PERF-001, F-PERF-002, F-PERF-003.
These three findings concern the memory model, throughput parity, and the absence
of any regression benchmark gate. Without addressing them the implementer has no
performance contract to code to and no gate to catch regressions before merge.

---

## Finding F-PERF-001: Streaming vs. Buffering — Spec Silent on Memory Model

**Severity:** HIGH
**Spec covers it:** N (silent)

### Observation

The existing classic-pcap path (src/reader.rs) is explicitly eager: it collects
all packets into `Vec<RawPacket>` before returning. NFR-PERF-002 (OPEN-DEBT,
NFR-VIO-001) documents this as a known limitation: "For very large captures the
all-in-memory model is a known limitation." The README claim of "multi-GB captures"
is already flagged as overstating capability under RAM constraint.

The pcapng spec (ADR-009, BC-2.01.009..018) says nothing about whether the pcapng
path streams packets or buffers them. This is not merely a style gap: `pcap-file`
2.0.0's `PcapNgReader`/`PcapNgParser` API exposes a block iterator
(`next_block()`), which enables streaming. The implementer could reasonably:
(a) iterate and immediately collect into `Vec<RawPacket>` (matching the classic
path's eager model), or
(b) stream blocks lazily and hand each `RawPacket` to the analyzer pipeline
without holding the full file in memory.

These have radically different memory profiles on a 1 GB capture.

### Root Cause

The spec inherits the classic path's all-in-memory model implicitly via
`PcapSource { packets: Vec<RawPacket>, datalink: DataLink }`, but this is not
stated as a deliberate constraint for the pcapng path. BC-2.01.012 (EPB parsing)
says "The resulting `RawPacket` is appended to the `PcapSource.packets` vector in
EPB encounter order" — which commits to the eager model in the BC's postconditions
without naming the memory consequence.

### Recommendation

The spec should explicitly declare which model the pcapng path uses, so the
implementer is not making an unguided architectural choice.

**If the decision is eager (match classic path):** Add to ADR-009 Consequences:
"The pcapng path uses the same all-in-memory `Vec<RawPacket>` model as the
classic-pcap path. Peak RSS for a pcapng capture of size N is approximately
N * 1.5 (Vec overhead plus raw packet copies). NFR-VIO-001 (streaming-refactor
debt) applies equally to the pcapng path."

**If the decision is streaming (deferred to a future cycle):** Add explicit note
to ADR-009 that streaming is deferred and that the current cycle deliberately
matches the classic eager model, with the same known limitation.

Either way, the spec must be explicit. The current silence allows a divergent
implementation that either (a) accidentally streams when the pipeline expects all
packets up-front, or (b) misses an obvious improvement opportunity.

**Recommended NFR addition:**

```
NFR-PERF-005 | Performance | pcapng reader uses the same all-in-memory
Vec<RawPacket> model as the classic-pcap path (PcapSource.packets populated
before analysis); streaming refactor is deferred to a future cycle per
NFR-VIO-001 | Peak RSS <= pcapng_file_size * 1.5 | Load test with 1 GB
pcapng fixture (maccdc2012 pcapng equivalent); measure RSS | P1 | NFR-VIO-001
(same streaming-debt as classic path) | OPEN
```

---

## Finding F-PERF-002: Throughput Parity — No NFR Inherits from Classic Path

**Severity:** HIGH
**Spec covers it:** N (no throughput NFR exists for either classic or pcapng path)

### Observation

NFR-PERF-002 documents the all-in-memory model but gives only a memory bound
(RSS <= file_size * 1.5). There is no existing throughput NFR for classic pcap:
no packets/sec target, no MB/sec target, no wall-clock budget for a file of a
given size. The NFR catalog has NFR-PERF-001 (zero-copy decode), NFR-PERF-003
(O(1) dispatch), NFR-PERF-004 (SIMD overlap detection) — but nothing that measures
end-to-end read throughput through the ingestion path.

The pcapng format carries per-block overhead that the classic path does not:
- Per-EPB interface lookup (interface_id → IDB table, O(1) amortized but cache-hot)
- Per-EPB timestamp conversion via `pcapng_timestamp_to_secs_usecs()` (integer
  arithmetic: divisions by ticks_per_sec, one branch on bit 7 of if_tsresol)
- Options parsing per IDB (one-time at IDB encounter; amortized cost negligible)
- Block-type dispatch per block in the stream (SHB/IDB/EPB/SPB/unknown arm; adds
  one match arm per block)

None of these are expensive in isolation. However, without a throughput baseline
and a regression budget, there is no way to detect if the block-type dispatch or
the timestamp conversion function introduces a measurable regression on a
multi-million-packet capture.

### Root Cause

The NFR catalog has never had a throughput NFR for reader ingestion. The classic
path was never benchmarked end-to-end (no `cargo bench` target for reader
throughput). Adding pcapng without a throughput target means there is no
performance acceptance criterion to test against.

### Recommendation

**Recommended NFR addition (reader throughput, shared classic + pcapng):**

```
NFR-PERF-006 | Performance | PcapSource::from_file ingestion throughput (classic
pcap and pcapng paths) meets minimum read rate on single-threaded execution:
>= 500 MB/s on a modern development machine (comparable to raw disk/mmap
read speeds; this is the floor, not a stretch goal) | Measured as
(pcap_file_size_bytes / wall_clock_seconds) using criterion bench on a
64 MB synthetic pcap/pcapng fixture with 1500-byte packets and no analyzer
involvement | Benchmark: bench/reader_throughput.rs; criterion with 10 samples
minimum | P1 | — | OPEN
```

Rationale for 500 MB/s floor: a 1 GB capture should complete ingestion in <= 2
seconds. Modern NVMe delivers 3-7 GB/s; the bottleneck is parsing, not I/O.
`tcpdump -r` reads classic pcap at roughly 600-900 MB/s on modern hardware.
500 MB/s is a deliberately conservative floor that a pure-Rust iterator over
already-opened bytes should comfortably exceed; any result below it signals that
something unexpected (allocation storm, per-byte copy, O(n^2) behavior) has been
introduced.

The benchmark fixture should be a small synthetic (64 MB, ~45,000 packets at 1500
bytes each) generated by the test harness — it must exist in both classic and
pcapng forms so the two paths can be benchmarked identically and compared.

---

## Finding F-PERF-003: No Benchmark Regression Gate Mandated for pcapng Path

**Severity:** HIGH
**Spec covers it:** N

### Observation

The project already has a criterion bench harness (NFR-PERF-004, Cargo.toml:50-52,
`[[bench]] name = "pipeline"`). However, NFR-PERF-004 exercises the overlap
detection hot path, not the reader ingestion path. No story (STORY-123 through
STORY-127 as planned) mandates adding a criterion benchmark for the pcapng reader,
and no F6 hardening item requires running the benchmark and asserting no regression
against a baseline.

Without a mandated benchmark:
1. The pcapng implementation ships with no measured throughput.
2. Regressions on the classic-pcap path (which is structurally adjacent to the
   new probe branch) cannot be detected.
3. The timestamp conversion helper (`pcapng_timestamp_to_secs_usecs`) is provable
   by Kani for correctness but never benchmarked for speed.

### Recommendation

**Recommended AC addition to the story that closes reader ingestion (STORY-125 or a
new STORY-126 bench story):**

```
AC-BENCH-001: A criterion benchmark bench/reader_throughput.rs MUST be added
that measures PcapSource::from_file throughput (MB/s) on:
  (a) a synthetic classic-pcap fixture (64 MB, ~45,000 x 1500-byte packets)
  (b) the same fixture in pcapng/EPB format
Baseline is captured by the implementer (cargo bench --bench reader_throughput
> .factory/cycles/feature-pcapng-reader/hardening/performance-baseline.md) before
any pcapng code is merged to develop. The pcapng path MUST NOT regress the classic
path by more than 10% on the same packet count and capture size.
```

**Recommended NFR addition (regression gate):**

```
NFR-PERF-007 | Performance | The pcapng reader ingestion path MUST NOT regress
classic-pcap reader throughput by more than 10% (measured via criterion on
identical synthetic fixture in both formats) | pcapng throughput >= 0.90 *
classic_throughput on same packet count / file size | cargo bench --bench
reader_throughput; compare criterion means | P1 | — | OPEN
```

This is the concrete budget that F6 hardening can enforce. The 10% budget accounts
for pcapng's per-EPB overhead (interface lookup, timestamp conversion branch) while
flagging anything more expensive as a regression worth investigating.

---

## Finding F-PERF-004: Per-Interface Timestamp Resolution Lookup — Hot Path Not Caching

**Severity:** MEDIUM
**Spec covers it:** Partial

### Observation

BC-2.01.014 correctly specifies the `pcapng_timestamp_to_secs_usecs(ts_high, ts_low,
if_tsresol)` pure-core helper and notes that "when `if_tsresol` is absent, callers
MUST pass `6`." BC-2.01.011 specifies that `if_tsresol` is extracted per IDB and
stored in "an interface table keyed by the interface's 0-based index."

The spec implicitly requires a per-EPB interface lookup (EPB.interface_id →
interface table entry → if_tsresol value) on every packet. This is the hot path
for a multi-million-packet file.

The spec does NOT require that this lookup be cached in a way that avoids repeated
HashMap traversal or indirect memory access. In most single-IDB captures (the
common case, as documented), the interface table has exactly one entry and the
lookup is trivially fast. However, for the spec to be silent on the data structure
of the interface table leaves the implementer free to use a HashMap (with hashing
overhead per lookup) or a Vec (O(1) index, cache-friendly).

The spec also does not acknowledge that in the common case (if_tsresol absent →
default = 6, single-IDB), the conversion simplifies to:
  ts_sec = ticks / 1_000_000
  ts_usecs = ticks % 1_000_000
This is a fast-path that avoids the base-10/base-2 branch entirely and could be
hoisted to a compile-time-constant path if the implementer knows the resolution
at IDB parse time.

### Recommendation

Add an implementation note (not a hard AC) to BC-2.01.011 or ADR-009:

"The interface table SHOULD be implemented as `Vec<InterfaceInfo>` (indexed by
0-based interface_id, O(1) random access, cache-contiguous) rather than a HashMap.
In files with a single IDB (the common case), the Vec has exactly one entry and
if_tsresol is read once at IDB parse time. The common-case fast path (if_tsresol
absent → microseconds default) SHOULD be recognized at IDB parse time and stored
as a flag so the EPB hot-path avoids the base-10/base-2 bit-7 branch on every
packet."

This is a MEDIUM finding because the spec's current silence will likely lead to
a correct but suboptimal implementation. It does not block correctness, but it
is worth capturing before implementation starts.

---

## Finding F-PERF-005: Memory Bound for 1 GB Capture — No AC Asserting O(1) Memory in Packet Count

**Severity:** MEDIUM
**Spec covers it:** N

### Observation

NFR-PERF-002 asserts "RAM usage <= pcap_file_size * ~1.5 (Vec header overhead)" for
the classic path. No equivalent is stated for pcapng. On a 1 GB pcapng file the same
bound should hold: the file is read once, packets are pushed into the Vec, and no
additional per-block retained data grows with packet count (options are discarded
after IDB parse, the interface table is bounded by the number of interfaces, not
by packets).

However, there is one pcapng-specific risk the spec does not address: pcap-file
2.0.0's block iterator may internally allocate per-block data (e.g., byte Vecs for
options, or intermediate Block enum variants with owned data) that is not immediately
dropped. If the internal `Block` enum is heap-allocated and kept alive longer than
the inner-loop iteration, peak RSS could exceed the classic-path bound.

The spec does not contain any AC or NFR asserting that peak RSS for a pcapng capture
is bounded by O(1) in packet count (beyond the Vec of RawPackets).

### Recommendation

Add to NFR-PERF-005 (recommended above, F-PERF-001) or as a standalone AC:

"Peak RSS for a pcapng capture of file size N MUST NOT exceed N * 2.0 at any
point during ingestion. This allows for the `Vec<RawPacket>` plus one additional
copy of the largest block's bytes as working memory. Verified by: `cargo bench
--bench reader_throughput` with `/usr/bin/time -v` or equivalent RSS measurement
on a 64 MB fixture; extrapolated to 1 GB by linearity (the bound is linear in
file size, not packet count)."

The 2.0x bound (versus 1.5x for classic pcap) allows headroom for pcap-file 2.0.0's
internal block representation before the bytes are converted to `RawPacket`. If the
implementation can demonstrate tighter bounds in practice, the NFR can be tightened.

---

## Finding F-PERF-006: No Large-Fixture Throughput Test in E2E Corpus

**Severity:** LOW
**Spec covers it:** Partial

### Observation

The E2E corpus (E2E-PCAPS.md) currently has no pcapng file in the large/stressor
category. `maccdc2012_00000.pcap` (~1 GB) is the classic-pcap stressor and is
listed as "link-only" (optional, not auto-fetched). When pcapng support lands there
will be no pcapng equivalent in the corpus for manual E2E throughput validation.

The research file (e2e-pcap-candidates.md) notes that "almost every modern
TLS-heavy public capture is pcapng" — meaning the pcapng path will eventually be
the primary path for large captures. The spec does not call for adding a large
pcapng fixture to the E2E corpus as part of this feature cycle.

### Recommendation

Add to the feature cycle checklist (cycle-manifest.md or as an AC on the
verification story):

"Before F7 convergence, identify and index at least one large (>= 100 MB) pcapng
fixture in E2E-PCAPS.md for throughput and robustness validation of the pcapng
path. Candidates: Wireshark `dump.pcapng` (if large enough), Netresec MACCDC 2012
pcapng equivalent, or a locally-generated 1 GB synthetic pcapng. This fixture is
used for manual E2E validation only (gitignored); it does not need to be
auto-fetched."

This is LOW severity because it does not block F3 implementation, but it should be
planned before F7 so that convergence can demonstrate scale behavior.

---

## Question Answers

### 1. Streaming vs. Buffering

The pcapng path is NOT explicitly required to stream. BC-2.01.012 Post-Condition 4
("The resulting `RawPacket` is appended to the `PcapSource.packets` vector in EPB
encounter order") implicitly commits to the same eager all-in-memory model as the
classic path. The spec is silent on whether this is a deliberate constraint or an
oversight. `pcap-file` 2.0.0's `PcapNgParser` supports streaming (block-by-block
`next_block()` iteration), so the library does NOT force buffering the whole section.
The implementer must be told explicitly: collect into Vec<RawPacket> now, streaming
refactor is deferred (matching classic path, NFR-VIO-001). **Spec coverage: Partial
(BC commits to Vec but does not name the memory consequence or defer streaming
explicitly).**

### 2. Throughput Parity

There is NO existing throughput NFR for classic pcap. NFR-PERF-002 is a memory bound
only. The pcapng path has measurable per-EPB overhead vs. classic (interface lookup +
timestamp conversion branch + block-type dispatch) but none of it is spec-bounded.
The pcapng timestamp conversion helper (`pcapng_timestamp_to_secs_usecs`) is pure
integer arithmetic (division, modulo, one branch). On a nanosecond-resolution file
(if_tsresol=9) it adds one extra division-by-1000 step vs. the classic path's direct
`ts_frac / 1_000` at reader.rs:73. This is negligible per packet but unverified at
scale. **Spec coverage: N (no throughput NFR exists for either path).**

### 3. Memory Bound

With a 1 GB capture, peak RSS is bounded implicitly by NFR-PERF-002's "pcap_file_size
* 1.5" rule, but this is not explicitly extended to the pcapng path and does not
account for pcap-file 2.0.0's internal block allocations. There is no AC asserting
O(1) memory in packet count for the pcapng path. **Spec coverage: N.**

### 4. Per-Interface Timestamp Resolution — Hot Path Cost

BC-2.01.012/011 implies a per-EPB interface lookup (EPB.interface_id → Vec/HashMap
entry → if_tsresol u8). The spec does NOT require caching the resolution or
recognizing the common-case fast path. On a single-IDB file with default
microsecond resolution, the hot path is: lookup Vec[0] (cache-hot), call
`pcapng_timestamp_to_secs_usecs(ts_high, ts_low, 6)`, which for the default case
simplifies to `ticks / 1_000_000` and `ticks % 1_000_000`. This is fast. The
spec should recommend (not require) Vec over HashMap for the interface table and
note the common-case fast path. **Spec coverage: Partial (semantics correct;
performance implementation guidance absent).**

### 5. Benchmark/Regression Gate

No story (STORY-123..127) mandates adding a criterion benchmark for the pcapng
reader path or enforcing a regression budget in F6. The existing pipeline bench
(NFR-PERF-004) does not exercise reader ingestion throughput. A criterion bench
and a 10% regression budget relative to classic pcap should be mandated before
implementation (F3) so that F6 hardening has a concrete gate to enforce. **Spec
coverage: N.**

---

## Recommended NFR Additions

The following NFRs do not currently exist in the NFR catalog. They should be added
as F2 spec evolution items before F3 stories are decomposed.

### NFR-PERF-005 (new): pcapng Reader Memory Model

| Field | Value |
|-------|-------|
| ID | NFR-PERF-005 |
| Category | Performance |
| Requirement | pcapng reader uses the same all-in-memory Vec<RawPacket> model as the classic-pcap path; streaming refactor is explicitly deferred per NFR-VIO-001. The pcapng path MUST NOT introduce additional per-packet retained heap allocations beyond the RawPacket.data Vec<u8>. |
| Target | Peak RSS <= pcapng_file_size_bytes * 2.0 during ingestion (1.5x for the Vec<RawPacket> + 0.5x headroom for pcap-file 2.0.0 internal block representation) |
| Validation Method | cargo bench --bench reader_throughput with /usr/bin/time -v or equivalent RSS measurement on a 64 MB synthetic pcapng fixture; assert peak_rss <= 128 MB |
| Priority | P1 |
| Risk Source | NFR-VIO-001 (streaming-refactor debt applies equally to pcapng path) |
| Status | OPEN |

### NFR-PERF-006 (new): Reader Ingestion Throughput Floor

| Field | Value |
|-------|-------|
| ID | NFR-PERF-006 |
| Category | Performance |
| Requirement | PcapSource::from_file ingestion throughput (both classic-pcap and pcapng paths) >= 500 MB/s on a modern development machine, measured on a 64 MB synthetic fixture with 1500-byte packets, no analyzer involvement |
| Target | >= 500 MB/s (floor; not a stretch goal — classic pcap should achieve 600-900 MB/s; 500 MB/s flags catastrophic regression) |
| Validation Method | criterion bench (bench/reader_throughput.rs); 10 samples minimum; report mean and p95 |
| Priority | P1 |
| Risk Source | — (new; no prior NFR) |
| Status | OPEN |

### NFR-PERF-007 (new): pcapng vs. Classic-pcap Throughput Parity

| Field | Value |
|-------|-------|
| ID | NFR-PERF-007 |
| Category | Performance |
| Requirement | The pcapng reader ingestion path MUST NOT regress classic-pcap reader throughput by more than 10% on an identical synthetic fixture (same packet count, same frame size, different outer format) |
| Target | pcapng_throughput_MBs >= 0.90 * classic_throughput_MBs on the same 64 MB / ~45,000-packet synthetic fixture |
| Validation Method | cargo bench --bench reader_throughput; compare criterion means for classic and pcapng bench targets; assert ratio >= 0.90 |
| Priority | P1 |
| Risk Source | Per-EPB overhead: interface lookup, timestamp conversion branch, block-type dispatch arm |
| Status | OPEN |

---

## Spec Baseline for These Recommendations

The existing NFR that grounds all three new NFRs is NFR-PERF-002 (OPEN-DEBT):

> "NFR-PERF-002 | Performance | Single eager pass: full pcap loaded into Vec<RawPacket>
> before analysis begins; NOT streaming | RAM usage <= pcap_file_size * ~1.5 (Vec header
> overhead) | Load test with 1 GB pcap; measure RSS | P1 | NFR-VIO-001 | OPEN-DEBT"

NFR-PERF-005 extends this bound to the pcapng path.
NFR-PERF-006/007 add the throughput dimension that NFR-PERF-002 never captured.

The timestamp conversion helper (BC-2.01.014) is Kani-provable for correctness
(no panic, ts_usecs in [0, 999999], division by zero impossible) but correctness
proofs do not bound execution time. NFR-PERF-006/007 fill that gap.

---

## Must-Address Before F3

The following three findings require spec additions before implementation stories
are handed to the implementer:

| Finding | Required Action | Owner |
|---------|----------------|-------|
| F-PERF-001 | ADR-009 Consequences: explicitly state pcapng path uses all-in-memory model; add NFR-PERF-005 | Product Owner / Architect |
| F-PERF-002 | Add NFR-PERF-006 (throughput floor) to NFR catalog | Product Owner |
| F-PERF-003 | Add NFR-PERF-007 (regression budget) to NFR catalog; add AC-BENCH-001 to STORY-125 or a new bench story | Product Owner / Story Writer |

Findings F-PERF-004, F-PERF-005, and F-PERF-006 can be addressed in F3 (as
implementation notes) or F6 (as hardening checks) without blocking story
decomposition.

---

## Notes on Measurement Environment

When NFR-PERF-006/007 benchmarks are executed, the following environment facts
must be recorded in the baseline document for reproducibility:

- Hardware: CPU model, core count, clock speed
- Memory: total RAM, available RAM during bench
- Storage: disk type (NVMe/SSD/HDD), read bandwidth (can be estimated via `dd`)
- Rust toolchain: stable version used
- Fixture: synthetic 64 MB pcapng (generated by test harness), sha256 of fixture

The criterion report (HTML or JSON) MUST be archived alongside the baseline.
