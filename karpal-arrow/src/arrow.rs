use crate::category::Category;

/// Arrow: a Category that can lift pure functions and operate on products.
///
/// Laws:
/// - arr(id) == id()
/// - arr(|a| g(f(a))) == compose(arr(g), arr(f))
/// - first(arr(f)) == arr(|(a, c)| (f(a), c))
/// - first(compose(f, g)) == compose(first(f), first(g))
pub trait Arrow: Category {
    /// Lift a pure function into an arrow.
    fn arr<A: Clone + 'static, B: Clone + 'static>(f: impl Fn(A) -> B + 'static) -> Self::P<A, B>;

    /// Apply an arrow to the first component of a pair, passing the second through.
    fn first<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<(A, C), (B, C)>;

    /// Apply an arrow to the second component of a pair.
    fn second<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<(C, A), (C, B)> {
        let swap_in = Self::arr(|(c, a): (C, A)| (a, c));
        let swap_out = Self::arr(|(b, c): (B, C)| (c, b));
        Self::compose(swap_out, Self::compose(Self::first(pab), swap_in))
    }

    /// `***`: apply two arrows in parallel on a product.
    fn split<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static, D: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<C, D>,
    ) -> Self::P<(A, C), (B, D)> {
        Self::compose(Self::second(g), Self::first(f))
    }

    /// `&&&`: feed input to two arrows and collect results as a pair.
    fn fanout<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<A, C>,
    ) -> Self::P<A, (B, C)> {
        let dup = Self::arr(move |a: A| {
            let a2 = a.clone();
            (a, a2)
        });
        Self::compose(Self::split(f, g), dup)
    }
}
