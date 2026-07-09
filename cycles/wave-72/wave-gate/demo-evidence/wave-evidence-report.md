# Wave-72 Integration Demo Evidence Report

**Wave:** 72
**Stories:** STORY-158, STORY-159, STORY-160, STORY-161
**Branch:** develop (HEAD c4eb1f4 — wave-72 merged)
**Recorded:** 2026-07-09
**Scrub gate:** PASS — zero `<home>/` or absolute host paths in output dir
**Recording method:** Transcript fallback (VHS attempted; VHS 0.11.0 viewport limitation
applies to wirerust unicode box-drawing output — see wave-71 VHS Recording Note for details;
tape files retained as runnable reference scripts with `<repo>` placeholder)

---

## Wave Claim Coverage

| Demo | Artifact | Wave Claim Evidenced |
|------|----------|---------------------|
| DEMO-001 | `DEMO-001-json-surface.txt` | JSON six-key envelope with `schema_version: "2"` (BC-2.11.037); lowercase `verdict`/`confidence` + snake_case `category` in findings (BC-2.11.036); BC-2.11.001 v1.9 six-key shape (STORY-160) |
| DEMO-002 | `DEMO-002-regression-surfaces.txt` | Terminal UPPERCASE Display tokens unchanged (`INCONCLUSIVE`, `LIKELY`, `MEDIUM`, `[Anomaly]`, `[Execution]`) + CSV PascalCase categories unchanged + no `schema_version` in either output (STORY-160 AC-160-005 / AC-160-006) |
| DEMO-003 | `DEMO-003-gates-governance.txt` | Action-pin-gate: 23 SHA-pinned remote refs, 0 violations + ADR-0012 ten-decision check + VP-024 `proof_file_hash` populated (STORY-161 FU-F6-KANI-CLEANUP) + CLAUDE.md Two Hash Disciplines codification (STORY-161) |

---

## DEMO-001: Combined JSON Surface

**File:** `DEMO-001-json-surface.txt`
**Command:** `./target/release/wirerust analyze --all --output-format json tests/fixtures/modbus-write.pcap`
**Pcap:** `tests/fixtures/modbus-write.pcap` (Modbus/TCP, 8 packets, 3 findings)

**Evidence:**

Six-key envelope confirmed (BC-2.11.001 v1.9):
```
['analyzers', 'findings', 'mitre_attack_version', 'mitre_domain', 'schema_version', 'summary']
```

`schema_version` value is the string `"2"` (BC-2.11.037):
```
schema_version: '2'
```

Findings enum values use lowercase/snake_case (BC-2.11.036):
```
[1] verdict='inconclusive'  confidence='medium'  category='anomaly'
[2] verdict='inconclusive'  confidence='medium'  category='anomaly'
[3] verdict='likely'        confidence='medium'  category='execution'
```

No PascalCase enum values present in JSON output. Eight analyzers active (TCP Reassembly,
DNS, HTTP, TLS, modbus, DNP3, EtherNet/IP, ARP).

---

## DEMO-002: Regression Surfaces

**File:** `DEMO-002-regression-surfaces.txt`
**Commands:**
- Terminal: `./target/release/wirerust analyze --all --no-color tests/fixtures/modbus-write.pcap`
- CSV: `./target/release/wirerust analyze --all --output-format csv tests/fixtures/modbus-write.pcap`

**Evidence:**

Terminal Display tokens unchanged (AC-160-005):
```
  [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: Report Server ID (FC 0x11) from unit 1 (x2)
  [Execution] LIKELY (MEDIUM) - Modbus write command observed: FC 0x10 from unit 1
```
Verdict and confidence shown in UPPERCASE; category in PascalCase — unaffected by
`serde(rename_all)` which governs only `Serialize`, not `Display`.

`schema_version` absent from terminal output — `grep -c schema_version` returns `0` (AC-160-006).

CSV categories PascalCase, no `schema_version` column (AC-160-006):
```
category,verdict,confidence,...
Anomaly,INCONCLUSIVE,MEDIUM,...
Execution,LIKELY,MEDIUM,...
```

---

## DEMO-003: Gates + Governance

**File:** `DEMO-003-gates-governance.txt`

