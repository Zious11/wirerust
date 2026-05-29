---
document_type: story
story_id: "STORY-031"
epic_id: "E-3"
version: "1.6"
status: draft
producer: story-writer
timestamp: 2026-05-27T12:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.001.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.002.md
  - .factory/specs/behavioral-contracts/ss-05/BC-2.05.003.md
input-hash: "4626b8a"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-021]
blocks: [STORY-032, STORY-033]
behavioral_contracts:
  - BC-2.05.001
  - BC-2.05.002
  - BC-2.05.003
verification_properties: [VP-004]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 12
target_module: src/dispatcher.rs
subsystems: [SS-05]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **Execute:** `/vsdd-factory:deliver-story STORY-031`

# STORY-031: Content-First Classification — TLS Signature, HTTP Method Prefix, Port Fallback

## Narrative
- **As a** forensic analyst
- **I want to** have TCP flows classified by their payload content first (TLS record type then HTTP method prefix) and only fall back to port-based heuristics when content is insufficient
- **So that** port-obfuscation attacks — such as running TLS on port 80 or HTTP on port 443 — are identified by what the data actually is, not by what port convention suggests

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.05.001 | TLS Content Signature Routes Flow to TLS Regardless of Port |
| BC-2.05.002 | HTTP Method Prefix Routes Flow to HTTP |
| BC-2.05.003 | Port Fallback: 443/8443->TLS, 80/8080->HTTP When Content Insufficient |

## Acceptance Criteria

### AC-001 (traces to BC-2.05.001 postcondition 1)
When the first bytes of reassembled TCP content have `data.len() >= 5 AND data[0] == 0x16 AND data[1] == 0x03`, the `classify` function returns `DispatchTarget::Tls` regardless of the flow's port numbers.
- **Test:** `test_tls_content_routes_tls_on_port_443` + `test_tls_content_wins_over_port_8080`

### AC-002 (traces to BC-2.05.001 invariant 2-3)
Content-first dispatch (INV-2) takes precedence: a TLS ClientHello on port 80 is routed to Tls, not Http. Only bytes 0 and 1 are checked (`0x16 0x03`); byte 2 (minor version) and bytes 3-4 (record length) are NOT checked.
- **Test:** `test_dispatcher_content_detection_tls_on_port_80`

### AC-003 (traces to BC-2.05.001 precondition 2)
When `data.len() < 5`, the TLS content check is skipped (falls through to HTTP check or port fallback).
- **Test:** `test_tls_check_skipped_below_len_5`

### AC-004 (traces to BC-2.05.002 postcondition 1)
When the TLS content check fails and `data.starts_with` one of the 10 HTTP method/version prefix byte strings (`b"GET "`, `b"POST "`, `b"PUT "`, `b"DELETE "`, `b"HEAD "`, `b"OPTIONS "`, `b"PATCH "`, `b"CONNECT "`, `b"TRACE "`, `b"HTTP/"`), the `classify` function returns `DispatchTarget::Http`.
- **Test:** `test_dispatcher_routes_http` (single-prefix legacy) + `test_all_http_method_prefixes_route_to_http` (comprehensive table-driven, all 10 prefixes including `HTTP/` response-first, covers BC-2.05.002 invariant 3 and EC-008)

### AC-005 (traces to BC-2.05.002 invariant 2-3)
`b"HTTP/"` handles server-initiated or response-first flows. Method prefix strings include a trailing space — `b"GET"` (3 bytes, no space) does NOT match. The comparison is case-sensitive; `b"get "` does not match.
- **Test:** `test_http_no_space_does_not_match` (Inv-3 — trailing space + case sensitivity) + `test_all_http_method_prefixes_route_to_http` (Inv-2 — HTTP/ response-first variant)

### AC-006 (traces to BC-2.05.002 invariant 1)
The HTTP content signature check is evaluated AFTER the TLS check (INV-2). Data starting with `0x16 0x03` cannot trigger the HTTP match because TLS check is first.
- **Test:** `test_tls_takes_priority_over_http_methods_check`

