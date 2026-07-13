# AC-162-005 — PR Title Uses `docs:` Semantic Prefix

**Story:** STORY-162  
**AC:** AC-162-005 (pull request title must use `docs:` semantic prefix)

---

## Verification Note

AC-162-005 is verified at PR-creation time by the pr-manager agent. The semantic PR title
format is enforced by CI via `amannn/action-semantic-pull-request`.

Per the story's AC-162-005 text:

> The pull request title uses the `docs:` semantic prefix (e.g.,
> `docs: LMR-003 template-conformance exemption + check-green-doc-tense guard tests`),
> consistent with the primary deliverable being a VP-INDEX governance amendment.
> The `bin/test_check_green_doc_tense.py` additions are supporting test changes; `docs:`
> is correct when the principal change is governance documentation (no production Rust
> changed, no new CI gate added).

The `docs:` prefix is the correct semantic type because:
- No production Rust source was changed (`src/` is untouched).
- No CI YAML was modified.
- The principal deliverable is a VP-INDEX governance amendment (documentation artifact).
- `bin/test_check_green_doc_tense.py` changes are supporting test additions; the `docs:`
  type covers the overall scope of the PR.

This AC is satisfied at PR creation — not by a file or command. The CI `semantic-pr`
check enforces the `docs:` (or other allowed) prefix at PR-open time.

---

## Result

| AC | Criterion | Verdict |
|----|-----------|---------|
| AC-162-005 | `docs:` prefix required; verified at PR-creation time by CI semantic-pr check | VERIFIED AT PR TIME |
