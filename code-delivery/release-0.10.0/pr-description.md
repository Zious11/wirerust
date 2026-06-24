## Summary

This PR merges the `release/0.10.0` branch into `main` and constitutes the v0.10.0 release of wirerust.

> **BREAKING CHANGE — DNP3 JSON output key rename.** See the DNP3 section below for the consumer migration note.

### What's in this release (PRs #310 / #312 / #313 / #314)

- **#313 — BREAKING: DNP3 summary key `total_parse_errors` → `parse_errors` [PC-014].** The `detail` map produced by the DNP3 analyzer now uses `"parse_errors"` instead of `"total_parse_errors"`, aligning DNP3 with sibling analyzers (HTTP, TLS, Modbus). JSON consumers reading DNP3 summary output **must** rename the key. See migration note below.

- **#310 — ARP findings output is unbounded — documented in `--arp` long-help [PC-015].** `--help`/`--arp` long_help now explicitly documents that the ARP findings list is unbounded. Includes BC-2.16.016 characterization test + CLI help Red Gate tests.

- **#312 — ARP `.expect()` spec correction + regression tests [PC-013].** Corrected the BC-2.16.004 invariant regression guards to v1.10 framing; tests now match the authoritative spec language.

- **#314 — Demo-evidence resync.** STORY-108 demo evidence and test comments updated to use `parse_errors` following the PC-014 rename.

### BREAKING: DNP3 `parse_errors` key rename — consumer migration

| Before (≤ v0.9.x) | After (≥ v0.10.0) |
|---|---|
| `detail["total_parse_errors"]` | `detail["parse_errors"]` |

**jq migration:**
```
# Before
jq '.[] | .detail.total_parse_errors'

# After
jq '.[] | .detail.parse_errors'
```

All other DNP3 output fields are unchanged. Non-DNP3 analyzers (ARP, HTTP, TLS, Modbus) are unaffected.

### CHANGELOG — v0.10.0

```
## [0.10.0] - 2026-06-24

### Breaking Changes

- **DNP3 analyzer output: renamed summary key `total_parse_errors` → `parse_errors`.**
  The `detail` map produced by the DNP3 analyzer now uses the key `"parse_errors"` instead of
  `"total_parse_errors"`, aligning DNP3 with sibling analyzers (HTTP, TLS, Modbus) that already
  use `"parse_errors"`. JSON consumers reading DNP3 summary output must migrate the key name.
  [PC-014, BC-2.15.020 v1.4, STORY-108 AC-010]

  **Migration:** Replace any lookup of `detail["total_parse_errors"]` with
  `detail["parse_errors"]` in your consumer. For `jq` users:
  `jq '.[] | .detail.total_parse_errors'` → `jq '.[] | .detail.parse_errors'`.
```

### Release checklist

- [x] Branch cut from develop @ 2b348a1
- [x] `version = "0.10.0"` in Cargo.toml / Cargo.lock
- [x] CHANGELOG.md `## [0.10.0] - 2026-06-24` section added
- [x] Commit `chore: release v0.10.0` (db541ce) on release/0.10.0
- [x] Branch pushed; CI running
- [ ] Human authorization to merge → main
- [ ] Tag `v0.10.0` on main after merge (devops-engineer)
- [ ] Merge main back into develop (devops-engineer)
