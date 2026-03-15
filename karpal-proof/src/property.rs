use core::marker::PhantomData;

/// Marker trait for algebraic properties that can be witnessed.
///
/// A `Property` names a specific algebraic law. Zero-sized types
/// implement this trait to declare what law they represent.
pub trait Property {
    /// Human-readable name of the property.
    const NAME: &'static str;
}

/// Property P implies property Q: if P holds, then Q also holds.
pub trait Implies<Q> {}

// ---------------------------------------------------------------------------
// Semigroup / Monoid properties
// ---------------------------------------------------------------------------

/// The binary operation is associative: `a ∘ (b ∘ c) = (a ∘ b) ∘ c`.
pub struct IsAssociative;
impl Property for IsAssociative {
    const NAME: &'static str = "associativity";
}

/// There exists a left identity: `e ∘ a = a`.
pub struct HasLeftIdentity;
impl Property for HasLeftIdentity {
    const NAME: &'static str = "left identity";
}

/// There exists a right identity: `a ∘ e = a`.
pub struct HasRightIdentity;
impl Property for HasRightIdentity {
    const NAME: &'static str = "right identity";
}

/// The type forms a monoid: associative with two-sided identity.
pub struct IsMonoid;
impl Property for IsMonoid {
    const NAME: &'static str = "monoid";
}

// IsMonoid implies each constituent property
impl Implies<IsAssociative> for IsMonoid {}
impl Implies<HasLeftIdentity> for IsMonoid {}
impl Implies<HasRightIdentity> for IsMonoid {}

// ---------------------------------------------------------------------------
// Group properties
// ---------------------------------------------------------------------------

/// Every element has a left inverse: `a⁻¹ ∘ a = e`.
pub struct HasLeftInverse;
impl Property for HasLeftInverse {
    const NAME: &'static str = "left inverse";
}

/// Every element has a right inverse: `a ∘ a⁻¹ = e`.
pub struct HasRightInverse;
impl Property for HasRightInverse {
    const NAME: &'static str = "right inverse";
}

/// The type forms a group: monoid with two-sided inverses.
pub struct IsGroup;
impl Property for IsGroup {
    const NAME: &'static str = "group";
}

impl Implies<IsMonoid> for IsGroup {}
impl Implies<IsAssociative> for IsGroup {}
impl Implies<HasLeftIdentity> for IsGroup {}
impl Implies<HasRightIdentity> for IsGroup {}
impl Implies<HasLeftInverse> for IsGroup {}
impl Implies<HasRightInverse> for IsGroup {}

// ---------------------------------------------------------------------------
// Commutativity
// ---------------------------------------------------------------------------

/// The binary operation is commutative: `a ∘ b = b ∘ a`.
pub struct IsCommutative;
impl Property for IsCommutative {
    const NAME: &'static str = "commutativity";
}

/// The type forms an abelian group: commutative group.
pub struct IsAbelianGroup;
impl Property for IsAbelianGroup {
    const NAME: &'static str = "abelian group";
}

impl Implies<IsGroup> for IsAbelianGroup {}
impl Implies<IsMonoid> for IsAbelianGroup {}
impl Implies<IsAssociative> for IsAbelianGroup {}
impl Implies<IsCommutative> for IsAbelianGroup {}
impl Implies<HasLeftInverse> for IsAbelianGroup {}
impl Implies<HasRightInverse> for IsAbelianGroup {}
impl Implies<HasLeftIdentity> for IsAbelianGroup {}
impl Implies<HasRightIdentity> for IsAbelianGroup {}

// ---------------------------------------------------------------------------
// Semiring / Ring / Field properties
// ---------------------------------------------------------------------------

/// Addition is commutative.
pub struct AdditivelyCommutative;
impl Property for AdditivelyCommutative {
    const NAME: &'static str = "additively commutative";
}

/// Multiplication distributes over addition.
pub struct IsDistributive;
impl Property for IsDistributive {
    const NAME: &'static str = "distributive";
}

/// Zero annihilates: `0 * a = a * 0 = 0`.
pub struct ZeroAnnihilates;
impl Property for ZeroAnnihilates {
    const NAME: &'static str = "zero annihilation";
}

/// The type forms a semiring.
pub struct IsSemiring;
impl Property for IsSemiring {
    const NAME: &'static str = "semiring";
}

impl Implies<AdditivelyCommutative> for IsSemiring {}
impl Implies<IsDistributive> for IsSemiring {}
impl Implies<ZeroAnnihilates> for IsSemiring {}

/// The type forms a ring: semiring with additive inverses.
pub struct IsRing;
impl Property for IsRing {
    const NAME: &'static str = "ring";
}

impl Implies<IsSemiring> for IsRing {}
impl Implies<AdditivelyCommutative> for IsRing {}
impl Implies<IsDistributive> for IsRing {}

/// The type forms a field: ring with multiplicative inverses (for nonzero).
pub struct IsField;
impl Property for IsField {
    const NAME: &'static str = "field";
}

impl Implies<IsRing> for IsField {}
impl Implies<IsSemiring> for IsField {}

// ---------------------------------------------------------------------------
// Lattice properties
// ---------------------------------------------------------------------------

/// The operation is idempotent: `a ∘ a = a`.
pub struct IsIdempotent;
impl Property for IsIdempotent {
    const NAME: &'static str = "idempotent";
}

