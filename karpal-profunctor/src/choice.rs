use crate::profunctor::Profunctor;

/// A `Choice` profunctor can lift a `P<A, B>` into `P<Result<A, C>, Result<B, C>>`.
///
/// This is the key ingredient for profunctor-encoded prisms.
/// Uses `Result<L, R>` as the sum type (idiomatic Rust, no custom `Either`).
pub trait Choice: Profunctor {
    fn left<A, B, C>(pab: Self::P<A, B>) -> Self::P<Result<A, C>, Result<B, C>>
    where
        A: 'static,
        B: 'static,
        C: 'static;

    fn right<A, B, C>(pab: Self::P<A, B>) -> Self::P<Result<C, A>, Result<C, B>>
    where
        A: 'static,
        B: 'static,
        C: 'static;
}