### AC-007 (traces to BC-2.05.003 postcondition 1-2)
When both TLS and HTTP content checks fail, `classify` falls back to port-based heuristics: ports 443 or 8443 return `DispatchTarget::Tls`; ports 80 or 8080 return `DispatchTarget::Http`.
- **Test:** `test_port_fallback_443_to_tls` + `test_port_fallback_8443_to_tls` + `test_port_fallback_80_to_http` + `test_port_fallback_8080_to_http`

### AC-008 (traces to BC-2.05.003 invariant 1-2)
TLS port check (443/8443) is evaluated before HTTP port check (80/8080) per source order. Port lookup uses `flow_key.lower_port()` and `flow_key.upper_port()` — the canonically ordered pair — so a flow on (src=8443, dst=9000) finds 8443 in the port slice.
- **Test:** `test_port_fallback_uses_canonical_port_ordering`

### AC-009 (traces to BC-2.05.003 invariant 3)
Port fallback is only reached when BOTH content checks fail (INV-2). A valid HTTP GET request on port 443 is classified as Http by content, not as Tls by port.
- **Test:** `test_http_content_on_port_443_routes_to_http`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| classify (content check logic) | src/dispatcher.rs:90-117 | pure-core (pure classification logic, no state mutation) |
| StreamDispatcher.on_data | src/dispatcher.rs:120-169 | effectful-shell (calls classify, mutates routes cache) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | TLS ClientHello on port 80 | Routed to Tls (content wins over port) |
| EC-002 | TLS on port 443 | Routed to Tls via content signature |
| EC-003 | data starts with 0x16 0x03 but is not valid TLS | Routed to Tls; TlsAnalyzer handles parse error |
| EC-004 | data.len() == 4 (one byte short) | Falls through to HTTP method check (test: `test_tls_check_skipped_below_len_5`) |
| EC-005 | data[0] == 0x16 but data[1] != 0x03 | Falls through to HTTP check (test: `test_tls_check_requires_byte1_equals_0x03`) |
| EC-006 | b"GET /index HTTP/1.1" on port 8081 | Routed to Http |
| EC-007 | b"GET" (no space, 3 bytes) on port 9999 | Not matched; falls to port fallback; returns None (port unknown) |
| EC-008 | b"HTTP/1.1 200 OK" (response-first) on port 9999 | Routed to Http (test: `test_all_http_method_prefixes_route_to_http` — HTTP/ prefix variant) |
| EC-009 | Unknown bytes on port 443 | Routed to Tls (port fallback) |
| EC-010 | Unknown bytes on port 8080 | Routed to Http (port fallback) (test: `test_port_fallback_8080_to_http`) |
| EC-011 | b"GET " on port 443 | Routed to Http (content wins over port 443 hint) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/dispatcher.rs (classify function) | pure-core | No I/O, no global state mutation; deterministic |
| src/dispatcher.rs (on_data) | effectful-shell | Mutates routes HashMap via cache insert |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| Referenced code (dispatcher.rs:90-169) | ~3,000 |
| Test files (dispatcher_tests.rs) | ~3,000 |
| BC files (3 BCs) | ~3,000 |
| Tool outputs overhead | ~1,500 |
| **Total** | **~13,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~7%** |

## Tasks (MANDATORY)

