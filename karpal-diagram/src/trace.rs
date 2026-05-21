#[cfg(any(feature = "std", feature = "alloc"))]
use karpal_arrow::{ArrowLoop, FnA};

use crate::tensor::Tensor;

/// Traced monoidal structure for closing a feedback wire.
///
/// `trace` consumes a morphism `(A, D) -> (B, D)` and hides the feedback
/// object `D`, yielding a morphism `A -> B`. In Rust's strict evaluation model
/// this first executable slice follows `ArrowLoop`: `D: Default` supplies the
/// initial feedback seed for single-pass execution.
pub trait Trace: Tensor {
    fn trace<A: Clone + 'static, B: Clone + 'static, D: Default + Clone + 'static>(
        morphism: Self::P<(A, D), (B, D)>,
    ) -> Self::P<A, B>;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Trace for FnA {
    fn trace<A: Clone + 'static, B: Clone + 'static, D: Default + Clone + 'static>(
        morphism: Self::P<(A, D), (B, D)>,
    ) -> Self::P<A, B> {
        Self::loop_arrow(morphism)
    }
}
