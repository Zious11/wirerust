# Demo Evidence Report — STORY-129

**Feature:** Per-finding `mitre_attack` array in JSON output (BC-2.11.035)
**Issue:** #64
**Story ID:** STORY-129
**Recorded:** 2026-06-23 (re-recorded on fix/ics-tactic-ids; original 2026-06-22 showed incorrect TA0007)
**Tool:** VHS 0.11.0
**Binary:** `target/release/wirerust` (v0.9.3, built from fix/ics-tactic-ids branch)
**Fixture:** `tests/fixtures/modbus-write.pcap`

---

## Coverage Map

| AC | Description | Recording | Path |
|----|-------------|-----------|------|
| AC-1, AC-4, AC-7, AC-8 | Known techniques produce fully-resolved 5-field objects; multi-tag order preserved; ICS tactic_id correct (T0888→TA0102, T0836→TA0106); `mitre_techniques` unchanged | AC-001-mitre-attack-json-enrichment.gif / .webm | `docs/demo-evidence/STORY-129/` |
| AC-9 | CSV output is additive-non-breaking: no `mitre_attack` column appears | AC-009-csv-unaffected.gif / .webm | `docs/demo-evidence/STORY-129/` |

---

## Recordings

### AC-001: mitre_attack array — JSON success path

**Command recorded:**
```
./target/release/wirerust analyze --json --modbus tests/fixtures/modbus-write.pcap \
  | python3 -m json.tool | grep -A 25 mitre_attack | head -60
```

**What it shows:**
- 3 findings are emitted from `modbus-write.pcap`
- Each finding's `mitre_attack` array is populated with fully-resolved technique objects
- Finding 1+2 (Modbus recon): T0888 → `name: "Remote System Information Discovery"`, `tactic_id: "TA0102"`, `tactic_name: "Discovery (ICS)"` (corrected from TA0007 "Discovery" which mapped to the Enterprise matrix)
- Finding 3 (write command): two entries — T1692.001 and T0836 — in declaration order (AC-4), both resolving to `tactic_id: "TA0106"` / `tactic_name: "Impair Process Control"` (AC-7, ICS matrix)
- The `reference` URL uses the verbatim technique ID (preserving dot separator for sub-techniques)
- The raw `mitre_techniques` array remains unchanged alongside `mitre_attack` (AC-8)

**Artifacts:**
- `AC-001-mitre-attack-json-enrichment.gif` (156 KB)
- `AC-001-mitre-attack-json-enrichment.webm` (149 KB)
- `AC-001-mitre-attack-json-enrichment.tape` (VHS source)

---

### AC-009: CSV output — non-breaking error path

**Command recorded:**
```
./target/release/wirerust analyze --csv --modbus tests/fixtures/modbus-write.pcap
```

**What it shows:**
- CSV output contains columns: `category,verdict,confidence,summary,evidence,mitre_techniques,source_ip,direction,timestamp`
- The `mitre_techniques` column shows raw technique IDs only (e.g., `T0888`, `T1692.001;T0836`)
- No `mitre_attack` column and no nested JSON objects appear — confirming the enrichment is JSON-only and the CSV reporter is unmodified (BC-2.11.035 postcondition 6)

**Artifacts:**
- `AC-009-csv-unaffected.gif` (110 KB)
- `AC-009-csv-unaffected.webm` (111 KB)
- `AC-009-csv-unaffected.tape` (VHS source)

---

## Real mitre_attack JSON Snippet

Captured from a live run on fix/ics-tactic-ids (2026-06-23). All three findings shown.

### findings[0] — T0888 Discovery (ICS) — CORRECTED (was TA0007, now TA0102)

```json
{
  "category": "Anomaly",
  "confidence": "Medium",
  "direction": "ClientToServer",
  "evidence": ["FC=0x11 TxnID=0x0001 UnitID=1"],
  "mitre_attack": [
    {
      "id": "T0888",
      "name": "Remote System Information Discovery",
      "reference": "https://attack.mitre.org/techniques/T0888/",
      "tactic_id": "TA0102",
      "tactic_name": "Discovery (ICS)"
    }
  ],
  "mitre_techniques": ["T0888"],
  "source_ip": "192.168.1.10",
  "summary": "Modbus recon: Report Server ID (FC 0x11) from unit 1",
  "timestamp": "2024-05-29T16:26:43Z",
  "verdict": "Inconclusive"
}
```

