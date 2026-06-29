// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Schubert intersection type system for the Industrial Algebra ecosystem.
//!
//! `karpal-schubert-types` (Phase 14) wraps `amari-enumerative`'s Schubert
//! calculus as type-level markers for use in Karpal's proof and verification
//! infrastructure.

pub mod intersection;
pub mod schubert_proven;
pub mod schubert_type;
pub mod schubert_typed;
pub mod verification;

pub use intersection::{Intersection, IntersectionKind, check_intersection};
pub use schubert_proven::{SchubertProven, compose_checks};
pub use schubert_type::SchubertType;
pub use schubert_typed::SchubertTyped;
pub use verification::{schubert_bundle, verify_schubert};
