# HS-F4-001 Frame C — Validated Triage (DF-VALIDATION-001)

| Field | Value |
|-------|-------|
| Finding | HS-F4-001 Frame C — degenerate all-zero ClientHello accepted by public CLI (JA3 emitted, `parse_errors=0`) instead of PC-9-rejected |
| Validated against | develop HEAD `8b52046` |
| Validator | vsdd-factory:research-agent |
| Date | 2026-06-30 |
| Verdict | **B (HOLDOUT-OVERSPECIFIED / SPEC-PROSE-WRONG) + C (pre-existing, out-of-fragmentation-delta)**; NOT A. Plus a distinct **artifact-fidelity defect** (BC prose + Red-Gate test do not match). |
| Confidence | High (parser behavior confirmed against vendored `tls-parser` 0.12.2 source) |

## 1. What the holdout observed (CONFIRMED CORRECT)

An assembled handshake message `[0x01,0x00,0x01,0x00]` + 256 zero bytes (msg_type=ClientHello,
body_len=256, all-zero body) fed through the public path
(`on_data → try_parse_records → ClientToServer carry drain → parse_tls_message_handshake`,
`src/analyzer/tls.rs:936`) is **accepted**: `parse_tls_message_handshake` returns
`Ok(ClientHello)`, the code matches the `Ok((_rem, …ClientHello(ch)))` arm
(`tls.rs:937-947`), sets `client_hello_seen=true`, calls `handle_client_hello`, and emits a JA3.
`parse_errors` stays 0. The holdout observation is factually correct.

## 2. The holdout-evaluator's HYPOTHESIS is WRONG

The hypothesis posited two divergent paths — a "strict lower-level decode seam" driven by the
Red-Gate test vs a "lenient JA3-extraction path" in the public CLI. **There is only one path.**
Per BC-2.07.038 EC-007, single-record ClientHellos also flow through the carry drain loop, and
both the Red-Gate test and the CLI reach the identical `parse_tls_message_handshake` call at
`tls.rs:936`. There is no separate lenient extraction seam.

The real divergence is **input content**, not code path:

- **Red-Gate test `test_BC_2_07_038_canonical_frame_rfc8446_s4`, Frame C**
  (`tests/tls_analyzer_tests.rs:9619-9620`) uses a body of **256 × `0xcc`**, NOT zeros:
  ```rust
  let mut frame_c_hs: Vec<u8> = vec![0x01, 0x00, 0x01, 0x00]; // body_len=256
  frame_c_hs.extend(vec![0xcc; 256]); // malformed body
  ```
  With `0xcc`, the session_id length byte = `0xcc` = 204 > 32 → tls_parser's
  `verify(be_u8, |&n| n <= 32)` (`tls_handshake.rs:482`) fails → `Err` → `parse_errors+1`.
  The test passes — but it exercises a *different input* than the spec's stated Frame C.

- **HS-F4-001 / BC AC-CANONICAL-FRAME Frame C** specify an **all-zero** body. All-zero is a
  structurally valid degenerate ClientHello (see §3) → `Ok` → accepted → `parse_errors=0`.

## 3. tls_parser 0.12.2 ground truth (vendored source, authoritative)

`~/.cargo/registry/.../tls-parser-0.12.2/src/tls_handshake.rs`:

- `parse_tls_handshake_client_hello` (L479-491): `version` (u16), `random` (32B),
  `sidlen = verify(be_u8, n<=32)` (L482), `ciphers_len` (u16),
  `ciphers = parse_cipher_suites(i, len)`, `comp_len` (u8), `comp`, `ext = opt(complete(length_data(be_u16)))`.
- `parse_cipher_suites` (L516-528): **`if len == 0 { return Ok((i, Vec::new())) }`** — an
  empty cipher-suite list is EXPLICITLY ACCEPTED, not an error.
- `parse_compressions_algs` (L530-542): `len == 0` → `Ok` empty.
- `parse_tls_message_handshake` (L874-907): `take(hl)` carves exactly `body_len` bytes as
  `raw_msg`, then `let (_, msg) = …parse_tls_handshake_msg_client_hello(raw_msg)…` —
  **the `_` DISCARDS the inner parser's remaining bytes**, so 216 trailing zero bytes inside the
  declared length are silently ignored. Returns `Ok`.

Trace of all-zero 256-byte body: version=0, random=0, sidlen=0 (ok), no sid, ciphers_len=0
(empty, ok), comp_len=0 (empty, ok), ext-len=0 → `Some(&[])`, 216 trailing zeros → remainder →
discarded. Net: **`Ok(degenerate ClientHello)`**. `handle_client_hello` then computes a JA3 from
version=0 / empty ciphers; no `parse_tls_extensions` failure (ext is empty) → `parse_errors`
stays 0. This reproduces the holdout observation exactly.

External corroboration (Perplexity `sonar-deep-research`, 2026-06-30): nom/`tls-parser` is a
syntactic parser without semantic validation; structurally well-formed ClientHellos with empty
cipher lists are accepted. RFCs (5246 §7.4.1.2, 8446 §4.1.2) treat an empty cipher list as a
protocol violation a *server* must reject, but passive fingerprinting tools (Zeek, Suricata, JA3)
are lenient and fingerprint whatever parses — consistent with accepting the degenerate hello.

## 4. What PC-9 actually scopes to

