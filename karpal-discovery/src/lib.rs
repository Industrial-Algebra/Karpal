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
//! modules, and their public traits. Later slices add functions, types,
//! macros, curated semantic overlays, and project inspection; the
//! category-theoretic planner, algebraic probes, and command surface arrive
//! with Lonis [`Block`] integration.
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

pub use catalog::{
    Catalog, CrateRecord, ItemKind, ItemRecord, MethodRecord, ModuleRecord, TraitRecord,
};
