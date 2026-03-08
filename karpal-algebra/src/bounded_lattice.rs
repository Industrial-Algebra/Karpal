use crate::lattice::Lattice;

/// A `Lattice` with top and bottom elements.
///
/// Laws:
/// - `a.join(bottom()) == a`
/// - `a.meet(top()) == a`
pub trait BoundedLattice: Lattice {
    fn top() -> Self;
    fn bottom() -> Self;
}

macro_rules! impl_bounded_lattice {
    ($($t:ty),*) => {
        $(
            impl BoundedLattice for $t {
                fn top() -> Self { <$t>::MAX }
                fn bottom() -> Self { <$t>::MIN }
            }
        )*
    };
}

impl_bounded_lattice!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

impl BoundedLattice for bool {
    fn top() -> Self {
        true
    }
    fn bottom() -> Self {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i32_bounded_lattice() {
        assert_eq!(i32::top(), i32::MAX);
        assert_eq!(i32::bottom(), i32::MIN);
    }

    #[test]
    fn bool_bounded_lattice() {
        assert_eq!(bool::top(), true);
        assert_eq!(bool::bottom(), false);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn join_bottom_identity(a in any::<i32>()) {
            prop_assert_eq!(a.join(i32::bottom()), a);
        }

        #[test]
        fn meet_top_identity(a in any::<i32>()) {
            prop_assert_eq!(a.meet(i32::top()), a);
        }

        #[test]
        fn join_top_absorb(a in any::<i32>()) {
            prop_assert_eq!(a.join(i32::top()), i32::top());
        }

        #[test]
        fn meet_bottom_absorb(a in any::<i32>()) {
            prop_assert_eq!(a.meet(i32::bottom()), i32::bottom());
        }
    }
}
