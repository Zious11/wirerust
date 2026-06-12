---
document_type: tech-debt-register
version: "1.1"
timestamp: 2026-06-01T00:00:00Z
---

# wirerust Tech Debt Register

> Canonical register of open technical debt items. Items are recorded here for
> tracking and future prioritization. Per DF-VALIDATION-001, no item may be filed
> as a GitHub issue without research-agent validation confirming the finding is
> sound and still open on the current develop branch.

## Open Items

| ID | Description | Priority | Source | Status |
|----|-------------|----------|--------|--------|
| O-07 | `rayon` declared in Cargo.toml but unused in `src/` — dead dependency | P2 | adversarial Pass 1 (LOW finding) | OPEN |
| O-08 | `src/analyzer/dns.rs` module doc-comment stale — references removed behavior | P3 | adversarial Pass 29 (observation O-1); recorded in domain-debt.md | OPEN |
| CR-001 | [Phase-5 secondary review, MEDIUM] dispatcher `pub` analyzer fields should be encapsulated before public-API hardening (W7.1 gate). Fields are currently `pub` for test access; encapsulation via accessor methods or `pub(crate)` + `#[cfg(test)]` should happen before the W7.1 public-API surface is frozen. | P2 | Phase-5 secondary code-review (code-reviewer agent, distinct-lens sonnet pass) | CLOSED — merged PR #177 (02e9c00) 2026-06-01 |
| CR-010 | [Phase-5 secondary review, MEDIUM] `tls/mod.rs` `try_parse_records` allocates record byte-slices before the `0x16` content-type guard fires — on long-lived sessions with many non-TLS packets this performs unnecessary heap allocation per packet. Consider guard-before-allocate reorder for the hot path. | P2 | Phase-5 secondary code-review | CLOSED — merged PR #176 (c115d61) 2026-06-01 |
| CR-011 | [Phase-5 secondary review, MEDIUM] No multi-analyzer end-to-end test exercises HTTP + TLS + DNS + reassembly + reporter together in a single packet stream. Individual-analyzer and integration-level tests cover narrower slices; a combined smoke-test would catch interaction bugs earlier. | P2 | Phase-5 secondary code-review | CLOSED — merged PR #178 (eab2eb1) 2026-06-01 |
| CR-002 | [Phase-5 secondary review, LOW] `handler.findings()` clones the findings Vec on every call; callers that only need to read findings may be able to borrow instead. | P3 | Phase-5 secondary code-review | OPEN |
| CR-003 | [Phase-5 secondary review, LOW] `ThreatCategory::Persistence` doc-comment describes Execution-class behavior — the variant is never constructed in the current codebase. Supersedes ADV-IMPL-P12-LOW-001 and ADV-IMPL-P01-LOW-001 (cosmetic/accepted class). | P3 | Phase-5 secondary code-review (absorbed from accepted adversary findings) | OPEN |
| CR-005 | [Phase-5 secondary review, LOW/SUGGESTION] `resolve_targets` could be written non-recursively to avoid stack depth concerns on very deep call graphs. Current depth is bounded in practice but a non-recursive form would be more defensive. | P3 | Phase-5 secondary code-review | OPEN |
| CR-006 | [Phase-5 secondary review, LOW] 5 `unwrap()` calls in `mod.rs` should be converted to `expect("...")` with descriptive panic messages for easier debugging when they fire. | P3 | Phase-5 secondary code-review | OPEN |
| CR-007 | [Phase-5 secondary review, LOW] `json.rs` infallible unwrap comment should be replaced with a more explicit `expect("...")` so the invariant is self-documenting at the call site. | P3 | Phase-5 secondary code-review | OPEN |
| CR-009 | [Phase-5 secondary review, LOW] HTTP analyzer traversal-encoding variants (chunked + compressed edge cases) are not exercised by current tests. Adding a few synthetic fixtures would raise confidence in the HTTP traversal path. | P3 | Phase-5 secondary code-review | OPEN |
| CR-012 | [Phase-5 secondary review, LOW] HashMap accessor consistency — some call sites use `entry().or_insert()` pattern while others use `get()` + `insert()` separately. Standardizing on one idiom would improve readability. | P3 | Phase-5 secondary code-review | OPEN |
| DRIFT-DNP3-DIRECTION-001 | DNP3 `source_ip` resolution is port-20000-heuristic-only. Direction-aware resolution (matching `modbus.rs` ~355-382, using TCP Direction signal) is NOT resolved by STORY-110 — threading `direction` into `Dnp3Analyzer::on_data` ripples into ~100 call sites across STORY-106..109 and is out of STORY-110 AC scope (AC-001..011 contain no direction-threading criterion; dispatcher arm calls `dnp3.on_data(flow_key, data, timestamp)` without direction). Current behavior: correct for standard flows (one endpoint on port 20000); returns `lower_ip` when neither endpoint is on port 20000 (non-standard/proxied capture). Documented at `src/analyzer/dnp3.rs` `resolve_master_ip`. Source: STORY-108 adversarial P2; re-deferred by STORY-110 adversarial P1 finding F-110-P1-001. Per DF-VALIDATION-001: do NOT file as GitHub issue without research-agent validation. | P3 | STORY-108 adversarial review (P2 master-resolution test-vacuity finding); re-deferred by STORY-110 adv P1 F-110-P1-001 | DEFERRED — post-v0.6.0 dedicated "DNP3 direction-aware source resolution" chore (Feature-8 follow-up backlog). Reason: threading TCP Direction into Dnp3Analyzer::on_data + direction-aware resolve_master_ip ripples into ~100 STORY-106..109 on_data call sites; out of STORY-110 AC scope. Port-20000 heuristic is correct for standard DNP3 flows (one endpoint on 20000); mis-resolves only when neither endpoint is on 20000 (non-standard/proxied capture). Re-deferred post-STORY-110 adversarial Pass-1 F-110-P1-001. |
| DRIFT-MITRE-EMITTED-LABEL-001 | Kani `EMITTED_IDS` array in `src/mitre.rs` labels T0835 and T0831 as analyzer-emitted, but neither is actually emitted by any analyzer (13 actual emitted IDs vs 15 labeled). Pre-existing, cross-story. VP-007 Sub-B proof is sound (asserts resolvability only, not emission). The label is inaccurate and should be corrected in a system-level catalogue-accuracy pass. Per DF-VALIDATION-001: do NOT file as GitHub issue without research-agent validation. | P3 | STORY-109 adversarial P13 observation | DEFERRED — target: system-level catalogue-accuracy pass; severity LOW |
| DRIFT-BC-2.15.024-EC006-PROSE-001 | BC-2.15.024 EC-006/Precondition-6 prose states that a bailed-flow frame increments `parse_errors`/`malformed_in_window`, which conflicts with BC-2.15.009 PC5 (immediate no-op). Implemented and tested behavior is the no-op (correct); STORY-109 EC-006 story-body corrected. BC-2.15.024 prose-refresh deferred to PO backlog. Per DF-VALIDATION-001: do NOT file as GitHub issue without research-agent validation. | P3 | STORY-109 adversarial review + BC-2.15.009 PC5 conflict | DEFERRED — target: PO backlog prose-refresh; severity LOW |

