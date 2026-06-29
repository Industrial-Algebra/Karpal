// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod bicategory;
pub mod coherence;
pub mod enriched;
pub mod ffunctor;
pub mod two_category;

pub use bicategory::Bicategory;
pub use coherence::{BicategoryPentagonIdentity, BicategoryTriangleIdentity, InterchangeIdentity};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use coherence::{
    HigherCoherenceCertificate, higher_coherence_certificates, verify_bicategory_pentagon,
    verify_bicategory_triangle, verify_interchange,
};
pub use enriched::EnrichedCategory;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use enriched::SetCategory;
pub use ffunctor::{FFunctor, FMonad, IdentityFFunctor};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use two_category::Cat;
pub use two_category::TwoCategory;
