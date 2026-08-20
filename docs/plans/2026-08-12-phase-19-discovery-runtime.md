# Phase 19 — `karpal-discovery`: the Agent-First Discovery Runtime

**Date:** 2026-08-10 → 2026-08-12 · **Status:** complete (PRs #132–#155) ·
**Target:** 0.9.0

The design record for Karpal's second Lonis vertical. This document
consolidates the slice-level decisions; each slice's PR carries its detail.

## Motivation

`karpal-index` answers "which items exist?" by string-scanning. Agents ask
different questions: *"I need to sequence dependent effectful steps"* (a
problem shape, not a name), *"what is this project doing, categorically?"*
(a join between usage and meaning), *"prove the library works"* (execution,
not assertion). Phase 19 builds the runtime that answers them — and, per the
dogfooding thesis, powers the answers with Karpal's own abstractions: *if the
library cannot power its own discovery, it cannot credibly power anyone
else's.*

## Architecture decision — the Lonis seam

The analysis substrate (catalog, overlay, inspector, imports, planner,
probes) is **lonis-independent** library code. Only the output layer — the
`KarpalPayload`/`Block` wrapping and the `karpal` binary — is gated on the
optional `lonis` feature. This kept nine of eleven slices mergeable while
Lonis's contract stabilized, and made the gated swap (git pins at rev
`4aa23a6` → crates.io `lonis-schema`/`lonis-core` 0.1, PR #148) a two-line
change with zero API breakage.

The binary speaks the ADR-0006 v0 provider surface (`--mode json manifest |
tools list | tools describe | call`), so any Lonis host — including the
`SubprocessProvider` from `lonis-core` 0.1 — adopts it without bespoke glue.
All provider tests drive the real binary through the real host adapter.

## Slices and what they settled

| Slice | PRs | Delivered | Decision settled |
|---|---|---|---|
| A — structural catalog | #132, #137, #143, #150 | `extract_workspace` → typed, deterministic, content-hashable `Catalog` (all item kinds, re-exports, impl graph) | proc-macros catalog under *importable* names (the derive name, never the fn name); re-exports recorded with renames |
| B — semantic overlay | #146, #147 | 83-concept curated overlay (problem shapes, `generalizes`/`composes_with`/`alternative_to`/`dual_of`), embedded via `include_str!` | drift gate = validation against the live catalog in CI; curation is data, not code |
| C — project inspector | #144, #145 | `inspect_workspace` → `ProjectSnapshot` (dep source discrimination, targets, no_std mode, Cargo.lock) | read-only TOML parse; no `cargo` spawn, ever |
| imports (B/C deferred) | #149 | `analyze_imports` → resolved/unresolved/globs + `concepts_used` | leaf-tolerant, re-export-aware resolution; unresolved = drift signal |
| D/E — provider + commands | #150 | ADR-0006 v0 surface; 4 → 9 tools | one static tool registry as single source of truth; tool names = payload wire kinds (derive-generated, cannot drift) |
| F — planner | #151 | `recommend` + `plan` | dogfood the typeclasses *honestly* (below) |
| G — probes | #152 | 5 probes (laws, Schubert, recursion, coherence) | each probe runs real library code and reports demonstrated checks |
| H — hardening | #154/#155 | goldens, `--index-compat`, publish-order gate, CHANGELOG | byte-golden static JSON; data-golden blocks with normalized provenance |

## Honest dogfooding

The planner uses `karpal-core` `Semigroup`/`Monoid` (score aggregation),
`karpal-algebra` `Lattice`/`BoundedLattice` (Pareto dominance as lattice
join), and `karpal-free` `Free` (plan construction and catamorphic
normalization) — real trait usage, load-bearing.

The topos crate's `SmallCategory`/`Presheaf`/Yoneda machinery is type-level
(GATs over static types). Forcing 83 runtime concepts through it would be
decorative; the runtime capability category is the overlay's relation graph.
**Decision (Justin):** the deeper Yoneda-recall story is deferred until
`karpal-topos` is battle-tested — candidate 1.0 sugar.

## The drift discipline

Three independent gates emerged, each earning its keep on real catches:

1. **Overlay drift** — validation against the live workspace caught a
   nonexistent `RanF` symbol and a dropped relation during curation (#147).
2. **Import drift** — self-analysis flagged 26 unresolved imports, all three
   of which were genuine catalog blind spots (consts, derive names,
   re-exports), fixed in #150.
3. **Output drift** — golden tests pin the wire; **publish-order drift** — a
   test asserting every workspace member appears in `publish.yml`, which
   failed immediately on two missing crates.

## Testing posture

104 → 137 tests across the phase (default / `--features lonis`), including
real-workspace tests (the drift gates, self-analysis, the embedded-overlay
end-to-end: problem shape → ranked recall → normalized plan), provider tests
through the real `SubprocessProvider`, and goldens verified deterministic
across runs. fmt / clippy (`-D warnings`, both feature states) / doc / the
`no_std` gate stay green throughout.

## Migration

`karpal-index` remains published (added to the publish order). The `karpal`
binary's `--index-compat` mode emits its JSON shapes over the new catalog —
field-set parity with documented divergences (`path` has no `:line`;
`subtraits` was always empty). IA-MCP's manifest does not invoke
`karpal-index`, so the compat mode is a forward guarantee, not an emergency.

## Documentation

mdBook pages in both languages (`guide/karpal-discovery.md`,
`reference/discovery.md` + Japanese counterparts), the legacy guide carries a
successor banner, and this document consolidates the phase record.