### 3a: Action-Pin-Gate

Replication of the `action-pin-gate` CI job logic across both workflow files
(`.github/workflows/ci.yml`, `.github/workflows/pr.yml`):
```
Workflow files scanned: 2
Remote action refs validated (SHA-pinned): 23
Violations (non-SHA, non-allowlisted): 0
ACTION-PIN-GATE: PASS -- 0 violations
Allowlisted: dtolnay/rust-toolchain@stable, dtolnay/rust-toolchain@nightly
```
All 23 remote action refs are pinned to 40-character hex SHAs. The two
`dtolnay/rust-toolchain` channel refs are explicitly allowlisted per CLAUDE.md policy.

### 3b: ADR-0012 Ten-Decision Check

`docs/adr/0012-protocols-catalog-and-coverage-gaps.md` contains exactly 10 `### Decision`
headings (Decision 1 through Decision 10):
```
### Decision 1: Hand-Curated Static Compile-Time Array
### Decision 2: Tri-State Vocabulary (Suricata-Derived)
...
### Decision 10: UDP Gap Classification Decoupled from `enable_dns`
```

### 3c: VP-024 proof_file_hash (STORY-161)

`proof_file_hash` field is populated (non-null) in VP-024 frontmatter, discharging
FU-F6-KANI-CLEANUP (deferred since Phase F6, 2026-06-16):
```
proof_file_hash: "48296b21a5bbce59750e6210da8d55be8bf7d3d4a1ed6719088dd4ef59a2c8a5"
```
64-char lowercase hex SHA-256 mini-Merkle over the two Kani proof sections
(`src/analyzer/arp.rs` kani_proofs block + `src/decoder.rs` kani_proofs block).

### 3d: CLAUDE.md Two Hash Disciplines (STORY-161)

`CLAUDE.md` contains the "Two Hash Disciplines" section (line 219+) codifying the
distinction between `input-hash` (MD5-first-7, advisory drift detection) and
`proof_file_hash` (SHA-256 mini-Merkle, integrity anchor for formal verification):
```
Two hash disciplines in this repository are deliberately distinct:
- input-hash: MD5-first-7 hex ... advisory drift detection for spec inputs
- proof_file_hash: SHA-256 mini-Merkle ... integrity anchor for formal verification
```

---

## Artifact Inventory

```
.factory/cycles/wave-72/wave-gate/demo-evidence/
├── DEMO-001-json-surface.tape     # VHS tape (reference; transcript is primary evidence)
├── DEMO-001-json-surface.txt      # Transcript: JSON envelope 6 keys + lowercase/snake_case enums
├── DEMO-002-regression-surfaces.tape  # VHS tape (reference; transcript is primary evidence)
├── DEMO-002-regression-surfaces.txt   # Transcript: terminal UPPERCASE + CSV PascalCase unchanged
├── DEMO-003-gates-governance.tape # VHS tape (reference; transcript is primary evidence)
├── DEMO-003-gates-governance.txt  # Transcript: action-pin-gate + ADR-0012 + VP-024 + CLAUDE.md
└── wave-evidence-report.md        # This file
```

---

## Scrub Gate (PG-W70-DEMO-SCRUB)

**Command:** `grep -rE '<host-path-pattern>' .factory/cycles/wave-72/wave-gate/demo-evidence/`
**Result:** Zero matches — all absolute host paths replaced with `<repo>` placeholder in tape files; transcripts contain only repo-relative paths.

All transcripts use repo-relative paths or anonymized `<repo>` placeholders. No absolute
host paths or tilde-form home references present in any artifact file.

---

## VHS Recording Note

VHS recording was attempted for all 3 demos. All tapes encounter the same VHS 0.11.0 viewport
limitation documented in the wave-71 evidence report: wirerust's unicode box-drawing separator
characters (U+2500 x 40) cause long output to scroll past the VHS visible viewport before
`Wait+Screen` matches, causing timeout. The transcript fallback pattern established in wave-71
is followed here.

The tape files are retained as runnable reference scripts (with `<repo>` placeholder) and
include comments documenting the limitation. The mitigation note from wave-71 (pipe through
`grep` or use `--json` with field extraction) is applied in the tape scripts.
