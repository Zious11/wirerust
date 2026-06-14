---
document_type: wave-schedule
level: ops
version: "1.3"  # v1.3 (Pass-28 Slice-D): T0855→T1692.001 remap per issue #222 / ATT&CK-ICS v19 (HS-INDEX:322); seeded-count clarified as v0.3.0 milestone figure
status: draft
producer: story-writer
timestamp: 2026-06-13T00:00:00Z
phase: 4
inputs:
  - .factory/stories/STORY-INDEX.md
  - .factory/stories/STORY-100.md
  - .factory/stories/STORY-101.md
  - .factory/stories/STORY-102.md
  - .factory/stories/STORY-103.md
  - .factory/stories/STORY-104.md
  - .factory/stories/STORY-105.md
  - .factory/stories/STORY-111.md
  - .factory/stories/STORY-112.md
  - .factory/stories/STORY-113.md
  - .factory/stories/STORY-114.md
  - .factory/stories/STORY-115.md
traces_to: .factory/stories/STORY-INDEX.md
feature_id: issue-007-modbus-analyzer, issue-009-arp-security-analyzer
github_issue: 7, 9
cycles:
  - v0.3.0-multitag   # Waves 31 — E-13 Multi-Tag Schema Migration
  - v0.4.0-modbus     # Waves 32-34 — E-14 Modbus TCP Analyzer
  - v0.7.0-arp        # Waves 40-44 — E-16 ARP Security Analyzer
---

# Wave Schedule: Feature #7 Modbus Analyzer + Multi-Tag Schema; Feature #9 ARP Security Analyzer (Waves 40-44)

> **Context:** Feature #7 (GitHub issue #7) introduces two release increments:
> v0.3.0 ships the multi-tag Finding schema migration (E-13); v0.4.0 ships
> the Modbus TCP analyzer (E-14). Waves 31–34 extend the existing wirerust
> wave graph (Waves 1–30, already delivered). Topological sort confirmed acyclic.

---

## Summary

| Metric | Value |
|--------|-------|
| New stories | 6 (STORY-100 through STORY-105) |
| New waves | 4 (Waves 31–34) |
| New story points | 58 |
| Release gates | v0.3.0 after Wave 31; v0.4.0 after Wave 34 |
| Critical path | STORY-100 → STORY-102 → STORY-103 → STORY-104 → STORY-105 (5 stories, 4 serial hops) |
| Max parallelism | Wave 31: STORY-101 ∥ STORY-102 (parallel after STORY-100) |

---

## Dependency Graph

```
(Wave 30, completed)
    STORY-099
        │
        │  ← v0.3.0 release gate after Wave 31 ───────────────────────┐
        ▼                                                               │
    STORY-100  [Wave 31, P0, 13 pts — E-13: field rename + catalog]    │
    depends_on: []  (no dep on wave-30; independent schema change)     │
        │                                                               │
        ├──────────────────────┐                                        │
        ▼                      ▼                                        │
    STORY-101              STORY-102                                    │
    [Wave 31, P0, 8 pts    [Wave 32, P0, 8 pts                         │
     E-13: reporter         E-14: Modbus pure core]                    │
     multi-tag]             depends_on: [STORY-100]                    │
    depends_on:             blocks: [STORY-103]                        │
    [STORY-100]                  │                                      │
    blocks: []               STORY-103  [Wave 33, P0, 8 pts            │
        │                    E-14: flow state + transaction]            │
        │  ← STORY-101 ─────► (no dep on 101; parallel lane)          │
        │                        │                                      │
        │                    STORY-104  [Wave 33, P0, 13 pts           │
        │                    E-14: detection emissions]                 │
        │                        │                                      │
        │                    STORY-105  [Wave 34, P0, 8 pts            │
        │                    E-14: dispatcher integration + CLI]        │
        │                        │                                      │
        │                        │  ← v0.4.0 release gate after Wave 34─┘
        └────────────────────────┘
```

