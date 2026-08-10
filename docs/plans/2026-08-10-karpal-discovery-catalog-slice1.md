# karpal-discovery — Slice 1: Structural Catalog Generator

Date: 2026-08-10
Status: in progress (parallel domain prep)
Branch: `feature/discovery-catalog`
Depends on: nothing (Lonis-independent domain work)
Blocks: discovery commands, planner, probes (all gated on Lonis's `Block` contract)

## Context — why this slice, why now

`karpal-discovery` is the **second Lonis vertical** (amari-discovery is vertical #1,
the reference impl; Lonis is the horizontal harness carrying the general `Block`
contract per ANIMA_ECOSYSTEM_DOCTRINE §2.7). The full discovery runtime —
discover/recommend/plan/probe commands emitting structured output — is **gated on
Lonis** delivering its `Block` contract + `Tool`/`Capabilities` reconciliation
(Lonis handoff 2026-08-10, work items #1–#2). Neither exists yet.

This slice builds the **Lonis-independent domain foundation**: the structural
catalog generator. It produces a static index of the Karpal workspace that every
later layer consumes. It has **no dependency on the lonis `Block` contract** and
**minimal rework risk**: the catalog is intermediate data (input to discovery),
not tool output. The lonis `Block` wrapping applies only at the *output* layer
(search results, recommendations, plans, probe results), which this slice does
not touch.

Reference implementation: `amari/amari-discovery/src/catalog/` (11.4k LOC, 9
generator modules). This slice is a focused subset; amari's wasm surface
generator is dropped (no karpal-wasm exists).

## Reference contrast — what replaces what

`karpal-index` (published 0.8.0, an IA-MCP dependency) is a **string-scanning**
indexer: it walks source files and matches `.contains(" for ")`, counts braces,
etc. — despite declaring `syn` as a dependency, it does not parse the AST. Its
`ApiItem` is flat and loses structure (generic bounds, associated types, cfg
gates).

`karpal-discovery` replaces that with a **real syn AST walker** producing a typed,
deterministic, content-hashable structural catalog. `karpal-index` is left
untouched in this slice (stays published; migration is a later, Lonis-gated
phase).

## Scope — in / out

**In (this slice):**
- New `karpal-discovery` library crate (no binary yet).
- `Catalog` data model: workspace crates (name, version, description, features,
  dependencies), public modules, and **public traits** (name, module path,
  supertraits, associated items, method signatures, docs, cfg gate).
- A `syn`-based workspace extractor (`syn::parse_file` + `visit::Visit`) walking
  every crate's `src/` tree.
- Deterministic serialization (sorted maps/sets) + content hash.
- TDD: unit tests against fixture sources; integration test against the real
  Karpal workspace (asserts e.g. `Functor` exists with correct shape).

**Out (later slices):**
- Public functions, types/structs/enums, macros, full impl-graph, cfg-gate
  resolution beyond the item level.
- Curated semantic overlays (concept names, problem shapes, cost hints).
- Project inspection (Cargo/Rust `ProjectSnapshot`).
- The category-theoretic planner, algebraic probes.
- Any command surface, protocol, lonis `Block` integration, the `karpal` binary.

## Design decisions

1. **Library-only for now.** No `karpal` binary; that arrives with the lonis
   output-layer integration (when the `Block` contract exists). A
   `generate_catalog` example can exercise the extractor for development.
2. **Provisional typed model, not lonis Blocks.** The `Catalog` is a plain
   `serde::Serialize` struct. Rewiring individual *results* to lonis `Block`s
   later does not touch the catalog itself.
3. **Deterministic by construction.** All collections are `BTreeMap`/`BTreeSet`
   so serialized output is byte-stable and content-hashable (mirrors
   amari-discovery's catalog hashing discipline).
4. **Trait-first extraction.** Karpal is trait-centric (`Functor`, `Monad`,
   `Adjunction`, …). Traits are the highest-value item kind and establish the
   `visit::Visit` walker pattern that later kinds (functions, types) reuse.
5. **Workspace-inheritance Cargo.toml** (karpal-style): `version.workspace =
   true` etc.; deps inline (the workspace has no `[workspace.dependencies]`).

## TDD outline

- **Unit (fixture):** a tiny in-tree fixture crate with one trait (with a
  supertrait, an associated type, two methods, a doc comment) and one module.
  Assert the extractor yields the expected `TraitRecord` shape.
- **Unit (parsing edge cases):** empty file, file with only private items,
  trait with `where` clause and generics.
- **Integration (real workspace):** build the catalog over `..` (the Karpal
  workspace root from the crate), assert `Functor` is present in `karpal-core`
  with `fmap` among its methods, and that crate count > 0. Kept hermetic by
  asserting on stable, well-known symbols only.

## Verification

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test  -p karpal-discovery --all-features
cargo doc   -p karpal-discovery --no-deps
```

(No `no_std` gate: discovery inherently walks the filesystem, so it is a `std`
crate. This is the one Karpal crate that does not target `no_std`.)

## Open questions deferred

- Whether the catalog should be generated at build time into a checked-in asset
  (amari-discovery checks in `catalog/generated.json`). Deferred to the
  hardening slice; this slice produces the generator, not the checked-in asset.
- Stable capability-ID shape (`karpal:<crate>:<module>:<symbol>`) — deferred to
  the protocol/output slice (lonis-gated). The catalog stores module paths
  suffixed for that later decision.
