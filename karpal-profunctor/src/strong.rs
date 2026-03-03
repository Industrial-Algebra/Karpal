use crate::profunctor::Profunctor;

/// A `Strong` profunctor can lift a `P<A, B>` into `P<(A, C), (B, C)>`.
///
/// This is the key ingredient for profunctor-encoded lenses.
pub trait Strong: Profunctor {
    fn first<A, B, C>(pab: Self::P<A, B>) -> Self::P<(A, C), (B, C)>
    where
        A: 'static,
        B: 'static,
        C: 'static;

    fn second<A, B, C>(pab: Self::P<A, B>) -> Self::P<(C, A), (C, B)>
    where
        A: 'static,
        B: 'static,
        C: 'static;
}
