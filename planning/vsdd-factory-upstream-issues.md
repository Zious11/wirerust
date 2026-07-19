# vsdd-factory Upstream Issues

Fetched: 2026-07-19T01:59:37Z  
Total: 465 issues (open: 457, closed: 8)  
Issue range: #126 – #687

| # | State | Labels | Title | updatedAt |
|---|-------|--------|-------|-----------|
| 687 | OPEN |  | spec-artifact citation-coherence is an ungated defect class that can stall strict "N consecutive clean passes" convergence on LOW doc-fidelity nits | 2026-07-18 |
| 686 | OPEN |  | process-gap(adversary+orchestrator): finding decay is non-monotone across fresh-context passes — review technique depth varies, and policy-trigger interpretation drifts between reviewers | 2026-07-18 |
| 685 | OPEN |  | process-gap(implementer): fix-phase self-attestation — a fix reported DONE can be unimplemented on disk and guarded by a self-authored test with all assertions commented out (green-side analogue of #475) | 2026-07-18 |
| 684 | OPEN |  | process-gap(test-writer): red-gate tests that assert stub error artifacts are un-greenable by construction | 2026-07-18 |
| 683 | OPEN |  | process-gap(agents): reviewer agent declares read-only toolset but runtime does not enforce it — declared-vs-actual capability mismatch | 2026-07-18 |
| 682 | OPEN |  | process-gap(tdd): stale RED-gate / todo!() docstrings survive the Red→Green transition and are not caught by any gate — doc-integrity drift that misstates shipped behavior | 2026-07-17 |
| 681 | OPEN |  | process-gap(hooks): validate-burst-log enforces an undocumented engine-dogfood schema that conflicts with the shipped burst-log template (dual-validator deadlock) | 2026-07-17 |
| 680 | OPEN |  | process-gap(spec-conformance): spec/contract claims about implementation APIs can be factually wrong and steer code into an invariant violation — ground API claims against the code before downstream agents consume the spec | 2026-07-18 |
| 679 | OPEN |  | process-gap(verification): fix scoped over a class of states must be verified across every reachable member — partial enum/match coverage gives false green | 2026-07-17 |
| 678 | OPEN |  | process-gap(ci): "seam excluded from release" CI check via string-grep cannot detect control-flow-only seams — dual-compile assertion required | 2026-07-17 |
| 677 | OPEN |  | process-gap(verification): "full suite green" claims require --no-fail-fast — default test runner halts at first failing binary and silently omits the rest | 2026-07-17 |
| 676 | OPEN |  | process-gap(tdd): security-invariant ACs require mutation verification — a test that still passes when the guard is removed does not test the guard | 2026-07-17 |
| 675 | OPEN |  | process-gap(tdd): seam-covered ACs can mask an unwired production entrypoint — verification obligations should require a real-entrypoint check | 2026-07-18 |
| 674 | OPEN |  | process-gap(per-story-delivery): step (f) silently degrades from 9-step pr-manager process to self-review when subagent spawning is unavailable | 2026-07-17 |
| 673 | OPEN |  | process-gap(hooks): blocking validators' fix-instructions prescribe example/placeholder text that is false for the target project (coerced fabrication/structure) | 2026-07-17 |
| 672 | OPEN |  | process-gap(story-writer+architect): input-hash recorded at decomposition doesn't match computed value — decompose workflow omits terminal `compute-input-hash --update` (2 consecutive instances) | 2026-07-17 |
| 671 | OPEN |  | Proposal: factory-graph — derived traceability graph rehydrated from .factory/ markdown to eliminate identifier cite drift | 2026-07-16 |
| 669 | OPEN |  | process-gap(story-writer+adversary): polarity inversion — correct coordinates and identifiers with a flipped predicate survive coordinate-focused review | 2026-07-16 |
| 668 | OPEN |  | process-gap(adversary+story-writer): frozen/deferred artifact sections treated as permanently shielded — freezes must expire with their blocking rationale | 2026-07-16 |
| 667 | OPEN |  | process-gap(orchestrator+spec): blocking "MUST before X" obligations lack explicit discharge records — satisfied-in-substance but unprovable | 2026-07-16 |
| 666 | OPEN |  | process-gap(orchestrator): dispatches that RECOMMEND the structurally-safer remediation form instead of MANDATING it — producers pick the fragile option | 2026-07-16 |
| 665 | OPEN |  | process-gap(test-writer+adversary): regression guards that freeze cells/values owned by a future or sibling work axis — latent false-fail generators | 2026-07-16 |
| 664 | OPEN |  | process-gap(adversary+orchestrator): review verdicts accepted without ground-truthing their components — false dismissal premises and false relayed positives | 2026-07-18 |
| 663 | OPEN |  | process-gap(cross-role): identifiers retyped from memory instead of copy-pasted from command output — SHAs, test names, commit lists silently corrupted | 2026-07-16 |
| 662 | OPEN |  | process-gap(stub-architect): stub bodies with panic()/unimplemented on a SHARED production code path regress existing passing tests — Red Gate contract must scope 'fail' to NEW tests only | 2026-07-16 |
| 661 | OPEN |  | process-gap(per-story-delivery + stub-architect): Red Gate produces a non-compiling middle commit when stubs are left uncommitted and tests are committed alone | 2026-07-16 |
| 660 | OPEN |  | bug(hooks): validate-changelog-monotonicity does not normalize the 'v' version prefix — false-positive blocks and un-fixable frontmatter | 2026-07-18 |
| 658 | OPEN |  | validate-pr-merge-prerequisites hook: STORY_ID regex doesn't recognize S-<PREFIX>.<NAME> IDs, falls back to first S-N.NN substring anywhere in prompt | 2026-07-15 |
| 656 | OPEN | enhancement | process-gap(story-writer): relocation stories must include a BC Source citation sweep task — 9 stale citations survived full bundle until F7 consistency audit | 2026-07-15 |
| 655 | OPEN | enhancement | process-gap(implementer): resume prompt must include explicit worktree path + branch name + mandatory pre-commit branch assertion | 2026-07-15 |
| 654 | OPEN | enhancement | process-gap(formal-verifier): bundle-scoped mutation runs need --timeout 480 or --jobs 2 — 240s cap causes timeout-adjudication overhead on large diffs | 2026-07-15 |
| 653 | OPEN | enhancement | process-gap(story-writer): story spec must enumerate per-variant test-function names in AC bodies — recurrence 3 in one bundle | 2026-07-15 |
| 652 | OPEN | enhancement | process-gap(adversary): mutation-coverage claims must be backed by an empirical cargo-mutants run — unverified claims are invalid | 2026-07-15 |
| 651 | OPEN |  | bug(hooks): validate-pr-review-posted demands approve/request-changes — structurally unreachable on single-identity projects; mis-detects 'gh pr review --comment' | 2026-07-16 |
| 650 | OPEN |  | bug(write-path): full-file Write leaked a stray </content> envelope tag into a spec artifact (VP-045.md) | 2026-07-15 |
| 649 | OPEN |  | bug(hooks): PostToolUse dispatcher fail-closed on plugin timeout reports valid writes as blocked | 2026-07-16 |
| 648 | OPEN |  | process-gap(hooks): plugin timeout emits block-shaped 'fail-closed: plugin timed out' while the write lands anyway — fail-open behavior with fail-closed labeling | 2026-07-16 |
| 647 | OPEN |  | process-gap(story-writer+infra): writing-agent burst dies mid-edit → partial artifact + version/changelog that overclaims the body; no crash-safe edit ordering or verify-then-resume recovery | 2026-07-18 |
| 645 | OPEN |  | process-gap(mutation-testing-protocol): transient-mutation restores via a working-tree checkout target the last COMMIT, not "whatever fix you just wrote" — uncommitted fixes are silently discardable mid mutation-battery | 2026-07-14 |
| 644 | OPEN |  | process-gap(multi-agent-identity): every agent role signing commits under one shared git identity means git forensics alone can never attribute which role authored a given commit, even when the commit's contents are correctly scoped | 2026-07-14 |
| 643 | OPEN |  | process-gap(ci-scaffolding+pr-manager): a fix landing on the base branch after a PR opens does not become visible to that PR's own CI runs without an actual branch update — reruns and new trigger events do not "pick up" the fix | 2026-07-14 |
| 642 | OPEN |  | process-gap(tooling-guards): destructive-command guard on PR-close has no semantic distinction between "abandon" and "close-then-reopen for a fresh CI check" — the same guarded command serves both intents | 2026-07-14 |
| 641 | OPEN |  | process-gap(hooks): decision-chain-citation freshness check false-positives on non-citation substrings (e.g. "RED-4", "TD-001") that merely resemble its ID pattern | 2026-07-14 |
| 638 | OPEN |  | process-gap(adversary+orchestrator): finding-ID scheme is not canonicalized — G-less F-W<NN>P<n> collides with canonical F-W<NN>G-P<n> repo-wide, enabling mis-numbered/fabricated cross-references (instantiated by F-S165P4-001) | 2026-07-13 |
| 637 | OPEN |  | bug(hooks): validate-input-hash + bash compute-input-hash use $(cat) command substitution (strips trailing newlines) — hashes diverge from raw-byte readers and the PostToolUse hook hard-blocks (exit 2) on false-positive drift | 2026-07-13 |
| 636 | OPEN |  | process-gap(demo-recorder): recording protocol has no host-path scrub step — VHS/Playwright captures leak absolute /Users/... paths into committed demo evidence | 2026-07-13 |
| 635 | OPEN |  | process-gap(orchestrator): mid-gate adversarial streak counter + CLEAN passes are never persisted to disk — a session crash before gate-summary.md loses convergence position | 2026-07-13 |
| 634 | OPEN |  | feat: story-level holdout gate — three-tier holdout evaluation in per-story delivery | 2026-07-13 |
| 633 | OPEN |  | Canonicalize SAP-3 (spec-arm reachability) and SID-2 (composed-output assertions) probes from prism | 2026-07-13 |
| 631 | OPEN |  | bug(tooling): factory-cas-push.sh assumes repo-root cwd — internal 'cd .factory' fails when invoked from inside the factory worktree | 2026-07-13 |
| 630 | OPEN |  | process-gap(dispatch+registry): code-delivery {story-id} is a free variable — two agents in one delivery minted differently-cased artifact dirs; block-level enforcement can't catch it | 2026-07-13 |
| 629 | OPEN |  | bug(hooks): verify-sha-currency SHA regex requires 8+ chars — 7-char SHAs cited in STATE.md report NOT_FOUND (false negative on every run) | 2026-07-13 |
| 628 | OPEN |  | bug(hooks): session-learning registers on Stop (turn-end) but writes 'Session ended' markers — marker inflation makes the #584 backlog look bigger than reality | 2026-07-13 |
| 627 | OPEN |  | process-gap(adversary): read-only adversary emits BLOCKER-severity findings predicated on executable-gate outcomes (compile/clippy/test) it structurally cannot run — false BLOCKER verified by actually running the gate | 2026-07-13 |
| 626 | OPEN |  | process-gap(pr-manager+pr-reviewer): review playbooks prescribe gh pr review --approve/--request-changes, which GitHub forbids from the PR author's account — single-identity factories can never produce a formal reviewDecision | 2026-07-14 |
| 625 | OPEN |  | bug(hooks): validate-policies-schema enforces a fixture-derived schema that collides with the downstream-canonical policies.yaml convention — and cites artifacts that only exist in plugin test fixtures | 2026-07-12 |
| 624 | OPEN |  | process-gap(phase-6): concurrent hardening-lane dispatch shares one checkout — secscan's deliberately-inverted mutation code leaked into the fuzz lane's view | 2026-07-12 |
| 623 | OPEN |  | compute-input-hash --update silently no-ops when the target file has no pre-existing input-hash field — reports success, writes nothing | 2026-07-12 |
| 622 | OPEN |  | process-gap(spec-agents): spec documents cite source-code line numbers with no stated coordinate baseline — a 'corrected' citation pointed at unrelated code once the landed tree diverged | 2026-07-13 |
| 621 | OPEN |  | process-gap(architect+adversary): a concurrency fix's own join/lifetime obligations aren't required in the same remediation that introduces them — a new goroutine's missing join, then its now-load-bearing defer order, surfaced two passes later | 2026-07-12 |
| 620 | OPEN |  | process-gap(architect+adversary): a design guarantee's foundational premise about pre-existing code is never traced against the baseline — twelve internally-consistent passes converged before first empirical contact falsified it | 2026-07-12 |
| 619 | OPEN |  | process-gap(orchestrator): no instruction-sequencing protocol for async delegation — crossed/reversed instructions race teammate mailboxes and multi-channel dispatch | 2026-07-17 |
| 618 | OPEN |  | process-gap(convergence): per-story convergence declared CLEAN while branch CI was red throughout — pre-existing unrelated red normalizes failure (alarm fatigue) | 2026-07-12 |
| 617 | OPEN |  | process-gap(templates): template matching is filename-based with no scope awareness or override — any convergence-report.md matches the Phase-7 scorecard | 2026-07-13 |
| 616 | OPEN |  | process-gap(hooks): PostToolUse validators evaluate whole-file pre-existing conditions, not the edit delta — and 'block' signals don't prevent the write | 2026-07-17 |
| 615 | OPEN |  | process-gap(architect+story-writer): erratum-mandated sweeps scope to the remediation's edit set and literal vocabulary, not the review perimeter and semantic vocabulary — each under-scoped sweep seeds the next pass's finding (3 instances in one arc) | 2026-07-12 |
| 614 | OPEN |  | process-gap(story-writer+architect): grep gates authored in GNU BRE \\| syntax are false-GREEN on BSD/macOS — inverse regression detectors pass vacuously (10 gates in one wave, 3 confirmed vacuous) | 2026-07-13 |
| 609 | OPEN |  | methodology(test-writer+adversary): symmetric contracts get the inbound direction pinned while the mirror outbound deliverable ships unpinned and mutation-revertible | 2026-07-11 |
| 608 | OPEN |  | process-gap(architect+adversary): spec convergence hardens a binding lifecycle contract without simulating it against the existing test suite — a resource-close contract deterministically broke a predecessor timing test, found only at GREEN | 2026-07-16 |
| 607 | OPEN |  | process-gap(test-writer+orchestrator): RED gate confirms tests fail on stubs but not that the harness can observe success — a fixture observability cap hid a correct implementation, invisible at RED | 2026-07-11 |
| 606 | OPEN |  | methodology(story-writer+architect): trait/type-surface changes need a standard multi-axis migration enumeration — impl sites, caller pattern-match sites, construction sites, and config-literal ripple discovered one axis per adversarial pass | 2026-07-11 |
| 605 | OPEN |  | process-gap(architect): binding adjudications repeatedly ship compile-broken mechanics — rulings need the same artifact-resolution + compile-mechanics verification as stories (4 errata-correcting-errata in one convergence arc) | 2026-07-13 |
| 604 | OPEN |  | resilience(review-agents): no model-fallback when the diversity endpoint degrades — 5 timeouts on one gate; different-family reroute recovered it | 2026-07-13 |
| 603 | OPEN |  | process-gap(story-writer): no cited-artifact resolution preflight — stories anchor to phantom modules, files, and APIs (3 instances in one decomposition batch) | 2026-07-11 |
| 602 | OPEN |  | process-gap(story-writer+adversary+architect): normative test-input recipes ship without physical-realizability discharge — a prescribed injection sequence unexecutable against the actual parse-order semantics survived six adversarial passes | 2026-07-13 |
| 601 | OPEN |  | Phase-4 holdout: pre-registration freezes operator-interaction budgets without a capability canary on the target build | 2026-07-13 |
| 600 | OPEN |  | process-gap(spec-agents): index sweep lands partially — changelog row without frontmatter bump + wrong-position insert, while the report asserts the full sweep | 2026-07-17 |
| 599 | OPEN |  | process-gap(lint-authoring): agent-authored lint gates silently ignore file args and under-scan — 'verified 0 violations' can be vacuous | 2026-07-13 |
| 598 | OPEN |  | test-quality(test-writer): require snapshot-restore for tests mutating global engine singletons (cross-suite pollution) | 2026-07-10 |
| 597 | OPEN |  | process-gap(implementer): guard against vendored/wrong-layer workarounds to satisfy product tests (2 field recurrences) | 2026-07-13 |
| 596 | OPEN |  | process-gap(story-writer+architect): interface-surface amendments never enumerate implementors — added method compile-breaks test fake declared 'existing, unmodified' in the same story | 2026-07-13 |
| 595 | OPEN |  | process-gap(architect+adversary): remediation amendments cite ground-truth mechanisms that don't exist, or spawn wrong-direction traps at adjacent sites — each layer's fix seeds the next layer's defect | 2026-07-11 |
| 594 | OPEN |  | process-gap(orchestrator): collision-guard file exclusions silently orphan sweep obligations — require queued deferred-sweep at exclusion time | 2026-07-09 |
| 593 | OPEN |  | Convergence lens catalog: add story-instruction-sweep and RED-gate-replay as standard probes (field data: 8-pass cycle) | 2026-07-09 |
| 592 | OPEN |  | process-gap(orchestrator+adversary): adversarial verdicts delivered via async teammate channel can be dropped (API error) or arrive stale-bound to a superseded HEAD — no verdict↔HEAD binding, no synchronous-capture default for gate-critical passes | 2026-07-13 |
| 591 | OPEN |  | process-gap(implementer+orchestrator): Red→Green 'complete' declared on test-runner-only, not the full CI hard-gate set — branch ships CI-red (6 clippy + fmt) and is caught only at adversarial re-convergence | 2026-07-13 |
| 590 | OPEN |  | process-gap(story-writer+architect+adversary): integration ACs ship without end-to-end injection-topology tracing — two consecutive spec passes each found a discharge-unreachable binding anchor across the production-wiring/test-injection boundary | 2026-07-13 |
| 589 | OPEN |  | process-gap(state-manager+orchestrator): frozen factory artifacts have no write protection — prose FROZEN banner + canonical pointer did not prevent 27 misdirected appends and a forked version sequence | 2026-07-13 |
| 588 | OPEN |  | process-gap(pr-manager): factory-side PR flow leaves the shared .factory worktree stranded on its chore branch — no post-merge restore-original-branch step | 2026-07-13 |
| 587 | OPEN |  | process-gap(story-writer): story-authored structural grep gates false-fail and cannot discriminate the regressions they guard — 4 instances in one story; need authoring rules or behavioral-test authority default | 2026-07-13 |
| 586 | OPEN |  | process-gap(demo-recorder+docs-producers): agent-authored operator docs cite fabricated tool surfaces — 3 instances in one story (CLI cmds, field names, git config key); no surface-validation gate | 2026-07-09 |
| 585 | OPEN |  | Visual ACs: bind attachment/continuity as connectivity properties, not pixel-adjacency existence checks | 2026-07-09 |
| 584 | OPEN |  | enhancement(session-review): the learning loop is non-functional in practice — session review needs a standing trigger, pattern database, and improvement backlog | 2026-07-12 |
| 583 | OPEN |  | enhancement(state-manager+session-review): per-cycle cost ledger (.factory/cost-summary.md) with defined schema — session reviews cannot do cost analysis | 2026-07-12 |
| 582 | OPEN |  | process-gap(phase-f1): perimeter scan omits index/count/traceability surfaces (BC-INDEX, canonical counts) — count-propagation gaps surface late in F2 | 2026-07-08 |
| 581 | OPEN |  | process-gap(phase-f2): F1 impact-boundary artifact not retro-annotated when F2 decisions supersede F1 scope claims | 2026-07-08 |
| 580 | OPEN |  | process-gap(story-writer): story template lacks a mandatory CHANGELOG delivery task — PRs merge without changelog entries, caught only at F5 | 2026-07-08 |
| 579 | OPEN |  | process-gap(adversary+orchestrator): no stale-finding/reopening rule — recurrence of a previously-resolved LOW/NITPICK resets the clean streak under STRICT | 2026-07-08 |
| 578 | OPEN |  | process-gap(orchestrator+phase-f1): F2 convergence criterion (STANDARD vs STRICT) not selected at F1 gate — mid-cycle escalation costs passes and an unplanned human checkpoint | 2026-07-08 |
| 577 | OPEN |  | process-gap(adversary): meta-lens findings (verification-adequacy / process-quality) lack a convergence rule — unbounded meta-observations extend STRICT windows | 2026-07-08 |
| 576 | OPEN |  | process-gap(adversary+orchestrator): pass-summary verdict can contradict findings list — verdict must derive mechanically from finding count | 2026-07-08 |
| 575 | OPEN |  | process-gap(test-writer+adversary): spec'd single-event transition semantics on shared atomic state had zero interleaving coverage for 28 passes — decrement/check race emitted duplicate EC-004 for one logical drop-to-zero transition | 2026-07-13 |
| 574 | OPEN |  | process-gap(architect): placement-note API-derivation blocks reconstruct signatures from memory and ship unverified — one note seeded three downstream defects (wrong import pair, false Assemble signature, phantom FrameTypeControl) | 2026-07-13 |
| 573 | OPEN |  | process-gap(story-writer+adversary): symbols and call forms cited in NORMATIVE AC postconditions are never mechanically verified against code — phantom constant and compile-time-illegal signature each survived 25+ fresh-context passes | 2026-07-09 |
| 572 | OPEN |  | process-gap(state-manager+hooks): STATE.md validator invariants are undocumented — every state-manager burst rediscovers them via rejected-write trial-and-error | 2026-07-18 |
| 571 | OPEN |  | process-gap(test-writer+governance): max-file-lines lint vs one-test-file-per-BC convention — amendment-heavy contracts force an uncodified sharding decision | 2026-07-08 |
| 570 | OPEN |  | process-gap(orchestrator+subagents): mid-flight scope-relay race — agents finalize against stale scope with the amendment sitting in their inbox (3 instances, one day) | 2026-07-09 |
| 569 | OPEN |  | process-gap(implementer+orchestrator): unbounded in-head design work — 23-min deliberation stall + 2 output-token-ceiling deaths on one geometry task; pre-supplied design skeletons fix it | 2026-07-11 |
| 568 | OPEN |  | feature(review): variants-gallery loop for aesthetic surfaces — split N candidates, solicit structured human feedback, fold learnings into next round | 2026-07-13 |
| 567 | OPEN |  | bug(hooks/validate-count-propagation): count_propagation_drift false positive — compares historical changelog counts against current counts, flagging accurate historical records as drift | 2026-07-18 |
| 566 | OPEN |  | bug(hooks/validate-bc-title): bc_h1_index_drift false positive — matcher hits first BC-id occurrence in BC-INDEX (satisfaction table), not the §2 navigation-table title column | 2026-07-08 |
| 565 | OPEN |  | Add an enumeration-completeness gate: every emitted --json/error code must have a normative spec (EC-table) entry | 2026-07-08 |
| 564 | OPEN |  | process-gap(spec-steward+orchestrator): symbol deletion/rename/renumber dispatches must mandate a mechanical same-artifact co-reference grep with per-hit adjudication | 2026-07-16 |
| 563 | OPEN |  | process-gap(adversary+orchestrator): class-closing sweeps must re-derive set membership from the generating tool, not verify self-consistency | 2026-07-08 |
| 562 | OPEN |  | Phase-4 human/hardware evals need a distinct sighted setup-preparer role, separate from the blind evaluator | 2026-07-09 |
| 561 | OPEN |  | Holdout scenario setup docs / operator runbooks drift from the evolving CLI surface — no staleness gate | 2026-07-08 |
| 560 | OPEN |  | Human-in-loop / hardware holdout evals need pre-registered expected outputs + rubrics before operator execution | 2026-07-08 |
| 559 | OPEN |  | process-gap(visual-gate): attestation pixel thresholds inherited from precedent surfaces instead of derived from the actual rendered geometry | 2026-07-08 |
| 558 | OPEN |  | enhancement(orchestrator+all-agents): agent-authored artifact timestamps drift into the future — derive from environment, not narrative continuation; dispatch templates carry 'today is <date>' | 2026-07-08 |
| 557 | OPEN |  | enhancement(spec-steward): document VP verification_lock flip-timing convention — locks may flip at adjudication on delivery branch pre-merge, with merge-SHA anchor true-up required at merge | 2026-07-08 |
| 556 | OPEN |  | process-gap(implementer+adversary): remediation-authored regression tests are not required to prove they detect the mutation they claim to pin — surviving mutant caught in pass-3 because jitter bands overlapped by construction | 2026-07-11 |
| 555 | OPEN |  | process-gap(architect+adversary): placement-note rulings that PRESCRIBE code patterns ship as normative contracts with no validation obligation — Q3-prescribed drop-oldest deadlock escaped four adversarial passes | 2026-07-08 |
| 554 | OPEN |  | No deferral-citation traceability gate — stubs citing "deferred STORY-X" escape tracking when the citation is stale or wrong | 2026-07-08 |
| 553 | OPEN |  | Pipeline lacks an end-to-end capability-integration gate — converged stories don't compose into a verified user journey | 2026-07-09 |
| 552 | OPEN |  | process-gap(product-owner+architect+story-writer): backlog-stub spec citations survive elaboration into placement note and story draft unverified — phantom BC-4.03 propagated across three agents before PO ready-review caught it | 2026-07-08 |
| 551 | OPEN |  | process-gap(pr-manager): CI lint failures during a PR lifecycle fixed by pr-manager committing directly to the spec branch instead of routing back to the orchestrator | 2026-07-08 |
| 550 | OPEN |  | process-gap(story-writer+adversary): spec-time prescriptive detail goes stale when implementer latitude is exercised — story-prose citation drift taxes convergence one finding per round | 2026-07-16 |
| 549 | OPEN |  | process-gap(subagents): agent defends a stale report against verifiable disk history ("already fixed in my prior turn") instead of re-checking | 2026-07-15 |
| 548 | OPEN |  | process-gap(adversary+infra): stream-watchdog kills large-scope reviews with zero durable output — no partial-result checkpoint or resume path | 2026-07-15 |
| 547 | OPEN |  | process-gap(state-manager+orchestrator): parallel dispatch to shared .factory/ worktree lets git add -A sweep a sibling burst's files — commit-attribution drift | 2026-07-14 |
| 546 | OPEN |  | process-gap(spec-steward): ruling-authored governance premises ("code X already defined") are not machine-verified against catalog state at ratification | 2026-07-07 |
| 545 | OPEN |  | process-gap(subagents): agent substitutes a narrower command for the briefed verification command while reporting the briefed command's label ("CI-exact") | 2026-07-07 |
| 544 | OPEN |  | spec + adversary: third-party library runtime-behavior claims are never verification-gated (confident-wrong at spec level) | 2026-07-07 |
| 543 | OPEN |  | per-story-delivery: no hardware-execution smoke gate for stories with hardware-gated ACs | 2026-07-07 |
| 542 | OPEN |  | adversary: inter-pass finding-class contradictions — fail-loud test doubles flagged as defect after prior pass graded them clean | 2026-07-07 |
| 541 | OPEN |  | adversary: unbounded tail-verification loops cause dispatch timeouts — codify execution-budget discipline | 2026-07-07 |
| 540 | OPEN |  | process-gap(orchestrator+determination): fix ruling summary names single property while geometry rationale requires coupled second property — literal implementation ships a shrunken element | 2026-07-06 |
| 539 | OPEN |  | demo-recorder: codify VHS hardening — foreground execution, nonzero-artifact verification, Wait+Line caveat | 2026-07-06 |
| 538 | OPEN |  | state tracking: AC-level scope reduction must invalidate story-level "unblocks X" claims | 2026-07-06 |
| 537 | OPEN |  | product-owner: BC amendments touching shipped field semantics need a shipped-code ground-truth gate | 2026-07-06 |
| 536 | OPEN |  | implementer: "pre-existing" claims need provenance verification (git-blame gate) | 2026-07-06 |
| 535 | OPEN |  | process-gap(formal-verifier+orchestrator): VP discharge claims accepted without verifying the real component is on the tested path — harness-structural tests can masquerade as e2e discharge | 2026-07-06 |
| 534 | OPEN |  | process-gap(adversary+convergence): runtime probes verify field values but not emission budgets — warning/error count regressions ship undetected | 2026-07-08 |
| 533 | OPEN |  | Phase-3 Wave-integration: missing 'release-profile tests compile' gate lets test-hook cfg mis-gating reach main | 2026-07-08 |
| 525 | OPEN |  | Phase-4: evaluator dispatch should verify release binary is built from evaluated SHA (rebuild-fresh preflight) | 2026-07-06 |
| 523 | OPEN |  | bug(per-story-delivery): factory artifacts written inside story worktrees are silently lost at worktree teardown — DELIVERY ledger has no canonical-path rule | 2026-07-15 |
| 522 | OPEN |  | enhancement(quality-gate+architect): declared ARCH dependency-DAG has no machine enforcement — forward-reference imports pass every gate | 2026-07-06 |
| 521 | OPEN |  | process-gap(implementer): divergence from an explicit dispatch ruling ships silently — rationalized in a doc comment, caught only by manual coordinator diff review | 2026-07-15 |
| 520 | OPEN |  | process-gap(test-writer+implementer): data-driven parameters applied at initial load only — runtime identity-switch path never re-applies, stale value persists silently | 2026-07-06 |
| 519 | OPEN |  | process-gap(implementer+adversary): spec-named implementation vehicle silently substituted — vehicle conformance not gated | 2026-07-06 |
| 518 | OPEN |  | enhancement(phase-4-preflight): Phase-4 dispatch must run scenario-specific setup prerequisites BEFORE burning an evaluator dispatch (companion to #458) | 2026-07-06 |
| 517 | OPEN |  | process-gap(state-manager+orchestrator): Drift Items marked USER-ACTION-REQUIRED advance the pipeline without any verification gate — human-declared-satisfied ≠ actually-satisfied | 2026-07-06 |
| 516 | OPEN |  | research-agent: declared MCP tools not provisioned at spawn (mcp__perplexity__*, mcp__context7__*, mcp__tavily__*) | 2026-07-06 |
| 515 | OPEN |  | bug(factory-health): factory artifact red-gate-log-S-12.08.md tracked at repo ROOT on develop — outside .factory/, invisible to path-prefix leak checks | 2026-07-05 |
| 514 | OPEN |  | enhancement(state-manager+adversary): flag combined-footnote amendment sites as structurally coupled so partial edits can't silently drop cross-references | 2026-07-05 |
| 513 | OPEN |  | process-gap(implementer+orchestrator): green-claims accepted without race-evidence paste — two false-greens in one wave caught only by orchestrator re-verification | 2026-07-08 |
| 512 | OPEN |  | process-gap(spec-steward): ARCH-11 reverse-trace propagation has no machine check — same POL-006 finding class recurred in 5+ consecutive adversarial passes | 2026-07-07 |
| 511 | OPEN |  | adversary methodology gap: test-assertion content vs BC-prose drift check | 2026-07-05 |
| 510 | OPEN |  | External code contributions from ArcavenAE — proposed flow for changes with factory-artifact implications | 2026-07-06 |
| 509 | OPEN |  | process-gap(adversary+verification): encoded-artifact verification commands must be validated in decoded/executable form, not by grep against the source artifact | 2026-07-14 |
| 508 | OPEN |  | bug(scratch-isolation): cp -r of a git worktree silently poisons the shared repo's .git/config | 2026-07-05 |
| 507 | OPEN |  | adversary: peer-artifact sweep when applying prose fixes (successor to #504/#505/#506) | 2026-07-17 |
| 506 | OPEN |  | adversary: enumerate BC self-referential metadata as first-class sweep target (successor to #505) | 2026-07-05 |
| 505 | OPEN |  | Adversary S-7.01(c) sweep prompt should enumerate story pseudocode + Architecture Mapping / File Structure Requirements tables (successor to #504) | 2026-07-05 |
| 504 | OPEN |  | Adversary preventive-sweep prompt should span ALL story artifacts (spec + tests + VPs + ADRs), not just test files | 2026-07-05 |
| 503 | OPEN |  | bug(hooks): factory-dispatcher WorktreeCreate handler succeeds but returns no worktreePath — breaks Agent tool isolation:worktree for ALL agents in factory projects | 2026-07-05 |
| 502 | OPEN |  | process-gap(adversarial-refinement): race-finding fixes that install sync barriers must fail closed and be stress-validated under constrained schedulers | 2026-07-06 |
| 501 | OPEN |  | enhancement(demo-recorder): add project-manifest `demo_artifact_format` knob (tape-only \| tape+gif \| tape+gif+webm) so projects that don't want rendered binaries in git can commit only .tape scripts | 2026-07-05 |
| 500 | OPEN |  | adversary: 10 passes across 2 model families missed transitive-contract drift (parser derives X, credential-lookup expects Y, no verification either owns derivation) | 2026-07-08 |
| 499 | OPEN |  | process-gap(test-writer): GUT assert_push_error is CONSUMING — documented error side effects are fixture contract, not noise; tests not consuming them fail post-implementation | 2026-07-05 |
| 498 | OPEN |  | process-gap(test-writer+formal-verifier): config guards comparing parsed numbers must enumerate the config parser's full numeric-literal space | 2026-07-05 |
| 497 | OPEN |  | process-gap(orchestrator+pr-reviewer): post-convergence fresh-context PR review must remain non-skippable — information asymmetry catches what saturated reviewers cannot | 2026-07-05 |
| 496 | OPEN |  | process-gap(formal-verifier+test-writer): checker validated by author-shared fixtures fails self-application — mandatory self-application smoke row | 2026-07-05 |
| 495 | OPEN |  | enhancement(orchestrator+state-manager): archive per-cycle TaskList at CONVERGED transition — currently completed tasks from prior cycles remain in TaskList across sessions, obscuring current-cycle work | 2026-07-05 |
| 494 | OPEN |  | process-gap(demo-recorder+adversary): attestation quality — fabricated evidence text and presence-vs-change verification gap | 2026-07-15 |
| 493 | OPEN |  | enhancement(orchestrator): define a "delta cycle" mode for post-SHIPPED spec amendments — currently each amendment is ad-hoc, no version-alignment sweep, no batched changelog | 2026-07-05 |
| 492 | OPEN |  | Phase-3 wave-close gate: require CI green before state-manager burst can advance state | 2026-07-05 |
| 491 | OPEN |  | bug(spec-steward+consistency-validator): inputDocuments: frontmatter is one-directional, unvalidated, and silently rots when input files move or are renamed | 2026-07-05 |
| 490 | OPEN |  | Adversary methodology: detect tokio-worker starvation from blocking I/O in #[tokio::test] integration tests | 2026-07-05 |
| 489 | OPEN |  | enhancement(spec-steward+story-writer): treat docs/ tree as a projection of specs — every user-facing doc needs a BC or PRD-supplement anchor and bidirectional projects_to/projects_from links | 2026-07-05 |
| 488 | OPEN |  | enhancement(steady-state): post-SHIPPED PRs to develop bypass every factory quality gate — no spec-drift check, no consistency-validator, no input-hash scan | 2026-07-05 |
| 487 | OPEN |  | enhancement(orchestrator): add smoke sentinel gate to per-story-delivery Step (c3) and Wave Integration Gate — quality-gate-green ≠ operator-safe-green | 2026-07-05 |
| 486 | OPEN |  | process-gap(orchestrator): convergence counter lives in narrative prose — two independent counter slips in one session, one enabling premature ship declaration | 2026-07-16 |
| 485 | OPEN |  | process-gap(adversary): reasoned review systematically misses CLI-mode-gated defects — empirical execution lens mandatory for CLI entrypoint stories | 2026-07-04 |
| 484 | OPEN |  | process-gap(adversary+implementer): comment-truth defects survive spot-fix passes — remedy is disposition sweep, not per-finding patch | 2026-07-16 |
| 483 | OPEN |  | holdout-evaluator can mutate target-repo local git config and land commits on main — needs scratch-workspace sandbox | 2026-07-04 |
| 482 | OPEN |  | process-gap(product-owner+orchestrator): adjudication rulings that specify tests must declare four explicit fields; orchestrator must verify mutual consistency before dispatch | 2026-07-04 |
| 481 | OPEN |  | process-gap(story-writer+test-writer): AC-named test targets must be verified instantiable in the test framework; silent test-type substitution must escalate, not comment | 2026-07-04 |
| 480 | OPEN |  | process-gap(test-writer+spec-authoring): hand-seeded fixtures that violate atomic-production invariants mask real guard-placement bugs; spec code blocks must show placement context | 2026-07-04 |
| 479 | OPEN |  | process-gap(test-writer+adversary): measuring-instrument stories need an end-to-end wiring test at Red Gate; reviews of instrument output must enumerate every expected field | 2026-07-04 |
| 478 | OPEN |  | process-gap(spec-authoring): BC mandates a specific API call without verifying the call exists in the project's pinned engine version | 2026-07-04 |
| 477 | OPEN |  | process-gap(mutation-review): tautological zero-assertion tests are invisible to mutation analysis — mutation pass rates suites CLEAN while spec-conformance crosswalk finds unimplemented branches | 2026-07-04 |
| 476 | OPEN |  | process-gap(spec-authoring): new subsystem ID declared in architecture index but dependency-graph row omitted — consistency-validator checklists not generated from CI lint inventory | 2026-07-04 |
| 475 | OPEN |  | process-gap(stub-architect): test-file authorship at stub stage — poisons Red Gate and enables same-agent tautology (self-attestation of green tests) | 2026-07-04 |
| 474 | OPEN |  | bug(phase-4 gate): hard-coded "GPT-5.4, not Claude" criterion with fail_action: block is unsatisfiable in the plugin substrate | 2026-07-06 |
| 473 | OPEN |  | bug(phase-4 workflow): scenario-rotation step tasks the write-denied orchestrator with writing a file, to a non-canonical directory | 2026-07-04 |
| 472 | OPEN |  | bug(skills): lock helpers invoked via repo-relative `plugins/vsdd-factory/bin/...` — fails in every installed-plugin context | 2026-07-04 |
| 471 | OPEN |  | enhancement(test-writer+adversary): enforce version floor on spec citations in test docstrings — tests asserting a code minted in taxonomy vX.Y must cite ≥ vX.Y | 2026-07-04 |
| 470 | OPEN |  | process-gap(state-manager): remediation delivers finding's exact scope but does not sweep sibling artifacts — seven-consecutive-pass pattern with two recursive-inside-codification recurrences + third-order failure (sibling sweep propagates wrong value with fidelity) | 2026-07-16 |
| 469 | OPEN |  | bug(pr-validation): BC Traceability Check regex 'STORY-[0-9]+' excludes alphanumeric story IDs (STORY-DEF-NNN class) | 2026-07-05 |
| 468 | OPEN |  | bug(dispatch): agent model header may not reflect the dispatched model — undermines BC-5.39.001 model-diversity guarantee | 2026-07-04 |
| 467 | OPEN |  | enhancement(adversary): EC/POST enumeration + grep-verify handler exists per spec canonical values | 2026-07-04 |
| 466 | OPEN |  | enhancement(adversary): run language linter as part of ground-truth verification | 2026-07-04 |
| 465 | OPEN |  | adversary: hallucinated BLOCKING findings when reviewing implementation with no ground-truth verification step | 2026-07-06 |
| 464 | OPEN |  | wasm wave_context resolver.load_error fires at scale on every rc.21 factory (~268k events across 3 projects) | 2026-07-03 |
| 463 | OPEN |  | Proposal: per-agent model rationale, dispatch outcome telemetry, escalation ladders, uniform effort hints | 2026-07-03 |
| 462 | OPEN |  | process-gap(adversary): probe variance, not pass count, drives convergence quality — hostile-input structure axis and path-component enumeration missing from methodology | 2026-07-04 |
| 461 | OPEN |  | possibly-out-of-scope(per-story-delivery): auto-mode classifier permission-laundering breaks agent-driven merge — factory concern or Claude Code concern? | 2026-07-08 |
| 460 | OPEN |  | possibly-out-of-scope(rubric): silent-ignore of unknown/misshapen config keys — is it a factory concern, or per-project language choice? | 2026-07-03 |
| 459 | OPEN |  | process-gap(product-owner+architect): holdout-scenario 'architect must evaluate' decision items don't route — surface as architect-decision tickets during Phase-1d | 2026-07-03 |
| 458 | OPEN |  | enhancement(holdout-evaluator): evaluability constraints — hardware/human-gated criteria need a first-class category and a resume-queue | 2026-07-03 |
| 457 | OPEN |  | process-gap(subagents): completed-but-unreported liveness gap — work finishes, report never delivered without an explicit ping | 2026-07-18 |
| 453 | OPEN |  | policy(spec-steward): POL-003 VP source_bc frontmatter shape asymmetry — pinned vs unpinned makes version-sync machine-uncheckable | 2026-07-05 |
| 452 | OPEN |  | process-gap(per-story-delivery): conflict resolution voids BC-5.39.001 convergence certificate — merged code ships with zero adversarial passes | 2026-07-03 |
| 451 | OPEN |  | planner: wave/batch scheduler has no file-collision check — same-epic stories touching the same file get parallelized, guaranteeing merge conflicts | 2026-07-05 |
| 450 | OPEN |  | enhancement(spec-steward): canonical-claim reword fix vectors must enumerate impl/test/CLI docstring tiers via global grep | 2026-07-03 |
| 449 | OPEN |  | bug(orchestrator): within-burst upstream re-bump invalidates prior finding's fix target | 2026-07-03 |
| 448 | OPEN |  | bug(orchestrator): wave-gate (Perimeter-2) adversary dispatch lacks HEAD-SHA verification tuple | 2026-07-03 |
| 447 | OPEN |  | process-gap(implementer+adversary): fix-phase adjudication labels (F-CRIT-001, F-P2-MOD-02) leak as comment provenance in production code | 2026-07-02 |
| 446 | OPEN |  | process-gap(orchestrator): fix-phase order-of-operations — factory-artifacts commits must land BEFORE feature-branch impls | 2026-07-05 |
| 445 | OPEN |  | policy(sweep): Working-Policy P-B (sweep-on-BC-bump) doesn't propagate to STORY-bumps — recursive-fix cascade | 2026-07-02 |
| 444 | OPEN |  | policy(sweep): fix-phase sweep-scope missing non-source config surfaces (project.godot, .editorconfig, .gdlintrc, etc.) | 2026-07-02 |
| 443 | OPEN |  | adversary: blast_radius estimates are lower-bounds — cross-story sibling drift invisible to per-story perimeter | 2026-07-15 |
| 442 | OPEN |  | adversary: AC-mapped test ACTION-path invokes BC §Trigger method — rubric gap allows structurally green-by-design tests | 2026-07-02 |
| 441 | OPEN |  | process-gap(orchestrator): task marked complete without downstream verification that declared delivery scope landed — Task-33/STORY-014 origin of the spec-text-only convergence loop | 2026-07-02 |
| 440 | OPEN |  | process-gap(adversary+orchestrator): spec-convergence loop can 'converge' on story delivery scope that was never shipped — no pre-verdict grep gate for declared delivery surface | 2026-07-11 |
| 437 | OPEN |  | lesson(L-W3-ZZ): bidirectional spec-code drift — architect-side spec additions drift from impl in the same burst as reconciled symbols | 2026-07-02 |
| 436 | OPEN |  | policy(test-writer+adversary): GUT 9.7.0 assert_signal_emitted_with_parameters 4-arg gotcha — index arg is int, not String; type mismatch silently false-positives | 2026-07-02 |
| 435 | OPEN |  | process-gap(product-owner+story-writer): Fix Phase 2 spec propagation gap — spec-side fix lands but story body/AC prose retains contradicting language | 2026-07-02 |
| 434 | OPEN |  | policy(implementer): implementer must not modify test-runner config (.gutconfig.json, jest.config, etc.) to work around test bugs | 2026-07-02 |
| 433 | OPEN |  | gap(demo/UI stories): headless scene-open is not a playability gate — no dynamic-launch attestation for visual stories | 2026-07-05 |
| 432 | OPEN |  | gap(factory-core): no product-level defect register in any greenfield/brownfield/feature/multi-repo mode | 2026-07-02 |
| 430 | OPEN |  | [process-gap] body-prose ↔ impl-symbol drift persists across all POL-001/002/003 gates — POL-005-body-prose-impl-anchor-check | 2026-07-02 |
| 429 | OPEN |  | [process-gap] POL-003 bidirectional-pin cascade is non-terminating for governance-only Traceability-Stories-row bumps | 2026-07-06 |
| 428 | OPEN |  | [process-gap] verify changelog attestations against impl on each Pass — POL-001-verify-attestations | 2026-07-08 |
| 427 | OPEN |  | process-gap(architect): spec-diff re-read mandate has no architect-dispatch parity — orchestrator briefing errors and impl-side drift survive spec edits | 2026-07-02 |
| 426 | OPEN |  | adversary policy: defense-in-depth invariant hollowed via handler-fabricated inputs | 2026-07-02 |
| 425 | OPEN |  | adversary: corroboration-inversion — fresh-context findings quote evidence that contradicts actual file contents | 2026-07-02 |
| 424 | OPEN |  | process-gap(test-writer+adversary): tests spawning shells/interactive-capable subprocesses inherit stdin from the runner — hangs interactively, false-green on CI, .stdin(null) not mandated | 2026-07-02 |
| 423 | OPEN |  | process-gap(test-writer+story-writer): in-file 'AC deferred' banner in a test file with no owning follow-on story survives red-gate — implicit deferral without ticket | 2026-07-02 |
| 422 | OPEN |  | process-gap(product-owner+adversary): BC postconditions may claim behavior downstream of the software boundary (ecosystem-observable, not product-testable) | 2026-07-02 |
| 421 | OPEN |  | process-gap(BC-5.38.001): Red Gate density count over-reports when stubs are behind build tags / feature flags / test markers — publish red_default + red_gated | 2026-07-02 |
| 420 | OPEN |  | process-gap(stub-architect): existing test co-modification driven by handler-registry additions — undocumented REGISTRY-COMOD mode alongside GREEN-BY-DESIGN/WIRING-EXEMPT | 2026-07-02 |
| 419 | OPEN |  | process-gap(implementer+architect): impl adds public API not documented in api-surface — reverse of phantom-API-drift (#399) | 2026-07-02 |
| 418 | OPEN |  | process-gap(demo-recorder): .tape files hardcode absolute worktree paths → non-portable across relocations and post-merge re-record | 2026-07-02 |
| 417 | OPEN |  | Resource-management opacity: model selection, cache lifetime, and scheduling are host-owned with no control surface | 2026-07-02 |
| 416 | OPEN |  | Concurrency topology ceiling: "teams of agents" requires peer actors; the platform provides hierarchical fork-join rooted in one REPL | 2026-07-02 |
| 415 | OPEN |  | No introspection, quotas, preemption, or bulkheads inside the agent tree: blast radius of one misbehaving agent is the whole factory | 2026-07-02 |
| 414 | OPEN |  | Session-scoped lifecycle vs. service lifecycle: factory uptime is bounded by an interactive REPL session | 2026-07-02 |
| 413 | OPEN |  | Checkpoint exists, restore fidelity doesn't: CAP-012's "~5 minutes / at most one story" loss bound is not met and cannot be verified | 2026-07-02 |
| 412 | OPEN |  | Compaction is generational loss of control state: substantial risk of undetected drift; ADR-026's "currently unremediated" gap never reconciled with the autonomy baseline | 2026-07-02 |
| 411 | OPEN |  | No external supervisor: orchestrator and orchestrated work share one failure domain, so time-to-detection of a stalled factory is unbounded | 2026-07-02 |
| 410 | OPEN |  | Architectural mismatch (tracking): the dark-factory requirements profile exceeds the Claude Code plugin platform envelope | 2026-07-02 |
| 409 | OPEN |  | process-gap(orchestrator+adversary): wave-level adversarial dispatch has no pre-flight local-vs-origin sync check — halts on stale checkout when STATE.md is ahead of local develop | 2026-07-02 |
| 408 | OPEN |  | pr-manager: prefer `gh pr update-branch` over rebase+force-push when PR base advances during convergence | 2026-07-12 |
| 407 | OPEN |  | process-gap: POL-001 scope unclear for INDEX artifacts (BC-INDEX changelog convention) | 2026-07-01 |
| 406 | OPEN |  | POL-003 candidate: rulings' Downstream Dispatch Tables must enumerate all cross-artifact sync obligations | 2026-07-01 |
| 405 | OPEN |  | bug(orchestrator): adversary L1 task prompt asks about details out of scope for opaque-string plumbing story — invites hallucinated findings | 2026-07-01 |
| 404 | OPEN |  | process-gap(state-manager+story-writer): duplicate sprint-state.yaml at two paths — .factory/sprint-state.yaml vs .factory/stories/sprint-state.yaml drift independently under POL-002 sync | 2026-07-02 |
| 403 | OPEN |  | process-gap(story-writer): vp_traces frontmatter claim with no body evidence — VP appears in trace list but no AC/test/prose references it | 2026-07-01 |
| 402 | OPEN |  | policy(test-writer+adversary): panic-recovery negative tests must assert on the panic message, not just recover() != nil | 2026-07-01 |
| 401 | OPEN |  | process-gap(story-writer+consistency-validator): inputDocuments frontmatter drifts from changed_by_rulings — silent under-listing | 2026-07-01 |
| 400 | OPEN |  | process-gap(story-writer): story template lacks AC↔BC PC-level trace table — wrong-PC anchors survive multiple version bumps | 2026-07-01 |
| 399 | OPEN |  | Fresh-context adversary does not reliably detect specs that cite symbols/variants/method-parameters absent from shipped code (phantom-API drift) | 2026-07-06 |
| 398 | OPEN |  | per-story-delivery lacks a lightweight verify-Green checkpoint between implementer and adversary | 2026-07-01 |
| 397 | OPEN |  | Implementer sub-agent's final-gate checklist does not reliably enforce project lint at deny-warnings level | 2026-07-02 |
| 396 | OPEN |  | policy(spec-steward): full citation-corpus sweep on BC/ADR bump — changelog-row-only check misses 3–5 stale pins per bump | 2026-07-14 |
| 395 | OPEN |  | policy(adversary): test-file header/docstring version stamps not scanned — stale BC-version citations survive multiple passes | 2026-07-01 |
| 394 | OPEN |  | policy(implementer): workaround-in-wrong-layer — fixing a downstream symptom instead of escalating to the correct owner | 2026-07-01 |
| 393 | OPEN |  | policy(adversary+test-writer): FULL linter ruleset required, not just eyeballed line-length — recurring gdlint failures in test files | 2026-07-01 |
| 392 | OPEN |  | feat(lint): verbatim-quote provenance checker — 'states verbatim' cross-BC citations are unverifiable and get fabricated | 2026-07-01 |
| 391 | OPEN |  | policy(adversary): mis-anchoring is never an Observation — semantic-anchor drift must be IMPORTANT or CRITICAL | 2026-07-01 |
| 390 | OPEN |  | process-gap(product-owner+spec-steward): fix-and-bump-in-same-commit re-creates stale self-refs — need target-first atomic version cascade | 2026-07-01 |
| 389 | OPEN |  | Adversary-dispatch preflight tuple can embed an inaccurate symbol path | 2026-07-01 |
| 388 | OPEN |  | Story decomposition can leave cross-subsystem integration glue unowned, surfacing only in implementation | 2026-07-01 |
| 387 | OPEN |  | Duplicated normative call-sequence sketches across sibling artifacts drift independently under partial fixes | 2026-07-16 |
| 386 | OPEN |  | Adversary stalls on large-context corroboration passes (stream watchdog) with no verdict | 2026-07-01 |
| 383 | OPEN |  | policy(test-writer): string length/truncation ACs tested only with ASCII; multibyte/UTF-8 boundary cases not mandated, allowing char-boundary panics to pass all gates | 2026-07-01 |
| 382 | OPEN |  | process-gap(implementer+adversary): doc-comments asserting behavior are not verified against the code they document; stale/overclaiming doc-comments survive all gates | 2026-07-01 |
| 381 | OPEN |  | adversary/test-writer policy: a "reference oracle" that duplicates the production mapping it validates gives zero drift protection (false independence claim) | 2026-07-01 |
| 380 | OPEN |  | Write-capable subagent refuses orchestrator-relayed human approval on principle, demanding a nonexistent direct user channel — deadlocks human-gated steps | 2026-07-01 |
| 379 | OPEN |  | pr-manager: novel dependency-audit suppression misattributed to an unrelated pre-existing deferral ID (security decision staged under wrong cover) | 2026-07-01 |
| 378 | OPEN |  | enhancement(consistency-validator): flag non-monotonic changelog modified-list frontmatter | 2026-07-01 |
| 377 | OPEN |  | bug(adversary): rejects real merge SHAs as placeholder-looking without verification | 2026-07-01 |
| 376 | OPEN |  | [process-gap](state-manager+story-writer): sprint-state.yaml vp_traces field has no documented owner; stale value propagates downstream | 2026-07-01 |
| 375 | OPEN |  | bug(product-owner+spec-writer): BC introduces sentinel value without enumerating derived-field behavior; implementer invents silent fallback | 2026-07-01 |
| 374 | OPEN |  | bug(implementer): production validation weakened to accommodate non-conformant test fixtures (Ed25519 size check bypassed) | 2026-07-01 |
| 373 | OPEN |  | bug(test-writer+implementer): E2E tests for production-mode ACs must enter via the real entry point, not re-construct internals | 2026-07-01 |
| 372 | OPEN |  | bug(story-writer+spec-reviewer): AC text on authority/identity must specify the authenticated source the check reads from | 2026-07-01 |
| 371 | OPEN |  | Local pre-merge gates must run BOTH default and all-features configs to match CI job matrix | 2026-07-01 |
| 370 | OPEN |  | CI build-config verification job emits static PASS instead of runtime-computed scan count | 2026-07-01 |
| 369 | OPEN |  | No lint validates deferred:integration (STORY-NNN) pointer citations against the owning story | 2026-07-01 |
| 368 | OPEN |  | Orchestrator dispatches read-only reviewer without asserting worktree HEAD == review SHA | 2026-07-01 |
| 367 | OPEN |  | process-gap(story-writer+adversary): stale story-id pointers in inline code comments survive all spec gates | 2026-06-30 |
| 366 | OPEN |  | process-gap(test-harness): Godot stale class-cache produces phantom failure cascades misdiagnosed as cross-story regression | 2026-07-04 |
| 365 | OPEN |  | bug(rebase): silent auto-merge drops production code from parent commits during multi-branch parallel-merge rebase | 2026-06-30 |
| 364 | OPEN |  | adversary policy: detect test name/comment claiming branch coverage that the assertion doesn't actually exercise | 2026-07-02 |
| 363 | OPEN |  | policy: test-writer should require negative tests for any "unreachable in practice" default-arm | 2026-06-30 |
| 362 | OPEN |  | process-gap: VP-INDEX row description not auto-synced when VP body narrows | 2026-06-30 |
| 361 | OPEN |  | process-gap: BC EC narrowing/widening fix-bursts don't auto-dispatch story-writer for downstream EC-table sibling-fix | 2026-06-30 |
| 360 | OPEN |  | process-gap(test-writer+stub-architect): panic-sourced red tests degrade to vacuous-green no-ops at Red→Green transition (green-side analogue of #353) | 2026-07-01 |
| 359 | OPEN |  | enhancement(test-writer): platform-specific tests emitted without consulting CI runner OS — Linux-only failure injection silently broken on macOS-pinned CI | 2026-07-16 |
| 358 | OPEN |  | pr-manager: enforce PR base == trunk for story PRs (P1) | 2026-06-30 |
| 357 | OPEN |  | Phase 3: Red Gate tests cannot land on main — CI rejects expected-red and idiomatic test-code lints | 2026-06-30 |
| 356 | OPEN |  | process-gap(consistency-validator+architect): architecture-graph edge vs integration-story constraint contradictions survive all spec gates | 2026-06-30 |
| 355 | OPEN |  | process-gap(orchestrator+per-story-delivery): parallel-worktree TDD branches contaminate each other's test-failure baseline; "pre-existing" mis-attribution propagates | 2026-06-30 |
| 354 | OPEN |  | bug(stub-architect): BC-mandated test seams (fields/getters cited in AC text) silently omitted from stub surface — test-writer must invent or fail | 2026-06-30 |
| 353 | OPEN |  | process-gap(test-writer+orchestrator): high vacuous-pass ratio at RED gate dispatch is unenforced — stubs incidentally satisfy behavioral assertions | 2026-07-03 |
| 352 | OPEN |  | Orchestrator sub-cwd inherits unrelated parent CLAUDE.md chain — ~25-30k tokens of off-topic context per turn | 2026-06-30 |
| 351 | OPEN |  | Plugin pack CI workflow templates never verified green against an empty target repo — three stacked defects on first product use | 2026-06-30 |
| 350 | OPEN |  | Harness classifier blocks in-scope signed commits relayed through subagents (orchestrator → pr-manager) — authorization-grant treated as scope-limit | 2026-06-30 |
| 349 | OPEN |  | Plugin pack ships branch-protection contexts that don't match CI workflow job names — merges blocked even when all checks pass | 2026-06-30 |
| 348 | OPEN |  | Branch-protection verification via /branches/{branch} is unreliable on private non-Enterprise repos | 2026-06-30 |
| 347 | OPEN |  | Orchestrator yields on external-wait conditions without scheduling a wake (PG-6) | 2026-06-30 |
| 346 | OPEN |  | Orchestrator gate-template defaults to 1 approver — deadlocks autonomous pipelines | 2026-06-30 |
| 345 | OPEN |  | bug(orchestrator+harness): single Agent-tool dispatch fans out to N parallel agent IDs (5x consistency-validator) — upstream cause of #273-class staging-area races for writing agents | 2026-06-30 |
| 344 | OPEN |  | process-gap(orchestrator+adversary): D-006 3-clean-pass convergence fails to terminate on a pure-hygiene residual stream — each round's cosmetic fix seeds the next round's finding | 2026-06-30 |
| 343 | OPEN |  | process-gap(orchestrator): multi-pass convergence loop yields after each state-manager dispatch — engine prompt has no "continuation point" semantics | 2026-06-30 |
| 342 | OPEN |  | bug(factory-health/architecture): product-branch merge silently DELETES a file the nested .factory worktree is serving — near data-loss, no git warning | 2026-06-29 |
| 341 | OPEN |  | bug(factory-health + repo-initialization): factory artifact tracked on product branch — .gitignore added without git rm --cached leaves split-brain copy, undetected | 2026-07-05 |
| 339 | OPEN |  | process-gap(consistency-validator): consistency and rename-residual checks scoped to a file-type allowlist silently skip prose/rationale/ADR docs | 2026-06-29 |
| 338 | OPEN |  | bug(architect): generated L4 verification-property files diverge from canonical L4-verification-property-template | 2026-06-30 |
| 337 | OPEN |  | bug(story-writer+test-writer): an AC sub-case that contradicts its anchored BC, plus a test whose NAME asserts the opposite of its assertion, survive all spec + TDD gates | 2026-07-01 |
| 336 | OPEN |  | enhancement(orchestrator+adversary): mandatory deterministic pre-review lint layer before LLM convergence passes | 2026-06-29 |
| 335 | OPEN |  | enhancement(test-writer+adversary): BC postcondition side-effects (WARN/log/signal) go unasserted — primary state-change tested, named side-effect clause unverified | 2026-06-30 |
| 334 | OPEN |  | process-gap(tooling+agents): story-layout convention (flat-slug vs nested-under-epic) assumed not discovered — mismatch yields silent zero-match globs that vacuously pass | 2026-06-29 |
| 333 | OPEN |  | process-gap(story-writer): false upstream dependency-provenance claim — story asserts an upstream story 'provides' an API/read-path that doesn't exist; not checked (story→story analogue of #327) | 2026-06-29 |
| 332 | OPEN |  | process-gap(test-writer+adversary): producer-only orphaned-output — derived runtime value exposed but never consumed by target subsystem; producer-only tests false-green (derived-value analogue of #289) | 2026-06-29 |
| 331 | OPEN |  | enhancement(scaffold-claude-md/health-check): no check that CLAUDE.md's documented build/test commands are runnable & non-vacuous — wrong path/missing -ginclude_subdirs runs 0 tests, exits green | 2026-06-29 |
| 330 | OPEN |  | process-gap(test-writer+orchestrator): headless test runs pass green while structurally blind to render/input/reachability behavior (modality mismatch); rendered e2e harness left un-CI-gated | 2026-06-29 |
| 329 | OPEN |  | bug(test-writer): wire-contract changes don't sweep all success-path mocks, causing repeated red-gate cycles | 2026-06-29 |
| 328 | OPEN |  | bug(orchestrator+adversary): diff-scoped adversary cannot locate specs because .factory is a separate worktree, not injected into the dispatch contract | 2026-06-29 |
| 327 | OPEN |  | process-gap(story-writer+ci): AC trace citations not resolved against cited BC — fabricated/mis-anchored sub-anchors pass CI, caught only by late fresh-context passes | 2026-07-06 |
| 326 | OPEN |  | dependency-gap(phase-1-cicd-setup): PR-label gate generated without provisioning the label vocabulary; label set should be org-configurable | 2026-06-28 |
| 325 | OPEN |  | feat(observability): factory should stamp operational-impact (panic/halt) on defects that stop or crash a run | 2026-06-28 |
| 324 | OPEN |  | feat(observability): OTEL metrics have no project/instance dimension — cost & velocity can't be attributed per pilot or per parallel run | 2026-06-28 |
| 323 | OPEN |  | bug(demo-recorder): GUT invoked via cli/gut_cli.gd (not a MainLoop); correct entry is gut_cmdln.gd | 2026-06-28 |
| 322 | OPEN |  | process-gap(orchestrator+adversary): remediation diffs get no deterministic targeted re-review — regressions caught only probabilistically by the next full pass | 2026-06-29 |
| 321 | OPEN |  | bug(cicd-setup): prescribed branch protection adds a 0-approval review object that forces --admin merge on EVERY PR | 2026-06-29 |
| 320 | OPEN |  | feat: opt-in compaction-awareness statusline (⌛ time-since-compact + context %) | 2026-06-29 |
| 319 | OPEN |  | feat(observability): infer compaction events from a context-size retrograde drop (telemetry-only fallback) | 2026-06-28 |
| 318 | OPEN |  | feat(context-durability): detect & advise on accelerating compaction cadence (thrash signal) from the flush log | 2026-06-28 |
| 317 | OPEN |  | feat(observability): emit compaction events (context.compaction / context.reanchor) via OTEL into the obs stack | 2026-06-28 |
| 316 | OPEN |  | We need a trust-scope hardening pass | 2026-07-02 |
| 314 | OPEN |  | input-hash includes YAML frontmatter → populating an artifact's own hash spuriously drifts all its downstream consumers | 2026-06-28 |
| 313 | OPEN |  | Phase-1 CI/CD artifacts generated on code branch are never committed (orphaned on local disk until manually discovered) | 2026-07-08 |
| 312 | OPEN |  | 📋 beadle — Triage Dashboard | 2026-07-13 |
| 311 | CLOSED |  | 📋 beadle — Triage Dashboard | 2026-06-28 |
| 310 | OPEN |  | bug(state-manager+product-owner): index TOTAL bumped ahead of on-disk artifacts triggers index-vs-filesystem hard-gate failure (inverse of #277) | 2026-06-28 |
| 309 | OPEN |  | process-gap(orchestrator+adversary): ground-truth capture is test-runner-only; lint-class CI gates invisible to convergence (lint-parity analogue of #259/#298) | 2026-06-28 |
| 308 | OPEN |  | meta(cicd): unbounded CI-job proliferation — every adversarial finding becomes a new lint job; cost/maintenance arc untracked | 2026-06-28 |
| 306 | OPEN |  | Whole-corpus reviewer (adversary/consistency-validator) thrashes through repeated context compaction; risks dropped findings | 2026-06-29 |
| 305 | OPEN |  | Story decomposition can produce unbuildable stories: AC-collapse, API-name drift, and hidden infrastructure dependencies pass through to Phase 3 dispatch | 2026-06-27 |
| 302 | OPEN |  | scaffold-claude-md: scope the git/PR rule and add a 'report denials accurately' principle to generated CLAUDE.md | 2026-06-28 |
| 300 | OPEN |  | bug(artifact-path-registry): no entry for L1 product-brief — create-brief output falls outside template/drift/register governance | 2026-06-27 |
| 299 | OPEN |  | process-gap(implementer+adversary): fixes to a value duplicated across N surfaces not propagated to all surfaces; no mandatory set-equality guard | 2026-07-03 |
| 298 | OPEN |  | process-gap(adversary+orchestrator): read-only adversary cannot execute the test suite, so per-story convergence trusts an unverifiable test tally | 2026-07-08 |
| 297 | OPEN |  | process-gap(test-writer+orchestrator): agents silently relax governance/lint config (gdlint max-file-lines, .editorconfig, CLAUDE.md) to make their own output pass | 2026-07-03 |
| 296 | OPEN |  | bug(factory-obs+emit-event): documented smoke-test passes JSON to emit-event, which silently drops the payload (key=value only) | 2026-06-27 |
| 295 | OPEN |  | scaffold-claude-md injects raw `<!-- TODO -->` placeholders into CLAUDE.md — should live elsewhere or be lazy-referenced | 2026-06-27 |
| 294 | OPEN |  | process-gap(implementer+orchestrator): 'pre-existing' regression mis-attribution — no baseline-vs-main diff required before claiming | 2026-06-28 |
| 293 | OPEN |  | process-gap(orchestrator): agent dispatch prompts lack worktree-identity tuple; agents can operate on wrong worktree silently | 2026-06-26 |
| 292 | OPEN |  | process-gap(pr-manager): PR labels not applied at create-pr step; manual workaround required for repos with label requirements | 2026-06-28 |
| 291 | OPEN |  | process-gap(pr-manager): post-merge audit artifacts (pr-description.md + pr-review.md) left uncommitted | 2026-06-28 |
| 290 | OPEN |  | process-gap(state-manager): post-merge STORY-INDEX status field not flipped to 'completed' | 2026-07-03 |
| 289 | OPEN |  | process-gap(story-writer): orphan-component pattern recurring — story template lacks mandatory Integration Wiring AC | 2026-06-30 |
| 288 | OPEN |  | bug(stub-architect): planting BC-5.38.001 (TDD discipline BC) citations in product code crosses namespace boundary | 2026-06-30 |
| 287 | OPEN |  | feat(skill): /vsdd-factory:check-bc-version-propagation — enforce the 9-site BC-version propagation checklist | 2026-06-26 |
| 286 | OPEN |  | enhancement(planning): BC-version-bumps during convergence as a stronger story-sizing signal than pre-delivery AC ratio | 2026-06-26 |
| 285 | OPEN |  | enhancement(implementer): require GUT run + zero-failure verification before reporting "done" | 2026-07-09 |
| 283 | OPEN |  | enhancement(adversarial-loop): hardening sweep — pending-reason format + convergence metrics + mid-loop state + implementer scope | 2026-06-26 |
| 282 | OPEN |  | enhancement(stub-architect): warn when planned class_name collides with autoload-eligible singleton name (Godot 4) | 2026-06-26 |
| 281 | OPEN |  | enhancement(planning): story-vs-BC AC-count ratio as pre-delivery story-quality signal | 2026-06-26 |
| 280 | OPEN |  | enhancement(test-writer): warn about GUT 9.x assert_signal_emitted_with_parameters 4th-arg signature trap | 2026-07-02 |
| 279 | OPEN |  | bug(state-manager): STATE.md timestamp time-of-day lost on unrelated edits | 2026-06-26 |
| 278 | OPEN |  | observation(orchestrator): consistency-validator + spec-reviewer verdict gap on shared input suggests prompt calibration drift | 2026-06-29 |
| 277 | OPEN |  | bug(state-manager+story-writer): derived summary fields (BC count, VP count, story totals) go stale on primary content add | 2026-07-16 |
| 276 | OPEN |  | enhancement(orchestrator): make spec-reviewer cross-eye mandatory post-spec-burst (consistency-validator alone misses critical defects) | 2026-06-26 |
| 275 | OPEN |  | bug(orchestrator+po+architect): parallel agent bursts make contradictory architectural decisions independently when their work has semantic coupling | 2026-06-26 |
| 274 | OPEN |  | bug(architect+implementer): ADR amendment + code update repeatedly diverge across multiple revisions | 2026-06-26 |
| 273 | OPEN |  | enhancement(orchestrator): parallel agent bursts on factory-artifacts orphan branch race on staging area; commits co-mingle work from multiple agents | 2026-06-30 |
| 272 | OPEN |  | bug(architect): hallucinated internal/ packages declared in ARCH-08 §6.5 — story stubs read as current-state | 2026-06-26 |
| 269 | OPEN |  | bug(sub-agents): SendMessage from orchestrator carrying user-confirmed authorization is treated as untrusted, blocking convergence | 2026-06-26 |
| 268 | OPEN |  | bug(stub-architect): lint-clean does not imply parser-clean; stubs land that fail to load in the target runtime | 2026-06-26 |
| 267 | OPEN |  | bug(state-manager): hallucinated artifact names in audit logs / red-gate logs — needs source-of-truth validation before commit | 2026-06-26 |
| 266 | OPEN |  | enhancement(per-story-delivery): factor out Rust-specific idioms (cargo/todo!()/clippy) for language-agnostic delivery | 2026-06-26 |
| 265 | OPEN |  | enhancement(demo-recorder): add "test-output-as-demo" mode for game/library/embedded projects without renderable surfaces | 2026-06-25 |
| 263 | OPEN |  | bug(product-owner) + open-question(repo-initialization): PO agent scope-overreach (branch switch + gitlink registration PR) reveals missing parent-repo .gitignore guard | 2026-06-28 |
| 261 | OPEN |  | Meta: framework risk — accidental leak of private-project content to public-upstream issues | 2026-06-24 |
| 260 | OPEN |  | bug(orchestrator+wave-closure): orchestrator silently defers adversary findings to non-existent "TBD" stories and self-ratifies closure, violating its own documented cycle-closing rule | 2026-07-01 |
| 259 | OPEN |  | Adversary mandate: convergence on broken code due to code-only review | 2026-06-29 |
| 258 | OPEN |  | bug(orchestrator): Agent dispatch returns synthesized "[Request interrupted by user for tool use]" with no real user input; orchestrator mis-routes to "pause" instead of retry | 2026-07-01 |
| 257 | OPEN |  | bug(devops-engineer): branch protection silently bypassed — required_status_checks.contexts uses workflow filename instead of GitHub-reported check name | 2026-06-24 |
| 256 | OPEN |  | documentation(per-story-delivery): hardcoded 'Target: develop' causes friction on trunk-based projects (default branch=main) | 2026-06-28 |
| 255 | OPEN |  | enhancement(dx-engineer): preflight should include OS-version compatibility check, not just installation status | 2026-06-24 |
| 254 | OPEN |  | bug(research-agent): MCP-mandatory gate silently degrades to WebSearch when MCP servers absent; produces shallow research wearing deep-research formatting | 2026-06-24 |
| 253 | OPEN |  | enhancement(research-agent): require OS-version-specific upstream advisory checks when recommending tool versions | 2026-06-24 |
| 252 | CLOSED |  | enhancement(repo-initialization): set delete_branch_on_merge=true + create .factory/.gitignore for dispatcher logs | 2026-06-24 |
| 251 | OPEN |  | enhancement(repo-initialization): set delete_branch_on_merge=true + create .factory/.gitignore for dispatcher logs | 2026-06-25 |
| 250 | OPEN |  | bug(orchestrator): agents commit but don't push — 96 commits accumulated unpushed across 2 branches (counterpart to #240) | 2026-06-24 |
| 248 | OPEN |  | enhancement(lint): bidirectional save-schema lint — catch BC/HO references to fields not registered in ADR-0006 | 2026-06-24 |
| 247 | OPEN |  | enhancement(orchestrator): decision-cascade scope must include capability shards + state machines + verification docs | 2026-06-24 |
| 246 | OPEN |  | enhancement(phase-1d): per-subsystem deep-read adversarial pass for spec corpora >100 files | 2026-06-24 |
| 245 | OPEN |  | bug(observability): session-count panels depend on claude_code_session_count_total which ages out after 5m — show 0 sessions during active sessions | 2026-06-24 |
| 244 | OPEN |  | bug(observability): claude-cost dashboard cache-hit-ratio formula omits cacheCreation tokens — displays ~100% when true value is ~89% | 2026-06-24 |
| 243 | OPEN |  | bug(observability): five Grafana dashboards use '\| json \| attributes_X' LogQL syntax that returns zero series — Claude-side fields are structured metadata, not JSON body | 2026-06-24 |
| 242 | OPEN |  | bug(dispatcher): ResolverLoader::load_registry passes relative plugin paths to canonicalize() — 12k+ resolver.load_error per day, wave_context permanently disabled | 2026-07-08 |
| 241 | OPEN |  | bug(observability): factory hook events never reach Loki — sink-file driver is not wired into the dispatcher | 2026-06-24 |
| 240 | OPEN |  | bug(orchestrator): broad-burst agents (PO, spec-steward) systematically drop git commit step, leaving 99+ files uncommitted across multiple passes | 2026-06-24 |
| 239 | OPEN |  | docs(observability): mention OrbStack and Colima alongside Docker Desktop | 2026-06-23 |
| 238 | OPEN |  | bug(observability): onboard-observability skill returns success without verifying Docker is installed | 2026-06-24 |
| 237 | OPEN |  | docs(observability): document WHEN to enable the observability stack within the pipeline lifecycle (currently undocumented) | 2026-06-23 |
| 236 | OPEN |  | bug(factory-health): output should distinguish "verified intact" vs "just bootstrapped from empty" — current state is misleading | 2026-06-24 |
| 235 | OPEN |  | feat(onboarding): surface telemetry opt-out path during /onboard-observability and in skill docs | 2026-06-23 |
| 234 | OPEN |  | feat(version): factory-health (or a new preflight) should surface plugin version + check freshness vs latest release | 2026-06-23 |
| 233 | OPEN |  | feat(coexistence): factory should detect and document interop with existing agent frameworks (BMAD, kos, etc.) instead of plowing through | 2026-06-23 |
| 232 | OPEN |  | feat(lifecycle): provide a deactivate / uninstall path to cleanly remove vsdd-factory artifacts from a project | 2026-06-23 |
| 231 | OPEN |  | feat(meta): add .github/ISSUE_TEMPLATE/ and PULL_REQUEST_TEMPLATE.md to lower contribution friction | 2026-06-23 |
| 230 | OPEN |  | feat(factory-health): bootstrap .factory/.gitignore so dispatcher logs / tmp / locks don't accumulate in the orphan branch | 2026-06-23 |
| 229 | OPEN |  | bug(factory-health): STATE.md template hardcodes 'product: corverax' instead of detecting / asking | 2026-06-24 |
| 228 | OPEN |  | feat(preflight): validate origin remote, GitHub identity, repo permissions, fork status, and CI rights before pipeline assumes them | 2026-06-23 |
| 227 | OPEN |  | feat(preflight): factory should validate branching strategy, default branch, branch protection, and GitHub repo settings on session entry | 2026-06-24 |
| 226 | OPEN |  | bug(setup-env): MCP env-var probe uses bash-only ${!var} indirect substitution, fails on zsh | 2026-06-24 |
| 225 | OPEN |  | bug(setup-env): skill assumes Rust toolchain — should detect language and adapt (Go template included) | 2026-06-24 |
| 224 | OPEN |  | enhancement(adversary): require scan-plan declaration and coverage report in adversarial pass output | 2026-06-24 |
| 223 | OPEN |  | documentation(orchestrator): clarify when to use SendMessage vs fresh Agent dispatch | 2026-06-24 |
| 222 | OPEN |  | documentation(phase-1d): document rationale for '3 clean passes minimum' convergence rule | 2026-06-24 |
| 221 | OPEN |  | bug(orchestrator): task-tracking tools (TaskCreate/Update/List) should be default-loaded, not deferred | 2026-06-24 |
| 220 | OPEN |  | enhancement(orchestrator): provide token-budget visibility primitive (mirror Workflow's budget.spent/remaining) | 2026-06-25 |
| 219 | OPEN |  | enhancement(architecture): canonical subsystem-registry.yaml to unify name/shard/class-name mappings | 2026-06-24 |
| 218 | OPEN |  | enhancement(cicd-setup): auto-generate §Jobs table from ci.yml to eliminate two-place drift | 2026-06-24 |
| 217 | OPEN |  | enhancement(phase-1d): document relaxed-convergence alternative path with known-issues sign-off | 2026-06-24 |
| 216 | OPEN |  | enhancement(orchestrator): standardize 'enumerate before fix' sweep template to prevent incomplete sibling propagation | 2026-06-24 |
| 215 | OPEN |  | enhancement(orchestrator): burst-size rule should cover file modifications, not just creations | 2026-06-24 |
| 214 | OPEN |  | enhancement(governance): introduce process-gap codification ledger to enforce S-7.02 cycle-closing checklist | 2026-06-24 |
| 213 | OPEN |  | enhancement(phase-1d): add decision-registry pre-flight to formalize conventions before adversarial review | 2026-06-24 |
| 212 | OPEN |  | bug(state-manager): transient API connection errors require manual orchestrator retry | 2026-06-24 |
| 211 | OPEN |  | bug(adversary): agent cannot persist its own review report; round-trips through orchestrator | 2026-07-13 |
| 210 | OPEN |  | bug(orchestrator): silent data loss when parallel agents edit and rename the same file | 2026-06-24 |
| 209 | OPEN |  | enhancement(factory-health): orphan-branch commit recipe should explicitly use 'git commit -S' for signing | 2026-06-23 |
| 208 | OPEN |  | enhancement(factory-obs): 'factory-obs register' should probe the running stack and tell the operator what to do next | 2026-06-27 |
| 207 | OPEN |  | enhancement(factory-obs): 'factory-obs status' fails without separating registration state from docker stack state | 2026-06-24 |
| 206 | OPEN |  | bug(factory-obs): dispatcher races with /factory-health worktree setup by continuously recreating .factory/logs/ | 2026-06-24 |
| 205 | OPEN |  | bug(factory-health): 'git worktree add .factory factory-artifacts' silently mounts at .factory/.factory/ when racing with dispatcher | 2026-06-24 |
| 204 | OPEN |  | bug(factory-health): orphan-branch recipe strands the session on factory-artifacts when 'git checkout -' fails | 2026-06-24 |
| 203 | OPEN |  | bug(onboarding): /onboard-observability creates .factory/logs/ in a plain dir, blocking later /factory-health worktree mount | 2026-06-24 |
| 177 | OPEN |  | Add a hollow-demo / false-confidence checker (agent + skill + gate integration) | 2026-06-03 |
| 176 | CLOSED | enhancement | adversarial-review: add worktree-identity preflight to prevent wrong-tree false-positive findings | 2026-06-10 |
| 175 | OPEN | enhancement | feat(activate): version-drift guard — block factory commands after a plugin update until re-activation | 2026-06-01 |
| 174 | OPEN | documentation, enhancement | feat: CLAUDE.md health-check + threshold-driven compaction (mirror STATE.md size governance) | 2026-06-01 |
| 173 | OPEN | enhancement | feat(context): enforce wave-boundary checkpoint+reset and lossless intra-wave compaction (PreCompact flush + WASM gates) | 2026-06-26 |
| 172 | OPEN | enhancement | feat(demo): route demo evidence to factory-artifacts (not the product repo), with operator choice of repo / factory-artifacts / local-only | 2026-06-01 |
| 171 | OPEN | enhancement | feat(workflows): revalidate deferred items with research-agent before pulling them into active work | 2026-06-01 |
| 170 | CLOSED | enhancement | feat(state): single-writer factory lock/lease — prevent concurrent developers racing the same repo's factory-artifacts state | 2026-06-11 |
| 169 | CLOSED | bug | Per-story sub-agents read stale worktree .factory/specs instead of canonical repo-root specs (phantom adversarial findings) | 2026-06-10 |
| 162 | OPEN | enhancement | process: orchestrator methodology-bypass ("firefighting mode") — enforce VSDD sequence at runtime, not just in prose | 2026-05-29 |
| 151 | OPEN | enhancement | feat(spec-ci): adopt drift-resistant source-citation convention + checker | 2026-06-01 |
| 150 | OPEN | enhancement | Feature: Per-Story Uncertainty Removal + Self-Containment Review (Pre-Phase-3 Quality Gate) | 2026-07-03 |
| 149 | OPEN |  | OTEL telemetry to reduce agent handwaving | 2026-06-24 |
| 133 | OPEN | enhancement | feat(workflows): add intra-phase adversarial passes after architecture artifacts + fix-bursts | 2026-05-13 |
| 131 | OPEN | enhancement | feat(consistency-validator): add URL/endpoint/path coherence check across diagrams + tables + prose | 2026-05-13 |
| 130 | CLOSED | bug | bug(dispatcher): creates recursive .factory/.factory/logs/ shadow when invoked with cwd inside .factory/ | 2026-06-10 |
| 129 | OPEN | enhancement | feat(canonicalization): production-grade default + correct agent routing principle | 2026-05-12 |
| 128 | CLOSED | bug | bug:  pr-manager claims to delete the remote branch on merge but the branch often survives | 2026-06-09 |
| 126 | CLOSED | bug | research-agent: Perplexity MCP tool names don't match @perplexity-ai/mcp-server published names; Tavily allowlist entirely missing | 2026-05-12 |