**Notes:**
- STORY-101 and STORY-102 are in different waves (31 vs 32) because STORY-102
  depends on STORY-100, which is in Wave 31. STORY-101 also depends on STORY-100
  and is colocated in Wave 31 (intra-wave serial: dispatch STORY-101 only after
  STORY-100 PR is merged and CI is green).
- STORY-103 and STORY-104 are both in Wave 33 (STORY-103 unblocks STORY-104;
  they must be dispatched sequentially within Wave 33).
- STORY-101 has no dependency on STORY-102/103/104/105 and does not block them.

---

## Wave Table

| Wave | Stories | Parallelism | Points | Release Gate |
|------|---------|-------------|--------|--------------|
| 31 | STORY-100, STORY-101 | Serial within wave (101 after 100) | 21 | v0.3.0 |
| 32 | STORY-102 | — | 8 | — |
| 33 | STORY-103, STORY-104 | Serial within wave (104 after 103) | 21 | — |
| 34 | STORY-105 | — | 8 | v0.4.0 |
| **TOTAL** | **6** | | **58** | |

---

## Release Gate Mapping

### v0.3.0 — After Wave 31

**Trigger:** Both STORY-100 and STORY-101 PRs merged; `cargo test --all-targets` green.

**Scope:** Multi-tag `Finding` schema migration.

**Deliverables:**
- `Finding.mitre_techniques: Vec<String>` (was `mitre_technique: Option<String>`)
- JSON output: singleton techniques now `["T1027"]` (array, not scalar)
- CSV column 6: renamed `mitre_techniques`; semicolon-join for multi-technique cells
- Terminal: multi-ID MITRE line (`MITRE: T1692.001, T0836`); tactic grouping by first element
- JSON report: `mitre_domain: "ics-attack"` and `mitre_attack_version: "ics-attack-v15"` envelope keys
- MITRE catalog: 21 seeded techniques at v0.3.0 (was 15; current canonical 25 post-DNP3/ARP); 6 new ICS arms: T0836, T0814, T0806, T0835, T0831, T0888

**Breaking change:** JSON `"mitre_technique"` scalar field replaced by `"mitre_techniques"` array.
CSV and terminal output are behavior-preserving for existing single-technique findings.

**Pre-release obligations (FLAG F4):** Verify `mitre_attack_version = "ics-attack-v15"` covers
all 7 ICS techniques (T0888, T1692.001, T0836, T0835, T0831, T0814, T0806) at
attack.mitre.org/resources/attack-data-and-tools/ before creating the v0.3.0 release tag.

### v0.4.0 — After Wave 34

**Trigger:** STORY-102, STORY-103, STORY-104, STORY-105 all PRs merged; full test suite green.

**Scope:** Modbus TCP analyzer (BC-2.14.001 through BC-2.14.025).

**Deliverables:**
- `--modbus` CLI flag (default off); `--all` includes Modbus; `--no-reassemble` warning
- `--modbus-write-burst-threshold` (default 20) and `--modbus-write-sustained-threshold` (default 10)
- `src/analyzer/modbus.rs`: MBAP parse, FC classification, flow state, 7 detectors, summarize()
- `DispatchTarget::Modbus` Rule 5 in `src/dispatcher.rs` (port 502, after content rules)
- VP-022 Kani proofs: parse_mbap no-panic, classify_fc total, pending-table bound
- VP-004 Kani oracle extended to cover Modbus branch
- 7 MITRE detection rules: T1692.001 (write), T0836 (register write), T0835 (coil write),
  T0831 (coordinated write T0831 co-tag), T0806 (burst/sustained rate), T0814 (DoS diagnostics),
  T0888 (recon 0x11/0x2B)

---

## Dispatch Ordering

### Wave 31 (v0.3.0 cycle)

1. Dispatch **STORY-100** (13 pts). Wait for PR merge + `cargo test --all-targets` green.
2. Dispatch **STORY-101** (8 pts). Depends on STORY-100 being merged.
3. Both PRs merged → **v0.3.0 release gate** → create release tag.

### Wave 32

4. Dispatch **STORY-102** (8 pts) after STORY-100 is merged (dependency).

### Wave 33

