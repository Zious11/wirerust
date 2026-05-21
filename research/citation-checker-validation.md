---
document_type: deferred-finding-validation
finding_id: P-CITE-PG
policy: DF-VALIDATION-001
research_agent: vsdd-factory:research-agent
validated_on: 2026-05-21
target_branch: develop
target_head: 0082a0c
verdict: VALIDATED-WITH-CHANGES
status: complete
---

# Deferred-Finding Validation — P-CITE-PG (Citation Checker)

## Verdict

**VALIDATED-WITH-CHANGES**

The underlying problem is real, well-documented, and still open on `develop` (verified
below). A follow-up issue is justified. However, the *proposal as written* — "build a
reusable line-citation checker into the VSDD engine" — treats a symptom. The recommended
change is to (a) reframe the issue around **citation convention**, not just a checker, and
(b) **not build a bespoke Rust binary**; either adopt an existing tool (Fiberplane Drift)
or ship a small dependency-free script. Details and required scope changes below.

## 1. Is the finding still open on `develop`?

Confirmed open. As of `develop` HEAD `0082a0c`:

- `.factory/STATE.md` Deferred Findings table still lists `P-CITE-PG` as
  **MANDATORY — 6 recurrences** with required action "Create a spec-CI citation-checker."
- The recurrence count is in fact understated. The STATE.md prose header (line 169) says
  "passes 4, 6, 8, 9, 10, 12"; the user-supplied summary says "4, 6, 8, 9, 10, 12, 14, 18,
  20, 21, 30" — 11 passes. Reading the per-pass log (STATE.md lines 95–128) confirms
  stale-anchor / mis-anchored-citation findings in passes 4, 6, 8, 9, 10, 12, 13, 14, 17,
  18, 20, 21, and 30, plus three large inter-pass remediation SWEEPs (~58, ~68, ~48
  defects). The finding is not only open, it is **the single most recurrent defect class
  in the entire Phase-1 adversarial gate** and has reset the convergence streak at least
  three times.
- No citation-checker tool exists in the repo or in the VSDD plugin `bin/` directory
  (verified: plugin `bin/` contains `lobster-parse`, `factory-*`, `compute-input-hash`,
  `research-cache`, etc. — no citation/anchor validator).
- Scale of the exposure: a content search across `.factory/specs/` found **1126
  `file.rs:NNN[-MMM]` citations across 249 files**. Every one is a hand-maintained line
  number subject to drift.

Conclusion: the finding is sound and unresolved. It passes DF-VALIDATION-001.

## 2. Is the problem real and well-known? (sources cited)

