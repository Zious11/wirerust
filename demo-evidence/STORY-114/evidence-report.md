# Demo Evidence Report — STORY-114

**Story:** D1 ARP Spoof Escalation + GARP-that-Conflicts (D2+D1) + MITRE Attribution + VP-007 5-Part Atomic Update
**Story ID:** STORY-114
**Branch:** `worktree-issue-9-story-114-arp-d1-spoof`
**Evidence committed on:** `factory-artifacts`
**Recorded:** 2026-06-15
**Toolchain:** VHS (CLI recordings), Rust stable

---

## Fixture Availability Note

No existing test fixture (`tests/fixtures/`) produces an IP→MAC rebind that triggers a D1 spoof finding under `--arp`. The `dns-remoteshell.pcap` fixture contains 4 ARP frames establishing 3 unique bindings with no rebind events (`spoof_findings: 0`). D1/D2+D1/D12 logic is demonstrated exclusively via unit tests in `src/analyzer/arp.rs` (module `story_114`) and integration tests in `tests/bc_2_16_story114_arp_tests.rs` — this is the intended verification strategy per the story's Test Plan.

---

## Recording Inventory

### AC-006-arp-spoof-threshold-flag.gif / .webm / .tape

**Acceptance Criteria:** AC-006 (BC-2.16.012 — `--arp-spoof-threshold` wiring)

**Command sequence:**
```
# 1. Confirm flag presence and default in --help
wirerust analyze --help 2>&1 | grep -A3 'arp-spoof-threshold'

# 2. Default threshold (3) — run without flag
wirerust analyze --arp tests/fixtures/dns-remoteshell.pcap --no-color 2>&1 | grep -A12 'ANALYZER: ARP'

# 3. Override to threshold=1 — flag accepted, no error
wirerust analyze --arp --arp-spoof-threshold 1 tests/fixtures/dns-remoteshell.pcap --no-color 2>&1 | grep -A12 'ANALYZER: ARP'
```

**Observed values (both runs):**
```
ANALYZER: ARP
  Packets analyzed: 4
  bindings_tracked: 3
  frames_analyzed: 4
  garp_findings: 0
  malformed_findings: 0
  malformed_frames: 0
  mismatch_findings: 0
  other_opcode_count: 0
  reply_count: 2
  request_count: 2
  spoof_findings: 0
  storm_findings: 0
```

**Help text excerpt:**
```
--arp-spoof-threshold <ARP_SPOOF_THRESHOLD>
    D1 spoof escalation threshold: number of MAC rebinds within ARP_FLAP_WINDOW_SECS (60 s)
    before a HIGH severity finding is emitted. Default: 3. Set to 1 to fire HIGH on the very
    first rebind. BC-2.16.012 primary deliverable (STORY-114). --arp-storm-rate is STORY-115
    [default: 3]
```

The flag is accepted without error at both `--arp-spoof-threshold 1` and the implicit default of 3. No D1 findings appear because the fixture has no IP→MAC rebind.

---

### AC-001-002-016-d1-spoof-unit-tests.gif / .webm / .tape

**Acceptance Criteria:** AC-001, AC-002, AC-016

**Command:**
```
cargo test test_d1 2>&1 | grep -E 'test analyzer|test result.*passed'
```

**Observed output:**
```
test analyzer::arp::story_114::test_d1_first_rebind_emits_medium ... ok
test analyzer::arp::story_114::test_d1_finding_evidence_contains_ips_and_macs ... ok
test analyzer::arp::story_114::test_d12_mismatch_carries_mitre_after_catalog ... ok
test analyzer::arp::story_114::test_d1_escalates_to_high_at_threshold ... ok
test analyzer::arp::story_114::test_d1_flap_window_reset ... ok
test analyzer::arp::story_114::test_d1_high_guard_prevents_second_high ... ok
test analyzer::arp::story_114::test_d1_threshold_1_high_on_first_rebind ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 60 filtered out
```

**Coverage per AC:**

