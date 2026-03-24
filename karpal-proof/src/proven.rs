use core::marker::PhantomData;

use karpal_algebra::{
    AbelianGroup, BoundedLattice, Field, Group, Lattice, Module, Ring, Semiring, VectorSpace,
};
use karpal_core::{Monoid, Semigroup};

use crate::property::*;

/// A value of type `T` accompanied by a compile-time witness
/// that property `P` holds for `T`.
///
/// Construction is deliberately restricted: safe constructors
/// require the relevant trait bound (e.g., `from_semigroup` requires
/// `T: Semigroup`). The trait implementation *is* the proof.
///
/// For properties verified externally (proptest, SMT solver, inspection),
/// use the unsafe [`Proven::axiom`] constructor.
pub struct Proven<P, T> {
    value: T,
    _property: PhantomData<P>,
}

// Manual trait impls: P is phantom, so bounds should only apply to T.

impl<P, T: core::fmt::Debug> core::fmt::Debug for Proven<P, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Proven")
            .field("value", &self.value)
            .finish()
    }
}

impl<P, T: Clone> Clone for Proven<P, T> {
    fn clone(&self) -> Self {
        Proven {
            value: self.value.clone(),
            _property: PhantomData,
        }
    }
}

impl<P, T: Copy> Copy for Proven<P, T> {}

impl<P, T: PartialEq> PartialEq for Proven<P, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<P, T: Eq> Eq for Proven<P, T> {}

impl<P, T: PartialOrd> PartialOrd for Proven<P, T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<P, T: Ord> Ord for Proven<P, T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<P, T: core::hash::Hash> core::hash::Hash for Proven<P, T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<P, T> Proven<P, T> {
    /// Create a witness without checking.
    ///
    /// # Safety
    ///
    /// Caller asserts that property `P` genuinely holds for type `T`.
    pub unsafe fn axiom(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }

    /// Access the inner value by reference.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Unwrap the witness, discarding the proof.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// If property `P` implies property `Q`, derive a new witness.
    pub fn derive<Q>(self) -> Proven<Q, T>
    where
        P: Implies<Q>,
    {
        Proven {
            value: self.value,
            _property: PhantomData,
        }
    }

    /// Combine two independent proofs into a conjunction.
    pub fn and<Q>(self, _other: Proven<Q, T>) -> Proven<And<P, Q>, T> {
        Proven {
            value: self.value,
            _property: PhantomData,
        }
    }
}