Yes. Line-number drift in docs-to-code links is a recognized, named problem ("documentation
rot" / "comment drift") with an established consensus best practice.

**What systems that link prose to source actually do:**

| System | Mechanism | Verdict on raw line numbers |
|--------|-----------|------------------------------|
| Sphinx `literalinclude` | Supports `:lines:` **and** `:start-after:` / `:end-before:` named string markers | Docs and community guidance (Simon Willison's TIL) recommend **named markers** precisely because "you don't need to update line numbers when code changes." |
| mdBook `{{#include}}` | Supports `file.rs:2:10` line ranges **and** named `ANCHOR`/`ANCHOR_END` comment regions | mdBook's own docs explicitly say to use **anchors instead of line numbers** "to avoid breaking your book when modifying included files." |
| PyMdown Snippets (Material for MkDocs) | Line-range syntax `file.md:4:6` | Line-based; brittle by the same mechanism. |
| MarkdownSnippets, Bluehawk | Named/region extraction from source comments | Region/symbol based — line numbers avoided. |
| GitHub permalinks | Line anchors are pinned **to a commit SHA** (`blob/<sha>#L10`) | Line numbers are only safe when frozen against an immutable commit; on a moving branch they drift. |
| Fiberplane Drift (purpose-built doc-rot linter) | Path + optional **symbol name** + **AST-hash signature** + git-SHA provenance | Deliberately uses **no line numbers at all**. |

The consensus across every mature system is the same: **raw line numbers on a moving
branch are an anti-pattern.** The robust alternatives, in rough order of strength, are
(1) symbol/named-anchor references, (2) content/AST-hash anchors, and (3) line numbers
*pinned to an immutable commit SHA*. Plain `file.rs:NNN` against `develop` is the one
combination everyone avoids.

## 3. Is line-number citation itself the anti-pattern? (be direct)

**Yes — and the proposal as written treats a symptom.** A checker that flags citations
pointing at blank/comment lines or past EOF will catch *some* drift, but it has a large
blind spot: when an inserted import or doc-comment shifts a block by N lines, the citation
very often still lands on a non-blank, non-comment code line — just the *wrong* code line.
The checker reports "ok" while the citation is semantically stale. This is exactly the
drift mode STATE.md describes ("an added import or doc-comment shifts every line below
it"). A blank/comment/EOF checker therefore provides false confidence against the dominant
failure mode.

The root cause is the citation *convention*, not the absence of a checker. The durable
fixes are convention changes:

- **Symbol references** — anchor to `decode_packet` or `TcpReassembler::new`, not to a
  line. Stable across any edit that does not rename/delete the symbol. wirerust BCs
  already name the symbol in prose (e.g. BC-2.04.001 names `TcpReassembler::new`), so the
  data to do this already exists; it is largely a formatting change.
- **Commit-pinned line numbers** — keep `file.rs:NNN` but require an accompanying
  `@<sha>` (the GitHub-permalink model). A citation is then verifiable and never silently
  drifts; it just goes "stale-needs-review" when the SHA is old.
- **Content/AST-hash anchors** — strongest, but heaviest to author by hand.

**Recommendation:** the follow-up issue should be reframed from "build a line-citation
checker" to "**adopt a drift-resistant citation convention for VSDD spec artifacts, plus a
checker that enforces it.**" A checker is still wanted — but it should validate a
*good* convention, not bless a fragile one. Building only the blank/comment/EOF checker
would be shipping a symptom-level mitigation and is likely to recur.

## 4. Feasibility and scope of the generic part

The narrowly-scoped mechanical checks are genuinely cheap and language-agnostic:

- **Past-EOF detection** — count lines in the target file; pure, trivial, language-neutral.
- **Citation-target-exists** — same; trivial.

The "blank line" and "comment-only line" checks are **only partly** language-agnostic:

- *Blank-line* detection is language-neutral.
- *Comment-only* detection is **not** language-agnostic. It needs per-language comment
  syntax: Rust/TS/Go `//` + `/* */`; Python `#` + triple-quoted strings (which are not
  even comments — docstrings are expressions); Go raw strings; etc. A naive `//` rule
  misclassifies. The proposal's step 3 (symbol-presence near cited lines) is *more*
  language-specific still and effectively needs a parser/tree-sitter to do well.

What breaks for non-Rust VSDD targets (TypeScript, Python, Go): the regex `\.rs:` anchor
pattern is Rust-specific; comment-syntax tables must be extended; Python's significant
whitespace and docstrings make line-class heuristics noisier. None of this is a blocker,
but it means the "generic" part is really "EOF + blank-line + extensible comment table,"
and the symbol-proximity part is not cheap or generic. Scoping the issue around only the
genuinely cheap checks (EOF + blank) plus a convention change is the defensible MVP.

**Inconclusive flag:** I could not quantify what fraction of the 11+ recurrences would
have been caught by a blank/comment/EOF checker vs. would have slipped through as
"wrong-but-plausible-line." STATE.md descriptions ("off-by-one citations", "line shifts")
strongly suggest a *majority* would slip through a blank/comment-only checker. This should
be spot-checked against 5–10 of the actual historical findings before committing scope.

## 5. Implementation-language recommendation

The proposal suggests a Rust binary compiled into the VSDD plugin. Assessment of the
distribution tradeoff for a Claude Code plugin (markdown skills + `bin/` helpers):

- The plugin's existing `bin/` helpers are **shell scripts** (`lobster-parse` is bash
  wrapping `yq`+`jq`; `factory-*` helpers are scripts). There is **no precedent for a
  compiled binary** in this plugin.
- A precompiled Rust binary means per-platform builds (macOS arm64/x64, Linux) committed
  to the plugin repo, or a release-download step — heavy for a markdown-centric plugin.
- "cargo build on install" is the worst option: forces a Rust toolchain on every plugin
  user regardless of their target project's language, and slows install.
- A **single-file script with no third-party dependencies** (POSIX `sh`, or Python 3
  stdlib which is present on every supported platform) matches the existing `bin/`
  convention, needs no build step, and is trivially portable. The EOF + blank-line checks
  are ~50 lines.

**Recommendation:** do **not** ship a Rust binary. If VSDD builds its own checker, make it
a dependency-free script consistent with the existing `bin/` helpers. Reserve a compiled
binary only if the symbol-proximity / tree-sitter feature is pursued, and even then prefer
wrapping an existing tool (next section).

## 6. Prior art to reuse

**Fiberplane Drift** (`github.com/fiberplane/drift`, MIT, latest **v0.10.0**, released
2026-05-08) is a near-exact match for this problem. It is a CLI documentation-rot linter
that binds spec/markdown files to source code, supports **Rust, TypeScript, Python, Go,
Zig, Java** via tree-sitter, uses path + symbol + AST-hash anchors (no line numbers),
stores bindings in a `drift.lock`, exits non-zero on staleness (`drift check`), and is
explicitly designed to run as a CI gate. It even ships a `setup` GitHub Action and an
`npx skills add fiberplane/drift` install path. Note Drift is written in **Zig** and would
need Zig 0.16.0 to build from source, but it distributes prebuilt binaries via GitHub
releases, so that is a download, not a build dependency.

Drift implements *every* durable mechanism recommended in section 3 and the cross-platform
language support flagged in section 4. The main caveat: adopting Drift means migrating
wirerust's 1126 line-citations to Drift's symbol/AST-anchor convention — real work, but it
is the *correct* work (it is the section-3 convention change), and `drift link` automates
the stamping.

**lychee** (`crates.io/crates/lychee`, latest **0.24.2**, published 2026-05-01) is *not* a
fit for source-line anchors — it checks URLs and HTML/markdown anchor fragments, not
`file.rs:NNN` source citations. Listed only to close the question: it does not solve this
problem and should not be wrapped for it.

MarkdownSnippets and Bluehawk solve *snippet injection* (pulling code into docs), not
*citation validation* (asserting a prose claim still matches a location). They are
adjacent, not reusable here.

**Recommendation on build-vs-adopt:** strongly prefer **adopt Drift** over building from
scratch. It is purpose-built, MIT-licensed, multi-language, CI-ready, and already
implements the convention change section 3 says is the real fix. A from-scratch VSDD
checker should be the fallback only if a dependency on an external Zig binary is judged
unacceptable for the plugin — in which case build the *minimal* dependency-free script
(section 5), and scope it to EOF + blank-line + commit-SHA-pinning enforcement rather than
re-implementing tree-sitter symbol resolution.

## Recommended Issue Framing (for the filed GitHub issue)

Title: `feat(spec-ci): adopt drift-resistant source-citation convention + checker`

Scope the issue as:
1. **Decide the convention** — symbol anchors and/or commit-SHA-pinned line numbers for
   all `file.rs:NNN` citations in VSDD spec artifacts. (This is the load-bearing change.)
2. **Adopt vs. build** — evaluate Fiberplane Drift v0.10.0 as the checker; fall back to a
   dependency-free `bin/` script only if an external binary is unacceptable.
3. **CI gate** — wire `drift check` (or the script) into spec-CI, analogous to
   `cargo fmt --check`, exit-non-zero on stale citations.
4. Treat the blank/comment/EOF-only checker as a *minimum* fallback, explicitly noting it
   does not catch wrong-but-plausible-line drift (the dominant failure mode).

This is non-blocking for the Phase-1 adversarial gate; per the Cycle-Closing Checklist a
follow-up story (this issue) satisfies the mandatory-codification requirement for a
6+-recurrence finding.

## Sources

- [Fiberplane — We built a linter for documentation rot](https://fiberplane.com/blog/drift-documentation-linter/)
- [GitHub — fiberplane/drift](https://github.com/fiberplane/drift)
- [mdBook — mdBook-specific features (`{{#include}}`, anchors vs. line numbers)](https://rust-lang.github.io/mdBook/format/mdbook.html)
- [Sphinx — Directives (`literalinclude`, `:start-after:`/`:end-before:`)](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- [Simon Willison — literalinclude with markers](https://til.simonwillison.net/sphinx/literalinclude-with-markers)
- [PyMdown Extensions — Snippets](https://facelessuser.github.io/pymdown-extensions/extensions/snippets/)
- [GitHub — SimonCropp/MarkdownSnippets](https://github.com/SimonCropp/MarkdownSnippets)
- [lychee — link checker (crates.io 0.24.2)](https://github.com/lycheeverse/lychee)
- [Claude Code — Plugins reference (`bin/` helpers)](https://code.claude.com/docs/en/plugins-reference)
- crates.io API — `lychee` `max_stable_version` 0.24.2, published 2026-05-01
- Local: `.factory/STATE.md` (Deferred Findings table, adversarial pass log), `.factory/policies.yaml`, `.factory/specs/` content search (1126 citations / 249 files), VSDD plugin `bin/` directory listing

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| WebSearch | 5 | doc-to-code linking practices, Sphinx/mdBook anchors, docs-as-tests tools, lychee, Claude Code plugin bin/ convention, Fiberplane Drift |
| WebFetch | 4 | Drift blog + repo (anchor format, language, license, version), crates.io lychee version |
| Perplexity | 0 | MCP Perplexity tools not invoked this run — see reliance note |
| Context7 | 0 | no library API surface needed |
| Tavily | 0 | not invoked |
| Read | 4 | STATE.md, policies.yaml, lobster-parse, sample BC file |
| Grep / Glob | 4 | citation-count census, spec-artifact enumeration, plugin bin/ listing |
| Training data | 1 area | general framing of "documentation rot" as a known class — corroborated by cited sources, not relied on for any version number or tool capability |

**Total external tool calls:** 9 (5 WebSearch + 4 WebFetch)
**Training data reliance:** low — every tool capability, version (Drift v0.10.0 2026-05-08,
lychee 0.24.2 2026-05-01), and best-practice claim is web-sourced; the finding's open
status is verified directly against local `develop` artifacts. Perplexity/Tavily/Context7
were not needed because the question resolved cleanly on WebSearch + primary-source
WebFetch and registry verification; this is flagged for transparency.
