use crate::hkt::{HKT2, ResultBF, TupleF};

/// Bifunctor: maps over both type parameters of a two-parameter type constructor.
///
/// Laws:
/// - Identity: `bimap(id, id, fab) == fab`
/// - Composition: `bimap(f . g, h . i, fab) == bimap(f, h, bimap(g, i, fab))`
pub trait Bifunctor: HKT2 {
    fn bimap<A, B, C, D>(
        fab: Self::P<A, B>,
        f: impl Fn(A) -> C,
        g: impl Fn(B) -> D,
    ) -> Self::P<C, D>;

    fn first<A, B, C>(fab: Self::P<A, B>, f: impl Fn(A) -> C) -> Self::P<C, B> {
        Self::bimap(fab, f, |b| b)
    }

    fn second<A, B, D>(fab: Self::P<A, B>, g: impl Fn(B) -> D) -> Self::P<A, D> {
        Self::bimap(fab, |a| a, g)
    }
}

impl Bifunctor for ResultBF {
    fn bimap<A, B, C, D>(
        fab: Result<B, A>,
        f: impl Fn(A) -> C,
        g: impl Fn(B) -> D,
    ) -> Result<D, C> {
        match fab {
            Ok(b) => Ok(g(b)),
            Err(a) => Err(f(a)),
        }
    }
}

impl Bifunctor for TupleF {
    fn bimap<A, B, C, D>(fab: (A, B), f: impl Fn(A) -> C, g: impl Fn(B) -> D) -> (C, D) {
        (f(fab.0), g(fab.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_bimap_ok() {
        let r: Result<i32, &str> = Ok(5);
        let result = ResultBF::bimap(r, |s| s.len(), |n| n * 2);
        assert_eq!(result, Ok(10));
    }

    #[test]
    fn result_bimap_err() {
        let r: Result<i32, &str> = Err("hello");
        let result = ResultBF::bimap(r, |s| s.len(), |n| n * 2);
        assert_eq!(result, Err(5));
    }

    #[test]
    fn result_first() {
        let r: Result<i32, &str> = Err("hi");
        let result = ResultBF::first(r, |s| s.len());
        assert_eq!(result, Err(2));
    }

    #[test]
    fn result_second() {
        let r: Result<i32, &str> = Ok(5);
        let result = ResultBF::second(r, |n| n * 3);
        assert_eq!(result, Ok(15));
    }

    #[test]
    fn tuple_bimap() {
        assert_eq!(TupleF::bimap((1, "hi"), |x| x + 1, |s| s.len()), (2, 2));
    }

    #[test]
    fn tuple_first() {
        assert_eq!(TupleF::first((1, "hi"), |x| x * 2), (2, "hi"));
    }

    #[test]
    fn tuple_second() {
        assert_eq!(TupleF::second((1, "hi"), |s| s.len()), (1, 2));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Identity: bimap(id, id, fab) == fab
        #[test]
        fn tuple_identity(a in any::<i32>(), b in any::<i32>()) {
            let fab = (a, b);
            let result = TupleF::bimap(fab, |x| x, |y| y);
            prop_assert_eq!(result, (a, b));
        }

        // Composition: bimap(f . g, h . i, fab) == bimap(f, h, bimap(g, i, fab))
        #[test]
        fn tuple_composition(a in any::<i16>(), b in any::<i16>()) {
            let fab = (a, b);
            let f = |x: i16| x.wrapping_add(1);
            let g = |x: i16| x.wrapping_mul(2);
            let h = |x: i16| x.wrapping_add(3);
            let i_fn = |x: i16| x.wrapping_mul(4);

            let left = TupleF::bimap(fab, |x| f(g(x)), |y| h(i_fn(y)));
            let right = TupleF::bimap(TupleF::bimap(fab, g, i_fn), f, h);
            prop_assert_eq!(left, right);
        }

        #[test]
        fn result_identity(x in any::<Result<i32, i32>>()) {
            let result = ResultBF::bimap(x, |a| a, |b| b);
            prop_assert_eq!(result, x);
        }

        #[test]
        fn result_composition(x in any::<Result<i16, i16>>()) {
            let f = |x: i16| x.wrapping_add(1);
            let g = |x: i16| x.wrapping_mul(2);
            let h = |x: i16| x.wrapping_add(3);
            let i_fn = |x: i16| x.wrapping_mul(4);

            let left = ResultBF::bimap(x, |a| f(g(a)), |b| h(i_fn(b)));
            let right = ResultBF::bimap(ResultBF::bimap(x, g, i_fn), f, h);
            prop_assert_eq!(left, right);
        }
    }
}
