use core::marker::PhantomData;

use karpal_core::hkt::HKT;

/// Right Kan Extension — `Ran G H A ≅ ∀R. (A → G R) → H R`.
///
/// Unlike most constructions in this crate, `Ran` is a **trait** rather than
/// a concrete type. This is because the universal quantifier `∀R` requires a
/// generic method, which cannot be made object-safe in Rust.
///
/// # Usage
///
/// Implement `Ran` for your own types to describe computations in CPS form:
///
/// ```rust,ignore
/// struct MyRan(i32);
///
/// impl Ran<OptionF, OptionF> for MyRan {
///     type Input = i32;
///     fn run_ran<R>(&self, k: impl Fn(i32) -> Option<R>) -> Option<R> {
///         k(self.0)
///     }
/// }
/// ```
///
/// # Relationship to Codensity
///
/// `Ran<F, F, A>` specialised to `G = H = F` is exactly `Codensity<F, A>`.
/// The `Codensity` type in this crate provides a concrete, ergonomic
/// implementation of that specialisation.
pub trait Ran<G: HKT, H: HKT> {
    /// The input type (corresponds to `A` in `∀R. (A → G R) → H R`).
    type Input;

    /// Run the Ran: given a continuation `k: A → G R`, produce `H R`.
    fn run_ran<R>(&self, k: impl Fn(Self::Input) -> G::Of<R>) -> H::Of<R>;
}

/// Map a function over a `Ran` implementation, producing a new `Ran`.
///
/// `ran_fmap(f: A → B)` transforms `Ran<G, H, Input=A>` into `Ran<G, H, Input=B>`:
///
/// ```text
/// new.run_ran(k: B → G R) = old.run_ran(|a| k(f(a)))
/// ```
pub fn ran_fmap<G, H, A, B, T, F>(ran: T, f: F) -> RanMapped<G, H, A, B, T, F>
where
    G: HKT,
    H: HKT,
    T: Ran<G, H, Input = A>,
    F: Fn(A) -> B,
{
    RanMapped {
        ran,
        f,
        _marker: PhantomData,
    }
}

/// A `Ran` with a mapped input, produced by [`ran_fmap`].
pub struct RanMapped<G: HKT, H: HKT, A, B, T, F> {
    ran: T,
    f: F,
    _marker: PhantomData<(G, H, A, B)>,
}

impl<G, H, A, B, T, F> Ran<G, H> for RanMapped<G, H, A, B, T, F>
where
    G: HKT,
    H: HKT,
    T: Ran<G, H, Input = A>,
    F: Fn(A) -> B,
{
    type Input = B;

    fn run_ran<R>(&self, k: impl Fn(B) -> G::Of<R>) -> H::Of<R> {
        let f_ref = &self.f;
        self.ran.run_ran(move |a| k(f_ref(a)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    struct SimpleRan(i32);

    impl Ran<OptionF, OptionF> for SimpleRan {
        type Input = i32;
        fn run_ran<R>(&self, k: impl Fn(i32) -> Option<R>) -> Option<R> {
            k(self.0)
        }
    }

    #[test]
    fn basic_ran() {
        let ran = SimpleRan(42);
        let result = ran.run_ran(|x| Some(x * 2));
        assert_eq!(result, Some(84));
    }

    #[test]
    fn ran_with_none() {
        let ran = SimpleRan(0);
        let result: Option<i32> = ran.run_ran(|_| None);
        assert_eq!(result, None);
    }

    #[test]
    fn ran_fmap_basic() {
        let ran = SimpleRan(10);
        let mapped = ran_fmap(ran, |x| x + 5);
        let result = mapped.run_ran(|x| Some(x * 2));
        // mapped.run_ran(k) = ran.run_ran(|a| k(f(a)))
        // = ran.run_ran(|a| k(a + 5))
        // = k(10 + 5) = k(15) = Some(15 * 2) = Some(30)
        assert_eq!(result, Some(30));
    }

    #[test]
    fn ran_fmap_identity() {
        let ran = SimpleRan(7);
        let mapped = ran_fmap(ran, |x: i32| x);
        let result = mapped.run_ran(Some);
        assert_eq!(result, Some(7));
    }

    #[test]
    fn ran_fmap_composition() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let left = ran_fmap(SimpleRan(5), move |a| g(f(a)));
        let right = ran_fmap(ran_fmap(SimpleRan(5), f), g);

        let left_result = left.run_ran(Some);
        let right_result = right.run_ran(Some);
        assert_eq!(left_result, right_result);
    }

    // Demonstrate relationship to Codensity: Ran<F, F, A> ≅ Codensity<F, A>
    #[test]
    fn ran_as_codensity() {
        // Ran<OptionF, OptionF> with Input = i32
        // is equivalent to Codensity<OptionF, i32>:
        //   ∀R. (i32 → Option R) → Option R
        let ran = SimpleRan(42);
        // "lower" via pure: k = Some
        let result = ran.run_ran(Some);
        assert_eq!(result, Some(42));
    }

    // Test with a different G and H
    struct VecToOption(Vec<i32>);

    impl Ran<OptionF, OptionF> for VecToOption {
        type Input = i32;
        fn run_ran<R>(&self, k: impl Fn(i32) -> Option<R>) -> Option<R> {
            // Apply k to the first element
            self.0.first().and_then(|&x| k(x))
        }
    }

    #[test]
    fn ran_different_impl() {
        let ran = VecToOption(vec![10, 20, 30]);
        let result = ran.run_ran(|x| Some(x.to_string()));
        assert_eq!(result, Some("10".to_string()));
    }

    #[test]
    fn ran_different_impl_empty() {
        let ran = VecToOption(vec![]);
        let result: Option<String> = ran.run_ran(|x| Some(x.to_string()));
        assert_eq!(result, None);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    struct PureRan(i32);

    impl Ran<OptionF, OptionF> for PureRan {
        type Input = i32;
        fn run_ran<R>(&self, k: impl Fn(i32) -> Option<R>) -> Option<R> {
            k(self.0)
        }
    }

    proptest! {
        // Functor identity: ran_fmap(id, r).run(k) == r.run(k)
        #[test]
        fn functor_identity(x in any::<i32>()) {
            let original = PureRan(x).run_ran(Some);
            let mapped = ran_fmap(PureRan(x), |a: i32| a).run_ran(Some);
            prop_assert_eq!(original, mapped);
        }

        // Functor composition: ran_fmap(g . f) == ran_fmap(g) . ran_fmap(f)
        #[test]
        fn functor_composition(x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);

            let left = ran_fmap(PureRan(x), move |a| g(f(a))).run_ran(Some);
            let right = ran_fmap(ran_fmap(PureRan(x), f), g).run_ran(Some);
            prop_assert_eq!(left, right);
        }
    }
}