1. [x] Write failing tests for AC-001 through AC-009 (test-writer)
2. [x] Verify Red Gate: all tests fail before implementation
3. [x] Implement TLS content signature check per BC-2.05.001 (`data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03`; returns Tls regardless of port)
4. [x] Implement HTTP method prefix check per BC-2.05.002 (10 prefixes including trailing space; `data.starts_with`; case-sensitive; after TLS check)
5. [x] Implement port fallback per BC-2.05.003 (443/8443->Tls, 80/8080->Http; TLS ports checked before HTTP ports; only reached when both content checks fail)
6. [x] Confirm TLS check takes priority over HTTP check
7. [x] Confirm content classification takes priority over port fallback
8. [x] Run all tests; verify all pass
9. [x] Update STATE.md
10. [x] (POST-PASS-1) Add 4 port-fallback branch tests (443/8443/80/8080) to cover BC-2.05.003 PC1-2 (F-W12P1-001)
11. [x] Add table-driven test for all 10 HTTP method prefixes including HTTP/ response-first (F-W12P1-002, F-W12P1-007)
12. [x] Add isolated `test_tls_check_skipped_below_len_5` (4-byte boundary, port 9999) — split from AC-007 test (F-W12P1-003, F-W12P1-004)
13. [x] Add `test_tls_check_requires_byte1_equals_0x03` for EC-005 (F-W12P1-006)
14. [x] Fix canonicalization docstring at `test_port_fallback_uses_canonical_port_ordering` + add explicit lower_port/upper_port assertions (F-W12P1-005)
15. [x] Strengthen `test_http_no_space_does_not_match` with positive control sub-case (F-W12P1-009)
16. [x] Rename `test_dispatcher_routes_tls` → `test_tls_content_wins_over_port_8080`; add `test_tls_content_routes_tls_on_port_443` baseline (Obs-4)
17. [x] Fix story line citations for classify (90-116 → 90-117) and on_data (120-160 → 120-169) (F-W12P1-010, F-W12P1-011)
18. [x] (POST-PASS-2 ADDITIONS) Remove 3 stale `test_dispatcher_port_fallback_short_data` references from AC-007 trace, Architecture Compliance Rules, and File Structure Requirements (F-W12P2-001 — rename orphan from pass-1)
19. [x] (POST-PASS-3) BC-2.05.002 re-anchor (DF-SIBLING-SWEEP-001 v2 BC pre-merge re-anchor doctrine extended) — added `test_all_http_method_prefixes_route_to_http` to VP-004 + Architecture Anchors; canonical-ordering test strengthened with parse_error_count discriminator (F-W12P3-001, F-W12P3-002)
20. [x] (POST-PASS-4) BC anchor-completeness extended to BC-2.05.001 + BC-2.05.003 (was BC-2.05.002 only in pass-3); TLS-bound port-fallback tests strengthened with `tls.parse_error_count() > 0` positive discriminator using `[0x16, 0x04, 0x01, 0x00, 0x01, 0xFF]` complete-record payload (5-byte non-TLS `[0x00..0x04]` didn't trigger TLS parse_errors due to truncated-record semantic); AC-005 trace updated to cite both Inv-2 and Inv-3 covering tests (F-W12P4-001, F-W12P4-002, F-W12P4-003, Obs-1)
21. [x] (POST-PASS-6) BC-2.05.002 EC-001 anchor fix (was port-9999 test cited for port-443 scenario; now correctly cites `test_http_content_on_port_443_routes_to_http`). Sibling sweep confirmed no other EC mis-anchors across 3 BCs. Codification candidate: anchor-completeness needs EC-scenario-match sub-rule (F-W12P6-001, F-W12P6-OBS-003).

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story in E-3 | N/A | N/A | N/A |
| STORY-021 (W11.L1) | BC pre-merge re-anchor doctrine: re-anchor BCs before merging any per-story adversarial fix | Always re-verify BC frontmatter array is non-empty before transitioning story to `ready` | Stale BC arrays cause spec-first gate failures |
| STORY-021 (W11.L2) | DF-ADVERSARY-METHODOLOGY-001 — adversarial review uses absolute paths for all grep/file references | Use absolute paths in all sibling-sweep grep commands and story references | Relative paths in sibling-sweeps produced false-negative grep results |
| STORY-021 (W11.L4) | Source-docstring propagation: when story body changes (line citations, AC traces), update any docstrings in test files that mirror those citations | Keep test docstrings in sync with story AC traces — if a test is renamed or re-anchored, update the story reference in the same commit | Silent divergence between test docstrings and story ACs is a class of finding in adversarial passes |
| STORY-021 (W11.L5) | Implementer-as-PR-executor pattern: the implementer writes the code AND opens the PR in the same dispatch | Use this pattern for STORY-031 PR (task 90) — do not dispatch a separate PR-creation agent | Splitting implementer and PR-opener adds a handoff gap where STATE.md can diverge |
| (process lesson) | When pass-1 renames a test, story-writer MUST grep ALL story sections (AC traces, Architecture Compliance Rules, File Structure Requirements, body prose) for the OLD test name and update each. Pass-1's sibling-sweep checked test file but not all story sections. Pattern recurrence indicates DF-SIBLING-SWEEP-001 v2 enforcement gap. | F-W12P2-001 in-pass resolution |
| (process lesson — codification candidate) | DF-SIBLING-SWEEP-001 v2 "BC pre-merge re-anchor" should explicitly cover anchor list COMPLETENESS, not just freshness: when a story adds a NEW test that covers an existing BC, the BC must be re-anchored to cite it (even if existing anchors are still valid). Pass-2 sweep updated BCs whose tests were renamed but missed BC-2.05.002 where pass-1 added a new comprehensive test without renaming anything | F-W12P3-003 process-gap |
| (process lesson — closure) | DF-SIBLING-SWEEP-001 v2 "BC pre-merge re-anchor" + anchor-completeness doctrine fully exercised in passes 2/3/4: BC-2.05.001 (pass-2), BC-2.05.002 (pass-3), BC-2.05.001/2/3 anchor expansion (pass-4). Pattern: each pass found the same gap in next sibling. Final codification: doctrine must apply to ALL BCs in story `behavioral_contracts:` frontmatter in a SINGLE sweep, not iteratively. | F-W12P4-001 closed; doctrine extended to anchor-completeness |
| (deferred observations — Obs-3/4/5) | Obs-3 (EC-003/EC-006 lack test citations): defer — both are implicitly covered. Obs-4 (`test_zero_attempt_budget_classifies_nothing` name misleading): defer — pre-existing P2.11 test, out of STORY-031 scope. Obs-5 (PSI codification candidate uncodified): realized via task 20 above and the PSI closure row. | pass-4 deferred per low severity |
| (process lesson — codification candidate v2) | Anchor-completeness doctrine needs sub-rule: "EC citations must EXACTLY exercise the scenario described in the EC row (specific port/value/condition named), not just the parent BC capability." Pass-6 caught BC-2.05.002 EC-001 citing port-9999 test for port-443 scenario. | F-W12P6-001 resolved; F-W12P6-OBS-003 for cycle-close codification |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Content-first takes precedence over port-based classification at all times (INV-2) | ADR 0001 / BC-2.05.001 invariant 2 | Unit tests: AC-002, AC-009 |
| TLS check (0x16 0x03) is first; HTTP check is second; port fallback is last | BC-2.05.002 invariant 1, BC-2.05.003 invariant 3 | Code review: source order in classify function |
| Loose TLS gate: only bytes 0 and 1 are checked; bytes 2-4 are NOT checked | BC-2.05.001 invariant 3 | Code review: confirm no additional byte checks |
| HTTP method prefixes include trailing space (e.g., `b"GET "` not `b"GET"`) | BC-2.05.002 invariant 3 | Code review: confirm prefix strings |
| All 10 HTTP method/version prefixes must trigger Http routing — coverage at AC-004 via `test_all_http_method_prefixes_route_to_http` table-driven test | BC-2.05.002 PC1, invariant 3 | Test enumerates `[GET, POST, PUT, DELETE, HEAD, OPTIONS, PATCH, CONNECT, TRACE, HTTP/]` |
| All 4 port-fallback branches (443→Tls, 8443→Tls, 80→Http, 8080→Http) must have explicit tests | BC-2.05.003 PC1-2, F-W12P1-001 | 4 distinct test functions named `test_port_fallback_{443,8443,80,8080}_to_*` |
| AC-003 (TLS length gate) and AC-007 (port fallback) MUST have separate tests — single test cannot independently verify both | F-W12P1-003 | `test_tls_check_skipped_below_len_5` (AC-003) vs `test_port_fallback_{443,8443,80,8080}_to_*` tests (AC-007) |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust std | 2024 edition (stable) | slice::starts_with, indexing |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| src/dispatcher.rs | modify | classify function (90-117): TLS check, HTTP prefix check, port fallback |
| tests/dispatcher_tests.rs | modify | Add/rename: `test_tls_content_routes_tls_on_port_443` (baseline TLS port 443), `test_tls_content_wins_over_port_8080` (renamed from `test_dispatcher_routes_tls`; content priority over Http port), `test_port_fallback_443_to_tls`, `test_port_fallback_8443_to_tls`, `test_port_fallback_80_to_http`, `test_port_fallback_8080_to_http`, `test_tls_check_skipped_below_len_5` (4-byte boundary), `test_tls_check_requires_byte1_equals_0x03` (EC-005), `test_all_http_method_prefixes_route_to_http` (table-driven all 10 prefixes); existing: test_dispatcher_content_detection_tls_on_port_80, test_dispatcher_routes_http, test_http_content_on_port_443_routes_to_http |

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-05-21 | story-writer | Initial story decomposition |
| 1.1 | 2026-05-27 | story-writer | Pass-1 adversarial remediation: (1) AC-001 trace updated from `test_dispatcher_routes_tls` (renamed) to `test_tls_content_routes_tls_on_port_443` + `test_tls_content_wins_over_port_8080`; (2) AC-003 trace updated to `test_tls_check_skipped_below_len_5` (isolated, F-W12P1-003); (3) AC-004 trace expanded with `test_all_http_method_prefixes_route_to_http` (F-W12P1-002); (4) AC-007 trace expanded with all 4 port-fallback branch tests (F-W12P1-001); (5) Architecture Mapping line citations fixed: classify 90-116→90-117, on_data 120-160→120-169 (F-W12P1-010/011); (6) File Structure Requirements line citations and test list updated; (7) Edge Case table: EC-004, EC-005, EC-008, EC-010 test citations added; (8) Architecture Compliance Rules: 3 new rules added for 10-prefix coverage, 4 port-fallback branch tests, and AC-003/AC-007 test separation; (9) Tasks 10-17 appended (all completed); (10) PSI updated with STORY-021 W11 learnings (W11.L1, W11.L2, W11.L4, W11.L5) |
| 1.2 | 2026-05-27 | story-writer | Pass-2 adversarial remediation (F-W12P2-001): removed 3 stale references to `test_dispatcher_port_fallback_short_data` (renamed to `test_port_fallback_443_to_tls` in pass-1): (1) AC-007 Test field — old name dropped, 4 live tests retained; (2) Architecture Compliance Rules row — example updated to `test_port_fallback_{443,8443,80,8080}_to_*` family; (3) File Structure Requirements existing list — stale entry removed; Task 18 appended (completed); PSI updated with process lesson on sibling-sweep story-body coverage gap |
| 1.3 | 2026-05-27 | story-writer | Pass-3 PO commit (factory 37ca765) — BC-2.05.002 re-anchor per DF-SIBLING-SWEEP-001 v2 extended doctrine: added `test_all_http_method_prefixes_route_to_http` to VP-004 + Architecture Anchors; canonical-ordering test strengthened with parse_error_count discriminator (F-W12P3-001, F-W12P3-002); Task 19 appended; PSI updated with F-W12P3-003 process-gap codification candidate for anchor COMPLETENESS rule |
| 1.4 | 2026-05-27 | story-writer | Pass-4 PO commit (factory d143939) — BC-2.05.001/002/003 all bumped to v1.4; AC-005 trace expanded to cite both Inv-2 (`test_all_http_method_prefixes_route_to_http`) and Inv-3 (`test_http_no_space_does_not_match`) covering tests (Obs-1); Task 20 appended recording pass-4 anchor-completeness sweep and TLS port-fallback discriminator strengthening; PSI closure row added codifying that anchor-completeness doctrine must apply to ALL BCs in a single sweep, not iteratively; Obs-3/4/5 deferred as low-severity |
| 1.5 | 2026-05-27 | story-writer | Pass-6 PO commit (factory 9339318) — BC-2.05.002 v1.5 input reflected; Task 21 appended: EC-001 anchor fix (port-9999 test replaced with `test_http_content_on_port_443_routes_to_http` matching port-443 scenario); sibling sweep confirmed no other EC mis-anchors across 3 BCs; PSI codification candidate row added for EC-scenario-match sub-rule (F-W12P6-001, F-W12P6-OBS-003) |
| 1.6 | 2026-05-28 | story-writer | DF-SIBLING-SWEEP-001 v4 propagation — BC-2.05.001 v1.5 and BC-2.05.003 v1.5 (EC table inline test citations added); input-hash recomputed: `81248d8` → `2c4392a` (sha256 over sorted cited-BC files BC-2.05.001/002/003, first 7 chars). No AC citation changes required. |
| 1.7 | 2026-05-29 | state-manager | input-hash corrected via canonical bin/compute-input-hash --update (prior value `2c4392a` was hand-computed sha256 over sorted inputs-file list; tool uses MD5 over inputs-order file list). New value: `4626b8a`. |