## Future Enhancements / Deferred v2 Ideas

> Items here are **parked observations** — not actioned, not filed as GitHub issues, and not
> in scope for any current cycle. Per DF-VALIDATION-001, filing as a GitHub issue requires
> research-agent validation first. Human decision to defer is noted per item.

| ID | Title | Observation | Scope / Direction | Status |
|----|-------|-------------|-------------------|--------|
| FE-001 | pcapng input format support | wirerust reads ONLY classic libpcap (.pcap) and rejects .pcapng files with "Failed to parse pcap header / wrong magic number" (pcapng magic `0a0d0d0a`). Hit during 2026-06-08 post-v0.1.0 local demo: both Wireshark and modern/macOS tcpdump default to pcapng, so a user supplying their own capture will commonly hit this. Also note: nanosecond-precision classic pcap (magic `a1b23c4d`) was not verified as supported. | Deferred to v2 per human decision (2026-06-08). Not in scope for v0.1.x. Minimum interim improvement (if ever desired, also deferred): clearer error message suggesting "convert to classic .pcap (e.g. `editcap -F libpcap in.pcapng out.pcap`)" instead of the raw magic-number error. Do NOT implement now. | deferred / v2 / not-filed |

**OBS (secondary, parked):** TLS SNI, JA3 fingerprints, and cipher suite are extracted internally
but only surfaced via threat findings, not as a benign connection inventory. Candidate for a v2
docs clarification or an optional inventory output mode. Parked, not actioned.

## Closed / Refuted Items

| ID | Description | Resolution |
|----|-------------|------------|
| CR-004 | [Phase-5 secondary review] "Blocking" claim: inner-HashMap JSON key ordering is non-deterministic — review alleged indexmap dependency causes non-deterministic `Value::Object` maps in serde_json. **EMPIRICALLY REFUTED — FALSE POSITIVE.** Orchestrator investigation confirmed: serde_json's dependency tree is itoa/memchr/serde/serde_core/ryu — NO indexmap dependency; `preserve_order` feature is OFF; therefore `serde_json::Map<K,V>` = `BTreeMap<K,V>` = alphabetically sorted keys. The indexmap crate in Cargo.lock is pulled by toml_edit/wasm-*/wit-* toolchain crates, not by serde_json. Binary run twice in separate processes (different HashMap seeds) on 3 live TLS fixtures produced byte-identical JSON output. All inner detail maps route through `json!()` / `.to_value()` → `Value::Object` → BTreeMap → deterministic. NOT a defect. Recorded here so the claim is not re-investigated in future sessions. | REFUTED 2026-06-01 — do NOT re-investigate without new evidence showing serde_json preserve_order was enabled |
