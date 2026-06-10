# Research Index

| Date | Type | Topic | File | Status |
|------|------|-------|------|--------|
| 2026-06-09 | domain | Modbus TCP protocol analyzer (ICS/OT forensics, feature #7) | modbus-tcp-research.md | complete |
| 2026-06-09 | general | Modbus analyzer F2 design decisions (write-burst window, MITRE recon FC mapping, multi-technique co-emission) | modbus-f2-design-decisions.md | complete |
| 2026-06-09 | general | F2 decomposition & sequencing (breaking Finding type change Vec<String> + Modbus analyzer; refactor-first wave plan, atomic vs parallel-change, TDD) | f2-decomposition-sequencing.md | complete |
| 2026-06-09 | general | Release sequencing: bundle vs. split the MITRE multi-tag schema break and the Modbus feature (semver, blast radius, OSS precedent) | f2-bundle-vs-split.md | complete |
| 2026-06-09 | general | F2 multi-tag output schema (multiple MITRE ATT&CK techniques per finding — JSON array vs nested, CSV semicolon-join, empty/absent, ordering) | f2-multitag-schema.md | complete |
| 2026-06-09 | general | F4-PIN: MITRE ATT&CK for ICS version pin for `mitre_attack_version` (v19.1 confirm; T0888/T0855/T0836/T0835/T0831/T0814/T0806/T0846 validity) | attack-ics-version-pin.md | complete (⚠️ STALE on T0855 — see dnp3-research.md §6/§7: T0855 is REVOKED in v19.1, replaced by T1692.001) |
| 2026-06-10 | domain | DNP3 (IEEE 1815) protocol analyzer (feature #8, TCP/20000) — DLL frame + CRC block math, transport/app FC table, addressing, MITRE mapping. ⚠️ T0803+T0855 REVOKED in v19.1; T0828≠Loss of Control (use T0827) | dnp3-research.md | complete |