### findings[2] — Multi-technique Impair Process Control (unchanged)

```json
{
  "category": "Execution",
  "confidence": "Medium",
  "direction": "ClientToServer",
  "evidence": ["FC=0x10 TxnID=0x0002 UnitID=1 ADU bytes 0..17"],
  "mitre_attack": [
    {
      "id": "T1692.001",
      "name": "Unauthorized Message: Command Message",
      "reference": "https://attack.mitre.org/techniques/T1692.001/",
      "tactic_id": "TA0106",
      "tactic_name": "Impair Process Control"
    },
    {
      "id": "T0836",
      "name": "Modify Parameter",
      "reference": "https://attack.mitre.org/techniques/T0836/",
      "tactic_id": "TA0106",
      "tactic_name": "Impair Process Control"
    }
  ],
  "mitre_techniques": ["T1692.001", "T0836"],
  "source_ip": "192.168.1.10",
  "summary": "Modbus write command observed: FC 0x10 from unit 1",
  "timestamp": "2024-05-29T16:26:45Z",
  "verdict": "Likely"
}
```

Envelope fields (top-level):
- `"mitre_attack_version": "ics-attack-19.1"`
- `"mitre_domain": "ics-attack"`

**Note — ARP (T0830):** No ARP pcap fixture exists in `tests/fixtures/`. The expected output
for T0830 would be `tactic_id: "TA0100"` / `tactic_name: "Collection (ICS)"`. This is
verified by unit tests in `tests/reporter_json_tests.rs` but cannot be demonstrated via a
live pcap recording without an ARP fixture.

---

## Acceptance Criteria Coverage

| AC | Status | Evidence |
|----|--------|---------|
| AC-1: single known technique, all 5 fields | DEMONSTRATED | AC-001 recording shows T0888 with id/name/tactic_id/tactic_name/reference |
| AC-2: unknown ID partial object | NOT RECORDED (unit-test covered) | `T9999` has no live fixture; AC-2 is covered by `test_BC_2_11_035_unknown_technique_id_never_lost` |
| AC-3: empty mitre_techniques omits key | NOT RECORDED (unit-test covered) | Covered by `test_BC_2_11_035_empty_mitre_techniques_omits_mitre_attack` |
| AC-4: multi-tag order preserved | DEMONSTRATED | AC-001 shows T1692.001 at index 0 and T0836 at index 1 |
| AC-5: duplicates not deduplicated | NOT RECORDED (unit-test covered) | Covered by `test_BC_2_11_035_duplicate_ids_not_deduplicated` |
| AC-6: sub-technique dot preserved | DEMONSTRATED | AC-001 shows `T1692.001` with dot in id and reference URL |
| AC-7: ICS tactic_id resolved | DEMONSTRATED | AC-001 shows `tactic_id: "TA0102"` for T0888 (Discovery ICS) and `tactic_id: "TA0106"` for T0836/T1692.001 (Impair Process Control); all ICS-matrix IDs, not Enterprise |
| AC-8: mitre_techniques unchanged | DEMONSTRATED | AC-001 output shows both `mitre_techniques` and `mitre_attack` coexisting |
| AC-9: CSV unaffected | DEMONSTRATED | AC-009 recording shows no mitre_attack column in CSV |
| AC-10: terminal unaffected | NOT RECORDED (unit-test covered) | Covered by `test_BC_2_11_035_terminal_unaffected` |

**Note on unrecorded ACs:** AC-2, AC-3, AC-5, AC-10 require synthetic inputs (unknown IDs,
empty vecs, duplicates) that have no corresponding fixture pcap file. These are definitively
covered by 13 unit tests in `tests/reporter_json_tests.rs`. The VHS recordings focus on the
live CLI path with a real pcap fixture that produces populated `mitre_attack` arrays.
