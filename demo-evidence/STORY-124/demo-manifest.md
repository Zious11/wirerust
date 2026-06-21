# STORY-124 Demo Manifest

**Story:** STORY-124 — pcapng IDB parse + interface DataLink whitelist + multi-IDB conflict  
**Epic:** E-19 | **Wave:** 52 | **Recorded:** 2026-06-20

---

## Demo → AC / Behavior Mapping

| Demo File | AC | BC | Behavior Demonstrated | Error Code |
|-----------|----|----|----------------------|-----------|
| `AC-008-multi-idb-conflict.gif` | AC-008 | BC-2.01.018 PC2 | Multi-IDB link-type conflict (ETHERNET vs LINUX_SLL) → E-INP-011 with `tcpdump -i any` / `single link type` hint | E-INP-011 |
| `AC-009-multi-idb-same-type.gif` | AC-009 | BC-2.01.018 PC1 | Multi-IDB same link type (2x ETHERNET) → accepted, analysis proceeds | None (success) |
| `AC-007-non-whitelisted-idb.gif` | AC-007 | BC-2.01.016 PC2 | Non-whitelisted IDB link type (IEEE802_11=105) → graceful E-INP-001, no panic, lists 5 supported types | E-INP-001 |
| `AC-001-be-idb-options.gif` | AC-001, AC-002 | BC-2.01.011 PC1+PC2 | Big-endian pcapng: linktype correctly decoded from BE bytes + if_tsresol option extracted | None (success) |
| `AC-TEST-idb-test-suite.gif` | AC-001..AC-011, VP-030 | All 3 BCs | Full IDB test suite: 27 tests pass (all ACs, whitelist, three-level precedence, proptest VP-030) | All E-INP codes |

---

## Headline Demo: AC-008 (Multi-IDB Conflict)

`AC-008-multi-idb-conflict.gif` is the primary PR embed. It shows the improved UX:

```
pcapng multi-interface link-type conflict: interface 0 has ETHERNET, interface 1 has LINUX_SLL
(hint: this commonly occurs with 'tcpdump -i any' captures that mix link types;
 wirerust requires a single link type per file) (E-INP-011)
```

This is the error message mandated by BC-2.01.018 AC-001(b): explicit interface indices, link type
names (not raw codes), and the `tcpdump -i any` remediation hint were all converged in the
adversarial review pass.

---

## Tape Sources (VHS v0.11.0)

All tapes run `wirerust analyze` against pre-built pcapng fixtures (in `fixtures/`):

- `AC-008-multi-idb-conflict.tape`
- `AC-009-multi-idb-same-type.tape`
- `AC-007-non-whitelisted-idb.tape`
- `AC-001-be-idb-options.tape`
- `AC-TEST-idb-test-suite.tape`

---

## ACs Without CLI-Level Demo (rationale)

AC-003, AC-004, AC-005, AC-006, AC-010, AC-011 are internal parser states covered by the test
suite recording. The `wirerust analyze` CLI surfaces these as generic errors; distinguishing
E-INP-008 from E-INP-013 at the CLI output level is indistinguishable from an end-user perspective.
All 27 unit/property tests pass as shown in `AC-TEST-idb-test-suite.gif`.
