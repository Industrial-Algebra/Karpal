# Discovery Feedback — karpal-discovery 0.9.0 vs. Knopper (2026-08-22)

**From:** the Knopper identity-restoration session. Ran `karpal.imports`,
`karpal.concepts`, and `karpal.recommend` (built `--features lonis` from
workspace) against the Knopper workspace (develop @ b180b60). Consumer: a
coding agent planning GA/Schubert integration and mining karpal's overlay for
design vocabulary. Consumer-side report:
`Knopper/docs/research/2026-08-22-discovery-amari-karpal.md`.

## What worked

1. **The 83-concept overlay is the product.** The concept summaries are
   concise, precise, and Rust-anchored ("A structure-preserving map …; in
   Rust, a context whose contained values can be transformed via `map`").
   They were the single most valuable artifact of the run: I mapped Knopper's
   architecture onto them by hand —
   - `Machine` (Context + Model + project) ≈ **Store comonad + ComonadEnv**
   - projection-injection seam ≈ **Optic**
   - `Effect<Msg>` (closed enum; async bridge open) ≈ **free-monad quotient**
   - `resolve_layout` ≈ **cata/hylo**
   - presence tones / merges ≈ **lattice / Heyting / semiring**
   That mapping exercise is what motivated this feedback.
2. **Honest zeros.** `karpal.imports` on a non-karpal workspace returns
   `resolved: 0` with no noise — correct and useful.
3. **Provider CLI manners.** Unknown tool → "see `karpal --mode json tools
   list`"; malformed input → the expected shape in the error. Good ergonomics
   for a Lonis provider.

## What didn't: `karpal.recommend` recall is too strict

Every non-canonical phrasing missed, even when the *concept's own summary*
contained the query vocabulary:

| Query | Result | Should have hit (its summary literally says…) |
| ----- | ------ | ---------------------------------------------- |
| "bidirectional focus on a part of a structure, get and put nested fields" | 0 | **optic** — "get/put-style access" |
| "context with state and a focus position, extract a value at the cursor" | 0 | **Store comonad** — "state + focus" |
| "merge two partial states with least upper bound join" | 0 | **lattice** — "binary meet and join" |
| "effectful map over a structure preserving shape, commuting two layers" | 0 | **traversable** — "commutes two layers" |
| "sequence dependent effectful steps" (README-canonical) | ✅ monad | — |

The recall tokens appear to be curated problem-shapes only; the summaries and
aliases are richer than the matcher uses. Cheapest fix with the biggest yield:
**index concept summaries + aliases as recall text** (bag-of-words or
BM25-lite over summary+aliases+problem shapes), or at minimum stem/synonymize
("get and put" ↔ "get/put"). The canonical-phrase-only behavior makes
`recommend` unusable for agents that don't already know the answer.

## The proposal: a doctrine/patterns aspect (maintainer's own musing, seconded)

The maintainer noted Lonis already has the facility for this. The Knopper run
is a concrete use case for it:

**A pattern = a named mapping from a foreign architecture shape to overlay
concepts, with detection signals.** Examples straight out of this session:

- *Elm-architecture machine* → Store comonad + ComonadEnv. Signals: a trait
  with `init`/`update`/`view`-shaped methods, associated `Msg`/`Model` types,
  single-dispatch messages. karpal already parses `syn` ASTs — detecting
  "trait with associated types named Msg/Model" is a structural query it can
  answer today.
- *FRP signal graph* → Traced monoidal / ArrowLoop. Signals: subscribe/push
  pairs, `Behavior`/`Signal` type names (cliffy-shaped).
- *Diff/draw pipeline* → recursion schemes (`cata`/`hylo`). Signals: tree
  fold over a node enum with a child-vector variant.

**A doctrine = maintainer-level positioning constraints that steer ranking.**
Example from Knopper's plan: "the GA substrate is the identity — never
recommend routing around geometry." With doctrine payloads, `recommend` could
rank a lens/prism suggestion *by whether it composes with or bypasses the
substrate*, and flag violations ("this approach stubs the geometry" — exactly
the failure mode the Rabbit Hole report caught in shipped code).

Together they'd make karpal-discovery useful against **non-karpal codebases as
an architecture-analysis engine** — which is how this session used it, by
hand. Suggested surface: `karpal.patterns detect --workspace X` → matched
patterns with evidence anchors (file:line of the firing signals), plus
doctrine metadata on recommend results.

## Minor notes

- `karpal.concepts` with empty query returned the full 83 — good; consider
  pagination metadata if the overlay grows.
- Import analysis on a foreign workspace could optionally report *near-miss*
  symbols (e.g., Knopper re-exports `IntoGeometric`/`FromGeometric` from
  cliffy — a "concept-adjacent but non-karpal" signal), though the honest
  zero is also fine.

— Knopper session, 2026-08-22.
