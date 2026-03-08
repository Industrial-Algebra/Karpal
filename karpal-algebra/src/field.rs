use crate::ring::Ring;

/// A `Ring` with multiplicative inverses for all non-zero elements.
pub trait Field: Ring {
    fn reciprocal(self) -> Self;

    fn div(self, other: Self) -> Self
    where
        Self: Sized,
    {
        self.mul(other.reciprocal())
    }
}

impl Field for f32 {
    fn reciprocal(self) -> Self {
        1.0 / self
    }
}

impl Field for f64 {
    fn reciprocal(self) -> Self {
        1.0 / self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {
            assert_approx_eq!($a, $b, 1e-10)
        };
        ($a:expr, $b:expr, $eps:expr) => {
            let (a, b) = ($a as f64, $b as f64);
            assert!(
                (a - b).abs() < $eps,
                "assertion failed: |{} - {}| = {} >= {}",
                a,
                b,
                (a - b).abs(),
                $eps
            );
        };
    }

    #[test]
    fn f64_reciprocal() {
        assert_approx_eq!(2.0f64.reciprocal(), 0.5);
        assert_approx_eq!(4.0f64.reciprocal(), 0.25);
    }

    #[test]
    fn f64_div() {
        assert_approx_eq!(10.0f64.div(4.0), 2.5);
    }

    #[test]
    fn f32_reciprocal() {
        assert!((2.0f32.reciprocal() - 0.5).abs() < 1e-6);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::semiring::Semiring;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn f64_multiplicative_inverse(a in proptest::num::f64::NORMAL.prop_filter("non-zero", |x| x.abs() > 1e-10)) {
            let result = a.mul(a.reciprocal());
            prop_assert!((result - 1.0).abs() < 1e-6, "a={}, a*a^-1={}", a, result);
        }

        #[test]
        fn f64_div_is_mul_reciprocal(
            a in proptest::num::f64::NORMAL.prop_filter("bounded", |x| x.abs() < 1e6),
            b in proptest::num::f64::NORMAL.prop_filter("non-zero bounded", |x| x.abs() > 1e-10 && x.abs() < 1e6)
        ) {
            let left = a.div(b);
            let right = a.mul(b.reciprocal());
            prop_assert!((left - right).abs() < 1e-6, "a/b={}, a*b^-1={}", left, right);
        }
    }
}
