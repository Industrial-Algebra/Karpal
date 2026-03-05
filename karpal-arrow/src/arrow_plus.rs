use crate::arrow_zero::ArrowZero;

/// ArrowPlus: an ArrowZero with an associative choice operation.
///
/// Laws:
/// - Associativity: plus(plus(f, g), h) == plus(f, plus(g, h))
/// - Left identity:  plus(zero_arrow(), f) == f
/// - Right identity: plus(f, zero_arrow()) == f
pub trait ArrowPlus: ArrowZero {
    fn plus<A: Clone + 'static, B: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<A, B>,
    ) -> Self::P<A, B>;
}
