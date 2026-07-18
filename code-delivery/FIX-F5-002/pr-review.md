# PR #412 Review — `docs(FIX-F5-002)` — Verdict: APPROVE

**PR:** #412 — `docs(FIX-F5-002): correct FIX-F5-001 demo-evidence provenance + JSON + CHANGELOG direction-parity accuracy`
**Branch:** `fix/FIX-F5-002` → `develop`
**Scope:** docs-only (no `.rs` changes)

## Scope confirmation

`git diff --name-only origin/develop...origin/fix/FIX-F5-002` returns exactly two files:

- `CHANGELOG.md`
- `docs/demo-evidence/FIX-F5-001/evidence-report.md`

Zero `.rs` files, no `Cargo.toml`, no `bin/`.

## Check-by-check (PG-W74 row-verify mandate)

| # | Check | Verdict | Evidence |
|---|-------|---------|----------|
| 1 | Docs-only gate | **PASS** | Diff = `CHANGELOG.md` + `evidence-report.md` only. |
| 2 | JSON accuracy row-verify | **PASS** | All serde annotations + emit-site values confirmed against tree. |
| 3 | Provenance accuracy | **PASS** | Report cites S-139/S-140 (PR #328); notes IEC-104 additionally sets `direction: Some(direction)`. |
| 4 | CHANGELOG direction-parity | **PASS** | FIX-P4-001 entry corrected to `direction: None` for DNP3/EtherNet/IP. |
| 5 | Security review | **PASS / N/A** | Docs-only; no code or trust-boundary surface. |

## JSON row-verify detail (verified against source tree)

- `src/findings.rs:98` — `ThreatCategory` `rename_all = "snake_case"`; emit site uses `ThreatCategory::Impact` (`src/analyzer/iec104.rs:382`) → `"impact"`. Matches report.
- `src/findings.rs:66` — `Confidence` `rename_all = "lowercase"`; emit site uses `Confidence::Medium` (`iec104.rs:384`) → `"medium"`. Matches report.
- `src/findings.rs:31` — `Verdict` `rename_all = "lowercase"`; STOPDT-act path binds `verdict` (Possible/Likely) → `"possible"`. Matches report.
- `src/reassembly/handler.rs:21-28` — `Direction` has NO `rename_all` (source comment documents default `"ClientToServer"`/`"ServerToClient"`) → `"ClientToServer"`. Matches report.
- Emit site `iec104.rs:382-396` — summary begins `"IEC-104 STOPDT-act received:"`, `evidence = ["CF1=0x{cf1:02X} (STOPDT-act)"]`; report's `"CF1=0x13 (STOPDT-act)"` (STOPDT-act CF1 = 0x13) matches.
- JSON key ordering in the After-block (category, verdict, confidence, summary, evidence, mitre_techniques, source_ip, timestamp, direction) matches the `Finding` struct field declaration order in `findings.rs`. serde serializes in declaration order, so the report is order-accurate as well as value-accurate.

## Provenance / direction-parity detail

- DNP3: `src/analyzer/dnp3.rs` sets `direction: None` at all emit sites (1051, 1111, 1162, 1246, …).
- EtherNet/IP: `src/analyzer/enip.rs` sets `direction: None` at all emit sites (486, 1087, 1193, 1239, 1274, 1309, 1381, 1459).
- IEC-104 sets `direction: Some(direction)` (`iec104.rs:396`). The claim "IEC-104's direction enrichment exceeds the DNP3/EtherNet/IP baseline" is factually correct.
- Year corrected to `2026-07-17` throughout (matches current date).

## Security note (explicit)

No security review required. This PR changes only Markdown documentation (`CHANGELOG.md` and a demo-evidence report). No code change, no parser/input-handling surface, no trust-boundary modification. Nothing to scan for CWE/CVE.

## Findings

No BLOCKING, WARNING, or NIT findings. The PR corrects three documented inaccuracies (provenance, fabricated JSON values, wrong year) plus a CHANGELOG direction-parity misstatement. Every corrected value now matches the actual source tree.

CHANGELOG-gate note: the CI trigger set (`src/`, `Cargo.toml`, `bin/`) is untouched, so `changelog-gate` does not require an entry — yet the PR adds a proper `[Unreleased]` FIX-F5-002 entry anyway, which is correct hygiene.

## Overall disposition: APPROVE
