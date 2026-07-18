---
document_type: blocking-issues-resolved
level: ops
version: "1.0"
status: archive
producer: state-manager
timestamp: 2026-07-18T04:30:00Z
cycle: "feature-iec104"
inputs: []
input-hash: "d41d8cd"
traces_to: STATE.md
---

# Resolved Carry-Forwards — feature-iec104

Carry-forward items marked RESOLVED/CLOSED during the feature-iec104 cycle, extracted from
STATE.md Active Carry-Forwards table on 2026-07-18. Open items remain in STATE.md.

| ID | Summary | Resolution | Resolved Date |
|----|---------|------------|---------------|
| IEC104-FINDING-DIRECTION-001 | RESOLVED (PR #410, D-464) — CLOSED. All 10 IEC-104 emit sites now direction: Some(...). Finding.direction was None though direction was known; direction threaded through process_u_frame + detect_iec104_threats; 11 direction-assertion tests (mod fix_p4_001). | CLOSED (D-464, PR #410 7e95f71) | 2026-07-17 |
| F5R2-01 / F5R2-02 / F-B1 / F-B2 | RESOLVED — CLOSED. F5R2-01/02 (2 MEDIUM doc-accuracy) fixed by FIX-F5-002 (PR #412 b356545); F-B1 (1 HIGH demo-evidence fabrication) fixed by FIX-F5-003 (PR #413 9eab53f); F-B2 (1 MEDIUM CHANGELOG mitre_techniques) fixed by FIX-F5-004 (PR #415 b36b884). All F5 round findings resolved. | CLOSED (D-468, 2026-07-17) | 2026-07-17 |
