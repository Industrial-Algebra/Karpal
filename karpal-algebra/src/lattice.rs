/// A type with join (supremum) and meet (infimum) operations.
///
/// Laws:
/// - Associativity: `a.join(b.join(c)) == a.join(b).join(c)` (and meet)
/// - Commutativity: `a.join(b) == b.join(a)` (and meet)
/// - Idempotency: `a.join(a) == a` (and meet)
/// - Absorption: `a.join(a.meet(b)) == a` and `a.meet(a.join(b)) == a`
pub trait Lattice: Sized {
    fn join(self, other: Self) -> Self;
    fn meet(self, other: Self) -> Self;
}

macro_rules! impl_ord_lattice {
    ($($t:ty),*) => {
        $(
            impl Lattice for $t {
                fn join(self, other: Self) -> Self {
                    core::cmp::max(self, other)
                }
                fn meet(self, other: Self) -> Self {
                    core::cmp::min(self, other)
                }
            }
        )*
    };
}

impl_ord_lattice!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

impl Lattice for bool {
    fn join(self, other: Self) -> Self {
        self || other
    }
    fn meet(self, other: Self) -> Self {
        self && other
    }
}

impl Lattice for f32 {
    fn join(self, other: Self) -> Self {
        f32::max(self, other)
    }
    fn meet(self, other: Self) -> Self {
        f32::min(self, other)
    }
}

impl Lattice for f64 {
    fn join(self, other: Self) -> Self {
        f64::max(self, other)
    }
    fn meet(self, other: Self) -> Self {
        f64::min(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i32_join_meet() {
        assert_eq!(3i32.join(5), 5);
        assert_eq!(3i32.meet(5), 3);
    }

    #[test]
    fn bool_join_meet() {
        assert_eq!(false.join(true), true);
        assert_eq!(false.meet(true), false);
    }

    #[test]
    fn f64_join_meet() {
        assert_eq!(1.5f64.join(2.5), 2.5);
        assert_eq!(1.5f64.meet(2.5), 1.5);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Join associativity
        #[test]
        fn join_associativity(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
            prop_assert_eq!(a.join(b).join(c), a.join(b.join(c)));
        }

        // Meet associativity
        #[test]
        fn meet_associativity(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
            prop_assert_eq!(a.meet(b).meet(c), a.meet(b.meet(c)));
        }

        // Join commutativity
        #[test]
        fn join_commutativity(a in any::<i32>(), b in any::<i32>()) {
            prop_assert_eq!(a.join(b), b.join(a));
        }

        // Meet commutativity
        #[test]
        fn meet_commutativity(a in any::<i32>(), b in any::<i32>()) {
            prop_assert_eq!(a.meet(b), b.meet(a));
        }

        // Idempotency
        #[test]
        fn join_idempotency(a in any::<i32>()) {
            prop_assert_eq!(a.join(a), a);
        }

        #[test]
        fn meet_idempotency(a in any::<i32>()) {
            prop_assert_eq!(a.meet(a), a);
        }

        // Absorption
        #[test]
        fn absorption_join_meet(a in any::<i32>(), b in any::<i32>()) {
            prop_assert_eq!(a.join(a.meet(b)), a);
        }

        #[test]
        fn absorption_meet_join(a in any::<i32>(), b in any::<i32>()) {
            prop_assert_eq!(a.meet(a.join(b)), a);
        }
    }
}
