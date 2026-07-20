// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Topos-theoretic constructions for the Industrial Algebra ecosystem.
//!
//! This crate builds the categorical infrastructure underlying structured
//! emptiness: small categories, presheaves (contravariant functors
//! `C^op → Set`), representable presheaves, sieves, and the Yoneda lemma.
//!
//! # Phase 16B (this release)
//!
//! - [`SmallCategory`] — small categories with morphisms-as-data
//! - [`Presheaf`] — contravariant functors `C^op → Set`
//! - [`Representable`] — the hom-presheaf `Hom_C(-, c)`
//! - [`Sieve`] / [`FiniteSieve`] — precomposition-closed families of morphisms
//! - Yoneda lemma as a computable bijection
//!
//! Future sub-phases (16C, 16D) will add the subobject classifier, finite
//! limits, Grothendieck topologies, and sheaves.
//!
//! # Encoding notes
//!
//! Objects of a small category are phantom marker types; morphisms `Mor<A, B>`
//! are values carrying runtime data. Direction is type-level (the `A`, `B`
//! phantom parameters) *and* runtime (the indices inside each concrete
//! morphism). Identity morphisms are constructed by concrete categories via
//! inherent methods, because Rust cannot extract object identity generically
//! from phantom types — see the design doc for the full rationale.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod presheaf;
pub mod representable;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod sieve;
pub mod small_category;
pub mod yoneda;

pub use presheaf::{ConstantPresheaf, Presheaf};
pub use representable::Representable;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use sieve::{FiniteSieve, Sieve};
pub use small_category::{ChainCat, ChainMor, ChainObj, DiscreteCat, SmallCategory};
pub use yoneda::{yoneda_apply, yoneda_extract};
