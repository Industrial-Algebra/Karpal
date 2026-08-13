# karpal-discovery

Structural catalog generator and project inspector for the Karpal workspace —
the domain foundation of the Karpal discovery vertical.

`karpal-discovery` walks a Karpal workspace checkout with a real `syn` AST
parser and produces a typed, deterministic, content-hashable catalog of the
public API surface: workspace crates, their public modules, public items
(traits, free functions, structs, enums, type aliases, macros), and the trait
implementation graph (queryable via `Catalog::implementors_of`). A companion
**project inspector** (`inspect_workspace`) parses `Cargo.toml` manifests
with a real TOML parser and produces a `ProjectSnapshot` — workspace meta,
per-crate package metadata, dependencies (with registry/path/git/workspace
source discrimination), feature flags, targets (explicit plus
conventionally auto-discovered bins/examples/tests/benches), inferred
platform constraints (the `no_std` linkage mode), and resolved dependencies
from `Cargo.lock`. Both are
read-only and carry no dependency on the lonis `Block` contract; together they
supersede `karpal-index`'s string-scanning indexer.

## Architecture

This crate is the **second Lonis vertical** (`amari-discovery` is the reference
implementation; [Lonis](https://github.com/Industrial-Algebra/Lonis) is the
horizontal harness carrying the general `Block` contract per the Anima Ecosystem
Doctrine §2.7). The catalog produced here is intermediate domain data — input
to discovery — and carries **no dependency on the lonis `Block` contract**. The
`Block` wrapping applies only at the *output* layer (search results,
recommendations, plans, probe results), which is gated on Lonis and built in
later slices.

This slice catalogues **public traits, functions, structs, enums, type
aliases, macros (declarative `macro_rules!` + the three procedural flavors),
and the trait implementation graph** (plus crate metadata and modules), and
adds the **project inspector** (`inspect_workspace` → `ProjectSnapshot`:
workspace meta, package metadata, dependencies with source discrimination,
features, explicit + conventionally auto-discovered targets, inferred
platform constraints (no_std mode), and resolved deps from `Cargo.lock` —
read-only,
no `cargo` spawn). Curated semantic
overlays, the category-theoretic planner, algebraic probes, and the command
surface arrive in later slices.

## Status

Worked example / domain prep. Not yet published; the published discovery binary
remains `karpal-index` (0.8.0). This is a `std`-only crate (it walks the
filesystem), so unlike the rest of Karpal it does not target `no_std`.

## License

Apache-2.0. See [LICENSE](../LICENSE) for details.
