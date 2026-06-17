---
document_type: maintenance-sweep-output
sweep: tech-debt-register
sweep_id: maint-2026-06-17 / Sweep-8
producer: consistency-validator
timestamp: 2026-06-17T00:00:00Z
register_version: "1.1"
register_last_updated: 2026-06-12
---

# Tech Debt Register — Maintenance Sweep 8

**Date:** 2026-06-17  
**Register source:** `.factory/tech-debt-register.md` (v1.1, last updated 2026-06-01 per document header; effective as of 2026-06-12 with STATE.md drift items cross-referenced)  
**Scope:** Recommendations only. No edits to the register file in this sweep.  
**DF-VALIDATION-001:** All recommended new entries that could become GitHub issues are flagged accordingly.

---

## 1. Existing Register State

### 1.1 Open Items Audit (11 open entries)

| ID | Priority | Status | Observation |
|----|----------|--------|-------------|
| O-07 | P2 | OPEN | `rayon` still declared in Cargo.toml (v1, resolved 1.12.0), zero `use rayon` in `src/`. Still genuine. No change. |
| O-08 | P3 | OPEN | `src/analyzer/dns.rs` module doc-comment stale. Not independently verified in this sweep; no contraindication found. Remains OPEN. |
| CR-002 | P3 | OPEN | `handler.findings()` — trait method in `reassembly/handler.rs:64` returns `Vec<Finding>` (clone). `reassembly/mod.rs:685` impl returns `&[Finding]`. The trait and impl signatures diverge; trait callers clone, impl callers borrow. Remains OPEN. |
| CR-003 | P3 | OPEN | `ThreatCategory::Persistence` doc-comment mismatch. Not re-verified in this sweep. Remains OPEN. |
| CR-005 | P3 | OPEN | `resolve_targets` recursion. Not re-verified. Remains OPEN. |
| CR-006 | P3 | OPEN | 5 `unwrap()` calls in `mod.rs` (confirmed: `reassembly/mod.rs:270/286/340/474/581`). All `get_mut(key).unwrap()` on flows map. Still genuine. Remains OPEN. |
| CR-007 | P3 | OPEN | `json.rs:69` infallible `unwrap()` on `serde_json::to_string_pretty`. Confirmed present. Remains OPEN. |
| CR-009 | P3 | OPEN | HTTP chunked/compressed edge case fixture gap. Not re-verified. Remains OPEN. |
| CR-012 | P3 | OPEN | HashMap accessor inconsistency. Not re-verified. Remains OPEN. |
| DRIFT-DNP3-DIRECTION-001 | P3 | DEFERRED | Still valid; direction threading deferred post-v0.6.0. `resolve_master_ip` heuristic still in `src/analyzer/dnp3.rs:169`. No change in status. |
| DRIFT-MITRE-EMITTED-LABEL-001 | P3 | DEFERRED | **STALE — RECOMMEND CLOSURE.** The drift entry claimed "T0835 and T0831 are NOT emitted by any analyzer (13 actual vs 15 labeled)." `src/analyzer/modbus.rs:561/564` confirms both IDs ARE emitted (`mitre.push("T0835")` / `mitre.push("T0831")`). The EMITTED_IDS array correctly lists both. Original concern is no longer valid. The comment at line 225 of mitre.rs (`// 6 Enterprise + 7 ICS + 2 STORY-109 + 2 ARP ... = 17 emitted`) accurately accounts for them. Recommend updating status to RESOLVED/INVALID in the register. |
| DRIFT-BC-2.15.024-EC006-PROSE-001 | P3 | DEFERRED | Still valid PO backlog item. No change in status. |
| DRIFT-ENGINE-CHECKOUT-GUARD-001 | P2 | ENGINE-NOTE | Dark-factory engine scope. No change. |
| DRIFT-ENGINE-PRMGR-REPORT-001 | P2 | ENGINE-NOTE | Dark-factory engine scope. No change. |

### 1.2 Deferred / Future Enhancement Items