5. Dispatch **STORY-103** (8 pts) after STORY-102 PR is merged.
6. Dispatch **STORY-104** (13 pts) after STORY-103 PR is merged.

### Wave 34 (v0.4.0 cycle)

7. Dispatch **STORY-105** (8 pts) after STORY-104 PR is merged.
8. STORY-105 merged → **v0.4.0 release gate** → create release tag.

---

## Critical Path

```
STORY-100 (13) → STORY-102 (8) → STORY-103 (8) → STORY-104 (13) → STORY-105 (8)
                                                                    ↑
                                                      Total: 50 pts, 4 serial hops
```

STORY-101 (8 pts) is off the critical path — it runs in parallel with STORY-102 after
STORY-100. Its late completion does not delay v0.4.0 (only delays v0.3.0 tag if it blocks
the Wave 31 gate).

---

## Parallelism Opportunities

| After | Parallel Candidates | Notes |
|-------|---------------------|-------|
| STORY-100 merged | STORY-101 ∥ STORY-102 | Both depend only on STORY-100; can be dispatched simultaneously |
| STORY-103 merged | STORY-104 (only option) | STORY-104 is the sole blocker; no other parallelism available |

---

## Affected Existing Stories (Revision Notes)

The following completed stories are affected by the STORY-100 schema migration.
Their test assertions are updated by STORY-100; no re-implementation is required.

| Story | Version Before | Version After | Scope of Change |
|-------|---------------|---------------|-----------------|
| STORY-069 | 1.4 | 1.5 | Finding struct tests: `mitre_technique: Some/None` → `mitre_techniques: vec!` |
| STORY-070 | 1.6 | 1.7 | JSON serialization tests: scalar → array; skip_serializing_if update |
| STORY-071 | 1.5 | 1.6 | MITRE catalog tests: 15→21 seeded IDs; VP-007 grep pattern update |
| STORY-078 | 1.5 | 1.6 | Terminal MITRE grouping: Vec[0] first-element; multi-ID render |
| STORY-079 | 1.4 | 1.5 | CSV column 6 header rename + join(";") encoding |
| STORY-080 | 1.3 | 1.4 | CSV optional-field encoding: Option::None → Vec::is_empty |

---

# Wave Schedule Extension: Feature #9 — ARP Security Analyzer (issue #9, v0.7.0)

> **Context:** Feature #9 (GitHub issue #9) introduces the ARP Security Analyzer (E-16).
> v0.7.0 ships the complete ARP detection stack. Waves 40–44 extend the existing wirerust
> wave graph (Waves 1–39, all delivered). The dependency chain is strictly linear; no
> parallelism within E-16. Topological sort confirmed acyclic (E-16 tail appended after
> Wave 39; one cross-epic edge STORY-110 → STORY-111).

---

## Summary (E-16 ARP)

| Metric | Value |
|--------|-------|
| New stories | 5 (STORY-111 through STORY-115) |
| New waves | 5 (Waves 40–44) |
| New story points | 47 |
| Release gate | v0.7.0 after Wave 44 |
| Critical path | STORY-110 → STORY-111 → STORY-112 → STORY-113 → STORY-114 → STORY-115 (5 E-16 stories, 4 serial hops; + 1 cross-epic hop from STORY-110) |
| Max parallelism | None — strictly linear within E-16 |

---

## Dependency Graph (E-16)

```
(Wave 39, delivered)
    STORY-110
        │  ← v0.7.0 release gate after Wave 44 ──────────────────────┐
        ▼                                                              │
    STORY-111  [Wave 40, P0, 5 pts — SS-02: etherparse migration]    │
    depends_on: [STORY-110]                                           │
        │                                                              │
        ▼                                                              │
    STORY-112  [Wave 41, P0, 8 pts — SS-16: ArpAnalyzer Stub + VP-024 Sub-A]  │
    depends_on: [STORY-111]                                           │
        │                                                              │
        ▼                                                              │
    STORY-113  [Wave 42, P0, 13 pts — SS-16: D2/D11/D12 detections + binding table]  │
    depends_on: [STORY-112]                                           │
        │                                                              │
        ▼                                                              │
    STORY-114  [Wave 43, P0, 13 pts — SS-16: emissions + VP-007 atomic]  │
    depends_on: [STORY-113]                                           │
        │                                                              │
        ▼                                                              │
    STORY-115  [Wave 44, P0, 8 pts — SS-02/SS-16: D3 storm + decode-time ARP gating + --arp-storm-rate CLI]  │
    depends_on: [STORY-114]                                           │
        │                                                              │
        └──────────────────────────── v0.7.0 release gate ────────────┘
```

