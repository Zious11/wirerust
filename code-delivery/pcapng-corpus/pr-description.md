## Summary

Adds 7 verified real-world pcapng captures to the E2E corpus auto-fetch script (`bin/fetch-e2e-pcaps`) and updates the tracked index (`tests/fixtures/E2E-PCAPS.md`) with a new "pcapng block-diversity suite" section.

**This PR changes only test-infrastructure and documentation files.** No production Rust code is modified. The capture files themselves are gitignored (`tests/fixtures/local-samples/`) and are not part of this PR — only the fetch script and index are committed.

---

## What changed

### `bin/fetch-e2e-pcaps`

Seven new `name|sha256|url` entries added to the `REAL_CAPTURES` array under a new `# --- pcapng block-diversity suite ---` comment block. Each entry has a sha256 verified locally against the fetched file. Two link-only entries added to the comment section (not auto-fetched):

| Capture | Feature exercised | Source |
|---------|-------------------|--------|
| `pcapng-example.pcapng` | SHB options (shb_comment/hw/os/app), 2×IDB (LINUX_SLL + ETHERNET), DSB (Decryption Secrets Block), NRB, EPB opt_comment; triggers E-INP-011 | Wireshark wiki SampleCaptures (Wireshark Foundation / SYNbit) |
| `220703_arp-storm-nrb.pcapng` | NRB (Name Resolution Block), 622 EPBs, single IDB, LE | Wireshark wiki SampleCaptures (Wireshark Foundation) |
| `dhcp-big-endian.pcapng` | Big-endian encoding (byte-order magic `1A 2B 3C 4D`), SHB+IDB+EPB all big-endian | Wireshark test suite (BSD-style) |
| `pcapng-comments.pcapng` | EPB-level `opt_comment` options on individual packets | Wireshark test suite (BSD-style) |
| `dtls12-dsb.pcapng` | DSB (Decryption Secrets Block, type `0x0000000A`), DTLS 1.2 TLS key log | Wireshark test suite (BSD-style) |
| `dhcp-nanosecond-test.pcapng` | IDB `if_tsresol = 0x09` (nanosecond timestamp resolution, base-10 exponent 9) | Wireshark test suite (BSD-style) |
| `http-brotli-isb.pcapng` | ISB (Interface Statistics Block, type `0x00000005`) skip/parse path | Wireshark test suite (BSD-style) |

Link-only additions to the comment section (not auto-fetched):
- `asyncrat_1hr.pcapng` (~4.6 MB) — Active Countermeasures; Cloudflare bot-blocked (HTTP 403)
- `042219_1000_7.pcapng` (~657 MB) — CUPID dataset (Colorado U / CC BY-SA 4.0); excluded due to size

### `tests/fixtures/E2E-PCAPS.md`

- New "pcapng block-diversity suite" table with all 7 captures: size, sha256, source, link type, pcapng feature exercised, and smoke-test result.
- New URL entries in the "Direct download URLs" section.
- New link-only table rows for `asyncrat_1hr.pcapng` and `042219_1000_7.pcapng`.
- New "pcapng block-diversity suite attribution" subsection covering Wireshark Foundation / BSD-style test captures and CUPID CC BY-SA 4.0.

---

## Why these captures

The pcapng reader stack (STORY-123–128, ADR-009) implements SHB, IDB, EPB, NRB, DSB, ISB block types and big-endian decoding. The corpus previously had no native pcapng fixtures that exercised these paths beyond what unit/integration tests cover. These 7 captures provide E2E validation of the full block-type surface, sourced from publicly available Wireshark samples and test suite files.

---

## Security review rationale

Security review is not required for this PR. The changes are:
1. A bash fetch script addition (download URLs for known public captures with sha256 verification)
2. A markdown documentation index update

There is no production code change, no change to any CI-gated test, no new executable logic beyond the existing `curl --retry 3` + sha256 verify pattern already present in the script. The URLs are public Wireshark Foundation and GitLab sources. sha256 pinning provides integrity assurance on the fetched artifacts.

---

## CI impact

No CI-gated tests change. CI does not execute `bin/fetch-e2e-pcaps` (captures are gitignored and the fetch script is not part of the CI pipeline). CI will run the standard Rust suite: `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`. This PR makes no Rust source changes, so all checks are expected green.

---

## Pre-merge checklist

- [x] Only `bin/fetch-e2e-pcaps` and `tests/fixtures/E2E-PCAPS.md` staged
- [x] No `tests/fixtures/local-samples/*.pcapng` staged (gitignored)
- [x] sha256 values verified locally for all 7 auto-fetch entries
- [x] URLs are public sources (Wireshark Foundation wiki + GitLab test captures)
- [x] Link-only entries are correctly commented (not in REAL_CAPTURES array)
- [x] Attribution section updated in E2E-PCAPS.md
- [x] Semantic PR title uses `test:` type per CLAUDE.md enforcement