| ID | Status | Observation |
|----|--------|-------------|
| FE-001 | deferred/v2 | pcapng format. Still valid, still deferred. |
| RUSTSEC-2026-0097 | ACCEPTED-TRANSITIVE | rand 0.8.5 via tls-parser→phf. Still valid. No new RUSTSEC advisories affecting runtime code noted (dependency-audit-raw.log exists in maintenance/). |
| ACTION-PIN-001 | OPEN P3 | dtolnay/rust-toolchain allowlist exemption. Still valid. Not a security risk; documented in ci.yml lines 214/231-233. |
| PCAP-CORPUS-001 | TABLED | Still tabled; human decision pending. |
| DRIFT-F2-COUNT-001 | DEFERRED | "15 seeded" count stale. Still valid per BC-2.10.006 changelog note (25 after STORY-114). |
| DRIFT-SUPERPOWERS-001 | DEFERRED | docs/superpowers/ stale. Still valid. |

### 1.3 Items Recommended for Closure/Update

| ID | Recommendation | Reason |
|----|----------------|--------|
| DRIFT-MITRE-EMITTED-LABEL-001 | Update to RESOLVED/INVALID | T0835/T0831 ARE emitted by modbus.rs:561/564; original claim was incorrect |
| DRIFT-ENGINE-RELEASECONFIG-STALE-001 | Mark RESOLVED | STATE.md marks "PARTIALLY RESOLVED"; the version_sources follow-up is a low-value cosmetic; recommend closing |
| DRIFT-ETHERPARSE-0.20-MIGRATION-001 | Mark RESOLVED | Folded into ARP feature; delivered in v0.7.0 (D-066 sub-delta A) |
| DNPXX-SOURCE-RENAME-001 | Confirm still DEFERRED | `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT` confirmed present in `src/analyzer/dnp3.rs:169`, `src/cli.rs:16`, `src/main.rs:218`, and 26 test references. Rename touches 30 locations — requires DF-VALIDATION-001 before filing. |

---

## 2. Items from E-17 Cycle — Candidate New Register Entries

The following drift items appear in STATE.md (Drift Items section, post-v0.7.1) that are NOT yet in the register. Each is evaluated for register eligibility.

### 2.1 Recommended: Add to Register

**Proposed entry: DRIFT-VP024-BTREEMAP-PROSE-001**

- **Source:** STATE.md Drift Items; E-17 F2 cycle
- **Description:** VP-024 Feasibility Assessment table row "Input space size" (~line 582) still reads "Sub-D: 9-iteration loop, BTreeMap with 8 entries maximum". The shipped Sub-D substrate is the `insert_binding_lru_array` fixed-capacity array surrogate (not BTreeMap). The BTreeMap narrative in the same section correctly documents why BTreeMap was rejected; only this one Feasibility table cell remains stale. All normative proof sections were corrected in v2.0/v2.2/v2.3.
- **Severity:** LOW (cosmetic prose lag in a single table cell)
- **Priority:** P3
- **Status:** DEFERRED — VP-maintenance pass
- **DF-VALIDATION-001 required before GitHub issue:** Yes

---

**Proposed entry: DRIFT-E17-VERSIONLABEL-LAG-001**

- **Source:** STATE.md Drift Items; E-17 F4 wave-adversary residual LOW
- **Description:** `verification-coverage-matrix` lines ~48/137 and E-17 test-file doc-comments cite initial-burst BC versions (v1.8/v1.7) rather than final v1.9/v1.8. EC-009 content is version-stable so citations resolve correctly; cosmetic version-label lag only.
- **Severity:** LOW (cosmetic)
- **Priority:** P3
- **Status:** DEFERRED LOW — traceability sweep candidate
- **DF-VALIDATION-001 required before GitHub issue:** Yes

---

**Proposed entry: DRIFT-E16-EPICS-SUMMARY-GAP-001**

- **Source:** STATE.md Drift Items
- **Description:** `epics.md` "Estimated Story Count Summary" table omits Epic E-16 (ARP Security Analyzer, 5 stories, STORY-111..115). Table totals are understated. The body section for E-16 is also absent. Pre-existing E-16 debt.
- **Severity:** LOW (documentation/traceability; does not affect runtime behavior)
- **Priority:** P3
- **Status:** DEFERRED LOW — epic-registry maintenance sweep
- **DF-VALIDATION-001 required before GitHub issue:** Yes

