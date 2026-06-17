## Release v0.7.1 — E-17 ARP QinQ/MACsec Offset Regression Hardening

**Type:** Test-and-docs patch release. No runtime behavior change.

### Summary

- Adds 10 regression tests for VLAN/QinQ (802.1ad double-tag)/MACsec link-extension ARP offset
  handling introduced in v0.7.0 (issue #253, STORY-116/117).
- Adds an off-by-8 SCI-accounting guard for MACsec-tagged ARP frames.
- Documents MACsec-over-ARP offset correctness as an evidence-backed limitation (no public
  on-wire MACsec+ARP capture exists; correctness proven by etherparse source, upstream proptests,
  and synthetic tests).
- Full VSDD F1–F7 CONVERGED.

### What Changed

The VLAN/QinQ/MACsec offset handling itself shipped in v0.7.0. This patch release adds the
regression guard layer that locks in correctness:

| File | Change |
|------|--------|
| `tests/bc_2_16_qinq_macsec_offset_tests.rs` | QinQ/MACsec offset regression tests |
| `tests/bc_2_16_e17_macsec_offset_tests.rs` | E-17 MACsec benign-truncated hardenening (F4 wave-adversarial Finding 1) |
| `CHANGELOG.md` | [0.7.1] entry |
| `Cargo.toml` | version bump 0.7.0 → 0.7.1 |

### Issue Reference

Closes / guards against regression of issue #253 (E-17 ARP QinQ/MACsec offset).

### CHANGELOG Entry

> **[0.7.1] - 2026-06-17**
>
> **Added**
> - Regression test coverage for VLAN / QinQ (802.1ad double-tag) / MACsec link-extension ARP
>   offset handling — 10 tests across `tests/bc_2_16_qinq_macsec_offset_tests.rs` and
>   `tests/bc_2_16_e17_macsec_offset_tests.rs` (issue #253, STORY-116/117). Includes an
>   off-by-8 SCI-accounting guard for MACsec-tagged ARP.
>
> **Notes**
> - No runtime behavior change: the VLAN/QinQ/MACsec offset handling itself shipped in 0.7.0;
>   this release adds regression guards. MACsec-over-ARP offset correctness is proven by
>   etherparse source + upstream proptests + synthetic tests and is documented as an
>   evidence-backed limitation (no public on-wire MACsec+ARP capture exists).

### Merge Method

Merge commit (`--merge`) per D-073 (repo allows merge-commits only).

### Pre-Merge Checklist

- [x] Branch `release/0.7.1` branched from `develop` (gitflow)
- [x] Target branch is `main` (release branch)
- [x] CHANGELOG [0.7.1] entry present
- [x] `Cargo.toml` version is 0.7.1
- [x] Human approval granted (v0.7.1 release)
- [x] CI checks passing (verified before merge)
- [ ] PR merged via merge-commit (--merge) per D-073
