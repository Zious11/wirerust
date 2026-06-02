# VP-008 Fuzz Campaign Evidence — `decode_packet` Never Panics on Arbitrary Input

- **VP:** VP-008 (`.factory/specs/verification-properties/vp-008-decode-packet-no-panic.md`)
- **Source contracts:** BC-2.02.007 (reject malformed input, no panic), BC-2.02.008 (reject unsupported link types), BC-2.02.009 (surface no-IP-layer error)
- **Proof method:** Fuzzing (cargo-fuzz / libFuzzer) — the method mandated by the VP
- **Target:** `fuzz/fuzz_targets/fuzz_decode_packet.rs`
- **Date executed:** 2026-06-01
- **Executor:** formal-verifier (Phase 6 formal hardening)
- **Result:** **PASS — ZERO crashes, ZERO panics, ZERO OOM/timeout/leak artifacts** over the full ≥5-minute run.

## Toolchain

| Item | Value |
|------|-------|
| cargo-fuzz | 0.13.1 |
| Rust toolchain | `nightly-2026-05-21` — `rustc 1.97.0-nightly (b954122bb 2026-05-20)` |
| CI-pinned nightly | `nightly-2026-05-21` |
| **Match to CI pin** | **YES — ran on the exact CI-pinned nightly-2026-05-21.** No divergence. |
| Host | aarch64-apple-darwin (Darwin 25.5.0) |
| Source tree | develop @ `48b61e5` (latest), worktree branch `verify/fuzz-vp008` |
| Build profile | release (optimized + debuginfo), ASan/libFuzzer instrumentation |

The locally-default nightly was `1.97.0-nightly (f964de49b 2026-05-07)`, which does NOT match the
CI pin. To eliminate toolchain divergence, the CI-pinned `nightly-2026-05-21` was installed via
rustup and used for both the build and the run. No divergence to document.

## Corpus Seeding

Seeded per the VP §Corpus Seeding section. The fuzz target consumes a raw **link-layer frame**
(`&[u8]`), not a full pcap file, so frames were extracted from the classic-pcap fixtures with a
stdlib-only parser (`fuzz/seed_corpus.py`) rather than fed whole files.

| Seed source | Count / note |
|-------------|--------------|
| Real link-layer frames extracted from `tests/fixtures/*.pcap`, `*.cap`, `*.trace` (Ethernet, Raw IPv4, Raw IP link types; capped ≤200 frames/file) | 1026 real frames sampled |
| Truncations of a representative sample of real frames at header boundaries (1, 2, 4, 8, 13/14/15 = ethernet, 16 = SLL, 19/20/21 = IPv4 min, 33/34, 39/40/41 = IPv6, 53/54, and one-short-of-full) | included |
| Synthetic adversarial/malformed frames: empty `&[]`, 1-byte, truncated ethernet (1–13 B), truncated IPv4/IPv6 headers, IPv4 with oversized total-length (0xffff) on tiny payload, IPv4 with huge-IHL options claim, IPv6 bogus next-header chain + oversized payload length, TCP with huge data-offset, truncated/full Linux SLL cooked headers, unknown ethertype, all-0xff jumbo (64 B and 1500 B), single & double VLAN-tagged then truncated | included |
| **Total unique seeds written (content-hash dedup)** | **1813** |
| Seeds libFuzzer loaded (after its own dedup) | 1812 |
| Seed corpus on-disk size | ~7.1 MB (401,114 B of actual loaded frame bytes) |

Note: the `fuzz/corpus/` directory is gitignored (`fuzz/.gitignore` lists `corpus/`,
`artifacts/`, `coverage/`, `target/`), per repo convention. Seeds are therefore kept locally
only and not committed. The seeding script `fuzz/seed_corpus.py` reproduces the corpus
deterministically from `tests/fixtures/`.

## Campaign Statistics

| Metric | Value |
|--------|-------|
| Command | `cargo +nightly-2026-05-21 fuzz run fuzz_decode_packet -- -max_total_time=300 -print_final_stats=1` |
| Wall-clock | 301 seconds (≥ the 5-minute no-crash bar) |
| Total executions | 21,775,342 |
| Average exec/s | 72,343 |
| Final coverage | 1049 edges (`cov`), 1743 features (`ft`) — coverage plateaued (no new edges for the final ~minutes) |
| Instrumented counters | 114,742 inline 8-bit counters / 114,742 PCs in the module |
| Minimized corpus | 450 units / 33 KB |
| New units added during run | 2059 |
| Peak RSS | 638 MB |
| Slowest unit | 0 s (sub-millisecond) |
| **Crashes / panics** | **0** |
| **OOM / timeout / leak artifacts** | **0** (`fuzz/artifacts/fuzz_decode_packet/` empty; no `crash-*`/`oom-*`/`timeout-*`/`leak-*` anywhere) |
| Process exit code | 0 |

## Interpretation

Over 21.7 million executions across all five whitelisted `DataLink` variants (ETHERNET, RAW,
IPV4, IPV6, LINUX_SLL) AND the three unsupported variants (IEEE802_11, NULL, LOOP), with a
representative seed corpus of real + truncated + adversarial frames, `decode_packet` never
panicked. Every input resolved to `Ok(ParsedPacket)` or `Err(anyhow::Error)`. This is empirical
evidence that BC-2.02.007/008/009's no-panic postcondition holds at the parser entry point and
the unsupported-link-type rejection arm.

**VP-008 fuzz-evidence bar (no crash after ≥5 min): SATISFIED.**

Per the VP's own Feasibility note, 24h+ continuous fuzzing is the eventual recommendation and the
`pcap_source.rs` secondary target remains an unowned/undelivered obligation; neither is in scope
for this Phase-6 5-minute gate. This run satisfies the spec's stated no-crash bar; coverage had
plateaued well before the timeout.

## Reproduction

```bash
# from a checkout of develop @ 48b61e5 (or worktree verify/fuzz-vp008)
python3 fuzz/seed_corpus.py tests/fixtures          # rebuild local seed corpus (gitignored)
cargo +nightly-2026-05-21 fuzz run fuzz_decode_packet -- -max_total_time=300 -print_final_stats=1
```