| AC | Test | Result |
|----|------|--------|
| AC-001 | `test_d1_first_rebind_emits_medium` — first rebind emits MEDIUM, mitre_techniques=["T0830","T1557.002"] | ok |
| AC-002 | `test_d1_escalates_to_high_at_threshold` — 3 rebinds within 60s → HIGH on 3rd | ok |
| AC-003 | `test_d1_high_guard_prevents_second_high` — after HIGH, subsequent rebinds are MEDIUM | ok |
| AC-004 | `test_d1_flap_window_reset` — after 60s, window resets and next rebind is MEDIUM | ok |
| AC-005 | `test_d1_threshold_1_high_on_first_rebind` — threshold=1 → HIGH on first rebind | ok |
| AC-016 | `test_d1_finding_evidence_contains_ips_and_macs` — evidence has conflicting IP + old MAC + new MAC | ok |
| AC-017 | `test_d12_mismatch_carries_mitre_after_catalog` — D12 mismatch finding carries T0830+T1557.002 | ok |

---

### AC-007-008-009-garp-conflict-tests.gif / .webm / .tape

**Acceptance Criteria:** AC-007, AC-008, AC-009

**Command:**
```
cargo test test_garp_conflicts 2>&1 | grep -E 'test analyzer|test result.*passed'
```

**Observed output:**
```
test analyzer::arp::story_114::test_garp_conflicts_d1_high_at_threshold ... ok
test analyzer::arp::story_114::test_garp_conflicts_garp_finding_upgrades_to_medium ... ok
test analyzer::arp::story_114::test_garp_conflicts_d1_also_emitted ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 64 filtered out
```

**Coverage per AC:**

| AC | Test | Result |
|----|------|--------|
| AC-007 | `test_garp_conflicts_garp_finding_upgrades_to_medium` — GARP with binding conflict → MEDIUM (not LOW), mitre=["T0830","T1557.002"] | ok |
| AC-008 | `test_garp_conflicts_d1_also_emitted` — same frame emits two findings: GARP MEDIUM + D1 MEDIUM | ok |
| AC-009 | `test_garp_conflicts_d1_high_at_threshold` — 3rd rebind as GARP conflict → GARP MEDIUM + D1 HIGH | ok |

---

### AC-011-012-mitre-catalog-vp007.gif / .webm / .tape

**Acceptance Criteria:** AC-011, AC-012

**Command sequence:**
```
# VP-007 drift guard (SEEDED=25, EMITTED=17)
cargo test vp007_catalog_drift_guard 2>&1 | grep -E 'vp007_catalog|test result.*passed'

# Integration tests: T0830+T1557.002 catalog resolution + seeded/emitted counts
cargo test --test bc_2_16_story114_arp_tests 2>&1 | grep -E 'test story_114|test result.*passed'
```

