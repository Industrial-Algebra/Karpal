# Discovery with the `karpal` Binary

`karpal-discovery` is the agent-first discovery runtime of Phase 19 — the second [Lonis](https://github.com/Industrial-Algebra/Lonis) vertical. Its `karpal` binary is both a human CLI and a conforming Lonis `SubprocessProvider`: AI-agent harnesses discover and invoke it through one uniform protocol, and humans use the same commands directly.

Where `karpal-index` string-scans source files, `karpal-discovery` builds a **typed, deterministic catalog** with a real `syn` AST parse, layers a **curated mathematical overlay** (83 concepts with problem shapes and relationships) over it, and answers questions at the level agents actually ask them: *"I need to sequence dependent effectful steps"* → `monad`.

## Installation

The binary is built with the `lonis` feature:

```sh
cargo install --path karpal-discovery --features lonis --bin karpal
```

The library itself is usable without the feature (the CLI and the Lonis output layer are optional):

```toml
[dependencies]
karpal-discovery = "0.9"
```

## The Lonis Provider Protocol

`karpal` speaks the ADR-0006 v0 provider surface — the same protocol `lonis` itself speaks, so any Lonis host can use it without bespoke glue:

```bash
$ karpal --mode json manifest
{"name":"karpal","version":"0.9.0","provider_type":"external-executable",
 "protocol_version":"0","tools":["karpal.search","karpal.detail",...],
 "display_name":"Karpal Discovery"}

$ karpal --mode json tools list
$ karpal --mode json tools describe karpal.recommend
{"name":"karpal:recommend","description":"Recall and Pareto-rank curated concepts...",
 "determinism":"deterministic","side_effects":"read-only","cost":"low"}
```

Invocation is ADR-0003: JSON on stdin, a block array on stdout, structured `ToolError` on stderr. In-process hosts use `lonis_core::SubprocessProvider`; the binary is equally usable from a shell:

```bash
$ echo '{"workspace": ".", "query": "Functor"}' | karpal call karpal.search
```

Every tool is deterministic, read-only, and low-cost — stated in each tool's contract.

## The Nine Tools

### `karpal.search` — catalog items

```json
{"workspace": ".", "query": "Functor"}
```

Case-insensitive substring search over every public item name (traits, functions, structs, enums, type aliases, macros, consts) in the workspace catalog.

### `karpal.detail` — one item, fully joined

```json
{"workspace": ".", "item": "Functor", "crate": "karpal-core"}
```

Returns the item with its docs, its implementors (from the trait implementation graph), and the overlay concepts anchored to it. For `Functor`:

```json
{
  "item": {"name": "Functor", "crate_name": "karpal-core",
           "module_path": "karpal_core::functor", "kind": "trait"},
  "docs": "Covariant functor: lifts a function `A -> B` into `F<A> -> F<B>`.",
  "implementors": ["CofreeF", "ComposeF", "EnvF", "FixF", "FreeF",
                   "IdentityF", "NonEmptyVecF", "OptionF", "ResultF", "VecF"],
  "concepts": [{"id": "functor", "stability": "stable", ...}]
}
```

### `karpal.concepts` — browse the overlay

```json
{"query": "sheaf"}
```

Searches curated concepts across ids, names, aliases, mathematical concepts, and **problem shapes**. An empty query lists all 83.

### `karpal.imports` — what a project actually uses

```json
{"workspace": "."}
```

Analyzes a workspace's `use` statements against its own catalog: resolved symbols (with per-file counts), unresolved imports (the drift signal — stale references to removed or renamed items), and the curated **concepts in use**.

### `karpal.recommend` — recall and rank for a goal

```json
{"goal": "sequence dependent effectful steps"}
```

The planner's recall layer: direct matches seed candidates and the relation graph expands them; ranking is Pareto dominance over (relevance, weight). Ask for the problem, get the concept — `monad`, ranked first, with evidence:

```json
{
  "concept_id": "monad",
  "stability": "stable",
  "relevance": 14,
  "evidence": ["match: problem shape",
               "relation: composes_with free-monad ↔ monad",
               "relation: dual_of comonad ↔ monad"]
}
```

### `karpal.plan` — orient, explore, verify

```json
{"goal": "monad"}
```

A candidate plan for the goal: orient on it, explore the top-ranked concepts (bounded to three), verify the drift gate. Plans are built as a `Free` monad and normalized (adjacent duplicate steps collapse).

### `karpal.probe_list` / `karpal.probe_describe` / `karpal.probe_run` — algebraic probes

```json
{"id": "schubert-intersection"}
```

Five probes, each running real library code: functor/monad laws on `Option`, the `karpal-proof` law checkers, Schubert intersections (**the structured-emptiness thesis live** — `Positive` vs `StructuralZero`), recursion-scheme agreement (`hylo ≡ cata ∘ ana`), and Mac Lane coherence witnesses. `karpal.probe_run` reports each check it demonstrated.

## `karpal-index` Compatibility

Legacy `karpal-index` invocations keep working through the compat mode:

```bash
$ karpal --index-compat search Functor --json
$ karpal --index-compat hierarchy Monad --json
```

The JSON shapes are the legacy ones (`ApiItem`, `Hierarchy`, `null` for not-found) over the new catalog. Divergences are documented in the crate README: `path` carries the module path (no line numbers), and `subtraits` is empty — as it always was.

## Next

- [Discovery Runtime Reference](../reference/discovery.md) — architecture, the library API, and how the planner dogfoods Karpal's own typeclasses.
- [karpal-index guide](./karpal-index.md) — the legacy binary this succeeds.
