// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

/// Marker trait for all optics.
///
/// This trait exists to unify the optic family under a single taxonomy.
/// Concrete optic types (Lens, Prism, etc.) implement this trait.
pub trait Optic {}
