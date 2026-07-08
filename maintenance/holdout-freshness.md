---
run: maint-2026-07-08
producer: holdout-evaluator
sweep: 4
binary: target/release/wirerust (built 2026-07-08, clean cargo build --release)
hs-index-version: "2.12"
scope: HS-INDEX P0 must-pass sample — 10 recent feature scenarios (HS-123..HS-132) + 11 older sample across epics
---

# Holdout Freshness — Sweep 4 (maint-2026-07-08)

## Summary Counts

| Metric | Count |
|--------|-------|
| Scenarios run | 21 |
| PASS | 21 |
| FAIL | 0 |
| STALE | 0 |
| Skipped for missing fixture | 0 fully; 1 partial (HS-131 case A/B/D — no UDP/53 DNS fixture) |

All sampled scenarios reflect current behavior. No new staleness detected. Repairs
applied in maint-2026-07-06 FIX-C (HS-061, HS-064, HS-066, HS-075) are confirmed
still consistent with the shipped binary at develop `b642c0f`.

## Sampled Scenarios — HS-123..HS-132 (feature-protocol-coverage, E-21, v0.12.0)

| HS ID | Case coverage | Result | Evidence |
|-------|---------------|--------|----------|
| HS-123 | A/B/C/D/E/F/G — partition counts + flag gating | PASS | `protocols --json` = 30 rows; `--supported`=7; `--unsupported`=23; `--supported --unsupported` → exit 2 (clap conflict); positional arg → exit 2 |
| HS-124 | A/B/C/D/E/F/G — EtherType canonical values | PASS | GOOSE `0x88B8 (35000)`; SV `0x88BA (35002)`; PROFINET RT/DCP `0x8892 (34962)`; EtherCAT `0x88A4 (34980)`; POWERLINK `0x88AB (34987)`; ARP EtherType `—`; all L2 rows show `[L2]` |
| HS-125 | A/B (BACnet/Modbus/GOOSE JSON schema) | PASS | Modbus/TCP UDP 502 `supported:true`; BACnet/IP UDP 47808 `supported:false`; GOOSE `ethertype:35000` integer (not hex string) |
| HS-126 | A/B/C/D — TCP/102 collision footnote row-triggered | PASS | Terminal footnote naming S7comm/S7comm-plus/IEC 61850 MMS/ICCP/TASE.2 appears with `--unsupported`, absent from `--supported` |
| HS-127 | A/B — `--coverage-gaps` is opt-in, not enabled by `--all` | PASS | `analyze --all --json` has no `coverage_gaps` key; `analyze --all --coverage-gaps --json` emits it |
| HS-128 | A — empty pcap: `caveat_l2` present, `entries:[]` | PASS | Empty pcap fixture (hs-empty.pcap) with `--coverage-gaps` emits caveat_l2 + entries=[] |
| HS-129 | A — UDP/47808 → state `known-unsupported`, name `BACnet/IP` | PASS | hs-bacnet-udp47808.pcap → 1 entry {port:47808, transport:UDP, state:known-unsupported, name:BACnet/IP, count:1} |
| HS-130 | A/B/C/D — TCP/102 dispatcher counter + `collision_note` | PASS | hs-tcp102.pcap + `--all --coverage-gaps --json` → TCP/102 entry with `collision_note` string naming all four protocols and `state:known-unsupported`; dual-gate: no counter without an analyzer |
| HS-131 | C — TCP/53 → state `unknown` (transport mismatch) | PASS (partial) | hs-tcp53.pcap → {port:53, transport:TCP, state:unknown} (Case C). Cases A/B/D not evaluated — no UDP/53 DNS pcap in fixtures dir |
| HS-132 | A/B — Known-good IT catalog + known-problematic BACnet corpus | PASS | hs-bacnet-corpus.pcap → known-unsupported UDP/47808 entry count=8; `protocols --json` counts 30/7/23 |

## Sampled Older Scenarios (~10 across epics)