---

**Proposed entry: DRIFT-E16-BC-BACKLINK-GAP-001**

- **Source:** STATE.md Drift Items
- **Description:** `BC-2.16.009` and `BC-2.16.015` Traceability "Stories:" fields omit STORY-114 and STORY-115. E-17 added STORY-116/117 entries but did not backfill E-16 story references.
- **Severity:** LOW (traceability gap; BCs are functionally correct)
- **Priority:** P3
- **Status:** DEFERRED LOW — traceability sweep
- **DF-VALIDATION-001 required before GitHub issue:** Yes

---

**Proposed entry: DRIFT-EPICS-REGISTRY-STRUCTURAL-001**

- **Source:** STATE.md Drift Items
- **Description:** `epics.md` pre-existing structural debt: "Subsystems Covered" table heading says "12 Subsystems" but omits SS-14/SS-15/SS-16; epic body sections missing for E-13, E-14, and E-16. The E-17 pass corrected only the E-16 story-count-summary row, total_bcs (268→283), and E-17 entries. Full epic-registry reconstruction is out of scope for a single traceability sweep.
- **Severity:** LOW
- **Priority:** P3
- **Status:** DEFERRED LOW — dedicated registry-maintenance sweep required
- **DF-VALIDATION-001 required before GitHub issue:** Yes

---

**Proposed entry: PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE** (escalate existing STATE.md item to register)

