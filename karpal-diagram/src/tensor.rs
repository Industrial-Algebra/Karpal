use karpal_arrow::Arrow;
#[cfg(any(feature = "std", feature = "alloc"))]
use karpal_arrow::FnA;

pub type AssocLeft<A, B, C> = ((A, B), C);
pub type AssocRight<A, B, C> = (A, (B, C));
pub type HexagonTarget<A, B, C> = (B, (C, A));

/// Monoidal structure for categories whose objects can be tensored in parallel.
///
/// In this initial encoding, the tensor product is modeled with Rust tuples.
pub trait Tensor: Arrow {
    /// Tensor two morphisms in parallel.
    fn tensor<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static, D: Clone + 'static>(
        left: Self::P<A, B>,
        right: Self::P<C, D>,
    ) -> Self::P<(A, C), (B, D)>;

    /// The left-associated product `((a, b), c) -> (a, (b, c))`.
    fn associate<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>()
    -> Self::P<AssocLeft<A, B, C>, AssocRight<A, B, C>> {
        Self::arr(|((a, b), c): AssocLeft<A, B, C>| (a, (b, c)))
    }

    /// The inverse associator `(a, (b, c)) -> ((a, b), c)`.
    fn associate_inv<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>()
    -> Self::P<AssocRight<A, B, C>, AssocLeft<A, B, C>> {
        Self::arr(|(a, (b, c)): AssocRight<A, B, C>| ((a, b), c))
    }

    /// Left unitor `((), a) -> a`.
    fn left_unitor<A: Clone + 'static>() -> Self::P<((), A), A> {
        Self::arr(|(_, a): ((), A)| a)
    }

    /// Inverse left unitor `a -> ((), a)`.
    fn left_unitor_inv<A: Clone + 'static>() -> Self::P<A, ((), A)> {
        Self::arr(|a: A| ((), a))
    }

    /// Right unitor `(a, ()) -> a`.
    fn right_unitor<A: Clone + 'static>() -> Self::P<(A, ()), A> {
        Self::arr(|(a, _): (A, ())| a)
    }

    /// Inverse right unitor `a -> (a, ())`.
    fn right_unitor_inv<A: Clone + 'static>() -> Self::P<A, (A, ())> {
        Self::arr(|a: A| (a, ()))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Tensor for FnA {
    fn tensor<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static, D: Clone + 'static>(
        left: Self::P<A, B>,
        right: Self::P<C, D>,
    ) -> Self::P<(A, C), (B, D)> {
        Self::split(left, right)
    }
}

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests {
    use super::*;
    use karpal_arrow::Semigroupoid;

    #[test]
    fn fna_tensor_runs_both_sides_in_parallel() {
        let left = FnA::arr(|n: i32| n * 2);
        let right = FnA::arr(|flag: bool| !flag);
        let combined = FnA::tensor(left, right);

        assert_eq!(combined((3, true)), (6, false));
    }

    #[test]
    fn associator_round_trips() {
        let assoc = FnA::associate::<i32, bool, &'static str>();
        let assoc_inv = FnA::associate_inv::<i32, bool, &'static str>();
        let round_trip = FnA::compose(assoc_inv, assoc);

        assert_eq!(round_trip(((4, true), "x")), ((4, true), "x"));
    }

    #[test]
    fn unitors_round_trip() {
        let left = FnA::compose(FnA::left_unitor::<i32>(), FnA::left_unitor_inv::<i32>());
        let right = FnA::compose(FnA::right_unitor::<i32>(), FnA::right_unitor_inv::<i32>());

        assert_eq!(left(5), 5);
        assert_eq!(right(5), 5);
    }
}
