# Discovery Runtime Reference

The `karpal-discovery` crate (Phase 19) is the agent-first discovery runtime: a typed structural catalog, a curated semantic overlay, project inspection, imported-symbol analysis, a category-theoretic planner, and algebraic probes — with the `karpal` binary as a conforming [Lonis](https://github.com/Industrial-Algebra/Lonis) `SubprocessProvider` (see the [guide](../guide/karpal-discovery.md) for CLI usage).

It is the **second Lonis vertical** (`amari-discovery` is the first). Where Amari's discovery dogfoods holographic recall and tropical ranking, Karpal's dogfoods **Karpal's own typeclasses** — the planner's score aggregation *is* `Semigroup`/`Monoid`, its ranking *is* a `BoundedLattice`, its plans *are* `Free` monads. If the library cannot power its own discovery, it cannot credibly power anyone else's.

## Architecture

```
        ┌────────────────────────── karpal-discovery (std-only) ─────────────────────────┐
        │                                                                                │
  extract.rs ─► Catalog ──┬──► overlay.rs   ConceptOverlay (83 curated concepts)        │
  (syn AST walk)          │   (include_str! + drift-gated)                              │
        │                 ├──► imports.rs    ImportsReport (use-statement analysis)     │
  inspect.rs ─► ProjectSnapshot (Cargo.toml, no cargo spawn)                            │
        │                 ├──► planner.rs    recommend() / plan() — dogfoods            │
        │                 │        karpal-core, karpal-algebra, karpal-free             │
  probes.rs   ────────────┴──► probe_catalog() / run_probe() — dogfoods                 │
                 karpal-proof, karpal-recursion, karpal-diagram, karpal-schubert-types  │
        └────────────────────────────────────────────────────────────────────────────────┘
                                    │  (lonis feature, optional)
                            payload.rs → Block<karpal.*> → main.rs: the karpal binary,
                            a SubprocessProvider with nine tools
```

The analysis substrate is **lonis-independent**: catalog, overlay, inspector, imports, planner, and probes are plain library code. Only the output layer (`Block` wrapping, the wire protocol, the binary) is gated on the `lonis` feature — which since Lonis 0.1.0 is a set of crates.io registry dependencies, so the crate and binary are publishable.

## The Structural Catalog (19-A)

`extract_workspace(root) -> Catalog` walks a workspace with a real `syn` parse and records, deterministically and content-hashably:

- **Crates**: name, version, description, features, dependencies, modules.
- **Public items**: traits (supertraits, methods, associated items), functions (signatures), structs, enums, type aliases, macros — declarative `macro_rules!` under `#[macro_export]`, and procedural macros catalogued under their *importable* names (a `#[proc_macro_derive(VerifySemigroup)]` fn is catalogued as `VerifySemigroup`; the fn name never leaves the proc-macro crate).
- **Re-exports**: `pub use` leaves including renames (`MonteCarloVerifier as AmariMonteCarloVerifier`), with globs and `std`/`core`/`alloc` origins skipped.
- **The implementation graph**: `Catalog::implementors_of("Functor")` → the types implementing it, workspace-wide.

The catalog's items carry qualified module paths and doc comments; `Catalog::find_item` and `Catalog::item_count` cover simple queries.

## The Concept Overlay (19-B)

`load_concept_overlay() -> ConceptOverlay` deserializes a **checked-in, hand-curated TOML** embedded via `include_str!` — a crates.io install needs no source checkout. Each `ConceptRecord` carries:

- an `id`, display `name`, `summary`, and search `aliases`;
- **`problem_shapes`** — problems phrased the way a user or agent would state them ("sequence dependent effectful steps", "patch locally-consistent data into a global section");
- `math_concepts`, qualified `symbol_refs` (`karpal-core::Functor`), a `StabilityTier`, and a `CostHint`.

Concepts are joined by directed `ConceptRelation`s: `generalizes` (mirroring verified trait supertraits), `composes_with`, `alternative_to` (e.g. `arrow-apply` ≅ `monad`), and `dual_of` (`comonad` ↔ `monad`, the contravariant hierarchy).

**The drift gate** — `ConceptOverlay::validate(&catalog)` — is the overlay's defining guarantee: every `symbol_ref` must resolve to a real catalog item, every relation endpoint must be a known id, no concept may float unanchored, and ids must be unique. A CI test validates the embedded overlay against the live workspace, so curation cannot outrun the code it describes. During curation this gate caught a genuinely nonexistent symbol (`RanF`) and a dropped relation — it earns its keep.

## The Project Inspector (19-C)

`inspect_workspace(root) -> ProjectSnapshot` is a read-only TOML/TOML-lock parse (no `cargo` spawn, no mutation): workspace metadata (members, resolver), per-crate package metadata, dependencies with **source discrimination** (`Registry`/`Path`/`Git`/`Workspace`), features, targets (explicit plus conventional auto-discovery of bins/examples/tests/benches), inferred platform constraints (the `no_std` linkage mode), and resolved dependencies from `Cargo.lock`. A `content_hash` covers the whole snapshot.

## Imported-Symbol Analysis (19-B/C)

`analyze_imports(root, &catalog) -> ImportsReport` parses a target project's `use` statements and resolves them against the catalog:

- **`resolved`** — qualified symbol refs with item kind, local names (aliases included), per-file spread, occurrence counts. Resolution is leaf-tolerant (`use karpal_core::functor::Functor` matches `Functor`) and re-export-aware (intra-crate renames, cross-crate re-exports, and external-origin re-exports resolved at the re-export site).
- **`unresolved`** — imports naming a catalog crate but no item of it: **the drift signal**. Pointing this at Karpal's own workspace surfaced three real catalog gaps (consts, derive names, re-exports), all fixed.
- **`globs`** — recorded by path, never expanded.

`ImportsReport::concepts_used(&overlay)` joins the resolved symbols to the curated concepts — the answer to "what is this project doing, categorically?"

## The Planner (19-F)

`recommend(goal, &overlay)` and `plan(goal, &recommendation)` power discovery with Karpal's own abstractions, at runtime:

| Stage | Substrate (real trait usage) |
|---|---|
| Score aggregation | `karpal-core` `Semigroup`/`Monoid` — evidence accumulates through `Monoid::combine`; exact id/name matches outrank substrings |
| Pareto ranking | `karpal-algebra` `Lattice`/`BoundedLattice` — strict dominance *is* lattice join (`a ⊔ c = a ∧ a ≠ c`); ranking is dominator-count then relevance, weight, id |
| Plan construction | `karpal-free` `Free` monad — plans are `Free<PlanF, ()>` built with `lift_f` + `chain`, consumed by a structural catamorphism; adjacent duplicate steps collapse |

Recall seeds from direct text matches across all curated fields, then expands along the relation graph one hop in both directions; every entry carries its evidence (`match: problem shape`, `relation: dual_of comonad ↔ monad`). A goal phrased as a *problem* — "sequence dependent effectful steps" — recalls `monad` and plans around it.

**An honesty note on the topos:** `karpal-topos`'s `SmallCategory`/`Presheaf`/Yoneda machinery is type-level (GATs over static types); forcing 83 runtime concepts through it would be decorative. The runtime capability category *is* the overlay's relation graph. A deeper Yoneda-based recall story is deferred until the topos crate is battle-tested — candidate 1.0 material.

## The Probes (19-G)

`probe_catalog()` and `run_probe(id)` — bounded, read-only, deterministic executions that dogfood the library:

| Probe | Dogfoods | Demonstrates |
|---|---|---|
| `functor-monad-laws` | `karpal-core` | functor identity/composition + monad identities/associativity on `Option` |
| `algebra-laws` | `karpal-proof`, `karpal-core`, `karpal-algebra` | the law checkers: associativity, identities, absorption on the planner's own `Score` lattice |
| `schubert-intersection` | `karpal-schubert-types` | `IntersectionKind` discrimination in Gr(2,4): σ₁·σ₁ **Positive** vs σ₂₂·σ₂₂ **StructuralZero** — [structured emptiness](../concepts/structured-emptiness.md), live |
| `recursion-eval` | `karpal-recursion` | `ana` builds Peano, `cata` tears it down, `hylo ≡ cata ∘ ana` |
| `coherence` | `karpal-diagram` | pentagon / triangle / hexagon `Rewrite` witnesses |

## Wire Contracts and Hardening (19-H)

- **Golden tests** pin the binary's output surface: provider JSON byte-for-byte (it is static), block payloads on their `data` with the volatile provenance timestamp normalized. Regeneration is deliberate (`KARPAL_UPDATE_GOLDENS=1`); a golden change is a contract change.
- **`--index-compat`** speaks the legacy `karpal-index` JSON shapes over the new catalog (see the [guide](../guide/karpal-discovery.md)).
- **A publish-order drift gate** asserts every workspace member appears in `publish.yml`'s publish sequence.

## Library Quickstart

```rust
use karpal_discovery::{extract_workspace, load_concept_overlay, analyze_imports, recommend, plan};

// Structural catalog
let catalog = extract_workspace(std::path::Path::new("."));

// Curated concepts — validated against the catalog (drift gate)
let overlay = load_concept_overlay();
overlay.validate(&catalog).expect("no drift");

// Which concepts does this project use?
let report = analyze_imports(std::path::Path::new("."), &catalog);
for concept in report.concepts_used(&overlay) {
    println!("in use: {} ({})", concept.id, concept.summary);
}

// Planner: ask for the problem, get ranked concepts and a plan
let recommendation = recommend("sequence dependent effectful steps", &overlay);
let plan = plan("sequence dependent effectful steps", &recommendation);
```

Everything here is read-only, deterministic, and offline — no `cargo` spawns, no network, no mutation.
