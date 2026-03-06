use crate::arrow::Arrow;

/// ArrowChoice: an Arrow that can route through sum types.
///
/// Uses `Result<L, R>` as the sum type, consistent with karpal-profunctor's Choice.
///
/// Laws:
/// - left(arr(f)) == arr(|r| r.map(f))
/// - left(compose(f, g)) == compose(left(f), left(g))
/// - compose(arr(Ok), f) == compose(left(f), arr(Ok))
pub trait ArrowChoice: Arrow {
    /// Route the Ok branch through the arrow, passing Err through.
    fn left<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<Result<A, C>, Result<B, C>>;

    /// Route the Err branch through the arrow, passing Ok through.
    fn right<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Self::P<A, B>,
    ) -> Self::P<Result<C, A>, Result<C, B>> {
        let mirror_in = Self::arr(|r: Result<C, A>| match r {
            Ok(c) => Err(c),
            Err(a) => Ok(a),
        });
        let mirror_out = Self::arr(|r: Result<B, C>| match r {
            Ok(b) => Err(b),
            Err(c) => Ok(c),
        });
        Self::compose(mirror_out, Self::compose(Self::left(pab), mirror_in))
    }

    /// `+++`: apply f on Ok, g on Err.
    fn splat<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static, D: Clone + 'static>(
        f: Self::P<A, B>,
        g: Self::P<C, D>,
    ) -> Self::P<Result<A, C>, Result<B, D>> {
        Self::compose(Self::right(g), Self::left(f))
    }

    /// `|||`: merge two arrows, one for each branch of Result.
    fn fanin<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Self::P<A, C>,
        g: Self::P<B, C>,
    ) -> Self::P<Result<A, B>, C> {
        let merge = Self::arr(|r: Result<C, C>| match r {
            Ok(c) | Err(c) => c,
        });
        Self::compose(merge, Self::splat(f, g))
    }
}
