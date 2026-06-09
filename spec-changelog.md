---
document_type: spec-changelog
title: "wirerust Specification Changelog"
status: active
producer: product-owner
---

# wirerust Specification Changelog

All notable changes to the specification artifacts (PRD, BCs, domain spec, architecture)
are recorded here. Entries follow MAJOR.MINOR versioning: MINOR for new capabilities
added without breaking existing BCs; MAJOR for breaking changes (BC retirement, interface
changes, invariant rewrites).

---

## [1.1] — 2026-06-09

### MINOR: SS-14 Modbus/ICS Analyzer — Feature #7

**Summary:** Added Modbus TCP protocol analyzer (SS-14, C-22) with 25 behavioral contracts,
VP-022 formal verification target, ADR-005 architecture decision, and 6 MITRE ATT&CK for
ICS technique mappings.

**Artifacts affected:**

| Artifact | Change | File |
|----------|--------|------|
| PRD | Version bump 1.0 → 1.1; Section 2.14 added (25 BCs); Section 7 RTM extended (25 rows); KD-003 and KD-005 sections updated | `.factory/specs/prd.md` |
| BC-INDEX | Version bump 1.0 → 1.1; SS-14 subsystem section added (25 rows); total BC count 219 → 244 | `.factory/specs/behavioral-contracts/BC-INDEX.md` |
| BC-2.14.001..022 | Created (F2 create burst, Groups A-G) | `.factory/specs/behavioral-contracts/ss-14/` |
| BC-2.14.023 | Created (Group H: --modbus CLI flag enablement) | `.factory/specs/behavioral-contracts/ss-14/BC-2.14.023.md` |
| BC-2.14.024 | Created (Group H: --modbus-write-threshold CLI flag) | `.factory/specs/behavioral-contracts/ss-14/BC-2.14.024.md` |
| BC-2.14.025 | Created (Group H: StreamDispatcher port-502 Rule 5 classification) | `.factory/specs/behavioral-contracts/ss-14/BC-2.14.025.md` |
| Architecture Delta | Created | `.factory/phase-f2-spec-evolution/architecture-delta.md` |
| PRD Delta | Created | `.factory/phase-f2-spec-evolution/prd-delta.md` |
| VP-022 | Designed (to be authored by formal-verifier in parallel) | `.factory/specs/verification-properties/VP-022.md` (pending) |
| ADR-005 | Created (binary ICS protocol integration decision) | `.factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md` |

**New MITRE ATT&CK for ICS techniques (6 total):**
- T0855 — Unauthorized Command Message (IcsImpairProcessControl)
- T0836 — Modify Parameter (IcsImpairProcessControl)
- T0814 — Denial of Service (IcsInhibitResponseFunction)
- T0806 — Brute Force I/O (IcsImpairProcessControl)
- T0835 — Manipulate I/O Image (IcsImpairProcessControl)
- T0831 — Manipulation of Control (IcsImpairProcessControl)

**MITRE catalog size:** 15 → 20 seeded technique IDs
(`SEEDED_TECHNIQUE_ID_COUNT = 15 → 20`; `EMITTED_IDS` extended from 6 to 12).

**Key constants introduced:**
- `MAX_PENDING_TRANSACTIONS = 256` (per-flow pending table cap)
- `WRITE_RATE_WINDOW_SECS = 1` (burst detection window)
- `DEFAULT_MODBUS_WRITE_THRESHOLD = 10` (writes/second before T0806 fires)

**CLI surface changes:**
- `--modbus` flag added to `analyze` subcommand (boolean, default false)
- `--modbus-write-threshold N` flag added (u32, default 10; zero rejected)
- `--all` expansion updated to include `--modbus`
- `needs_reassembly` expression updated: `enable_http || enable_tls || enable_modbus`

**Dispatcher changes:**
- `DispatchTarget::Modbus` variant added (4th variant after Http, Tls, None)
- `StreamDispatcher.modbus: Option<ModbusAnalyzer>` field added
- `classify` Rule 5: port 502 → `DispatchTarget::Modbus` (after content rules 1-2 and TLS/HTTP port rules 3-4)
- `modbus_analyzer()` and `take_modbus_analyzer()` accessors added
- `on_data` and `on_flow_close` Modbus routing arms added
- VP-004 `classify_oracle` must be extended with Rule 5

**Spec debt resolved:**
- O-04 partially resolved: T0855 (previously catalogued-but-never-emitted) is now actively
  emitted by ModbusAnalyzer. Updated in PRD Section 1.5 Out of Scope note.

---

## [1.0] — 2026-05-20

### Initial specification (brownfield ingestion)

Initial PRD and BC set produced by brownfield ingestion of develop HEAD. 219 active BCs
across ss-01 through ss-13 (BC-2.01.001..BC-2.13.004). Includes: 218 ingestion-batch BCs,
6 retired (BC-ABS-004..009), 5 pass-4 additions (BC-2.11.020..024), 2 F2 pcap-timestamp
additions (BC-2.04.055, BC-2.09.007).
