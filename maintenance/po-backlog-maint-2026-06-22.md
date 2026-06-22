---
document_type: po-backlog
sweep: maint-2026-06-22
sweep_number: 4
source: maintenance/holdout-freshness.md
produced_by: product-owner
date: 2026-06-22
status: open
github_issue_status: NOT FILED — pending research-agent validation per DF-VALIDATION-001
---

# Product-Owner Backlog — Maintenance Sweep 4 (maint-2026-06-22)

Source sweep report: `.factory/maintenance/holdout-freshness.md`
Source sweep report: `.factory/maintenance/sweep-report-2026-06-22.md`

This document records PO-actionable items surfaced by Sweep 4 holdout freshness analysis.
No GitHub issues are filed from this document — DF-VALIDATION-001 requires research-agent
validation before any issue is created from a factory finding.

---

## Section 1 — Holdout Assertion Staleness (Intentional Behavior Changes)

These scenarios fail due to intentional product changes, not bugs. The assertions must be
updated to reflect the new correct behavior. No code changes are needed.

### Item PO-S4-001: HS-064 and HS-075 — JSON schema top-level key count assertion stale

**Affected scenarios:** HS-064 (json-reporter-schema-and-encoding), HS-075 (json-skipped-packets-always-present)

**Current assertion:** Both scenarios assert the JSON output has "exactly 3 top-level keys":
`analyzers`, `findings`, `summary`.

**Why it is stale:** PR #209 intentionally added `mitre_attack_version` and `mitre_domain`
as top-level envelope fields for ATT&CK-for-ICS v19.1 compliance. The live schema now has
exactly 5 top-level keys. The underlying behavior is correct; the assertion predates the envelope.

**Required action:** Relax the assertion in both HS-064 and HS-075 from "exactly 3 top-level
keys" to "at least the following 5 keys must be present":
- `analyzers` (present, non-null)
- `findings` (present, array)
- `summary` (present, non-null)
- `mitre_attack_version` (present, string)
- `mitre_domain` (present, string)

No new behavior is being introduced; this is a wording-only fix to match current reality.

**Effort:** Low. Wording-only edits to two HS files.

**Priority:** P1 — affects sweep FAIL-STALE classification; does not gate any release.

---

### Item PO-S4-002: HS-108 Case A — "stdout empty under --json" sub-assertion stale

**Affected scenario:** HS-108 (pcapng-zero-packet-notice-end-to-end), Case A

**Current assertion:** Sub-assertion states "stdout empty under --json".

**Why it is stale:** The JSON reporter correctly emits a valid empty-summary skeleton when
there are zero packets (consistent with the postconditions of HS-075 and BC-2.01.009 PC6).
stdout is NOT empty; it contains the skeleton JSON object. The overall Case A/B/C contract
passes; only the sub-assertion wording is wrong.

**Required action:** Update the Case A sub-assertion to: "stdout contains a valid JSON
empty-summary skeleton (analyzers/findings/summary/mitre_attack_version/mitre_domain
keys all present; findings array is empty)".

**Effort:** Low. Wording-only edit to one HS file.

**Priority:** P1.

---

### Item PO-S4-003: HS-090 and HS-098 — Misleading `--json <pcap>` invocation form

**Affected scenarios:** HS-090 (end-to-end-pcap-to-json-report), HS-098 (end-to-end-pcap-to-csv-report)

**Current text:** Verification steps use the form `wirerust analyze --json <pcap>`.

**Why it is misleading:** `--json` is an optional-value flag in clap. When written as
`--json <pcap>`, clap consumes the pcap path as the value of `--json` (treating it as an
output target) rather than as a positional argument. This causes an error (or unexpected
behavior) and does not invoke the JSON reporter against the pcap. The actual correct form
is `wirerust analyze --output-format json <pcap>`.

**Shipped behavior is correct and safe** — no pcap is overwritten; clap errors safely.
Only the verification text in the HS files is wrong.

**Required action:** Normalize all invocation forms in HS-090 and HS-098 verification
steps from `--json <pcap>` to `--output-format json <pcap>` (or the canonical equivalent
per the current CLI surface).

