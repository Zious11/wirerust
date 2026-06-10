# E2E PCAP samples — index

The large packet captures used for **manual end-to-end validation** of the
analyzers are **not stored in git** (they exceed GitHub's 100 MB push limit and
a shared storage backend is still undecided — see `.factory/STATE.md`
`PCAP-CORPUS-001`). They live, gitignored, under `tests/fixtures/local-samples/`.

This file is the **tracked index** so any developer can reproduce the local
E2E set. To fetch/regenerate everything:

```bash
bin/fetch-e2e-pcaps
```

That downloads the real captures and regenerates the synthetic one into
`tests/fixtures/local-samples/`, verifying every checksum.

## Captures

| File | Size | sha256 | Source | Protocols | Validates |
|------|------|--------|--------|-----------|-----------|
| `4SICS-GeekLounge-151020.pcap` | 25 MB | `8c6ee02dc26b1b5298a7c9b4dc83cc779bd2a3219d5c5cbc51e3d4d325763bc2` | [Netresec 4SICS](https://www.netresec.com/?page=PCAP4SICS) | Modbus/502, DNP3, S7, HTTP, DNS, … | parse robustness / mixed-protocol scale; light Modbus |
| `4SICS-GeekLounge-151021.pcap` | 134 MB | `7365b0ea475b76bf79b207fd8f83baa45e4449aead5da6a9214bbcffbc5fa7de` | [Netresec 4SICS](https://www.netresec.com/?page=PCAP4SICS) | Modbus/502, DNP3, S7, HTTP, TLS, … | recon detection (FC 0x2B/0x11); throughput |
| `4SICS-GeekLounge-151022.pcap` | 200 MB | `82529c23906416dc73d7f1926a0d38b82527f1f2a7ff8c6f755ce3208feb9643` | [Netresec 4SICS](https://www.netresec.com/?page=PCAP4SICS) | Modbus/502 (heavy), DNP3, TLS, … | full Modbus detector set: writes (T0835/T0836/T1692.001), burst (T0806), recon (T0888); DoS finding-cap; determinism |
| `modbus-large.pcap` | ~7 KB | `1286603a7c83ca28de7eb46bc93271acd86ce3121f8fe695a744491cc22e5966` | synthetic — `tests/fixtures/mk_modbus_large_pcap.py` | Modbus/502 (5 crafted flows) | every Modbus detector class in isolation (recon, write-burst, coil/register/control writes, diagnostics DoS) |

> A tiny committed fixture, `tests/fixtures/modbus-write.pcap` (8 packets), is
> tracked in git and used by the automated test suite — it is **not** part of
> this local-only set.

## Direct download URLs (real captures)

| File | URL |
|------|-----|
| 151020 | `https://share.netresec.com/s/xYj2qCNbsLEAd6M/download/4SICS-GeekLounge-151020.pcap` |
| 151021 | `https://share.netresec.com/s/camL59aoxbCRyyZ/download/4SICS-GeekLounge-151021.pcap` |
| 151022 | `https://share.netresec.com/s/gw6Y2QzJHqDD5pr/download/4SICS-GeekLounge-151022.pcap` |

## Attribution

The 4SICS Geek Lounge captures are from Netresec's public 4SICS ICS-lab
collection: <https://www.netresec.com/?page=PCAP4SICS>. Per the source's
request, **credit CS3Sthlm / 4SICS** if these captures are redistributed or
used in training material. They are not redistributed via this repo.

## Adding a capture

1. Drop the `.pcap` in `tests/fixtures/local-samples/` (gitignored).
2. Add a row to the table above with its `sha256` (`shasum -a 256 <file>`),
   size, source URL (or generator), protocols, and what it validates.
3. Add its URL + checksum to `bin/fetch-e2e-pcaps` so others can fetch it.

This index is the lightweight precursor to the full `PCAP-CORPUS-001` corpus
(orphan-branch manifest + tiered/cached CI runner) — once a storage backend is
chosen, these rows migrate into that manifest.