**Observed output:**
```
test mitre::vp007_format_tests::vp007_catalog_drift_guard ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 66 filtered out

test story_114_mitre::test_t0830_and_t1557_002_resolves_in_catalog ... ok
test story_114_mitre::test_vp007_all_17_emitted_ids_resolve ... ok
test story_114_mitre::test_vp007_seeded_25_emitted_17 ... ok
test story_114_cli::test_cli_arp_spoof_threshold_default_3 ... ok
test story_114_cli::test_cli_arp_spoof_threshold_parsed ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Coverage per AC:**

| AC | Test | Result |
|----|------|--------|
| AC-011 | `test_t0830_and_t1557_002_resolves_in_catalog` — `technique_info("T0830")` → `("Adversary-in-the-Middle", LateralMovement)`, `technique_info("T1557.002")` → `("Adversary-in-the-Middle: ARP Cache Poisoning", CredentialAccess)` | ok |
| AC-012 | `vp007_catalog_drift_guard` — SEEDED_TECHNIQUE_ID_COUNT=25, SEEDED_TECHNIQUE_IDS.len()=25, EMITTED_IDS.len()=17; `test_vp007_seeded_25_emitted_17` — all 25 IDs resolve via public API | ok |
| AC-006 | `test_cli_arp_spoof_threshold_parsed` — `--arp-spoof-threshold 1` parsed correctly | ok |
| AC-006 | `test_cli_arp_spoof_threshold_default_3` — absent flag defaults to 3 | ok |

---

### AC-all-full-test-suite.gif / .webm / .tape

**Acceptance Criteria:** All (regression gate)

**Command:**
```
/path/to/run_tests_full.sh
# which runs: cargo test --all-targets 2>&1 | grep '^test result' | awk sum
```

**Observed output:**
```
Total passed: 1552 | Expected: 1552
```

Zero failures across all 50 test binaries. The full suite confirms STORY-114 implementation is non-regressing across all prior stories.

---

## ARP Summary — dns-remoteshell.pcap

The only ARP-containing fixture in the repo is `dns-remoteshell.pcap`. It produces:

| Field | Value |
|-------|-------|
| frames_analyzed | 4 |
| bindings_tracked | 3 |
| request_count | 2 |
| reply_count | 2 |
| spoof_findings | 0 |
| garp_findings | 0 |
| mismatch_findings | 0 |
| malformed_findings | 0 |
| storm_findings | 0 |

No D1 finding is triggered because no IP→MAC rebind occurs in this fixture. This is expected and explicitly documented above.

---

## AC Coverage Matrix

| AC | Description | Demo Artifact | Status |
|----|-------------|---------------|--------|
| AC-001 | D1 first rebind emits MEDIUM + MITRE | AC-001-002-016-d1-spoof-unit-tests | PASS |
| AC-002 | D1 escalates to HIGH at threshold=3 | AC-001-002-016-d1-spoof-unit-tests | PASS |
| AC-003 | One-shot HIGH guard (subsequent = MEDIUM) | AC-001-002-016-d1-spoof-unit-tests | PASS |
| AC-004 | Flap window reset after 60s | AC-001-002-016-d1-spoof-unit-tests | PASS |
| AC-005 | threshold=1 → HIGH on first rebind | AC-001-002-016-d1-spoof-unit-tests | PASS |
| AC-006 | `--arp-spoof-threshold` CLI flag wired | AC-006-arp-spoof-threshold-flag + AC-011-012-mitre-catalog-vp007 | PASS |
| AC-007 | GARP+conflict upgrades to MEDIUM | AC-007-008-009-garp-conflict-tests | PASS |
| AC-008 | GARP+conflict also emits D1 | AC-007-008-009-garp-conflict-tests | PASS |
| AC-009 | GARP+conflict D1 HIGH at threshold | AC-007-008-009-garp-conflict-tests | PASS |
| AC-010 | GARP without conflict → LOW only (regression) | AC-all-full-test-suite | PASS |
| AC-011 | T0830→LateralMovement, T1557.002→CredentialAccess | AC-011-012-mitre-catalog-vp007 | PASS |
| AC-012 | vp007_catalog_drift_guard SEEDED=25, EMITTED=17 | AC-011-012-mitre-catalog-vp007 | PASS |
| AC-013 | IcsImpact Display = "Impact (ICS)" — verify only | AC-all-full-test-suite (existing F5 tests pass) | PASS |
| AC-014 | MitreTactic::Impact != MitreTactic::IcsImpact | AC-all-full-test-suite | PASS |
| AC-015 | HS-008 already correct — verify only | No change required (D-069) | N/A |
| AC-016 | D1 evidence has IP + old MAC + new MAC | AC-001-002-016-d1-spoof-unit-tests | PASS |
| AC-017 | D12 mismatch carrying MITRE after catalog seeding | AC-001-002-016-d1-spoof-unit-tests | PASS |

---

## Worktree Cleanliness Verification

After committing evidence to `factory-artifacts`, the story worktree must remain clean:

```
git -C /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-114 status --short
# Expected: (empty — no output)
```