| HS ID | Epic | Contract Focus | Result | Evidence |
|-------|------|----------------|--------|----------|
| HS-001 | E-1 (BC-2.01.009) | pcapng/pcap link-type gate + zero-packet notice | PASS | hs-empty.pcap (LINKTYPE_ETHERNET, 0 packets) → notice on stderr, exit 0 |
| HS-011 | E-6 (BC-2.08.001..004) | DNS statistics never emit findings | PASS | DNS analyzer produces zero findings across all fixtures |
| HS-045 | E-3 (BC-2.05.008) | dispatcher without analyzer flags | PASS | `analyze --no-reassemble --json` → `analyzers:[]` |
| HS-061 | E-4 (BC-2.06.023 v1.6) | HTTP detail map = 10 keys incl. `dropped_map_entries` | PASS | HTTP detail keys = {dropped_map_entries, methods, non_http_flows, parse_errors, poisoned_bytes_skipped, recent_uris, status_codes, top_hosts, transactions, user_agents} (repair maint-2026-07-06 FIX-C confirmed) |
| HS-064 | E-8 (BC-2.11.001..005) | JSON top-level = 5 keys (adds mitre_attack_version, mitre_domain) | PASS | Keys = [analyzers, findings, mitre_attack_version, mitre_domain, summary] (repair FIX-C confirmed) |
| HS-066 | E-5 (BC-2.07.031 v1.5) | TLS detail map = 10 keys | PASS | Detail keys = {buffer_saturation_drops, cipher_suites, dropped_map_entries, handshake_reassembly_overflows, ja3_hashes, ja3s_hashes, parse_errors, tls_versions, top_snis, truncated_records} (repair FIX-C confirmed) |
| HS-075 | E-8 (BC-2.11.001..002) | JSON reporter skipped_packets always present + 5 top-level keys | PASS | `summary.skipped_packets` = 0 present, 5 top-level keys (repair FIX-C confirmed) |
| HS-085 | E-9 (BC-2.12.007) | `--reassemble` + `--no-reassemble` mutually exclusive | PASS | clap error, exit 2 |
| HS-086 | E-10 (BC-2.13.001..004) | Obsolete flags rejected | PASS | All 4 (`--threats`, `--beacon`, `--filter`, `--verbose`) → clap "unexpected argument" |
| HS-097 | E-9 (BC-2.12.012) | Non-existent target descriptive error | PASS | `Error: Target not found: /tmp/does-not-exist-xyz.pcap` |
| HS-100 | E-9 (BC-2.12.021) | Summary JSON protocol keys use Debug/CamelCase format | PASS | `summary.protocols` key = `"Tcp"` (CamelCase Debug), not `"TCP"` |

## HS-INDEX-ENIP-WAVE-DRIFT-001 — Confirmation

**Confirmed. HS-INDEX v2.12 states the ENIP feature holdouts trace to waves/stories that do
not match the authoritative story registry.**

HS-INDEX.md v2.12 has three loci referencing ENIP as `waves 63-68` / `STORY-131..STORY-141`:

- Line 746 (feature-holdouts summary table row):
  `| EtherNet/IP (waves 63-68) | 13 seeds (DNP3 convention) | 13 (HS-110..HS-122) | CONCRETE — authored v0.11.0-feature-enip |`
- Line 758 (SS-17 EtherNet/IP section preamble):
  `> Stories: STORY-131..STORY-141 (waves 63-68).`
- Line 784 (EtherNet/IP feature holdout summary row):
  `| Stories | STORY-131..STORY-141 |`

Ground truth per STORY-INDEX.md v3.1 (v2.8 2026-06-24 note) + individual STORY-13n frontmatter:

| Story | Wave | Epic |
|-------|------|------|
| STORY-130 | 58 | E-20 |
| STORY-131 | 58 | E-20 |
| STORY-132 | 59 | E-20 |
| STORY-133 | 59 | E-20 |
| STORY-134 | 60 | E-20 |
| STORY-135 | 60 | E-20 |
| STORY-136 | 60 | E-20 |
| STORY-137 | 60 | E-20 |
| STORY-138 | 61 | E-20 |
| STORY-139 | 62 | E-20 (EC-X1/EC-X2 detection-correctness fix, DRIFT-ENIP-DIRECTION-001) |
| STORY-140 | 63 | **E-15 (DNP3)** — not E-20 |
| STORY-141 | 64 | **E-14 (Modbus)** — not E-20 |

Additional corroboration: HS-110's own frontmatter cites `STORY-130.md` as the source
story (not STORY-131). So the HS-INDEX assertion "STORY-131..STORY-141" excludes the
first ENIP story (STORY-130), and includes two stories (STORY-140, STORY-141) that are
not ENIP at all.

