---
document_type: research-note
story_id: STORY-162
wave: "73"
produced_by: research-agent
date: 2026-07-10
---

# STORY-162 Research Note

Supporting decisions for wave-73 STORY-162 delivery (LMR-003 template-conformance
exemption + `check-green-doc-tense` main() guard self-tests).

Three questions researched, per-question findings, recommendations, and confidence.

---

## Q1. Hermetic Python test patterns for CLI exit-code precision (AC-162-003) and repo-root detection (AC-162-004)

### 1a. Exact-exit-code assertion (exit 1 vs exit 2 disambiguation)

**Finding.** The idiomatic modern-Python pattern (2020-2026) is to design `main()`
to **return** an integer exit code and confine `sys.exit()` to the `if __name__
== "__main__":` wrapper. Tests then call `main()` directly and assert on the
returned integer — no `SystemExit` interception required. `check-green-doc-tense`
already follows this design: `def main() -> int:` returns `1`, `2`, or `0`, and
line 409 does `sys.exit(main())`. This means AC-162-003 can be satisfied by
`assert mod.main() == 1` — the exact-value assertion is directly supported by
the existing code shape. [Python `sys.exit` docs][1]; [pytest `pytest.main`
returns exit code, does not raise SystemExit][2]; [Real Python CLI testing][3];
[Brian Okken, arrange-act-assert idiom][4].