---

## Wave Table (E-16)

| Wave | Stories | Parallelism | Points | Release Gate |
|------|---------|-------------|--------|--------------|
| 40 | STORY-111 | — | 5 | — |
| 41 | STORY-112 | — | 8 | — |
| 42 | STORY-113 | — | 13 | — |
| 43 | STORY-114 | — | 13 | — |
| 44 | STORY-115 | — | 8 | v0.7.0 |
| **TOTAL** | **5** | | **47** | |

---

## Release Gate: v0.7.0 — After Wave 44

**Trigger:** STORY-111 through STORY-115 PRs merged; `cargo test --all-targets` green; `cargo clippy --all-targets -- -D warnings` clean.

**Scope:** ARP Security Analyzer (BC-2.16.001 through BC-2.16.015).

**Deliverables:**
- etherparse upgraded to 0.20; `DecodedFrame` enum with `Ip` and `Arp` variants; `ArpFrame` struct
- BC-2.02.009 revised to v1.6: third decode path `Ok(DecodedFrame::Arp(...))` for Ethernet/IPv4 ARP frames
- `src/analyzer/arp.rs`: `ArpAnalyzer` struct, `process_arp`, binding table (cap 65,536 = MAX_ARP_BINDINGS), per-MAC storm counters (cap 4,096 = MAX_STORM_COUNTERS), D1 spoofing, D2 GARP, D3 storm detections, `summarize()`
- ARP gating at decode-time in `src/main.rs`: `DecodedFrame::Arp(frame)` branch routes to `ArpAnalyzer::process_arp` (BC-2.16.011). ARP does NOT pass through `classify()` / `src/dispatcher.rs` — it is intercepted before StreamDispatcher (arp-architecture-delta §4.4).
- `--arp` CLI flag (default off); `--all` includes ARP
- VP-024 Kani proofs (4 sub-groups / 6 properties): Sub-A = 3 Kani (verify_extract_arp_frame_safety, verify_extract_arp_frame_eth_ipv4_correctness, verify_extract_arp_frame_none_on_bad_size); Sub-B = 1 Kani (verify_classify_garp_total); Sub-C = 1 proptest (test_binding_table_last_write_wins — NOT Kani); Sub-D = 1 Kani (verify_binding_table_cap)
- VP-007 atomic update: SEEDED 23→25 / EMITTED 15→17 (`vp007_catalog_drift_guard` green)
- VP-008 fuzz harness updated for `Result<DecodedFrame>` return type
- MITRE detections: T0830 + T1557.002 only on D1 spoof + GARP-that-conflicts escalation (BC-2.16.014); benign D2 GARP emits mitre_techniques=[]; T0814 deferred (D3 storm, per DF-VALIDATION-001)

---

## Dispatch Ordering (E-16)

1. Dispatch **STORY-111** (5 pts) after STORY-110 is merged. Wait for PR merge + CI green.
2. Dispatch **STORY-112** (8 pts) after STORY-111 is merged.
3. Dispatch **STORY-113** (13 pts) after STORY-112 is merged.
4. Dispatch **STORY-114** (13 pts) after STORY-113 is merged.
5. Dispatch **STORY-115** (8 pts) after STORY-114 is merged.
6. STORY-115 merged → **v0.7.0 release gate** → create release tag.

---

## Critical Path (E-16)

```
STORY-111 (5) → STORY-112 (8) → STORY-113 (13) → STORY-114 (13) → STORY-115 (8)
Total: 47 pts, 4 serial hops — no off-critical-path work; all 5 stories are on the critical path.
```