**Correct values:**
- ENIP core (matching HS-110..HS-122 as authored): STORY-130..STORY-138, waves 58-61.
- ENIP inclusive of the EC-X detection-correctness follow-up story: STORY-130..STORY-139, waves 58-62.
- The `waves 63-68` claim in HS-INDEX v2.12 has no support anywhere in STORY-INDEX or the
  individual story frontmatter.

**Recommended remediation (metadata-only edit, no scenario changes):**
- Line 746: `EtherNet/IP (waves 58-62)` (or `58-61` if excluding STORY-139).
- Line 758: `Stories: STORY-130..STORY-139 (waves 58-62).`
- Line 784: `STORY-130..STORY-139` (or `130..138` if excluding EC-X follow-up).
- Bump HS-INDEX version to v2.13 with an entry documenting the ENIP-drift correction.

## Wave-70/71 Coverage Check

Recently shipped behavior in unreleased/wave 70-71 scope:

| Story | Change | Existing holdout coverage | Assessment |
|-------|--------|--------------------------|-----------|
| STORY-149 (wave 70) | TLS carry-path restructured for single-borrow; Criterion `tls_fragmented/3-record-carry-drain` benchmark; single-borrow invariant source-inspection tests | HS-055/HS-062/HS-066/HS-068 continue to guard TLS handshake behavior; single-borrow property is source-lint-shaped, not user-observable | Legitimately no new holdout — behavior-preserving perf recovery + internal invariant guard. Existing TLS holdouts remain the behavioral safety net. |
| STORY-150 (wave 71) | TLS drain-loop DRY unification in `process_handshake_carry`; Kani VP-039 3/3 re-verified | HS-055/HS-062/HS-066/HS-068 (TLS handshake behavior) | Legitimately no new holdout — behavior-preserving refactor + formal proofs. |
| STORY-156 (wave 71) | BC-2.16.016 ARP unbounded-findings regression pin: `summarize()` has no `dropped_findings` key; ARP `long_help` covers unbounded-findings semantics | No holdout asserts the *absence* of `dropped_findings` in the ARP `summarize()` map, and no holdout exercises the CLI `long_help` text. HS-INDEX has no ARP-summary shape holdout in the greenfield namespace (feature-ARP holdouts HS-W40..W44 focus on detection semantics, not summary key sets) | **Gap — minor.** A one-scenario ARP summary shape holdout (analogous to HS-061 for HTTP / HS-066 for TLS) would pin the "13 keys, no dropped_findings" contract. Not urgent — the in-tree regression test does the job — but candidate for a future author-up in the greenfield namespace. |
| STORY-157 (wave 71) | Tooling only: `bin/compute-input-hash` edge cases (empty inputs → `d41d8cd`; inline comment stripping); PG-HASH-HOOK-DIVERGENCE documented in CLAUDE.md | Factory tooling, not product surface | Legitimately no product holdout — this is factory-side tooling, exercised by `bin/test_compute_input_hash.py`. |

## Deferred Holdouts HS-110..HS-122 (ENIP feature — per D-267)

Standing status only (no evaluation performed):

- All 13 concrete scenario files present on disk under `.factory/holdout-scenarios/HS-110..HS-122`.
- All 13 registered in HS-INDEX v2.12 §"Feature Holdouts (SS-17 EtherNet/IP, v0.11.0-feature-enip)"
  with `lifecycle_status: active` and `introduced: v0.11.0-feature-enip`.
- All 13 have `last_evaluated: null` — consistent with formal-eval deferral per D-267.
- Frontmatter drift: the ENIP feature index section still cites wave/story ranges that
  disagree with authoritative story data — see HS-INDEX-ENIP-WAVE-DRIFT-001 above.
  (Frontmatter/index metadata only; individual HS-110..HS-122 file bodies were not
  re-audited in this sweep.)

## Anomalies / Findings

- **No new stale scenarios detected.** The four previously-stale scenarios (HS-061, HS-064,
  HS-066, HS-075) repaired in maint-2026-07-06 FIX-C are now consistent with the shipped
  binary; their assertions pass.
- **HS-INDEX-ENIP-WAVE-DRIFT-001 confirmed** (still unresolved as of v2.12). See §above.
- **Minor coverage gap:** STORY-156 ARP-summary "no dropped_findings" contract is guarded
  by the in-tree regression test but has no HS-INDEX-registered holdout. Non-blocking.
- **Fixture directory has no DNS UDP/53 pcap;** HS-131 case A/B/D cannot be exercised
  without either creating one (scapy) or borrowing from the in-tree DNS test corpus.
