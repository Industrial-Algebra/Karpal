#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use crate::{AlgebraicSignature, Obligation, Origin};

/// Named collection of related proof obligations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationBundle {
    pub name: String,
    pub origin: Origin,
    obligations: Vec<Obligation>,
}

impl ObligationBundle {
    pub fn new(name: impl Into<String>, origin: Origin) -> Self {
        Self {
            name: name.into(),
            origin,
            obligations: Vec::new(),
        }
    }

    pub fn with(mut self, obligation: Obligation) -> Self {
        self.obligations.push(obligation);
        self
    }

    pub fn push(&mut self, obligation: Obligation) {
        self.obligations.push(obligation);
    }

    pub fn obligations(&self) -> &[Obligation] {
        &self.obligations
    }

    pub fn into_obligations(self) -> Vec<Obligation> {
        self.obligations
    }

    pub fn semigroup(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::new(name, origin.clone()).with(Obligation::associativity_in(
            "associativity",
            origin,
            signature,
            "combine",
        ))
    }

    pub fn monoid(name: impl Into<String>, origin: Origin, signature: &AlgebraicSignature) -> Self {
        Self::new(name, origin.clone())
            .with(Obligation::associativity_in(
                "associativity",
                origin.clone(),
                signature,
                "combine",
            ))
            .with(Obligation::left_identity_in(
                "left_identity",
                origin.clone(),
                signature,
                "combine",
                "identity",
            ))
            .with(Obligation::right_identity_in(
                "right_identity",
                origin,
                signature,
                "combine",
                "identity",
            ))
    }

    pub fn group(name: impl Into<String>, origin: Origin, signature: &AlgebraicSignature) -> Self {
        Self::monoid(name, origin.clone(), signature)
            .with(Obligation::left_inverse_in(
                "left_inverse",
                origin.clone(),
                signature,
            ))
            .with(Obligation::right_inverse_in(
                "right_inverse",
                origin,
                signature,
            ))
    }

    pub fn semiring(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::new(name, origin.clone())
            .with(Obligation::associativity_in(
                "add_associativity",
                origin.clone(),
                signature,
                "add",
            ))
            .with(Obligation::associativity_in(
                "mul_associativity",
                origin.clone(),
                signature,
                "mul",
            ))
            .with(Obligation::additive_commutativity_in(
                "add_commutativity",
                origin.clone(),
                signature,
            ))
            .with(Obligation::left_identity_in(
                "add_left_identity",
                origin.clone(),
                signature,
                "add",
                "zero",
            ))
            .with(Obligation::right_identity_in(
                "add_right_identity",
                origin.clone(),
                signature,
                "add",
                "zero",
            ))
            .with(Obligation::left_identity_in(
                "mul_left_identity",
                origin.clone(),
                signature,
                "mul",
                "one",
            ))
            .with(Obligation::right_identity_in(
                "mul_right_identity",
                origin.clone(),
                signature,
                "mul",
                "one",
            ))
            .with(Obligation::left_distributivity_in(
                "left_distributivity",
                origin.clone(),
                signature,
            ))
            .with(Obligation::right_distributivity_in(
                "right_distributivity",
                origin,
                signature,
            ))
    }

    pub fn lattice(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::new(name, origin.clone())
            .with(Obligation::associativity_in(
                "meet_associativity",
                origin.clone(),
                signature,
                "meet",
            ))
            .with(Obligation::associativity_in(
                "join_associativity",
                origin.clone(),
                signature,
                "join",
            ))
            .with(Obligation::commutativity_in(
                "meet_commutativity",
                origin.clone(),
                signature,
                "meet",
            ))
            .with(Obligation::commutativity_in(
                "join_commutativity",
                origin.clone(),
                signature,
                "join",
            ))
            .with(Obligation::idempotency_in(
                "meet_idempotency",
                origin.clone(),
                signature,
                "meet",
            ))
            .with(Obligation::idempotency_in(
                "join_idempotency",
                origin.clone(),
                signature,
                "join",
            ))
            .with(Obligation::absorption_in("absorption", origin, signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sort;

    #[test]
    fn monoid_bundle_contains_expected_laws() {
        let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
        let bundle = ObligationBundle::monoid("sum", Origin::new("karpal-core", "Sum<i32>"), &sig);
        assert_eq!(bundle.obligations().len(), 3);
    }

    #[test]
    fn semiring_bundle_contains_expected_laws() {
        let sig = AlgebraicSignature::semiring(Sort::Int, "add", "mul", "zero", "one");
        let bundle = ObligationBundle::semiring("ring", Origin::new("karpal-algebra", "i32"), &sig);
        assert_eq!(bundle.obligations().len(), 9);
    }
}
