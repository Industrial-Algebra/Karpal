// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::arrow::Arrow;

/// ArrowLoop: an Arrow with a loop/fixpoint combinator.
///
/// Takes an arrow from `(A, D)` to `(B, D)` and produces an arrow from `A` to `B`,
/// where `D` is the "feedback" type threaded through the loop.
///
/// # Strict evaluation semantics
///
/// In Haskell, `loop` relies on laziness to tie the knot: the output `D` is
/// fed back as the input `D` through a recursive `let` binding. Rust is
/// strict, so [`loop_arrow`](ArrowLoop::loop_arrow) uses **single-pass
/// evaluation**: `D: Default` provides the initial feedback seed, the arrow
/// runs once, and the output `D` is discarded.
///
/// This is correct for use cases where:
/// - The feedback does not affect the output (e.g., tracing, logging)
/// - `D::default()` is the correct initial state
/// - Only one iteration is needed
///
/// For **iterative convergence** (true fixpoint computation), use
/// [`loop_fixpoint`], which requires `D: PartialEq` and iterates until the
/// feedback stabilizes.
///
/// # Law (single-pass)
///
/// `loop_arrow(f)(a) == f((a, D::default())).0`
///
/// This is NOT the same as Haskell's `loop` law, which requires laziness.
/// The categorical `loop` law holds only under the fixpoint interpretation
/// (see [`loop_fixpoint`]).
///
/// [`loop_fixpoint`]: crate::fn_arrow::loop_fixpoint
pub trait ArrowLoop: Arrow {
    fn loop_arrow<A: Clone + 'static, B: Clone + 'static, D: Default + Clone + 'static>(
        f: Self::P<(A, D), (B, D)>,
    ) -> Self::P<A, B>;
}
