// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::semigroupoid::Semigroupoid;

/// Category: a Semigroupoid with an identity morphism.
///
/// Laws:
/// - Left identity:  compose(id(), f) == f
/// - Right identity: compose(f, id()) == f
pub trait Category: Semigroupoid {
    fn id<A: Clone + 'static>() -> Self::P<A, A>;
}
