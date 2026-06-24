# PC-013: Idiomatic Handling of Provably-Unreachable Internal Invariants in Rust

**Type:** general (technology / Rust style)
**Date:** 2026-06-24
**Scope:** wirerust (network protocol analyzer, single-crate Rust 2024 library + CLI)
**Question:** For a site where a key's presence is established by an earlier `map.get(&k)` and
re-asserted a few statements later via `map.get_mut(&k).expect("present: checked above")` (entry
provably cannot be removed in between, panic-freedom separately proven), what is the idiomatic
choice among (1) keep `.expect("documented invariant")`, (2) convert to a silent
`if let Some(entry) = ... { }` graceful-skip, (3) use `unreachable!()` / `debug_assert!`?

---

## TL;DR Recommendation

**Keep the loud assertion. Do NOT convert to a silent skip.**

For a truly "this cannot happen by construction" internal invariant in a library, the idiomatic
Rust choice (2024–2026) is to **fail loud** via a panic-class mechanism — `.expect("...")`,
`assert!`, or `unreachable!()` — with a self-documenting message. Converting a loud `.expect()`
into a silent `if let Some(...) { } /* else: drop the work */` is the one option that the
mainstream Rust guidance specifically flags as an **anti-pattern**: it reclassifies a library
bug as a normal condition and masks logic errors.

**Concrete preference for this site:** keep `.expect("present: checked above")` (Option 1). It is
the most diagnostically specific choice for an `Option`-presence invariant, and it preserves
fail-loud-in-release semantics. `unreachable!()` (Option 3a) is acceptable and almost equivalent;
`debug_assert!`-only (Option 3b) is **not** sufficient on its own because it is elided in release.

Confidence: **High.** Official sources (The Rust Book, std docs, Rust API Guidelines) and
community practice converge. Guidance is genuinely split only on a *narrower* question (panic vs.
explicit `Result`/error-return for internal bugs), **not** on panic vs. silent-skip — see
"Where Guidance Is Split."

---

## 1. The Core Principle: Fail Loud on Broken Invariants, Fail Recoverably on Expected Failures

Rust draws a sharp line between two categories:

- **Recoverable errors** (expected failure modes: malformed input, I/O failure, resource
  exhaustion) → represent with `Result<T, E>` so the caller decides what to do.
- **Unrecoverable bad states** (broken invariant, violated contract, impossible state) →
  `panic!` because the program "can't handle" this and proceeding on invalid data is wrong or
  dangerous.

The Rust Book chapter *"To `panic!` or Not to `panic!`"* states this directly: panicking is
advisable when a "bad state" has been reached — an assumption, guarantee, contract, or invariant
has been broken — and "code after this point relies on not being in this bad state," with no good
way to encode the constraint in the type system. Conversely, when failure is *expected* (a parser
hitting malformed bytes, an HTTP rate limit), return `Result`.
[Rust Book ch. 9.3](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)

The Book *explicitly endorses `.expect()`* for exactly the scenario in this ticket: when "you
have some other logic that ensures the `Result` will have an `Ok` value, but the logic isn't
something the compiler understands," it is acceptable to `.expect()` and document the reasoning.
[Rust Book ch. 9.2](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html),
[ch. 9.3](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)

The `map.get(&k)` → `map.get_mut(&k).expect("present: checked above")` pattern *is* this case:
prior logic establishes the invariant; the compiler cannot see it; `.expect()` both enforces and
documents it.

**"Fail loud vs. fail silent" is precisely the relevant principle here.** The invariant violation
at the second `get_mut` is, by construction, a *library bug* — not a user-facing condition. The
fail-loud mechanisms (`expect`/`assert!`/`unreachable!`) are "fail-safe": they stop the program
from proceeding in a state that contradicts its own assumptions. A silent skip is "fail-open": it
proceeds without enforcing the constraint.

---

## 2. The Specific Risk of Converting `.expect()` → Silent Skip

