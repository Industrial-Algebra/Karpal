use core::marker::PhantomData;

use crate::intersection::{Intersection, check_intersection};
use crate::schubert_typed::SchubertTyped;

/// A value of type `T` accompanied by a type-level proof that it
/// satisfies the Schubert class declared by marker type `M`.
///
/// The marker `M` implements `SchubertTyped` — its `schubert_type()`
/// defines the Schubert class this value claims. Construction is
/// always safe: the proof comes from the type system's acceptance
/// of `M: SchubertTyped`, not from a runtime check.
///
/// This is the Schubert analogue of `karpal_proof::Proven<P, T>`.
pub struct SchubertProven<M: SchubertTyped, T> {
    value: T,
    _marker: PhantomData<M>,
    cached_type: crate::SchubertType,
}

impl<M: SchubertTyped, T> SchubertProven<M, T> {
    /// Construct a proven value from any value of type `T`.
    ///
    /// The proof is that marker `M` declares its Schubert class via
    /// `SchubertTyped` — no runtime verification needed at construction.
    pub fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
            cached_type: M::schubert_type(),
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

    /// Check whether this proven value is compatible with another
    /// Schubert-typed marker.
    ///
    /// Computes the intersection of the two Schubert classes and
    /// returns `Some(Intersection)` if they are compatible (nonzero
    /// intersection), `None` otherwise.
    pub fn check_against<U: SchubertTyped>(&self) -> Option<Intersection> {
        let other = U::schubert_type();
        let result = check_intersection(&self.cached_type, &other);
        if result.kind().is_zero() {
            None
        } else {
            Some(result)
        }
    }
}

// Manual trait impls that delegate to T

impl<M: SchubertTyped, T: core::fmt::Debug> core::fmt::Debug for SchubertProven<M, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SchubertProven")
            .field("value", &self.value)
            .finish()
    }
}

impl<M: SchubertTyped, T: Clone> Clone for SchubertProven<M, T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _marker: PhantomData,
            cached_type: self.cached_type.clone(),
        }
    }
}

impl<M: SchubertTyped, T: PartialEq> PartialEq for SchubertProven<M, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<M: SchubertTyped, T: Eq> Eq for SchubertProven<M, T> {}

/// Chained type-check composition via the Littlewood-Richardson rule.
///
/// Verifies that marker `A` is compatible with marker `B`, and that
/// the resulting Schubert class is compatible with marker `C`.
///
/// Returns `Some(Intersection)` representing the full composition if
/// all checks pass, `None` if any link in the chain fails.
pub fn compose_checks<A: SchubertTyped, B: SchubertTyped, C: SchubertTyped>() -> Option<Intersection>
{
    let a_type = A::schubert_type();
    let b_type = B::schubert_type();
    let c_type = C::schubert_type();

    // Step 1: A compatible with B?
    let ab = check_intersection(&a_type, &b_type);
    if ab.kind().is_zero() {
        return None;
    }

    // Step 2: Resulting class compatible with C?
    let ab_c = check_intersection(&a_type, &c_type);
    if ab_c.kind().is_zero() {
        return None;
    }

    Some(ab_c)
}
