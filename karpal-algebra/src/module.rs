use crate::abelian::AbelianGroup;
use crate::ring::Ring;

/// A module over a ring R — an abelian group with scalar multiplication.
///
/// Laws:
/// - `a.scale(R::one()) == a`
/// - `a.scale(r).scale(s) == a.scale(r.mul(s))`
/// - `a.combine(b).scale(r) == a.scale(r).combine(b.scale(r))`
/// - `a.scale(r.add(s)) == a.scale(r).combine(a.scale(s))`
pub trait Module<R: Ring>: AbelianGroup {
    fn scale(self, scalar: R) -> Self;
}

impl Module<f32> for f32 {
    fn scale(self, scalar: f32) -> Self {
        self * scalar
    }
}

impl Module<f64> for f64 {
    fn scale(self, scalar: f64) -> Self {
        self * scalar
    }
}

impl<F: crate::field::Field + AbelianGroup> Module<F> for (F, F) {
    fn scale(self, scalar: F) -> Self {
        (self.0.mul(scalar.clone()), self.1.mul(scalar))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semiring::Semiring;
    use karpal_core::Semigroup;

    #[test]
    fn f64_scale() {
        assert!((3.0f64.scale(2.0) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn f64_scale_one() {
        assert!((5.0f64.scale(f64::one()) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn tuple_scale() {
        let v = (1.0f64, 2.0f64).scale(3.0);
        assert!((v.0 - 3.0).abs() < 1e-10);
        assert!((v.1 - 6.0).abs() < 1e-10);
    }

    #[test]
    fn tuple_scale_distributes_over_add() {
        let a = (1.0f64, 2.0);
        let b = (3.0f64, 4.0);
        let sum_scaled = a.combine(b).scale(2.0);
        let scaled_sum = a.scale(2.0).combine(b.scale(2.0));
        assert!((sum_scaled.0 - scaled_sum.0).abs() < 1e-10);
        assert!((sum_scaled.1 - scaled_sum.1).abs() < 1e-10);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::semiring::Semiring;
    use karpal_core::Semigroup;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn scale_one_identity(a in -100.0f64..100.0) {
            prop_assert!((a.scale(f64::one()) - a).abs() < 1e-10);
        }

        #[test]
        fn scale_compatibility(
            a in -10.0f64..10.0,
            r in -10.0f64..10.0,
            s in -10.0f64..10.0
        ) {
            let left = a.scale(r).scale(s);
            let right = a.scale(r.mul(s));
            prop_assert!((left - right).abs() < 1e-6, "left={}, right={}", left, right);
        }

        #[test]
        fn scale_distributes_over_group_add(
            a in -10.0f64..10.0,
            b in -10.0f64..10.0,
            r in -10.0f64..10.0
        ) {
            let left = a.combine(b).scale(r);
            let right = a.scale(r).combine(b.scale(r));
            prop_assert!((left - right).abs() < 1e-6, "left={}, right={}", left, right);
        }

        #[test]
        fn scale_distributes_over_scalar_add(
            a in -10.0f64..10.0,
            r in -10.0f64..10.0,
            s in -10.0f64..10.0
        ) {
            let left = a.scale(r.add(s));
            let right = a.scale(r).combine(a.scale(s));
            prop_assert!((left - right).abs() < 1e-6, "left={}, right={}", left, right);
        }
    }
}
