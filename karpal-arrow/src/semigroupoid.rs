use karpal_core::hkt::HKT2;

/// Semigroupoid: morphisms that can be composed.
///
/// Laws:
/// - Associativity: compose(f, compose(g, h)) == compose(compose(f, g), h)
pub trait Semigroupoid: HKT2 {
    fn compose<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Self::P<B, C>,
        g: Self::P<A, B>,
    ) -> Self::P<A, C>;
}