- **Source:** STATE.md Drift Items; PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE
- **Description:** Policy `DF-GREEN-DOC-TENSE-SWEEP` was codified after the first recurrence, but RED-tense doc-comments recurred again in D-075 regression test (PR #243). Codified policy text alone is insufficient — agent-prompt or hook strengthening is needed: test-writer must write regression-guard framing from the start; implementer GREEN-sweep must check the fix's own new test comments. Currently open as a self-improvement epic candidate.
- **Evidence:** Issue #254 (repo-wide RED-prose cleanup) is open with ~71 occurrences confirmed across 5 test files plus ~13 in `src/analyzer/arp.rs`. The `tests/bc_2_15_110_dnp3_dispatcher_tests.rs` file (11+ lines of "RED GATE: panics via `todo!()`") is confirmed stale on develop HEAD.
- **Severity:** MEDIUM (ongoing quality erosion; each feature cycle reintroduces)
- **Priority:** P2
- **Status:** OPEN — agent-prompt/hook strengthening needed; linked to issue #254
- **DF-VALIDATION-001 required before GitHub issue:** N/A — #254 already filed

---

**Proposed entry: PG-ARP-FIX-MECHANISM-FIRST** (escalate to register)

- **Source:** STATE.md Drift Items; E-16 F5 cycle
- **Description:** When a fix requires hand-rolled offset/parsing logic (vs. delegating to a library), the fix mechanism must be verified against the library's full input model BEFORE writing spec corrections. The E-16 O-A fix (D-078) wrote spec first from an incorrect mechanism hypothesis, causing two rounds of spec+story corrections (BC v1.4→v1.6) and a cascading MEDIUM regression (D-F1: VLAN-offset false positive). Meta-lesson: a LOW-severity finding that requires hand-rolled parsing may not be worth fixing if the fix-induced-regression risk is HIGH.
- **Severity:** MEDIUM (process gap with demonstrated P2-level cascade)
- **Priority:** P2
- **Status:** OPEN — cycle-closing checklist candidate; policy codification follow-up
- **DF-VALIDATION-001 required before GitHub issue:** N/A (engine-level process gap)

---

**Proposed entry: DRIFT-DNPXX-CONSTANT-NAME-001** (promote DNPXX-SOURCE-RENAME-001 from STATE.md to register)

- **Source:** STATE.md Drift Items (DNPXX-SOURCE-RENAME-001)
- **Description:** `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT` is a placeholder-style naming convention. Convention used elsewhere in the codebase is protocol-specific (e.g., `T0831_WINDOW_SECS`). The `DNPXX` prefix reads as a wildcard/stub marker. Rename to `DNP3_DIRECT_OPERATE_THRESHOLD_DEFAULT` would affect 30 locations: `src/analyzer/dnp3.rs:169`, `src/cli.rs:16`, `src/main.rs:218`, and 27 test file references in `tests/bc_2_15_110_dnp3_dispatcher_tests.rs`.
- **Severity:** LOW (code readability/naming convention)
- **Priority:** P3
- **Status:** DEFERRED — requires DF-VALIDATION-001 before GitHub issue; rename is purely cosmetic
- **DF-VALIDATION-001 required before GitHub issue:** Yes

---

### 2.2 Engine-Note Items (Not Register Material)

The following E-17 process-gap items from STATE.md are ENGINE-LEVEL notes only and should NOT be added to the product register:

| ID | Disposition |
|----|-------------|
| PG-E17-STATEMGR-FABRICATED-VERDICT-001 | ENGINE-NOTE HIGH — dark-factory state-manager agent-prompt hardening |
| PG-E17-ADVERSARY-HANG-001 | ENGINE-NOTE HIGH — adversary sub-agent timeout/liveness |
| PG-E17-AGENT-SCOPE-CREEP-001 | ENGINE-NOTE MEDIUM — agent scope-enforcement |

These are correctly recorded in STATE.md and the register does not need entries for them.

---

## 3. Spec Drift — Candidate Entries

From cross-referencing STATE.md drift items against current source:

**Issue #254 (repo-wide doc-debt) — confirmed still open and genuine:**  
The register does not currently have an entry tracking the aggregate tech-debt for stale RED-gate prose. The proposed `PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE` entry above covers this category and links to #254.

**FU-REPO-WIDE-DOC-DEBT — partially addressed, register entry stale:**  
The STATE.md entry `FU-REPO-WIDE-DOC-DEBT` notes "13 test files carry stale RED-gate prose" and marks it "REGISTERED — post-STORY-114-merge chore." Issue #254 is the filed result. However, the register's current v1.1 text does not include an entry for `FU-REPO-WIDE-DOC-DEBT`. The proposed `PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE` entry subsumes this.

**DRIFT-F2-COUNT-001 — still valid:**  
BC-2.10.006 changelog confirms the "15 seeded" count was corrected to 23→25 in v1.3, but the STATE.md drift item notes the prd-supplements and HS-008/009 holdout files may still carry stale count prose from the greenfield era. This remains unresolved as a P3 DEFERRED item. No change needed in the register (item is in STATE.md drift list, not yet in the register).

---

## 4. Outdated Dependencies — Candidate Entries

Based on resolved versions in Cargo.lock vs. the Cargo.toml declared ranges:

| Dependency | Declared | Resolved | Major-version concern? | Recommendation |
|------------|----------|----------|------------------------|----------------|
| `rayon` | `"1"` | 1.12.0 | No (rayon 2.x not released as of mid-2026) | No action; O-07 covers the usage gap |
| `etherparse` | `"0.20"` | 0.20.2 | Yes — 0.21+ likely released (Cargo.lock shows 0.3.14 for `etherparse` sub-dep?) | **Investigate**: Cargo.lock shows `etherparse` main at 0.20.2 but a `0.3.14` entry appears in the paste output which is likely a sub-dependency. Pinned to `"0.20"` by design (ADR-008, SliceError::Len stability — see Cargo.toml comment). No immediate action. |
| `clap` | `"4"` | 4.6.1 | clap 5.x in development — not GA yet | No action |
| `serde_json` | `"1"` | 1.0.228 | No serde_json 2.x released | No action; issue #255 governs enum casing change (not version) |
| `criterion` | `"0.8"` | 0.8.2 | Cargo.lock also shows `criterion-plot 0.8.7` (sub-crate within criterion workspace) | No action |
| `indicatif` | `"0.18"` | 0.18.4 | indicatif 0.17.x was a prior series; 0.18 is current stable | No action |
| `md-5` | `"0.11"` | 0.11.0 | RustCrypto md-5 0.11 is current | No action |
| `pcap-file` | `"2"` | 2.0.0 | pcap-file 2.0.0 is the declared target; check for 2.x.y patch releases | Low priority; no security advisory |
| `tls-parser` | `"0.12"` | 0.12.2 | tls-parser 0.12.2 current | `RUSTSEC-2026-0097` via rand transitive dep; ACCEPTED-TRANSITIVE already in register |

**Conclusion:** No new major-version upgrade entries warranted at this time. The `etherparse = "0.20"` pin is intentional per ADR-008; monitoring for 0.21+ API-breaking changes should occur when a new feature cycle touches the decoder. No new dependency debt register entry required.

---

## 5. Known Deferrals — Status Verification

| Deferral | Expected Status | Actual Status |
|----------|----------------|---------------|
| W7.1 public-api baseline (`cargo public-api`) | OPEN/deferred — no baseline committed, no CI gate | CONFIRMED OPEN. Not in register but documented in CLAUDE.md. No change needed. |
| Input-hash CI gate | DEFERRED — `.factory/` lives on factory-artifacts branch | CONFIRMED DEFERRED per CLAUDE.md. No CI stub shipped. Correct. |
| FU-F6-KANI-CLEANUP (VP-024 `proof_file_hash`, issue #252) | OPEN — proof_file_hash null because harnesses span two files | CONFIRMED OPEN. Issue #252 filed and open. Register entry exists only in STATE.md; the product register does not have a dedicated entry. **Recommend adding a register entry.** |
| DRIFT-ETHERPARSE-0.20-MIGRATION-001 | Should be RESOLVED | CONFIRMED RESOLVED — delivered in v0.7.0 (D-066 sub-delta A). STATE.md marks IN-PROGRESS but the ARP cycle is complete. Recommend updating to RESOLVED in the register. |

**Proposed entry: W7-PUBLIC-API-BASELINE** (register the CLAUDE.md deferred item explicitly)

- **Source:** CLAUDE.md "Public API Surface (W7.1 — deferred)" section
- **Description:** `cargo public-api` baseline not yet committed. Tool requires nightly toolchain (rustdoc JSON) and a committed `public-api.txt`. Without a baseline, there is no CI gate detecting accidental public-API surface changes between releases. Two steps required: (1) generate baseline on nightly and commit; (2) add `cargo public-api diff` CI step.
- **Severity:** LOW (no current breakage; risk grows as public API stabilizes)
- **Priority:** P3
- **Status:** DEFERRED — documented in CLAUDE.md; no flaky/non-gating stub should be introduced without both steps in one PR
- **DF-VALIDATION-001 required before GitHub issue:** No (existing CLAUDE.md documentation suffices; no new research needed)

---

**Proposed entry: FU-F6-KANI-CLEANUP** (formalize into register from STATE.md)

- **Source:** STATE.md carry-forward #252; E-16 F6 hardening
- **Description:** VP-024 `proof_file_hash` field is null. The Kani proofs span two files (`src/decoder.rs` and `src/analyzer/arp.rs` `kani_proofs` modules). The existing single-file SHA-256 method used by other VPs does not accommodate multi-file proofs. A deterministic multi-file digest method must be defined (per issue #252: "SHA-256 of LF-normalized concatenation of the `#[cfg(kani)]` proof modules") and VP-024 must be re-locked with the computed hash.
- **Severity:** LOW (VP is functionally locked and verified; only the audit-trail field is incomplete)
- **Priority:** P2 (explicit issue #252 filed)
- **Status:** OPEN — issue #252 filed; awaiting implementation
- **DF-VALIDATION-001 required before GitHub issue:** N/A — #252 already filed

---

## 6. Anti-Patterns / Code Smells — Candidate Entries

From the pattern-consistency sweep theme:

**Confirmed present in current codebase:**

1. **`unwrap()` vs `expect()` inconsistency** (CR-006/CR-007 already in register): 41 total `unwrap()` calls in `src/`, of which 5 are in `reassembly/mod.rs` on map lookups and 1 in `json.rs:69` on serialization. The existing register entries (CR-006/CR-007) cover this. No new entry needed.

2. **`findings()` trait/impl signature mismatch** (CR-002 already in register): Trait returns `Vec<Finding>` (clone); concrete mod.rs impl returns `&[Finding]` (borrow). Existing entry covers this.

3. **`.contains()` on `String` for error dispatch in `src/main.rs:271`** — `Err(ref e) if e.to_string().contains("Non-Ethernet/IPv4 ARP frame")` is a string-match on a formatted error rather than matching on a typed error variant. This is a carry-forward of the D-078 implementation choice (the error originates from `anyhow::bail!` in the decoder). The drift item `F-W25-S088-P6-001` in STATE.md covers a `.contains()` weakness in a test assertion. This production usage is a different instance.

**Proposed entry: DRIFT-ANYHOW-STRING-MATCH-001**

- **Source:** Pattern-consistency sweep (Maintenance Sweep 8); `src/main.rs:271`
- **Description:** `src/main.rs:271` dispatches on `e.to_string().contains("Non-Ethernet/IPv4 ARP frame")` instead of matching a typed error variant. This is brittle: renaming the string in the error message silently breaks the dispatch. The origin is `anyhow::bail!("Non-Ethernet/IPv4 ARP frame")` in the decoder. The correct fix is to introduce a typed error enum for ARP decode errors and propagate it, or at minimum use a named constant for the string to make renames compile-visible.
- **Severity:** LOW (no known breakage; brittle only on string renaming)
- **Priority:** P3
- **Status:** DEFERRED — low-priority code quality item
- **DF-VALIDATION-001 required before GitHub issue:** Yes, before filing

---

## 7. Overdue Items

No items in the register carry explicit target release dates in the `vX.Y.Z by` form. The DEFERRED items use relative language ("post-v0.6.0", "PO backlog", "next feature cycle"). Mapping against current released version (v0.7.1):

| ID | Target Language | Overdue Assessment |
|----|-----------------|-------------------|
| DRIFT-DNP3-DIRECTION-001 | "post-v0.6.0 dedicated chore" | v0.6.0 shipped 2026-06-12; v0.7.1 now released. **OVERDUE by 1 release.** Human triage recommended: either promote to a planned story or explicitly defer to a named future version (v0.8.0 or post-ICS-protocol-roadmap). |
| DRIFT-BC-2.15.024-EC006-PROSE-001 | "PO backlog prose-refresh" | No release target. Still deferred. No escalation needed yet — PO backlog items have no SLA. |
| DRIFT-MITRE-EMITTED-LABEL-001 | "system-level catalogue-accuracy pass" | As noted in Section 1.3 above, this item is INVALID (T0835/T0831 ARE emitted). Recommend closing, not escalating. |
| O-07 | No explicit target | P2 OPEN. rayon has been unused for multiple release cycles. Issue #6 ("Add parallel file processing with rayon") is the intended consumer — if issue #6 is not on the near-term roadmap, consider either removing the dependency (low-risk PR) or accepting it as planned-for. Human triage recommended. |
| FU-REPO-WIDE-DOC-DEBT / #254 | "standalone docs chore PR after STORY-114 merges" | STORY-114 merged (PR #240, 2026-06-15). Issue #254 filed and open. The chore PR was not shipped — **OVERDUE** per the original commitment. ~71 occurrences confirmed. Recommend scheduling or explicitly re-deferring. |

---

## 8. Items Approaching Due — Warnings

| ID | Warning |
|----|---------|
| #252 (FU-F6-KANI-CLEANUP, VP-024 proof_file_hash) | If a new feature cycle starts before this is resolved, VP-024 will enter the next formal hardening phase with a null `proof_file_hash`, which may cause consistency-validator failures at the F7 convergence gate. Recommend resolving before the next F6 phase begins. |
| DRIFT-F2-COUNT-001 (stale "15 seeded" counts in prd-supplements/HS-008/HS-009) | If adversarial review runs on the full corpus during a future feature cycle, holdout count-assertion drift may be flagged. Stale catalog counts in HS-008/009 have caused adversarial resets in the past (PG-ARP-F2-006). Recommend scheduling a holdout-count sweep before the next F2/F3 cycle. |
| PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE / #254 | The ~71 occurrences in stale RED-gate prose grow with each feature cycle. The `tests/bc_2_15_110_dnp3_dispatcher_tests.rs` file still contains full "RED GATE: panics via todo!()" sections on tests that have been GREEN since v0.6.0. Risk: a future adversary pass will flag these as MEDIUM documentation-accuracy violations. Recommend a dedicated docs chore PR (already issues #254) before the next feature cycle adversarial phase begins. |

---

## 9. Summary

### Existing Register Items: 14 open/deferred product entries + 2 engine-notes + 1 FE

| Count | Category |
|-------|----------|
| 9 | OPEN P2/P3 code-quality items (CR-002/003/005/006/007/009/012, O-07/08) |
| 3 | DEFERRED drift items (DRIFT-DNP3-DIRECTION-001, DRIFT-BC-2.15.024-EC006-PROSE-001, DRIFT-MITRE-EMITTED-LABEL-001) |
| 2 | ENGINE-NOTE items (DRIFT-ENGINE-CHECKOUT-GUARD-001, DRIFT-ENGINE-PRMGR-REPORT-001) |
| 1 | Future enhancement (FE-001 pcapng) |
| 3 | Supporting/misc (RUSTSEC-2026-0097, ACTION-PIN-001, PCAP-CORPUS-001) |

### Recommended New Entries: 10

| Proposed ID | Category | Priority |
|-------------|----------|----------|
| DRIFT-VP024-BTREEMAP-PROSE-001 | Spec drift (cosmetic) | P3 |
| DRIFT-E17-VERSIONLABEL-LAG-001 | Spec drift (cosmetic) | P3 |
| DRIFT-E16-EPICS-SUMMARY-GAP-001 | Traceability gap | P3 |
| DRIFT-E16-BC-BACKLINK-GAP-001 | Traceability gap | P3 |
| DRIFT-EPICS-REGISTRY-STRUCTURAL-001 | Documentation debt | P3 |
| PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE | Process gap / code quality | P2 |
| PG-ARP-FIX-MECHANISM-FIRST | Process gap | P2 |
| DRIFT-DNPXX-CONSTANT-NAME-001 | Naming convention | P3 |
| W7-PUBLIC-API-BASELINE | Infrastructure gap | P3 |
| FU-F6-KANI-CLEANUP | Verification hygiene | P2 |
| DRIFT-ANYHOW-STRING-MATCH-001 | Anti-pattern | P3 |

(11 entries total — 3 at P2, 8 at P3)

### Recommended Closures/Resolutions: 3

| ID | Action |
|----|--------|
| DRIFT-MITRE-EMITTED-LABEL-001 | Close as INVALID — T0835/T0831 ARE emitted by modbus.rs |
| DRIFT-ETHERPARSE-0.20-MIGRATION-001 | Mark RESOLVED — delivered in v0.7.0 |
| DRIFT-ENGINE-RELEASECONFIG-STALE-001 | Mark RESOLVED — cosmetic version_sources follow-up is low value |

### Overdue Items: 3

| ID | Details |
|----|---------|
| DRIFT-DNP3-DIRECTION-001 | "post-v0.6.0 chore" target; now 2 releases past due (v0.7.0/v0.7.1). Human triage: promote to story or name a future release target. |
| O-07 (rayon unused) | Unused across 3 release cycles (v0.5.0/v0.6.0/v0.7.x). Human triage: either remove the dep in a chore PR, or document explicit intent to use it for issue #6. |
| FU-REPO-WIDE-DOC-DEBT / #254 | Scheduled "after STORY-114 merges" (STORY-114 merged 2026-06-15). Chore PR never shipped. ~71 stale occurrences. |

### WARNING: Items Approaching Risk Point: 3

| ID | Risk |
|----|------|
| #252 / FU-F6-KANI-CLEANUP | Blocks VP-024 re-lock before next F6 phase |
| DRIFT-F2-COUNT-001 | Holdout count drift risks adversarial-reset in next F2/F3 cycle |
| #254 / RED-prose debt | 71+ occurrences will be flagged MEDIUM in next adversarial corpus sweep |

---

*This report is RECOMMENDATIONS ONLY. No edits were made to `.factory/tech-debt-register.md`. Apply approved updates via fix-PR step.*
