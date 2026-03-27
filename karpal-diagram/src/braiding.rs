#[cfg(any(feature = "std", feature = "alloc"))]
use karpal_arrow::FnA;

use crate::tensor::{AssocLeft, HexagonTarget, Tensor};

/// Braided monoidal categories admit a coherent swap of tensor factors.
pub trait Braiding: Tensor {
    /// Swap the two tensor factors.
    fn braid<A: Clone + 'static, B: Clone + 'static>() -> Self::P<(A, B), (B, A)> {
        Self::arr(|(a, b): (A, B)| (b, a))
    }

    /// A simple executable form of the hexagon law.
    fn hexagon_forward<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>()
    -> Self::P<AssocLeft<A, B, C>, HexagonTarget<A, B, C>> {
        Self::arr(|((a, b), c): AssocLeft<A, B, C>| (b, (c, a)))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Braiding for FnA {}

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests {
    use super::*;

    #[test]
    fn braid_swaps_tuple_positions() {
        let braid = FnA::braid::<i32, &'static str>();
        assert_eq!(braid((7, "left")), ("left", 7));
    }

    #[test]
    fn hexagon_forward_has_expected_shape() {
        let hex = FnA::hexagon_forward::<i32, bool, &'static str>();
        assert_eq!(hex(((1, true), "x")), (true, ("x", 1)));
    }
}
