//! Integration tests for karpal-proof-derive macros.

use karpal_core::{Monoid, Semigroup};
use karpal_proof::{VerifyCommutative, VerifyGroup, VerifyMonoid, VerifySemigroup};

#[derive(Clone, Copy, Debug, PartialEq, Eq, VerifySemigroup)]
#[verify(strategy = "(0i16..100).prop_map(AdditiveS)")]
struct AdditiveS(i16);

impl Semigroup for AdditiveS {
    fn combine(self, other: Self) -> Self {
        AdditiveS(self.0.wrapping_add(other.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, VerifyMonoid)]
#[verify(strategy = "(0i16..100).prop_map(AdditiveM)")]
struct AdditiveM(i16);

impl Semigroup for AdditiveM {
    fn combine(self, other: Self) -> Self {
        AdditiveM(self.0.wrapping_add(other.0))
    }
}

impl Monoid for AdditiveM {
    fn empty() -> Self {
        AdditiveM(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, VerifyGroup)]
#[verify(strategy = "(-50i16..50).prop_map(AdditiveG)")]
struct AdditiveG(i16);

impl Semigroup for AdditiveG {
    fn combine(self, other: Self) -> Self {
        AdditiveG(self.0.wrapping_add(other.0))
    }
}

impl Monoid for AdditiveG {
    fn empty() -> Self {
        AdditiveG(0)
    }
}

impl karpal_algebra::Group for AdditiveG {
    fn invert(self) -> Self {
        AdditiveG(self.0.wrapping_neg())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, VerifyCommutative)]
#[verify(strategy = "(-50i16..50).prop_map(AdditiveC)")]
struct AdditiveC(i16);

impl Semigroup for AdditiveC {
    fn combine(self, other: Self) -> Self {
        AdditiveC(self.0.wrapping_add(other.0))
    }
}

// ---------------------------------------------------------------------------
// Test type: a lattice (min/max)
// ---------------------------------------------------------------------------

use karpal_proof::VerifyLattice;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, VerifyLattice)]
#[verify(strategy = "(0i32..1000).prop_map(MinMax)")]
struct MinMax(i32);

impl karpal_algebra::Lattice for MinMax {
    fn meet(self, other: Self) -> Self {
        MinMax(self.0.min(other.0))
    }

    fn join(self, other: Self) -> Self {
        MinMax(self.0.max(other.0))
    }
}

// ---------------------------------------------------------------------------
// Test type: a semiring (wrapping arithmetic)
// ---------------------------------------------------------------------------

use karpal_proof::VerifySemiring;

#[derive(Clone, Copy, Debug, PartialEq, Eq, VerifySemiring)]
#[verify(strategy = "(-10i16..10).prop_map(WrapRing)")]
struct WrapRing(i16);

impl karpal_algebra::Semiring for WrapRing {
    fn add(self, other: Self) -> Self {
        WrapRing(self.0.wrapping_add(other.0))
    }

    fn mul(self, other: Self) -> Self {
        WrapRing(self.0.wrapping_mul(other.0))
    }

    fn zero() -> Self {
        WrapRing(0)
    }

    fn one() -> Self {
        WrapRing(1)
    }
}

// ---------------------------------------------------------------------------
// Test type: ring (wrapping arithmetic with negate)
// ---------------------------------------------------------------------------

use karpal_proof::VerifyRing;

#[derive(Clone, Copy, Debug, PartialEq, Eq, VerifyRing)]
#[verify(strategy = "(-10i16..10).prop_map(WrapRingN)")]
struct WrapRingN(i16);

impl karpal_algebra::Semiring for WrapRingN {
    fn add(self, other: Self) -> Self {
        WrapRingN(self.0.wrapping_add(other.0))
    }

    fn mul(self, other: Self) -> Self {
        WrapRingN(self.0.wrapping_mul(other.0))
    }

    fn zero() -> Self {
        WrapRingN(0)
    }

    fn one() -> Self {
        WrapRingN(1)
    }
}

impl karpal_algebra::Ring for WrapRingN {
    fn negate(self) -> Self {
        WrapRingN(self.0.wrapping_neg())
    }
}
