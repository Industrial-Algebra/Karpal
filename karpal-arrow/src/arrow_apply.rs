use crate::arrow::Arrow;

/// ArrowApply: an Arrow that can apply arrows from within the computation.
///
/// Equivalent in power to Monad (ArrowApply ≅ Monad via Kleisli).
pub trait ArrowApply: Arrow {
    fn app<A: Clone + 'static, B: Clone + 'static>() -> Self::P<(Self::P<A, B>, A), B>;
}
