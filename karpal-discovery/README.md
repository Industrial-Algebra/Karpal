# karpal-discovery

Agent-first discovery runtime for the Karpal workspace — the second Lonis
vertical. A typed structural catalog, a curated mathematical overlay, project
inspection, imported-symbol analysis, a category-theoretic planner, and
algebraic probes, exposed through the `karpal` binary: a conforming
[Lonis](https://github.com/Industrial-Algebra/Lonis) `SubprocessProvider`
with nine tools.

Where `karpal-index` string-scans source files, this crate parses real `syn`
ASTs and answers questions at the level agents ask them: ask
`karpal.recommend` for *"sequence dependent effectful steps"* and it recalls
`monad` — ranked, with evidence.

## Install

```sh
# The binary (Lonis provider surface + CLI):
cargo install --path karpal-discovery --features lonis --bin karpal

# The library (lonis-free):
cargo add karpal-discovery
```

This is a `std`-only crate (it walks the filesystem), so unlike the rest of
Karpal it does not target `no_std`.

## Library quickstart

```rust
use karpal_discovery::{extract_workspace, load_concept_overlay, analyze_imports, recommend};

// Structural catalog: crates, modules, public items, re-exports, impl graph.
let catalog = extract_workspace(std::path::Path::new("."));

// Curated overlay: 83 concepts, problem shapes, relationships.
let overlay = load_concept_overlay();
overlay.validate(&catalog).expect("no drift"); // the CI drift gate

// What does this project use? (resolved imports → concepts)
let report = analyze_imports(std::path::Path::new("."), &catalog);
for concept in report.concepts_used(&overlay) {
    println!("in use: {}", concept.id);
}

// Ask for the problem, get ranked concepts.
let rec = recommend("sequence dependent effectful steps", &overlay);
```

Also: `inspect_workspace` → `ProjectSnapshot` (Cargo metadata, dependency
source discrimination, features, targets, platform constraints, resolved
deps — read-only, no `cargo` spawn) and `probe_catalog()`/`run_probe()`
(algebraic probes).

## The `karpal` binary

A conforming Lonis `SubprocessProvider` (ADR-0006 v0):
`karpal --mode json manifest | tools list | tools describe | call` —
JSON on stdin, blocks on stdout, structured `ToolError` on stderr. In-process
hosts use `lonis_core::SubprocessProvider`. Every tool is deterministic,
read-only, low-cost.

| Tool | Answers |
|---|---|
| `karpal.search` | which public items match a name? |
| `karpal.detail` | one item: docs, implementors, anchored concepts |
| `karpal.concepts` | browse curated concepts (ids, aliases, problem shapes) |
| `karpal.imports` | which symbols (and concepts) does a workspace use? |
| `karpal.recommend` | recall + Pareto-rank concepts for a goal |
| `karpal.plan` | orient / explore (top-3) / verify plan for a goal |
| `karpal.probe_list` | the registered probes |
| `karpal.probe_describe` | what one probe demonstrates and dogfoods |
| `karpal.probe_run` | run one probe, report each check |

Legacy invocations keep working: `karpal --index-compat search|detail|crates|hierarchy [--json]`
emits `karpal-index`'s JSON shapes over the new catalog.

## Architecture

The analysis substrate is **lonis-independent**; only the output layer (`Block`
wrapping and the binary) is gated on the optional `lonis` feature — crates.io
registry deps since Lonis 0.1.0, so the crate and binary are publishable.

- **Catalog (19-A)** — `extract_workspace` → `Catalog`: deterministic,
  content-hashable; public items of every kind (macros under their importable
  names — the derive name, never the proc-macro fn name), `pub use` re-exports
  with renames, and the trait implementation graph.
- **Overlay (19-B)** — `load_concept_overlay` → `ConceptOverlay`: 83 curated
  concepts (problem shapes, `generalizes`/`composes_with`/`alternative_to`/
  `dual_of` relations, stability/cost tiers), embedded via `include_str!` and
  validated against the catalog — references to missing symbols are drift, and
  CI rejects them.
- **Inspector (19-C)** — `inspect_workspace` → `ProjectSnapshot`: read-only
  Cargo metadata (no `cargo` spawn).
- **Imports** — `analyze_imports` → `ImportsReport`: `use`-statement resolution
  (leaf-tolerant, re-export-aware); unresolved catalog-crate imports are the
  drift signal. Pointing it at Karpal itself surfaced and fixed three catalog
  gaps.
- **Planner (19-F)** — `recommend` + `plan`: **dogfoods Karpal's own
  typeclasses at runtime** — `karpal-core` `Semigroup`/`Monoid` score
  aggregation, `karpal-algebra` `BoundedLattice` Pareto ranking (strict
  dominance *is* lattice join), `karpal-free` `Free`-monad plan construction
  with catamorphic normalization.
- **Probes (19-G)** — five bounded, read-only, deterministic probes running
  real library code: functor/monad laws (`karpal-core`), law checkers
  (`karpal-proof`), Schubert intersections — the structured-emptiness thesis
  live (`karpal-schubert-types`), recursion-scheme agreement
  (`karpal-recursion`), coherence witnesses (`karpal-diagram`).
- **Hardening (19-H)** — output-contract golden tests pin the wire surface
  (regeneration is deliberate: `KARPAL_UPDATE_GOLDENS=1`); the `--index-compat`
  migration mode; a publish-order drift gate over `publish.yml`.

Honesty note: the topos crate's Yoneda machinery is type-level; the runtime
capability category is the overlay's relation graph. A deeper Yoneda recall
story is deferred until `karpal-topos` is battle-tested (1.0 candidate).

## Documentation

The mdBook covers the runtime in both languages: [Discovery with
`karpal`](../book/src/guide/karpal-discovery.md) (guide) and [Discovery
Runtime](../book/src/reference/discovery.md) (reference), each with a Japanese
counterpart.

## Status

Debuts in 0.9.0. `karpal-index` remains published for compatibility;
deprecation follows consumer migration.

## License

Apache-2.0. See [LICENSE](../LICENSE) for details.
