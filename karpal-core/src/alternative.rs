use crate::applicative::Applicative;
use crate::plus::Plus;

/// Alternative: Applicative + Plus with no extra methods (blanket impl).
///
/// Laws:
/// - Distributivity: `ap(alt(f, g), x) == alt(ap(f, x), ap(g, x))`
/// - Annihilation: `ap(zero(), x) == zero()`
pub trait Alternative: Applicative + Plus {}

impl<F: Applicative + Plus> Alternative for F {}

#[cfg(test)]
mod law_tests {
    use crate::alt::Alt;
    use crate::apply::Apply;
    use crate::hkt::OptionF;
    use crate::plus::Plus;
    use proptest::prelude::*;

    proptest! {
        // Distributivity: ap(alt(f, g), x) == alt(ap(f, x), ap(g, x))
        #[test]
        fn option_distributivity(x in any::<i16>()) {
            let f: Option<fn(i16) -> i16> = Some(|a| a.wrapping_add(1));
            let g: Option<fn(i16) -> i16> = Some(|a| a.wrapping_mul(2));

            let left = OptionF::ap(OptionF::alt(f, g), Some(x));
            let right = OptionF::alt(OptionF::ap(f, Some(x)), OptionF::ap(g, Some(x)));
            prop_assert_eq!(left, right);
        }

        // Annihilation: ap(zero(), x) == zero()
        #[test]
        fn option_annihilation(x in any::<Option<i32>>()) {
            let left = OptionF::ap(OptionF::zero::<fn(i32) -> i32>(), x);
            let right: Option<i32> = OptionF::zero();
            prop_assert_eq!(left, right);
        }
    }
}
