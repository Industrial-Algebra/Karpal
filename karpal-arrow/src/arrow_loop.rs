use crate::arrow::Arrow;

/// ArrowLoop: an Arrow with a loop/fixpoint combinator.
///
/// Takes an arrow from `(A, D)` to `(B, D)` and produces an arrow from `A` to `B`,
/// where `D` is the "feedback" type threaded through the loop.
///
/// In Haskell, `loop` relies on laziness to tie the knot. Rust is strict, so
/// `D: Default` provides the initial feedback seed and the implementation uses
/// single-pass evaluation.
pub trait ArrowLoop: Arrow {
    fn loop_arrow<A: Clone + 'static, B: Clone + 'static, D: Default + Clone + 'static>(
        f: Self::P<(A, D), (B, D)>,
    ) -> Self::P<A, B>;
}