/// Meet and join satisfy absorption: `a ∧ (a ∨ b) = a`.
pub struct IsAbsorptive;
impl Property for IsAbsorptive {
    const NAME: &'static str = "absorptive";
}

/// The type forms a lattice.
pub struct IsLattice;
impl Property for IsLattice {
    const NAME: &'static str = "lattice";
}

impl Implies<IsIdempotent> for IsLattice {}
impl Implies<IsAbsorptive> for IsLattice {}
impl Implies<IsCommutative> for IsLattice {}
impl Implies<IsAssociative> for IsLattice {}

/// The type forms a bounded lattice: lattice with top and bottom.
pub struct IsBoundedLattice;
impl Property for IsBoundedLattice {
    const NAME: &'static str = "bounded lattice";
}

impl Implies<IsLattice> for IsBoundedLattice {}
impl Implies<IsIdempotent> for IsBoundedLattice {}
impl Implies<IsAbsorptive> for IsBoundedLattice {}

// ---------------------------------------------------------------------------
// Functor / Monad properties
// ---------------------------------------------------------------------------

/// `fmap(id, fa) == fa`.
pub struct FunctorIdentity;
impl Property for FunctorIdentity {
    const NAME: &'static str = "functor identity";
}

/// `fmap(g . f, fa) == fmap(g, fmap(f, fa))`.
pub struct FunctorComposition;
impl Property for FunctorComposition {
    const NAME: &'static str = "functor composition";
}

/// The type is a lawful functor.
pub struct IsLawfulFunctor;
impl Property for IsLawfulFunctor {
    const NAME: &'static str = "lawful functor";
}

impl Implies<FunctorIdentity> for IsLawfulFunctor {}
impl Implies<FunctorComposition> for IsLawfulFunctor {}

/// `pure(a) >>= f == f(a)`.
pub struct MonadLeftIdentity;
impl Property for MonadLeftIdentity {
    const NAME: &'static str = "monad left identity";
}

/// `m >>= pure == m`.
pub struct MonadRightIdentity;
impl Property for MonadRightIdentity {
    const NAME: &'static str = "monad right identity";
}

/// `(m >>= f) >>= g == m >>= (|x| f(x) >>= g)`.
pub struct MonadAssociativity;
impl Property for MonadAssociativity {
    const NAME: &'static str = "monad associativity";
}

/// The type is a lawful monad.
pub struct IsLawfulMonad;
impl Property for IsLawfulMonad {
    const NAME: &'static str = "lawful monad";
}

impl Implies<IsLawfulFunctor> for IsLawfulMonad {}
impl Implies<MonadLeftIdentity> for IsLawfulMonad {}
impl Implies<MonadRightIdentity> for IsLawfulMonad {}
impl Implies<MonadAssociativity> for IsLawfulMonad {}

// ---------------------------------------------------------------------------
// Compound property: conjunction of two properties
// ---------------------------------------------------------------------------

/// Both properties P and Q hold simultaneously.
///
/// Use `Proven::and()` to combine two witnesses into `Proven<And<P, Q>, T>`,
/// then `derive()` to extract either constituent via the `Implies` impls.
///
/// Note: Due to Rust's coherence rules, only `Implies<P>` (the first
/// component) is provided as a blanket impl. To extract `Q`, use the
/// dedicated `Proven<And<P, Q>, T>::derive_second()` method.
pub struct And<P, Q>(PhantomData<(P, Q)>);
impl<P: Property, Q: Property> Property for And<P, Q> {
    const NAME: &'static str = "conjunction";
}

// Only the first component as a blanket — second would conflict when P=Q.
impl<P, Q> Implies<P> for And<P, Q> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_names() {
        assert_eq!(IsAssociative::NAME, "associativity");
        assert_eq!(IsMonoid::NAME, "monoid");
        assert_eq!(IsGroup::NAME, "group");
        assert_eq!(IsAbelianGroup::NAME, "abelian group");
        assert_eq!(IsSemiring::NAME, "semiring");
        assert_eq!(IsRing::NAME, "ring");
        assert_eq!(IsField::NAME, "field");
        assert_eq!(IsLattice::NAME, "lattice");
        assert_eq!(IsBoundedLattice::NAME, "bounded lattice");
        assert_eq!(IsLawfulFunctor::NAME, "lawful functor");
        assert_eq!(IsLawfulMonad::NAME, "lawful monad");
    }

    // Compile-time test: implication chains
    fn _assert_implies<P: Implies<Q>, Q>() {}

    #[test]
    fn implication_lattice_compiles() {
        _assert_implies::<IsMonoid, IsAssociative>();
        _assert_implies::<IsMonoid, HasLeftIdentity>();
        _assert_implies::<IsGroup, IsMonoid>();
        _assert_implies::<IsGroup, IsAssociative>();
        _assert_implies::<IsAbelianGroup, IsGroup>();
        _assert_implies::<IsAbelianGroup, IsCommutative>();
        _assert_implies::<IsField, IsRing>();
        _assert_implies::<IsRing, IsSemiring>();
        _assert_implies::<IsBoundedLattice, IsLattice>();
        _assert_implies::<IsLawfulMonad, IsLawfulFunctor>();
        _assert_implies::<And<IsAssociative, IsCommutative>, IsAssociative>();
        // Second component extracted via Proven::derive_second(), not Implies
    }
}