**Effort:** Low. Wording-only edits to two HS files.

**Priority:** P1.

---

## Section 2 — Holdout Coverage Gap (MAJOR)

The HS-INDEX declares 73 feature-holdout seeds across four shipped, finding-producing
analyzers. Zero corresponding HS files exist on disk for any of these seeds.

### Summary Table

| Feature | Seeds declared | HS files on disk | Gap severity |
|---------|---------------|-----------------|--------------|
| DNP3 (waves 35-39) | 32 | 0 | HIGH — security-relevant ICS detections with MITRE mappings |
| ARP (waves 40-44) | 28 | 0 | HIGH — T0830/T1557.002 detections (arpspoof.pcap: 7 findings) |
| Finding-collapse (wave 47) | 13 | 0 | MEDIUM — live `(xN)` terminal behavior unguarded |
| Modbus (no seeds yet) | 0 declared | 0 | MEDIUM — 47 findings on modbus-large.pcap; no holdout coverage at all |

**Combined unguarded shipped surface:** All four of these features are active in production
(shipped since v0.6.0 / v0.7.0 / v0.8.0 / v0.9.x). A regression in any of them would
not be caught by the holdout suite.

---

### Item PO-S4-004: DNP3 Holdout Scenarios — 32 seeds, 0 files authored

**Seeds declared:** HS-W35-001 through HS-W39-005 (32 seeds in HS-INDEX Feature Holdouts,
SS-15 DNP3, waves 35-39).

**Evidence of shipped behavior:**
- Analyzer active under `wirerust analyze --all`.
- 1108 findings on `dnp3dataset_capture.pcap` across detections including MITRE ICS grouping.
- Stories: STORY-105 through STORY-109 (waves 35-39) delivered.

**Required action (recommended):** Author concrete holdout files for the 32 DNP3 seeds.
Minimum authoring scope per seed: setup pcap/command, concrete expected assertions for
key postconditions. Prioritize P0 seeds first (most are P0 per the HS-INDEX seed table).

**Alternatively:** Formally accept the coverage gap with documented rationale and set a
target cycle for authoring (e.g., v1.0 hardening cycle).

**Priority:** HIGH. DNP3 is a security-relevant ICS analyzer with MITRE ATT&CK ICS mappings.
A regression would be invisible to the holdout suite.

---

### Item PO-S4-005: ARP Holdout Scenarios — 28 seeds, 0 files authored

**Seeds declared:** HS-W40-001 through HS-W44-004 (28 seeds in HS-INDEX Feature Holdouts,
SS-16 ARP, waves 40-44).

**Evidence of shipped behavior:**
- Analyzer active under `wirerust analyze --all` and `--arp`.
- 7 findings on `arpspoof.pcap`: T0830 (Exploitation of Remote Services) and T1557.002
  (ARP Cache Poisoning) MITRE technique attributions.
- Stories: STORY-111 through STORY-115 (waves 40-44) delivered.

**Required action (recommended):** Author concrete holdout files for the 28 ARP seeds.
Priority seeds: HS-W42-001 through HS-W42-007 (GARP/binding-table/D11/D12 detections)
and HS-W43-001 through HS-W43-005 (D1 IP spoof / MITRE attachment / VP-007).

**Alternatively:** Formally accept the coverage gap with documented rationale.

**Priority:** HIGH. ARP poisoning detection is a security-critical feature. The binding
table and MITRE attribution logic are unguarded by any holdout.

---

### Item PO-S4-006: Finding-Collapse Holdout Scenarios — 13 seeds, 0 files authored

**Seeds declared:** HS-W47-001 through HS-W47-013 (13 scenarios in HS-INDEX Feature
Holdouts, SS-11 Finding-Collapse, wave 47).

**Note:** HS-INDEX records these as "FULLY AUTHORED — not seeds — with complete setup
descriptions, commands, and expected assertion lists." However, no files exist at
`.factory/holdout-scenarios/HS-W47-*.md` or in `.factory/feature/wave-holdout-scenarios/`.

**Evidence of shipped behavior:**
- `(xN)` terminal collapse behavior is live in production.
- Default-ON collapse, `--no-collapse` flag, K=3 cap, and JSON/CSV invariant all ship.
- Story: STORY-118 (wave 47) delivered.

