use karpal_core::hkt::HKT;

/// Monad transformer: lifts an inner monadic computation into the transformer stack.
///
/// Laws:
/// - `lift(pure(a))` == `pure(a)` (transformer's pure)
/// - `lift(chain(m, f))` == `chain(lift(m), |a| lift(f(a)))`
///
/// The `Clone` bound on `M::Of<A>` is required because some transformers
/// (ReaderT, StateT) wrap the lifted value in a closure that may be called
/// multiple times.
pub trait MonadTrans<M: HKT>: HKT {
    /// Lift an inner monadic value into the transformer.
    fn lift<A: 'static>(ma: M::Of<A>) -> Self::Of<A>
    where
        M::Of<A>: Clone;
}
