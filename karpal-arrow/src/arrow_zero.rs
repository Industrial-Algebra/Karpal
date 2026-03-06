use crate::arrow::Arrow;

/// ArrowZero: an Arrow with a zero (failing/empty) morphism.
///
/// Laws:
/// - compose(zero_arrow(), f) == zero_arrow()  (left absorption)
pub trait ArrowZero: Arrow {
    fn zero_arrow<A: Clone + 'static, B: Clone + 'static>() -> Self::P<A, B>;
}
