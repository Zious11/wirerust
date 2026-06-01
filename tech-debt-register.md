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
| CR-001 | [Phase-5 secondary review, MEDIUM] dispatcher `pub` analyzer fields should be encapsulated before public-API hardening (W7.1 gate). Fields are currently `pub` for test access; encapsulation via accessor methods or `pub(crate)` + `#[cfg(test)]` should happen before the W7.1 public-API surface is frozen. | P2 | Phase-5 secondary code-review (code-reviewer agent, distinct-lens sonnet pass) | OPEN |
| CR-010 | [Phase-5 secondary review, MEDIUM] `tls/mod.rs` `try_parse_records` allocates record byte-slices before the `0x16` content-type guard fires — on long-lived sessions with many non-TLS packets this performs unnecessary heap allocation per packet. Consider guard-before-allocate reorder for the hot path. | P2 | Phase-5 secondary code-review | OPEN |
| CR-011 | [Phase-5 secondary review, MEDIUM] No multi-analyzer end-to-end test exercises HTTP + TLS + DNS + reassembly + reporter together in a single packet stream. Individual-analyzer and integration-level tests cover narrower slices; a combined smoke-test would catch interaction bugs earlier. | P2 | Phase-5 secondary code-review | OPEN |
| CR-002 | [Phase-5 secondary review, LOW] `handler.findings()` clones the findings Vec on every call; callers that only need to read findings may be able to borrow instead. | P3 | Phase-5 secondary code-review | OPEN |
| CR-003 | [Phase-5 secondary review, LOW] `ThreatCategory::Persistence` doc-comment describes Execution-class behavior — the variant is never constructed in the current codebase. Supersedes ADV-IMPL-P12-LOW-001 and ADV-IMPL-P01-LOW-001 (cosmetic/accepted class). | P3 | Phase-5 secondary code-review (absorbed from accepted adversary findings) | OPEN |
| CR-005 | [Phase-5 secondary review, LOW/SUGGESTION] `resolve_targets` could be written non-recursively to avoid stack depth concerns on very deep call graphs. Current depth is bounded in practice but a non-recursive form would be more defensive. | P3 | Phase-5 secondary code-review | OPEN |
| CR-006 | [Phase-5 secondary review, LOW] 5 `unwrap()` calls in `mod.rs` should be converted to `expect("...")` with descriptive panic messages for easier debugging when they fire. | P3 | Phase-5 secondary code-review | OPEN |
| CR-007 | [Phase-5 secondary review, LOW] `json.rs` infallible unwrap comment should be replaced with a more explicit `expect("...")` so the invariant is self-documenting at the call site. | P3 | Phase-5 secondary code-review | OPEN |
| CR-009 | [Phase-5 secondary review, LOW] HTTP analyzer traversal-encoding variants (chunked + compressed edge cases) are not exercised by current tests. Adding a few synthetic fixtures would raise confidence in the HTTP traversal path. | P3 | Phase-5 secondary code-review | OPEN |
| CR-012 | [Phase-5 secondary review, LOW] HashMap accessor consistency — some call sites use `entry().or_insert()` pattern while others use `get()` + `insert()` separately. Standardizing on one idiom would improve readability. | P3 | Phase-5 secondary code-review | OPEN |

## Closed / Refuted Items

| ID | Description | Resolution |
|----|-------------|------------|
| CR-004 | [Phase-5 secondary review] "Blocking" claim: inner-HashMap JSON key ordering is non-deterministic — review alleged indexmap dependency causes non-deterministic `Value::Object` maps in serde_json. **EMPIRICALLY REFUTED — FALSE POSITIVE.** Orchestrator investigation confirmed: serde_json's dependency tree is itoa/memchr/serde/serde_core/ryu — NO indexmap dependency; `preserve_order` feature is OFF; therefore `serde_json::Map<K,V>` = `BTreeMap<K,V>` = alphabetically sorted keys. The indexmap crate in Cargo.lock is pulled by toml_edit/wasm-*/wit-* toolchain crates, not by serde_json. Binary run twice in separate processes (different HashMap seeds) on 3 live TLS fixtures produced byte-identical JSON output. All inner detail maps route through `json!()` / `.to_value()` → `Value::Object` → BTreeMap → deterministic. NOT a defect. Recorded here so the claim is not re-investigated in future sessions. | REFUTED 2026-06-01 — do NOT re-investigate without new evidence showing serde_json preserve_order was enabled |