impl<P, Q, T> Proven<And<P, Q>, T> {
    /// Extract the second component from a conjunction proof.
    ///
    /// (The first component is available via `.derive()` since
    /// `And<P, Q>: Implies<P>`.)
    pub fn derive_second(self) -> Proven<Q, T> {
        Proven {
            value: self.value,
            _property: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Safe constructors: trait bound IS the proof
// ---------------------------------------------------------------------------

impl<T: Semigroup> Proven<IsAssociative, T> {
    /// Witness associativity from a `Semigroup` implementation.
    pub fn from_semigroup(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }
}

impl<T: Monoid> Proven<IsMonoid, T> {
    /// Witness monoid laws from a `Monoid` implementation.
    pub fn from_monoid(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }
}

impl<T: Group> Proven<IsGroup, T> {
    /// Witness group laws from a `Group` implementation.
    pub fn from_group(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }
}

impl<T: AbelianGroup> Proven<IsAbelianGroup, T> {
    /// Witness abelian group from an `AbelianGroup` implementation.
    pub fn from_abelian(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }
}

impl<T: Semiring> Proven<IsSemiring, T> {
    pub fn from_semiring(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }
}

impl<T: Ring> Proven<IsRing, T> {
    pub fn from_ring(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }
}

impl<T: Field> Proven<IsField, T> {
    pub fn from_field(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }
}

impl<T: Lattice> Proven<IsLattice, T> {
    pub fn from_lattice(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }
}

impl<T: BoundedLattice> Proven<IsBoundedLattice, T> {
    pub fn from_bounded_lattice(value: T) -> Self {
        Proven {
            value,
            _property: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Trait forwarding: Proven<P, T> inherits T's algebraic structure
// ---------------------------------------------------------------------------

impl<P, T: Semigroup> Semigroup for Proven<P, T> {
    fn combine(self, other: Self) -> Self {
        Proven {
            value: self.value.combine(other.value),
            _property: PhantomData,
        }
    }
}

impl<P, T: Monoid> Monoid for Proven<P, T> {
    fn empty() -> Self {
        Proven {
            value: T::empty(),
            _property: PhantomData,
        }
    }
}

impl<P, T: Group> Group for Proven<P, T> {
    fn invert(self) -> Self {
        Proven {
            value: self.value.invert(),
            _property: PhantomData,
        }
    }
}

impl<P, T: AbelianGroup> AbelianGroup for Proven<P, T> {}

impl<P, T: Lattice> Lattice for Proven<P, T> {
    fn meet(self, other: Self) -> Self {
        Proven {
            value: self.value.meet(other.value),
            _property: PhantomData,
        }
    }

    fn join(self, other: Self) -> Self {
        Proven {
            value: self.value.join(other.value),
            _property: PhantomData,
        }
    }
}

impl<P, T: BoundedLattice> BoundedLattice for Proven<P, T> {
    fn top() -> Self {
        Proven {
            value: T::top(),
            _property: PhantomData,
        }
    }

    fn bottom() -> Self {
        Proven {
            value: T::bottom(),
            _property: PhantomData,
        }
    }
}

impl<P, T: Semiring> Semiring for Proven<P, T> {
    fn add(self, other: Self) -> Self {
        Proven {
            value: self.value.add(other.value),
            _property: PhantomData,
        }
    }

    fn mul(self, other: Self) -> Self {
        Proven {
            value: self.value.mul(other.value),
            _property: PhantomData,
        }
    }

    fn zero() -> Self {
        Proven {
            value: T::zero(),
            _property: PhantomData,
        }
    }

    fn one() -> Self {
        Proven {
            value: T::one(),
            _property: PhantomData,
        }
    }
}

impl<P, T: Ring> Ring for Proven<P, T> {
    fn negate(self) -> Self {
        Proven {
            value: self.value.negate(),
            _property: PhantomData,
        }
    }
}

impl<P, T: Field> Field for Proven<P, T> {
    fn reciprocal(self) -> Self {
        Proven {
            value: self.value.reciprocal(),
            _property: PhantomData,
        }
    }
}

impl<P, R: Ring, T: Module<R>> Module<R> for Proven<P, T> {
    fn scale(self, scalar: R) -> Self {
        Proven {
            value: self.value.scale(scalar),
            _property: PhantomData,
        }
    }
}

impl<P, F: Field, T: VectorSpace<F>> VectorSpace<F> for Proven<P, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_semigroup_and_access() {
        let p = Proven::<IsAssociative, i32>::from_semigroup(42);
        assert_eq!(*p.value(), 42);
        assert_eq!(p.into_inner(), 42);
    }

    #[test]
    fn from_monoid() {
        let p = Proven::<IsMonoid, i32>::from_monoid(10);
        assert_eq!(*p.value(), 10);
    }

    #[test]
    fn from_group() {
        let p = Proven::<IsGroup, i32>::from_group(-5);
        assert_eq!(*p.value(), -5);
    }

    #[test]
    fn from_abelian() {
        let p = Proven::<IsAbelianGroup, i32>::from_abelian(7);
        assert_eq!(*p.value(), 7);
    }

    #[test]
    fn derive_weaker_property() {
        let group_proof = Proven::<IsGroup, i32>::from_group(3);
        let monoid_proof: Proven<IsMonoid, i32> = group_proof.derive();
        assert_eq!(*monoid_proof.value(), 3);

        let assoc_proof: Proven<IsAssociative, i32> = monoid_proof.derive();
        assert_eq!(*assoc_proof.value(), 3);
    }

    #[test]
    fn derive_abelian_chain() {
        let abelian = Proven::<IsAbelianGroup, i32>::from_abelian(5);
        let _commutative: Proven<IsCommutative, i32> = abelian.derive();
    }

    #[test]
    fn and_conjunction() {
        let assoc = Proven::<IsAssociative, i32>::from_semigroup(1);
        // Safety: i32 addition is commutative (verified by proptest elsewhere)
        let comm: Proven<IsCommutative, i32> = unsafe { Proven::axiom(1) };
        let both = assoc.and(comm);
        let _: Proven<IsAssociative, i32> = both.derive();
    }

    #[test]
    fn semigroup_forwarding() {
        let a = Proven::<IsAssociative, i32>::from_semigroup(3);
        let b = Proven::<IsAssociative, i32>::from_semigroup(4);
        let c = a.combine(b);
        assert_eq!(*c.value(), 7);
    }

    #[test]
    fn monoid_forwarding() {
        let e = Proven::<IsMonoid, i32>::empty();
        assert_eq!(*e.value(), 0);
    }

    #[test]
    fn group_forwarding() {
        let p = Proven::<IsGroup, i32>::from_group(5);
        let inv = p.invert();
        assert_eq!(*inv.value(), -5);
    }

    #[test]
    fn lattice_forwarding() {
        let a = Proven::<IsLattice, bool>::from_lattice(true);
        let b = Proven::<IsLattice, bool>::from_lattice(false);
        assert_eq!(*a.meet(b).value(), false);
    }

    #[test]
    fn semiring_forwarding() {
        let a = Proven::<IsSemiring, i32>::from_semiring(3);
        let b = Proven::<IsSemiring, i32>::from_semiring(4);
        assert_eq!(*a.add(b).value(), 7);
    }

    #[test]
    fn ring_forwarding() {
        let a = Proven::<IsRing, i32>::from_ring(5);
        assert_eq!(*a.negate().value(), -5);
    }

    #[test]
    fn field_forwarding() {
        let a = Proven::<IsField, f64>::from_field(4.0);
        let r = a.reciprocal();
        assert!((r.value() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn unsafe_axiom() {
        let p: Proven<IsCommutative, i32> = unsafe { Proven::axiom(42) };
        assert_eq!(*p.value(), 42);
    }

    #[test]
    fn from_semiring() {
        let p = Proven::<IsSemiring, i32>::from_semiring(10);
        assert_eq!(*p.value(), 10);
    }

    #[test]
    fn from_ring() {
        let p = Proven::<IsRing, i32>::from_ring(10);
        assert_eq!(*p.value(), 10);
    }

    #[test]
    fn from_field() {
        let p = Proven::<IsField, f64>::from_field(2.5);
        assert!((p.value() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn from_lattice() {
        let p = Proven::<IsLattice, bool>::from_lattice(true);
        assert_eq!(*p.value(), true);
    }

    #[test]
    fn from_bounded_lattice() {
        let p = Proven::<IsBoundedLattice, bool>::from_bounded_lattice(false);
        assert_eq!(*p.value(), false);
    }

    #[test]
    fn derive_second_from_and() {
        let assoc = Proven::<IsAssociative, i32>::from_semigroup(10);
        let comm: Proven<IsCommutative, i32> = unsafe { Proven::axiom(10) };
        let both = assoc.and(comm);
        let extracted: Proven<IsCommutative, i32> = both.derive_second();
        assert_eq!(*extracted.value(), 10);
    }

    #[test]
    fn proven_preserves_equality() {
        let a = Proven::<IsMonoid, i32>::from_monoid(5);
        let b = Proven::<IsMonoid, i32>::from_monoid(5);
        let c = Proven::<IsMonoid, i32>::from_monoid(6);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn proven_preserves_ordering() {
        let a = Proven::<IsMonoid, i32>::from_monoid(3);
        let b = Proven::<IsMonoid, i32>::from_monoid(5);
        assert!(a < b);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::law_check;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proven_semigroup_associativity(a in -100i16..100i16, b in -100i16..100i16, c in -100i16..100i16) {
            let pa = Proven::<IsAssociative, i16>::from_semigroup(a);
            let pb = Proven::<IsAssociative, i16>::from_semigroup(b);
            let pc = Proven::<IsAssociative, i16>::from_semigroup(c);
            let left = pa.combine(pb).combine(pc);
            let pa2 = Proven::<IsAssociative, i16>::from_semigroup(a);
            let pb2 = Proven::<IsAssociative, i16>::from_semigroup(b);
            let pc2 = Proven::<IsAssociative, i16>::from_semigroup(c);
            let right = pa2.combine(pb2.combine(pc2));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn proven_monoid_identity(a in -100i16..100i16) {
            let pa = Proven::<IsMonoid, i16>::from_monoid(a);
            let e = Proven::<IsMonoid, i16>::empty();
            prop_assert_eq!(pa.combine(e), Proven::from_monoid(a));
            let pa2 = Proven::<IsMonoid, i16>::from_monoid(a);
            let e2 = Proven::<IsMonoid, i16>::empty();
            prop_assert_eq!(e2.combine(pa2), Proven::from_monoid(a));
        }

        #[test]
        fn proven_group_inverse(a in -100i16..100i16) {
            let pa = Proven::<IsGroup, i16>::from_group(a);
            let inv = pa.clone().invert();
            let result = pa.combine(inv);
            prop_assert_eq!(result, Proven::from_group(0));
        }

        #[test]
        fn law_check_semigroup_via_trait(a in -100i16..100i16, b in -100i16..100i16, c in -100i16..100i16) {
            law_check::check_associativity(
                a, b, c,
                |x, y| x.wrapping_add(y),
            ).unwrap();
        }

        #[test]
        fn law_check_commutativity_i16(a in -100i16..100i16, b in -100i16..100i16) {
            law_check::check_commutativity(
                a, b,
                |x, y| x.wrapping_add(y),
            ).unwrap();
        }

        #[test]
        fn law_check_inverse_i16(a in -100i16..100i16) {
            law_check::check_left_inverse(a, 0i16, |x, y| x.wrapping_add(y), |x| x.wrapping_neg()).unwrap();
            law_check::check_right_inverse(a, 0i16, |x, y| x.wrapping_add(y), |x| x.wrapping_neg()).unwrap();
        }

        #[test]
        fn law_check_distributivity_i16(a in -50i16..50i16, b in -50i16..50i16, c in -50i16..50i16) {
            law_check::check_left_distributivity(
                a, b, c,
                |x, y| x.wrapping_add(y),
                |x, y| x.wrapping_mul(y),
            ).unwrap();
        }
    }
}