Where `SystemExit` must be intercepted (e.g., an inner function calls
`sys.exit()` unconditionally), `unittest.assertRaises(SystemExit) as cm` +
`self.assertEqual(cm.exception.code, N)` or pytest's `with pytest.raises(
SystemExit) as excinfo: ... assert excinfo.value.code == N` is the documented
pattern. [Pytest getting-started][5]; [DEV community: testing exit codes with
pytest][6].

**For patching `_collect_rust_files` to `[]`:** the existing AC-158-005 block
already does exactly this (`mod._collect_rust_files = lambda _r: []`) with a
try/finally restoring the original. This manual-patch-then-restore is a valid
in-stdlib pattern; a pytest-style `monkeypatch.setattr` would be cleaner but the
codebase's self-tests are pure-stdlib-only (per story AC and existing style in
`test_compute_input_hash.py`), so the try/finally pattern is the right match.

**Recommendation (Q1a):** For AC-162-003, use the try/finally patch pattern
already established by AC-158-005, but **assert `mod.main() == 1`** (exact
value) instead of `!= 0`. Combine with the AC-162-004 hermetic fixture (below)
so the repo-root guard passes and only the zero-file guard can fire.

**Confidence:** HIGH. This is well-documented and the tool already returns
integer codes.

### 1b. Hermetic repo-root detection — three-option comparison

The story's AC-162-004(c) lists three approaches:

| Approach | Assessment |
|----------|------------|
| (a) Extract `_find_repo_root(start: Path) -> Path \| None` helper and unit-test it | **Idiomatic and preferred.** Direct precedent: Black's `find_project_root` helper is tested this way. [psf/black tests][7] Aligns with pytest's `tmp_path` guidance. [pytest tmp_path][8] |
| (b) Monkey-patch `Path(__file__).resolve` | **Not recommended.** No authoritative docs endorse patching `__file__` or `Path.resolve`. Python's importlib docs warn that `__file__` and `__spec__.origin` can drift when patched. [Python importlib docs][9] Fragile and non-idiomatic. |
| (c) Env-var override (`WIRERUST_REPO_ROOT`) consulted by `main()` | **Acceptable and already used in-repo.** `bin/compute-input-hash` supports this override (per CLAUDE.md: "Override with `WIRERUST_REPO_ROOT=/path/to/repo` if auto-detection fails"). Consistent with pytest `monkeypatch.setenv` guidance for hermetic env-var control. [pytest monkeypatch][10] |

**Recommendation (Q1b):** **Adopt Option (a) — extract `_find_repo_root(start:
Path) -> Path | None`.** Rationale:

1. It is the strongest hermeticity: no env-var pollution, no `__file__`
   patching, no reliance on the live repo layout.
2. It matches the Black precedent, which is the most widely cited Python CLI
   using this exact sentinel-walking pattern. [psf/black][7]
3. It is a minor, low-risk refactor of `bin/check-green-doc-tense` lines
   354-367 — just lift the walk into a small helper and call it from `main()`.
4. It enables both AC-162-003 and AC-162-004 with one shared testing surface.

**Fallback if refactor is out-of-scope:** Option (c) env-var override
(`WIRERUST_REPO_ROOT`) is the second-best. It is already the canonical in-repo
pattern per `bin/compute-input-hash` and CLAUDE.md.

**Confidence:** HIGH.

### 1c. Consistency with `bin/test_compute_input_hash.py` (the story's named canonical pattern)

The story names `bin/test_compute_input_hash.py` as the "canonical pattern for
hermetic tests in `bin/` tools using `tempfile.TemporaryDirectory()` +
`WIRERUST_REPO_ROOT` override." Reading that file:

- It uses `tempfile.TemporaryDirectory()` extensively (every test).
- It does **not** actually use `WIRERUST_REPO_ROOT` — it passes `tmp_dir`
  directly as the `repo_root` argument to `compute_hash(story, tmp_dir)`,
  because `compute_hash` takes `repo_root` as an explicit parameter.
- It uses `exec(compile(...))` to load `compute-input-hash` (a different
  loading style than `SourceFileLoader`).

**Implication:** The truly canonical in-repo pattern is **"pass the repo root
as an explicit parameter to a pure helper"** — which is Option (a) with a
different name. The story's mention of `WIRERUST_REPO_ROOT` in AC-162-003 is
technically inaccurate for the existing canonical pattern; the actual canonical
technique is helper-with-explicit-param.

**Recommendation:** Option (a) is fully consistent with the canonical in-repo
pattern (helper-with-explicit-param testing, not env-var overriding).
`bin/check-green-doc-tense` should factor `_find_repo_root(start: Path)` and
`_collect_rust_files(repo_root: Path)` as pure, injectable helpers — mirroring
`compute_hash(story, repo_root)`.

**Confidence:** HIGH. Verified by direct file read.

---

## Q2. Governance precedent for AC-162-001 Option A (extend closed allowlist) vs Option B (bounded exemption clause)

**Finding.** Industry precedent is **split but patterned**:

- **Option A (explicit enumeration) is dominant in immutable-normative-document
  regimes:** IETF RFCs (RFC 2026 / BCP 9, RFC 8126 registry design, IESG errata
  processing statement), ISO/IEC technical standards, Rust RFC process (post-
  acceptance minor amendments only), Kubernetes KEPs, and ADR practice all
  favor **explicit enumeration** for closed allowlists in locked documents.
  Changes are made by explicit amendment (add named entries) with modified-log
  citation. [RFC 2026][11]; [RFC 8126][12]; [IESG errata guidance][13]; [Rust
  RFC process][14].
- **Option B (bounded exemption/tailoring) is dominant in risk-based tailoring
  regimes:** NIST SP 800-53 tailoring guidance and ISO 42001 reference-control
  lists use abstract classes of permissible modifications — but critically,
  those regimes were **designed from the outset for tailoring**, not
  retrofitted. [NIST SP 800-53][15]; [NIST SP 800-53 PL-11 tailoring][16].
- **Retrofitting Option B into a document originally designed with a closed
  allowlist is uncommon and typically weakens the closed-list assurance
  property** by introducing interpretive complexity (auditors must evaluate
  conditions, not just membership).

**Assessment for LMR-003.** LMR-003 is a governance rule protecting **locked
VP documents** (integrity-focused, assurance-mechanism-role). Its closed
allowlist (currently `kani_version:` only) is a strict-normative-contract
allowlist, not a tailoring baseline. This aligns LMR-003 with the IETF /
Rust-RFC / ADR precedent cluster where **Option A is the mainstream pattern**.

**However**, the STORY-162 AC-162-001 Option B has an important safety property
the generic exemption clause pattern lacks: it embeds **objectively-testable
sentinel conditions** (`inputs: []` MUST be empty; `input-hash:` MUST be exactly
`d41d8cd`; both fields co-required; modified-log citation required). This turns
what would normally be interpretive judgment into a mechanical validator check
— exactly the "embedded controls" pattern NIST tailoring guidance recommends
when Option B is used. [NIST tailoring / POA&M documentation controls][15][16].

**Recommendation (Q2):** **Option A (extend the allowlist) is closer to mainstream
governance precedent** for a strict-normative-contract closed allowlist like
LMR-003, and better preserves the intent of closure (membership-testable, no
interpretation needed). Add two rows to the LMR-003 allowlist table for
`inputs:` and `input-hash:` with the permitted-meaning column narrowly worded:
"template-conformance provenance; MUST be `[]` / `d41d8cd` respectively;
modified-log citation required."

If the implementer prefers Option B for expressive brevity, it is defensible
**because** the exemption conditions are sentinel-value-testable (`inputs: []`
and `input-hash: d41d8cd` are constants, not conditions of judgment), which
is exactly the property NIST tailoring guidance requires for a robust bounded
exemption. Option B is therefore not out of family — but Option A is the safer
default for a document whose closed allowlist is an assurance primitive.

**Confidence:** MEDIUM-HIGH. Precedent is patterned but not universal; both
options are defensible. Option A is the lower-risk / more-conventional choice.

---

## Q3. importlib-loaded script testing pitfalls (SourceFileLoader + spec_from_loader)

**Finding.** Three concrete pitfalls documented:

1. **sys.modules caching / module-level state leakage.** Python's import system
   consults `sys.modules` first and reuses cached module objects. [Python import
   reference][17] Loading via `SourceFileLoader` and executing the module puts
   it into `sys.modules` under the given name. Subsequent tests that mutate
   module attributes (e.g., `mod._collect_rust_files = lambda ...`) persist
   until the module is explicitly reloaded or the attribute is restored. The
   existing AC-158-005 block in `test_check_green_doc_tense.py` correctly uses
   try/finally to restore `mod._collect_rust_files`. New AC-162-003/004 tests
   MUST follow the same discipline — or use a fresh module load per test.
   [Python `reload` caveats — cached refs may persist][18].

2. **`__file__` resolution semantics inside the loaded module.** The loader
   sets `__file__` to the source file path passed to `SourceFileLoader`. So
   inside `check-green-doc-tense`, `Path(__file__).resolve()` resolves to the
   **actual `/Users/zious/Documents/GITHUB/wirerust/bin/check-green-doc-tense`
   path**, not to anything under a temp tree. This is the root cause of the
   AC-162-004(c) requirement: hermetic tests cannot rely on
   `Path(__file__).resolve()` to point inside the temp fixture. Python's
   importlib docs additionally warn that manually reassigning `__file__` does
   NOT update `__spec__.origin`, creating divergence. [importlib docs on
   `__file__` vs `__spec__.origin`][9]

3. **Monkey-patching module attributes without cross-test contamination.** The
   existing self-test uses direct attribute mutation (`mod._collect_rust_files
   = lambda ...`) with try/finally restore. This is safe **provided** every
   test using the pattern also restores. A safer alternative is per-test module
   reload using `importlib.reload(mod)`, but the cached-references caveat from
   Python's docs means reload can still leave stale references in other places.
   [Python module reload behavior][18] The try/finally-restore approach is
   sufficient for the current test file structure (single-threaded, sequential,
   in-process).

**Recommendation (Q3):**

- Continue the try/finally attribute-restore discipline for all new tests. Do
  not introduce `importlib.reload`.
- Explicitly recognize that `Path(__file__).resolve()` inside the loaded module
  will point to the real `bin/` path — this is exactly why AC-162-004(c)
  requires one of the three approaches (helper extraction, `__file__` monkey-
  patch, or env-var override). Aligns with the Q1b recommendation.
- If future tests need to inspect the loaded module's `__spec__`, be aware
  `__spec__.origin` may differ from `__file__` if any code mutates one; the
  test file currently does not do this and should not start.

**Confidence:** HIGH for pitfalls 1 and 2 (well-documented). MEDIUM for the
specific recommendation on `reload` (documented caveats, but limited direct
evidence for testing scenarios).

---

## Summary of Three Recommendations

1. **Q1 — Hermetic testing (AC-162-003 / AC-162-004):** Extract
   `_find_repo_root(start: Path) -> Path | None` as a pure helper in
   `bin/check-green-doc-tense` (Option (a) from AC-162-004(c)). Test it
   directly with `tempfile.TemporaryDirectory()` for both the `.git`-sentinel
   and `.factory/`-sentinel arms. For AC-162-003, use the existing try/finally
   patch pattern on `_collect_rust_files` and assert `mod.main() == 1` exactly.
   This mirrors the canonical in-repo pattern of `test_compute_input_hash.py`
   (helper-with-explicit-param) and Black's `find_project_root`.
   **Confidence: HIGH.**

2. **Q2 — LMR-003 amendment shape (AC-162-001):** Prefer **Option A (extend
   the allowlist)** — it matches mainstream governance precedent (IETF, ISO,
   Rust-RFC, ADR) for immutable-normative closed allowlists. Add two narrowly-
   worded rows (`inputs:` and `input-hash:`) with sentinel-value constraints
   and a modified-log citation requirement. Option B is defensible **only
   because** its exemption conditions are sentinel-testable (`[]` / `d41d8cd`
   are constants), but Option A is the safer default.
   **Confidence: MEDIUM-HIGH.**

3. **Q3 — importlib pitfalls:** Continue the existing try/finally attribute-
   restore discipline; recognize that `Path(__file__).resolve()` inside a
   `SourceFileLoader`-loaded module points to the real file (not the temp
   tree), which is the root cause of AC-162-004(c). Do not use
   `importlib.reload` — cached references make it unreliable. Do not mutate
   `__file__` — Python docs warn `__spec__.origin` will drift.
   **Confidence: HIGH.**

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity `perplexity_research` (PRIMARY)** | 2 | Q1 (hermetic Python CLI testing patterns, importlib pitfalls — reasoning_effort=high) + Q2 (governance precedent for closed allowlist vs bounded exemption — reasoning_effort=medium) |
| Read (local files) | 4 | STORY-162 spec, `bin/check-green-doc-tense`, `bin/test_check_green_doc_tense.py`, `bin/test_compute_input_hash.py` — verified in-repo canonical pattern claims |
| Training data | minimal | Only for tool-selection reasoning; all substantive claims sourced from cited web references or verified from in-repo files |

**Total MCP tool calls:** 2 `perplexity_research` (high + medium effort).
**Training data reliance:** LOW — all substantive claims traced to cited sources
or to direct in-repo file inspection.

---

## Sources

[1] Python 3 docs, `sys.exit`. https://docs.python.org/3/library/sys.html
[2] Pytest usage docs, `pytest.main()` returns exit code (v6.2). https://docs.pytest.org/en/6.2.x/usage.html
[3] Real Python — Testing Python CLI Apps. https://realpython.com/python-cli-testing/
[4] Brian Okken — pytest arrange-act-assert talk. https://www.youtube.com/watch?v=43mwW9IEo8M
[5] Pytest getting-started (`pytest.raises(SystemExit)`). https://docs.pytest.org/en/stable/getting-started.html
[6] DEV community — Testing exit codes with pytest. https://dev.to/boris/testing-exit-codes-with-pytest-1g27
[7] psf/black `find_project_root` tests. https://github.com/psf/black/blob/main/tests/test_black.py
[8] Pytest `tmp_path` fixture. https://docs.pytest.org/en/stable/how-to/tmp_path.html
[9] Python 3 `importlib` docs (`__file__` / `__spec__.origin` semantics). https://docs.python.org/3/library/importlib.html
[10] Pytest `monkeypatch`. https://docs.pytest.org/en/stable/how-to/monkeypatch.html
[11] IETF RFC 2026 / BCP 9 — The Internet Standards Process. https://datatracker.ietf.org/doc/html/rfc2026
[12] IETF RFC 8126 — IANA Considerations. https://datatracker.ietf.org/doc/html/rfc8126
[13] IESG statement on RFC errata processing. https://datatracker.ietf.org/doc/statement-iesg-iesg-processing-of-rfc-errata-for-the-ietf-stream-20210507/
[14] Rust RFC process. https://github.com/rust-lang/rfcs
[15] NIST SP 800-53 rev 5. https://nvlpubs.nist.gov/nistpubs/specialpublications/NIST.SP.800-53r5.pdf
[16] NIST SP 800-53 PL-11 (tailoring). https://csf.tools/reference/nist-sp-800-53/r5/pl/pl-11/
[17] Python import reference (sys.modules caching). https://docs.python.org/3/reference/import.html
[18] Python module reload caveats (educational reference). https://www.youtube.com/watch?v=W9hisiG0Vq8
[19] RFC 8174 / RFC 2119 — normative key words. https://datatracker.ietf.org/doc/rfc8174/
