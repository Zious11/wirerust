# Adversarial Spec Review — feature-enip-v0.11.0 (SS-17), Pass 4

VERDICT: FAIL — 1 HIGH, 4 MEDIUM, 2 LOW (0 CRITICAL). Novelty: MODERATE. Core normative content CONFIRMED CLEAN (endianness, CIP service table, EMITTED 17→20, frame-walk, Kani non-vacuity, canonical holdout, write-burst 50, --all includes --enip). Residual drift concentrated in anchor/RTM/capability layer.

## Findings (all REMEDIATED)

- F-P4-01 (HIGH): BC-2.17.020/023 Architecture-Module mis-anchored cli.rs to SS-10 (MITRE/mitre.rs); corrected to SS-12 (CLI/Entry).
- F-P4-02 (MEDIUM): VP-INDEX:8 changelog VP-032 harness names — Sub-B totality→biconditional; added 5th harness vp032_cip_service_request_partition; "4 harnesses"→"4 sub-properties; 5 Kani harnesses".
- F-P4-03 (MEDIUM): PRD §2.10 O-04 catalogue-only enumeration missing T1693.001 (listed 7, claimed 8); added → 8 (5 Enterprise + 3 ICS).
- F-P4-04 (MEDIUM): CAP-17 stale — BC range 24→25 BCs; OA-001 default 20→50.
- F-P4-05 (MEDIUM): PRD §7 RTM BC-2.10.005 detail 25 seeded (13 ICS)→28 seeded (16 ICS) + 3 ENIP seeds.
- F-P4-06 (LOW): BC-2.17.020 OA-001 stale-open labels → RESOLVED=50.
- F-P4-07 (LOW): PRD:422 catalogue-only-delta prose reworded.

## Confirmed-Clean Axes (carried from Pass 3)

- Endianness (little-endian throughout SS-17 BCs): CLEAN
- CIP service table completeness (BC-2.17.010): CLEAN
- EMITTED count (SS-17: 17→20 techniques): CLEAN
- Frame-walk / length-prefix soundness: CLEAN
- Kani non-vacuity (VP-032 Sub-A/B/C/D): CLEAN
- Canonical holdout scenario coverage: CLEAN
- write-burst default=50 (OA-001 pending-human-confirm): CLEAN
- --all flag includes --enip: CLEAN

## Convergence Note

This is the 4th consecutive FAIL but severity is decaying monotonically:
Pass 1: 4C/7H/3M/3L → Pass 2: 4C/3H/3M/2L → Pass 3: 3C/4H/4M/0L → Pass 4: 0C/1H/4M/2L.
All anchor-layer residues now cleared. Core normative content confirmed clean across all 4 passes.
Pass 5 expected to converge (counter target: 3 consecutive CLEANs from 0/3).
