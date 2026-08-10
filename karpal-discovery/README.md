# karpal-discovery

Structural catalog generator for the Karpal workspace — the domain foundation
of the Karpal discovery vertical.

`karpal-discovery` walks a Karpal workspace checkout with a real `syn` AST
parser and produces a typed, deterministic, content-hashable catalog of the
public API surface: workspace crates, their public modules, and their public
items — traits, free functions, structs, enums, and type aliases. It is the
successor to `karpal-index`'s string-scanning indexer.

## Architecture

This crate is the **second Lonis vertical** (`amari-discovery` is the reference
implementation; [Lonis](https://github.com/Industrial-Algebra/Lonis) is the
horizontal harness carrying the general `Block` contract per the Anima Ecosystem
Doctrine §2.7). The catalog produced here is intermediate domain data — input
to discovery — and carries **no dependency on the lonis `Block` contract**. The
`Block` wrapping applies only at the *output* layer (search results,
recommendations, plans, probe results), which is gated on Lonis and built in
later slices.

This slice catalogues **public traits, functions, structs, enums, and type
aliases** (plus crate metadata and modules). Macros, the full impl-graph,
curated semantic overlays, project inspection, the category-theoretic planner,
algebraic probes, and the command surface arrive in later slices.

## Status

Worked example / domain prep. Not yet published; the published discovery binary
remains `karpal-index` (0.8.0). This is a `std`-only crate (it walks the
filesystem), so unlike the rest of Karpal it does not target `no_std`.

## License

Apache-2.0. See [LICENSE](../LICENSE) for details.