This is the crux of the ticket. The conversion is **not semantics-neutral**. It changes
"assert this is true, crash loudly if not" into "treat the impossible as routine and quietly drop
the work." Concrete risks, all sourced:

1. **It masks logic bugs.** If a future refactor introduces a path that removes the entry (or the
   "panic-freedom proof" silently rots), the `.expect()` would have pointed a maintainer straight
   at the violated assumption with the message "present: checked above." The silent skip instead
   produces *partial/incorrect results with no signal at all* — the hardest class of bug to
   diagnose. The Book's whole rationale for panicking on bad state is to "prevent your code from
   operating on bad data" and "alert the person ... that there's a bug."
   [Rust Book ch. 9.3](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)

2. **It reclassifies a bug as a documented behavior without saying so.** A silent skip is only
   defensible if "entry absent" is an *expected* condition that is part of the API/behavioral
   contract — in which case the idiomatic representation is an explicit `Option`/`Result` return,
   not a buried `if let` that drops work. For an internal invariant that is provably unreachable,
   absence is by definition *not* expected.
   [Rust API Guidelines — documentation](https://rust-lang.github.io/api-guidelines/documentation.html)

3. **Community consensus calls this an anti-pattern.** Forum discussion is consistent: an
   `Option` that "should always be `Some`" by a prior check/contract is a case where panicking is
   "by definition allowed/expected ... as something exceptional," and the value should be
   "squashed as soon as possible" with `.expect()`/`assert!` and a relevant message — not
   silently absorbed.
   [users.rust-lang.org — unwrap/expect vs unreachable](https://users.rust-lang.org/t/unwrap-expect-vs-unreachable/122275),
   [users.rust-lang.org — panic/unreachable/error in a library](https://users.rust-lang.org/t/panic-unreachable-and-error-in-a-library-written-for-non-programmers/50490)

**Bottom line:** converting loud `.expect()` to a silent skip trades a loud, debuggable failure
for a silent, undebuggable one. That is the wrong direction for an internal invariant.

---

## 3. `expect()` vs `unreachable!()` vs `debug_assert!` — Self-Documentation & Release Behavior

| Mechanism | Release behavior | Self-doc quality for *this* site | Verdict for an `Option`-presence invariant |
|---|---|---|---|
| `.expect("present: checked above")` | **Panics** (not elided) | **Best** — message ties directly to the value + the earlier check | **Recommended** |
| `unreachable!("checked present above")` | **Panics** (not elided) | Good — signals "this branch is impossible"; slightly more generic semantics; needs an explicit `else` arm | Acceptable, near-equivalent |
| `assert!(map.contains_key(&k))` | **Panics** (not elided) | Good — boolean-condition invariant, costs an extra lookup | Acceptable but redundant with the `get_mut` |
| `debug_assert!(entry.is_some())` | **Elided by default** in release | Documents intent, but does NOT enforce in release | **Insufficient alone** — see below |
| Silent `if let Some(...) { }` | No panic; work dropped | **None** — no signal on violation | **Anti-pattern** for this invariant |

Key facts, sourced:

- **`unreachable!()` is just `panic!` with a fixed message.** std docs: "`unreachable!` is just a
  shorthand for `panic!` with a fixed, specific message," for code "the compiler can't determine
  is unreachable"; "if the determination ... proves incorrect, the program immediately terminates
  with a `panic!`." Critically, std docs warn it is for code *truly logically unreachable by your
  program's logic* — **not** for paths bad input could reach (those are error conditions).
  [std `unreachable!`](https://doc.rust-lang.org/std/macro.unreachable.html)

- **`expect` is more specific than `unreachable!` for value invariants.** Forum guidance:
  "`unreachable!()` is a panic for internal errors with a *generic* error message" whereas
  `None.unwrap()` / `.expect(msg)` gives a more specific message — so for a "this `Option` must be
  `Some`" invariant, `.expect()` wins on diagnostics. `unreachable!()` is the better fit for
  structurally-impossible *branches* (e.g., a match arm that cannot occur).
  [users.rust-lang.org — unwrap/expect vs unreachable](https://users.rust-lang.org/t/unwrap-expect-vs-unreachable/122275)

- **`debug_assert!` is elided in release by default.** std docs: "`debug_assert!` statements are
  only enabled in non-optimized builds by default"; an unchecked (elided) assertion "allows a
  program in an inconsistent state to keep running." It is for checks "too expensive to be present
  in a release build." Therefore relying on `debug_assert!` *alone* for a correctness-critical
  invariant means the invariant is **unenforced in production** — the violated state would proceed
  silently in release, which is the very failure mode we are trying to avoid.
  [std `debug_assert!`](https://doc.rust-lang.org/std/macro.debug_assert.html)

  `debug_assert!` is a legitimate *additional* layer (cheap, loud-in-dev) layered on top of
  correct release behavior, but not a replacement for `.expect()`/`assert!` when the invariant
  must hold in release.

**`unwrap` vs `expect`:** When you do assert success, prefer `.expect(msg)` over `.unwrap()`.
Clippy's `unwrap_used` lint nudges toward `expect` precisely so a human-readable message records
"why this can't fail." Many practitioners treat an `.expect()` message as a sign-post that the
panic is *intentional and reasoned*, distinct from a thoughtless unwrap.
[users.rust-lang.org — unwrap/expect vs unreachable](https://users.rust-lang.org/t/unwrap-expect-vs-unreachable/122275),
[internals — prohibit unwrap thread](https://internals.rust-lang.org/t/prohibit-the-use-of-unwrap-and-increase-panics-privacy/20263)

---

## 4. Guidance for Security-Sensitive / Network-Parsing Code (directly relevant to wirerust)

wirerust parses untrusted network/PCAP input, so the panic-vs-degrade tension matters. The
guidance resolves cleanly once you separate the two categories:

- **Untrusted input (the bytes on the wire / file):** expected to be malformed/adversarial. This
  is a *recoverable* failure. Idiomatic handling is `Result`/error-typed parse failures or
  fault-tolerant "error node" constructs — NOT panics, and NOT silent unconditional drops without
  a record. This is the established pattern for robust parsers (e.g., emit an error node in the
  AST / a typed parse error and continue).
  [Rust Book ch. 9.3](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html),
  [users.rust-lang.org — fault-tolerant parsing with nom](https://users.rust-lang.org/t/fault-tolerant-parsing-with-nom/97961)

- **Internal invariants of the analyzer's own state** (the `map.get`/`get_mut` case): violation
  means the analyzer is in a state its own logic says is impossible — a *bug*. The Book is
  explicit that when continuing "could be insecure or harmful," panicking is the right call: it
  stops the system from operating under incorrect assumptions, which in a security tool could
  otherwise yield subtly wrong findings or analysis. Security best practice favors **fail-safe**
  (refuse to proceed) over **fail-open** (proceed without enforcing the constraint); a silent skip
  is the fail-open option.
  [Rust Book ch. 9.3](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)

- **DoS caveat.** The one legitimate concern with panics in a network service is denial-of-service
  if a panic crashes the whole process on attacker-influenced input. The mitigation is
  *architectural* (per-unit isolation / panic boundaries / ensuring untrusted-input paths use
  `Result`, not panic), **not** silencing internal-invariant assertions. This concern applies to
  input-driven panics; it does **not** justify converting a provably-unreachable internal
  invariant into a silent skip, because by construction that site is not reachable by input.
  [users.rust-lang.org — recover future chain from panic (HTTP server security)](https://users.rust-lang.org/t/how-to-recover-future-chain-from-panic-need-to-close-security-vulnerability-for-http-server/13980)

**Net:** For wirerust specifically — keep untrusted-input parse failures recoverable (`Result` /
typed errors), and keep internal "can't happen by construction" invariants loud
(`expect`/`unreachable!`). The ticket's site is the latter category.

---

## 5. Is a "Fail-Safe Degradation: Silently Skip" Contract Good or Anti-Pattern?

For **internal (non-I/O) invariants: anti-pattern.** A "silently skip if absent" contract is only
appropriate when absence is a *genuinely expected, documented* condition (an I/O / input-driven
failure mode that the API contract describes). For an invariant that is "provably unreachable by
construction," absence is a bug, and silently skipping it:

- masks the bug,
- produces partial/incorrect output with no signal,
- and — note the terminology — is actually **fail-OPEN, not fail-safe**. The "fail-safe" framing
  is misapplied here: the truly fail-safe action for a violated internal invariant is to refuse to
  proceed (panic), not to quietly continue.

This is consistent across the Book, the API Guidelines, and community practice.
[Rust Book ch. 9.3](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html),
[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/documentation.html),
[users.rust-lang.org — unwrap/expect vs unreachable](https://users.rust-lang.org/t/unwrap-expect-vs-unreachable/122275)

If a maintainer *wants* skip-on-absence behavior, the correct move is to make it an explicit,
documented part of the API (return `Option`/`Result`, or log the skip) — not to bury it in an
`if let` that drops work.

---

## 6. Where Guidance Is Genuinely Split (flagged honestly)

The split in the Rust ecosystem is **NOT** "panic vs. silent-skip." It is the narrower question of
**panic vs. explicit error-return for internal bugs**:

- Some teams adopt strict `clippy::unwrap_used` / `expect_used` / `panic` lint configs and prefer
  to surface even internal-invariant violations as typed errors (`ok_or(FatalInternalError)?`)
  rather than panics — especially in high-availability / never-crash environments.
  [Clippy `unwrap_used`/`expect_used` interaction issue](https://github.com/rust-lang/rust-clippy/issues/9222),
  [users.rust-lang.org — "expect_used lint is useless?"](https://users.rust-lang.org/t/the-expect-used-lint-is-useless/79074)

- The mainstream position (Rust Book, std, API Guidelines, BurntSushi's "unwrapping isn't evil")
  holds that `unwrap`/`expect`/`assert!`/`unreachable!` are *legitimate and idiomatic* for
  signaling bugs / genuinely-unreachable states, provided the message documents the reasoning.
  [Rust Book ch. 9.2–9.3](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html),
  [BurntSushi — "Study Rust error handling" / unwrapping isn't evil](https://blog.burntsushi.net/)

Both camps **agree** that silently dropping work on a violated internal invariant is wrong. The
disagreement is only over loud-panic vs. loud-typed-error. For wirerust — a CLI/analyzer where a
violated internal invariant is a developer-facing bug, not an end-user-handleable error — the
panic-class choice (`.expect`/`unreachable!`) is the simpler, more idiomatic, and more
diagnostically useful option, and aligns with the project's existing "panic-freedom is separately
proven" posture (the proof is the justification that the panic is unreachable; the `.expect()` is
the tripwire if the proof ever breaks).

Note: Searches did not surface a specific, on-point statement from Niko Matsakis / withoutboats /
matklad on *this exact* `map.get`/`get_mut` micro-pattern. matklad's writing on assertions
("materialize the mental map of invariants in source via asserts") supports the general
fail-loud-with-assertions philosophy but does not address the silent-skip alternative directly.
[matklad — Parsing Advances (2025)](https://matklad.github.io/2025/12/28/parsing-advances.html)
This is flagged as a (minor) gap rather than guessed.

---

## 7. Recommended Action for the wirerust Site(s)

1. **Keep `.expect("present: checked above")`** (Option 1). It is idiomatic, loud in release,
   and the most diagnostically specific for an `Option`-presence invariant. Optionally tighten the
   message to name the invariant and its enforcer, e.g.
   `.expect("key present: inserted above in this fn and never removed before this point")`.

2. **Do NOT** convert to a silent `if let Some(...) { }` skip (Option 2) — anti-pattern; masks
   bugs; fail-open.

3. `unreachable!()` (Option 3a) is an acceptable equivalent if the code reads more naturally as an
   `else` branch; `.expect()` is marginally preferred for the value-presence case.

4. If desired, **add** a `debug_assert!(map.contains_key(&k))` *adjacent to the earlier check* as a
   cheap dev-time tripwire, but **do not** rely on `debug_assert!` alone (Option 3b) for the
   release-build guarantee.

5. **Best of all where feasible:** restructure to a single lookup
   (`if let Some(entry) = map.get_mut(&k) { ... }`) so the invariant becomes
   compiler-checked and the second-lookup invariant disappears entirely. This is the type-system
   route Rust idiom prefers when practical; fall back to `.expect()` only when a single borrow is
   not structurally possible.
   [users.rust-lang.org — unwrap/expect vs unreachable](https://users.rust-lang.org/t/unwrap-expect-vs-unreachable/122275)

---

## Sources (authoritative, verified via fetch/search 2026-06-24)

Primary / authoritative:
- Rust Book ch. 9.3 *To panic! or Not to panic!* — https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html
- Rust Book ch. 9.2 *Recoverable Errors with Result* (endorses `.expect()` for logic-guaranteed success) — https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
- std `unreachable!` macro docs — https://doc.rust-lang.org/std/macro.unreachable.html
- std `debug_assert!` macro docs — https://doc.rust-lang.org/std/macro.debug_assert.html
- std `panic!` macro docs — https://doc.rust-lang.org/std/macro.panic.html
- Rust API Guidelines (documentation: Errors / Panics sections) — https://rust-lang.github.io/api-guidelines/documentation.html
- Clippy lint index — https://rust-lang.github.io/rust-clippy/master/

Community / recognized practice:
- BurntSushi (Andrew Gallant) — error handling / "unwrapping isn't evil" — https://blog.burntsushi.net/
- matklad (Aleksey Kladov) — Parsing Advances (asserts materialize invariants) — https://matklad.github.io/2025/12/28/parsing-advances.html
- users.rust-lang.org — unwrap/expect vs unreachable — https://users.rust-lang.org/t/unwrap-expect-vs-unreachable/122275
- users.rust-lang.org — panic/unreachable/error in a library — https://users.rust-lang.org/t/panic-unreachable-and-error-in-a-library-written-for-non-programmers/50490
- users.rust-lang.org — fault-tolerant parsing with nom — https://users.rust-lang.org/t/fault-tolerant-parsing-with-nom/97961
- users.rust-lang.org — recover future chain from panic (HTTP server security) — https://users.rust-lang.org/t/how-to-recover-future-chain-from-panic-need-to-close-security-vulnerability-for-http-server/13980
- Clippy unwrap_used/expect_used interaction — https://github.com/rust-lang/rust-clippy/issues/9222

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 1 | Deep multi-source synthesis (high reasoning_effort) on idiomatic handling of provably-unreachable internal invariants, expect vs unreachable vs debug_assert, silent-skip risk, security/parsing guidance, fail-safe contracts |
| Perplexity perplexity_search | 2 | Raw ranked sources: unreachable/expect/debug_assert best practice; matklad assertions & parsing-code panic vs degradation |
| WebFetch | 1 | Verified std `unreachable!` docs directly (reachable-by-input vs truly-unreachable distinction) |
| Training data | 1 area | General framing of fail-safe vs fail-open terminology — corroborated by sourced material, not relied on for claims |

**Total MCP tool calls:** 3 (1 perplexity_research + 2 perplexity_search) + 1 WebFetch
**Training data reliance:** low — every substantive claim is tied to an official Rust source
(Book / std docs / API Guidelines) or a cited community source verified this session.
