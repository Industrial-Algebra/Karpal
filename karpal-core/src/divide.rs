use crate::contravariant::{Contravariant, PredicateF};

/// Divide: the contravariant analogue of Apply.
///
/// Given a way to split `C` into `(A, B)`, and contravariant functors over
/// `A` and `B`, produce a contravariant functor over `C`.
///
/// Laws:
/// - Associativity: `divide(f, divide(g, a, b), c) == divide(h, a, divide(i, b, c))`
///   (where f/g/h/i are appropriate splitting functions)
pub trait Divide: Contravariant {
    fn divide<A: 'static, B: 'static, C: 'static>(
        f: impl Fn(C) -> (A, B) + 'static,
        fa: Self::Of<A>,
        fb: Self::Of<B>,
    ) -> Self::Of<C>;
}

impl Divide for PredicateF {
    fn divide<A: 'static, B: 'static, C: 'static>(
        f: impl Fn(C) -> (A, B) + 'static,
        fa: Box<dyn Fn(A) -> bool>,
        fb: Box<dyn Fn(B) -> bool>,
    ) -> Box<dyn Fn(C) -> bool> {
        Box::new(move |c| {
            let (a, b) = f(c);
            fa(a) && fb(b)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_divide() {
        let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
        let is_even: Box<dyn Fn(i32) -> bool> = Box::new(|x| x % 2 == 0);

        // Split a tuple into its components, check both predicates
        let both: Box<dyn Fn((i32, i32)) -> bool> =
            PredicateF::divide(|pair: (i32, i32)| pair, is_positive, is_even);

        assert!(both((3, 4))); // 3 > 0 && 4 is even
        assert!(!both((-1, 4))); // -1 not > 0
        assert!(!both((3, 3))); // 3 is not even
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Associativity (simplified): testing that divide composes correctly
        #[test]
        fn predicate_associativity(x in any::<i8>(), y in any::<i8>(), z in any::<i8>()) {
            let pa: Box<dyn Fn(i8) -> bool> = Box::new(|a| a > 0);
            let pb: Box<dyn Fn(i8) -> bool> = Box::new(|b| b > 0);
            let pc: Box<dyn Fn(i8) -> bool> = Box::new(|c| c > 0);

            // divide(id, divide(id, a, b), c) applied to (x, y, z)
            let pa2: Box<dyn Fn(i8) -> bool> = Box::new(|a| a > 0);
            let pb2: Box<dyn Fn(i8) -> bool> = Box::new(|b| b > 0);
            let pc2: Box<dyn Fn(i8) -> bool> = Box::new(|c| c > 0);

            let ab = PredicateF::divide(|pair: (i8, i8)| pair, pa, pb);
            let left = PredicateF::divide(
                |triple: (i8, i8, i8)| ((triple.0, triple.1), triple.2),
                ab,
                pc,
            );

            let bc = PredicateF::divide(|pair: (i8, i8)| pair, pb2, pc2);
            let right = PredicateF::divide(
                |triple: (i8, i8, i8)| (triple.0, (triple.1, triple.2)),
                pa2,
                bc,
            );

            prop_assert_eq!(left((x, y, z)), right((x, y, z)));
        }
    }
}