**Required action:** Confirm whether the authored content from HS-INDEX exists as concrete
HS files somewhere in the repo (different path?). If not, materialize the 13 authored
scenarios into `.factory/holdout-scenarios/HS-W47-NNN.md` files using the content
already captured in the HS-INDEX seeds section.

**Priority:** MEDIUM. Lower than DNP3/ARP due to lower security criticality, but the
collapse logic affects every terminal output path.

---

### Item PO-S4-007: Modbus Holdout Coverage — 0 seeds declared, 0 files

**Gap type:** No seed declarations and no scenario files.

**Evidence of shipped behavior:**
- Modbus analyzer ships and finds 47 findings on `modbus-large.pcap`.
- Analyzer active under `wirerust analyze --all`.
- MITRE ICS mappings present in Modbus findings.

**Required action:** Author at minimum a coverage skeleton (seeds) for Modbus holdout
scenarios, then promote to concrete scenarios in the following cycle. Minimum viable
scenarios: happy-path Modbus traffic (findings emitted), benign traffic (no false
positives), and at least one MITRE-tagged finding check.

**Priority:** MEDIUM. Modbus is a shipped ICS protocol analyzer with no holdout coverage
at all — not even seeds.

---

## Section 3 — Prioritization

| Item | Description | Severity | Effort | Recommended cycle |
|------|-------------|----------|--------|------------------|
| PO-S4-001 | HS-064 / HS-075 JSON key count stale | STALE | Low | Current / next sweep |
| PO-S4-002 | HS-108 Case A wording stale | STALE | Low | Current / next sweep |
| PO-S4-003 | HS-090 / HS-098 invocation form stale | STALE | Low | Current / next sweep |
| PO-S4-004 | DNP3 — 32 seeds unimplemented | MAJOR | High | v1.0 hardening cycle (prioritize first) |
| PO-S4-005 | ARP — 28 seeds unimplemented | MAJOR | High | v1.0 hardening cycle (prioritize second) |
| PO-S4-006 | Finding-collapse — 13 seeds unimplemented | MEDIUM | Medium | v1.0 hardening cycle |
| PO-S4-007 | Modbus — 0 seeds, 0 files | MEDIUM | Medium | v1.0 hardening cycle |

**Recommended execution order:**
1. PO-S4-001, -002, -003: wording-only fixes; batch in a single PR.
2. PO-S4-004 (DNP3) and PO-S4-005 (ARP): security-relevant; author together in v1.0 cycle.
3. PO-S4-006 and PO-S4-007: follow in same cycle after DNP3/ARP.

---

## Section 4 — DF-VALIDATION-001 Compliance Note

Per CLAUDE.md and policy `DF-VALIDATION-001` in `.factory/policies.yaml`:

> Deferred or open findings MUST be validated by the research agent before being filed
> as GitHub issues. No issue is created from an unvalidated finding.

None of the items in this document have been filed as GitHub issues. They are recorded
here as PO backlog only. Research-agent validation is required before any issue is opened.

---

## Appendix — Source Sweep Evidence

| Evidence artifact | Path |
|------------------|------|
| Holdout freshness sweep | `.factory/maintenance/holdout-freshness.md` |
| Full sweep report | `.factory/maintenance/sweep-report-2026-06-22.md` |
| HS-INDEX (seed declarations) | `.factory/holdout-scenarios/HS-INDEX.md` |
| HS-064 scenario file | `.factory/holdout-scenarios/HS-064-json-reporter-schema-and-encoding.md` |
| HS-075 scenario file | `.factory/holdout-scenarios/HS-075-json-reporter-skipped-packets-always-present.md` |
| HS-090 scenario file | `.factory/holdout-scenarios/HS-090-end-to-end-pcap-to-json-report.md` |
| HS-098 scenario file | `.factory/holdout-scenarios/HS-098-end-to-end-pcap-to-csv-report.md` |
| HS-108 scenario file | `.factory/holdout-scenarios/HS-108-pcapng-zero-packet-notice-end-to-end.md` |