BC-2.07.038 PC-9 (L98-108) is defined **entirely in terms of the `parse_tls_message_handshake`
return value**: "If `parse_tls_message_handshake` returns `Err(_)` … `parse_errors` is incremented
by exactly 1." It does NOT mandate rejecting empty cipher lists or trailing padding. Since the
parser returns `Ok` for the all-zero body, PC-9 simply does not fire, and dispatching it as a
ClientHello is exactly PC-3a. **The code is conformant to PC-9 as written.**

The defect is in the BC PROSE, not the code: PC-9's malformed-body examples list
"zero-length cipher suite list" (L100) and the AC-CANONICAL-FRAME Frame C narrative (L220-229)
and the HS-F4-001 registry (line 56-58) all assert the all-zero body yields `Err`/`parse_errors+1`.
**This is factually false** for tls_parser 0.12.2 — an empty cipher list is accepted (L517).

## 5. Does the leniency predate STORY-144?

Behaviorally **yes** (verdict C). Constraints: this agent has no shell/git access, so no
`git blame`/`git log -L` line citation is possible — this is a reasoned inference from code+spec,
flagged as such.

- The carry path itself is new (BC-2.07.038 `introduced: fix-tls-clienthello-frag`), so the exact
  lines `tls.rs:936-956` are new code.
- But PC-9(a) itself states the behavior is "parity with the single-record path … where
  `parse_tls_plaintext` failure increments `parse_errors`." The pre-fix single-record path used
  `parse_tls_plaintext`, which internally calls the same `parse_tls_message_handshake`/ClientHello
  parser and accepts degenerate hellos identically. So accepting an all-zero degenerate ClientHello
  (and ignoring trailing intra-length bytes) is a **pre-existing tls_parser property**, unchanged by
  the fragmentation work. It is orthogonal to STORY-144/145/146.
- (Minor staleness: PC-9(a)'s "tls.rs L787-789 … parse_tls_plaintext" citation is now stale — the
  single-record `parse_tls_plaintext` path was folded into the carry path; L787-789 is now the
  ServerToClient record clone.)

## 6. Verdict & routing

**Verdict: B + C (co-applied). NOT A.** The public CLI's acceptance of a degenerate all-zero
ClientHello conforms to BC-2.07.038 PC-9 as literally written, matches tls_parser 0.12.2's
documented behavior, matches real-world lenient JA3 tooling, and the leniency predates the
fragmentation cycle. No fragmentation-correctness regression exists. No code fix is warranted on
correctness grounds.

There IS, however, a real **artifact-fidelity defect** (spec ↔ test ↔ registry mismatch) worth
correcting:

1. **Refine the holdout + BC prose (product-owner / spec-author).**
   - HS-F4-001 Frame C (holdout-scenarios.md:56-58): change the expected outcome OR the input.
     Either (a) keep "all-zero body" and change the expectation to "accepted as degenerate
     ClientHello; `parse_errors=0`; `client_hello_seen=true`; JA3 emitted" (matches reality), or
     (b) change the input to a body that actually fails to parse (e.g. session_id length > 32, as
     the test uses) if a malformed-reject case is the intent.
   - BC-2.07.038 AC-CANONICAL-FRAME Frame C (L220-229) and PC-9 example list (L100): remove/correct
     the false claim that an all-zero body / "zero-length cipher suite list" yields `Err`. Replace
     with an example that genuinely fails (session_id length > 32, truncated extension, odd cipher
     list length).
   - Fix the stale `parse_tls_plaintext` L787-789 citation in PC-9(a).

2. **Test fidelity (test-writer).** `test_BC_2_07_038_canonical_frame_rfc8446_s4` Frame C currently
   uses `0xcc` while the spec text says "all-zero". After the BC is corrected, align the test body
   to the corrected spec (and document why `0xcc`, not zeros, is the malformed vector — because
   zeros parse as a degenerate-valid hello).

3. **Backlog (future correctness sweep, optional, NOT this cycle).** If product wants stricter
   structural validation (reject empty-cipher-suite / trailing-padded ClientHellos as an evasion
   anomaly, possibly emitting a finding rather than a JA3), file a separate maintenance item. This
   is a deliberate policy change with real-world-tooling tradeoffs (lenient fingerprinting is the
   norm), not a bug, and is orthogonal to fragmentation.

**No GitHub issue should assert a fragmentation correctness regression.** Per DF-VALIDATION-001,
the only issue-worthy item is the artifact-fidelity correction (items 1–2), routed to
product-owner/spec-author and test-writer.

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 2 | (1) tls-parser 0.12.2 ClientHello empty-cipher + trailing-byte behavior; (2) RFC 8446/5246 + JA3 tooling treatment of degenerate ClientHellos |
| WebSearch | 1 | Locate authoritative tls-parser ClientHello source (session_id ≤ 32 check) |
| WebFetch | 3 | Attempted GitHub/Fossies/docs.rs source fetch (404/401 — fell back to vendored crate source) |
| Read (vendored tls-parser 0.12.2 source) | 3 | Authoritative parser behavior: `parse_tls_handshake_client_hello`, `parse_cipher_suites`, `parse_tls_message_handshake` |
| Read/Grep (project) | — | BC-2.07.038, holdout registry, `src/analyzer/tls.rs`, Red-Gate test |
| Training data | 1 area | nom combinator semantics (cross-checked against vendored source — not relied upon) |

**Total MCP tool calls:** 2 (both `perplexity_research`).
**Training data reliance:** low — the load-bearing parser behavior was confirmed against the
exact vendored `tls-parser` 0.12.2 source on disk, not model knowledge.

**Constraint note:** No shell/git access in this profile; the "predates STORY-144" finding (§5) is
a reasoned code+spec inference, not a `git blame` citation.
