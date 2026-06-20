---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-19T04:00:00Z
cycle: feature-pcapng-reader
inputs: [STATE.md]
input-hash: pending
traces_to: STATE.md
---

# Lessons Learned — feature-pcapng-reader

<!-- Durable lessons from this cycle for future VSDD factory runs.
     Organized by category: agent-level, process-level, infrastructure-level.
     Each lesson is numbered continuously and includes the pass/burst
     where it was discovered. -->

## Agent-Level

1. **[process-gap] State-manager must not edit spec/ADR content** — During the D-139 commit burst the state-manager directly edited ADR content (mergecap hint wording) in addition to state tracking files. State-manager scope is restricted to state/index files (STATE.md, cycle logs, manifests, burst-logs). Spec and ADR content is the architect's lane. The ADR-009 edit had to be reverted by the architect and re-applied correctly, adding a reconciliation round-trip. Always route spec/ADR edits through the architect agent.
   _Discovered: D-139 burst, 2026-06-19. Reconciled: D-140, 2026-06-19._

## Process-Level

2. **[process-gap] BCs must not reach WRITTEN status with VP-NNN = — (O-1 from ADV-F2-PASS1)** — All 10 new BCs (BC-2.01.009..018) were advanced to `[WRITTEN]` status with all VP-NNN cells set to `—` (unassigned). The factory pipeline's convergence rubric (DF-CANONICAL-FRAME-HOLDOUT-001) blocks convergence for any BC without a VP assignment. Advancing a BC to WRITTEN without completing VP assignment and holdout-fixture designation is a process gap that creates a hard blocker at convergence. Going forward: VP-NNN assignment, VP-INDEX registration, and holdout fixture designation MUST be completed as part of the same spec-evolution burst that writes the BC content, before the BC is marked WRITTEN.
   _Discovered: adversarial spec review pass-1 (O-1), 2026-06-19. Decision: D-141._

3. **[process-gap] error-taxonomy input-hash must not be left as N/A (O-2 from ADV-F2-PASS1)** — The error-taxonomy file's `input-hash` field was set to `N/A` rather than a computed 7-character hash per DF-INPUT-HASH-CANONICAL-001. The error taxonomy is an input file for multiple story files; any story listing error-taxonomy as an input will compute an incorrect hash if the taxonomy's own hash is absent. Going forward: `bin/compute-input-hash` MUST be run to populate the hash for every factory artifact that serves as a story input, before F3 story decomposition begins.
   _Discovered: adversarial spec review pass-1 (O-2), 2026-06-19. Decision: D-141._

4. **[process-gap] Per-file-isolation ACs must not be inserted without an owning implementation story (C-1 from ADV-F2-PASS1)** — BC-2.01.018 AC-002 and E-INP-011/012 notes describe directory-mode per-file isolation (one file's error does not abort others). This claim was inserted during F2 spec evolution without verifying that a story exists to own the main.rs loop refactor. The existing code (`main.rs:241-244` uses `?`) falsely satisfies this AC — the first error aborts the run. An AC that requires implementation work in a file outside the story's scope is untestable and creates a false completeness signal. Going forward: before writing an AC that requires implementation work, verify that a story is scoped to own that work. If no owning story exists, either create one or retract the AC.
   _Discovered: adversarial spec review pass-1 (C-1), 2026-06-19. Decision: D-141._

## Infrastructure-Level

<!-- Reserved for infrastructure-level lessons discovered during this cycle. -->

## Spec Authoring

5. **[process-gap] BE/LE byte-order-magic shorthand is read-convention-dependent and caused a 4-document error chain** — The shorthand "0xVALUE → endianness" (e.g., "BE magic 0x4D3C2B1A") is ambiguous because the same numeric value can be stated as either the u32 constant or the on-disk byte sequence depending on the reader's assumed byte order. In the pcapng BOM case, writing "BE byte-order magic 0x4D3C2B1A" without specifying the read convention caused the ADR-009 author to use the LE on-disk byte sequence (bytes `4D 3C 2B 1A`) as the BE BOM — which is incorrect. This error propagated from ADR-009 rev 4 → BC-2.01.010 v1.4 changelog annotation → HS-103 v1.0 Case A test vector before being caught in the re-audit. **Rule:** Specifications MUST state wire values as explicit on-disk byte sequences rather than hex integers when endianness is material. For the pcapng BOM: always write "on-disk bytes `1A 2B 3C 4D` (big-endian section)" and "on-disk bytes `4D 3C 2B 1A` (little-endian section)" — never "0x1A2B3C4D read big-endian" without specifying the read direction. This is a candidate factory policy.
   _Discovered: re-audit consistency pass, 2026-06-19. Decision: D-143._

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | DF-STATE-MANAGER-SCOPE-001: state-manager must not write spec/ADR content | State-manager agent boundaries | proposed |
| 2 | DF-BC-VP-ASSIGNMENT-001: VP-NNN assignment + VP-INDEX registration + holdout fixture designation MUST be completed in the same burst that writes a BC to WRITTEN status | Spec-evolution / product-owner + architect | proposed |
| 3 | DF-INPUT-HASH-NEVER-NA-001: factory artifact input-hash fields MUST be computed (not set to N/A) before any dependent story's hash is generated | Spec-evolution / state-manager | proposed |
| 4 | DF-AC-OWNING-STORY-001: an AC that requires implementation work in a file outside the current story scope MUST have an owning story identified before the AC is written | Spec-evolution / product-owner | proposed |
| 5 | DF-WIRE-VALUE-BYTE-SEQUENCE-001: wire values MUST be stated as explicit on-disk byte sequences when endianness is material; hex integer shorthand ("0xVALUE") without a specified read convention is prohibited for endianness-sensitive fields | Spec-evolution / all spec authors | proposed |
