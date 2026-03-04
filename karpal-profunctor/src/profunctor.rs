pub use karpal_core::hkt::HKT2;

/// A profunctor is contravariant in its first argument and covariant in its second.
///
/// Laws:
/// - Identity: `dimap(id, id, p) == p`
/// - Composition: `dimap(f . g, h . i, p) == dimap(g, h, dimap(f, i, p))`
pub trait Profunctor: HKT2 {
    fn dimap<A: 'static, B: 'static, C, D>(
        f: impl Fn(C) -> A + 'static,
        g: impl Fn(B) -> D + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<C, D>;

    fn lmap<A: 'static, B: 'static, C>(
        f: impl Fn(C) -> A + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<C, B> {
        Self::dimap(f, |b| b, pab)
    }

    fn rmap<A: 'static, B: 'static, D>(
        g: impl Fn(B) -> D + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<A, D> {
        Self::dimap(|a| a, g, pab)
    }
}
