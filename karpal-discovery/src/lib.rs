// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! # karpal-discovery
//!
//! Structural catalog generator for the Karpal workspace — the domain
//! foundation of the Karpal discovery vertical.
//!
//! `karpal-discovery` walks a Karpal workspace checkout with a real `syn`
//! AST parser and produces a typed, deterministic, content-hashable
//! [`Catalog`] of the public API surface: workspace crates, their public
//! modules, and their public items — traits, free functions, structs, enums,
//! type aliases, and the trait implementation graph. A companion project
//! inspector ([`inspect_workspace`]) parses `Cargo.toml` manifests into a
//! [`ProjectSnapshot`] (workspace meta, package metadata, dependencies with
//! source discrimination, features, targets (explicit plus conventionally
//! auto-discovered), inferred platform constraints (the no_std linkage mode),
//! and resolved dependencies from `Cargo.lock`. A curated concept overlay
//! ([`load_concept_overlay`]) layers mathematical concept names and
//! relationships over the catalog (embedded, validated against it).
//! Imported-symbol analysis ([`analyze_imports`]) then resolves a target
//! project's `use` statements against the catalog — which symbols (and hence
//! which curated concepts, via [`ImportsReport::concepts_used`]) a project
//! actually consumes. With the `lonis` feature, the `karpal` binary is a
//! conforming `SubprocessProvider` hosting `karpal.search` / `karpal.detail`
//! / `karpal.concepts` / `karpal.imports`. Later slices add the
//! category-theoretic planner, algebraic probes, and hardening.
//!
//! ## Architecture note
//!
//! This crate is the **second Lonis vertical** (`amari-discovery` is the
//! reference impl; Lonis is the horizontal harness carrying the general
//! `Block` contract per ANIMA_ECOSYSTEM_DOCTRINE §2.7). The catalog produced
//! here is intermediate domain data — input to discovery — and carries **no
//! dependency on the lonis `Block` contract**. The `Block` wrapping applies
//! only at the *output* layer (search results, recommendations, plans, probe
//! results), which is gated on Lonis and not built here.
//!
//! ## Features
//!
//! This is a `std`-only crate: it walks the filesystem, so it does not target
//! `no_std`. It is the one Karpal crate without a `no_std` gate.
//!
//! [`Block`]: https://github.com/Industrial-Algebra/Lonis

#![forbid(unsafe_code)]

pub mod catalog;
pub mod extract;
pub mod imports;
pub mod inspect;
pub mod overlay;
#[cfg(feature = "lonis")]
pub mod payload;

pub use catalog::{
    Catalog, CrateRecord, EnumRecord, FunctionRecord, ImplRecord, ItemKind, ItemRecord,
    MacroFlavor, MacroRecord, MethodRecord, ModuleRecord, StructRecord, TraitRecord,
    TypeAliasRecord,
};
pub use imports::{GlobImport, ImportsReport, ResolvedImport, UnresolvedImport, analyze_imports};
pub use inspect::{
    CrateSnapshot, DepKind, DepSource, LibTarget, LockfileSnapshot, NamedTarget, PackageMeta,
    PlatformConstraints, ProjectSnapshot, ResolvedPackage, StdMode, TargetsRecord, WorkspaceMeta,
    inspect_workspace,
};
pub use overlay::{
    ConceptOverlay, ConceptRecord, ConceptRelation, CostHint, OverlayDrift, RelationKind,
    StabilityTier, load_concept_overlay,
};
