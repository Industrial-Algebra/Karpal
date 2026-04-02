#[cfg(any(feature = "std", feature = "alloc"))]
use karpal_arrow::FnA;

use crate::braiding::Braiding;

/// Symmetric monoidal categories have a braiding that is its own inverse.
pub trait Symmetry: Braiding {
    fn symmetry<A: Clone + 'static, B: Clone + 'static>() -> Self::P<(A, B), (A, B)> {
        Self::compose(Self::braid::<B, A>(), Self::braid::<A, B>())
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Symmetry for FnA {}

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests {
    use super::*;

    #[test]
    fn symmetry_is_involutive_for_fna() {
        let symmetry = FnA::symmetry::<i32, bool>();
        assert_eq!(symmetry((3, true)), (3, true));
    }
}
