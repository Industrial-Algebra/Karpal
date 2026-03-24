#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec, vec::Vec};
#[cfg(feature = "std")]
use std::{format, string::String, vec, vec::Vec};

use karpal_proof::{
    AdditivelyCommutative, HasLeftIdentity, HasLeftInverse, HasRightIdentity, HasRightInverse,
    IsAbsorptive, IsAssociative, IsCommutative, IsDistributive, IsIdempotent, IsMonoid, Property,
};

use crate::signature::AlgebraicSignature;

/// Verification philosophy tier, following the roadmap's three-tier model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationTier {
    /// Illegal states are unrepresentable in Rust's type system.
    Impossible,
    /// Statistical evidence makes violations rare in practice.
    Rare,
    /// Runtime or search-based discovery catches emergent behavior.
    Emergent,
    /// External theorem prover or proof assistant evidence.
    External,
}

/// Target proof language / export dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofDialect {
    SmtLib2,
    Lean4,
}

/// Source location for a generated proof obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    pub crate_name: String,
    pub item_path: String,
}

impl Origin {
    pub fn new(crate_name: impl Into<String>, item_path: impl Into<String>) -> Self {
        Self {
            crate_name: crate_name.into(),
            item_path: item_path.into(),
        }
    }
}

/// A declared variable or constant used by an obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub name: String,
    pub sort: Sort,
}

impl Declaration {
    pub fn new(name: impl Into<String>, sort: Sort) -> Self {
        Self {
            name: name.into(),
            sort,
        }
    }
}

/// Sorts supported by the backend-agnostic obligation IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sort {
    Bool,
    Int,
    Real,
    Named(String),
}

impl Sort {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }
}

/// Backend-agnostic term language for proof obligations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Var(String),
    Bool(bool),
    Int(i64),
    App { function: String, args: Vec<Term> },
    Eq(Box<Term>, Box<Term>),
    And(Vec<Term>),
    Or(Vec<Term>),
    Not(Box<Term>),
    Implies(Box<Term>, Box<Term>),
}

impl Term {
    pub fn var(name: impl Into<String>) -> Self {
        Self::Var(name.into())
    }

    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn int(value: i64) -> Self {
        Self::Int(value)
    }

    pub fn app(function: impl Into<String>, args: impl IntoIterator<Item = Term>) -> Self {
        Self::App {
            function: function.into(),
            args: args.into_iter().collect(),
        }
    }

    pub fn eq(left: Term, right: Term) -> Self {
        Self::Eq(Box::new(left), Box::new(right))
    }

    pub fn and(terms: impl IntoIterator<Item = Term>) -> Self {
        Self::And(terms.into_iter().collect())
    }

    pub fn or(terms: impl IntoIterator<Item = Term>) -> Self {
        Self::Or(terms.into_iter().collect())
    }

    pub fn negate(term: Term) -> Self {
        Self::Not(Box::new(term))
    }

    pub fn implies(lhs: Term, rhs: Term) -> Self {
        Self::Implies(Box::new(lhs), Box::new(rhs))
    }
}

/// A backend-agnostic proof obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obligation {
    pub name: String,
    pub property: &'static str,
    pub declarations: Vec<Declaration>,
    pub assumptions: Vec<Term>,
    pub conclusion: Term,
    pub origin: Origin,
    pub tier: VerificationTier,
}

impl Obligation {
    /// Create a new obligation for property `P`.
    pub fn for_property<P: Property>(
        name: impl Into<String>,
        origin: Origin,
        tier: VerificationTier,
        conclusion: Term,
    ) -> Self {
        Self {
            name: name.into(),
            property: P::NAME,
            declarations: Vec::new(),
            assumptions: Vec::new(),
            conclusion,
            origin,
            tier,
        }
    }

    pub fn with_decl(mut self, name: impl Into<String>, sort: Sort) -> Self {
        self.declarations.push(Declaration::new(name, sort));
        self
    }

    pub fn with_assumption(mut self, term: Term) -> Self {
        self.assumptions.push(term);
        self
    }

    /// Build an associativity obligation for a binary operator symbol.
    pub fn associativity(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        op: impl Into<String>,
    ) -> Self {
        let op = op.into();
        let (a, b, c) = (Term::var("a"), Term::var("b"), Term::var("c"));
        let left = Term::app(
            op.clone(),
            vec![Term::app(op.clone(), vec![a.clone(), b.clone()]), c.clone()],
        );
        let right = Term::app(
            op.clone(),
            vec![a.clone(), Term::app(op, vec![b.clone(), c])],
        );

        Self::for_property::<IsAssociative>(
            name,
            origin,
            VerificationTier::External,
            Term::eq(left, right),
        )
        .with_decl("a", carrier.clone())
        .with_decl("b", carrier.clone())
        .with_decl("c", carrier)
    }

    pub fn associativity_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
        role: &str,
    ) -> Self {
        Self::associativity(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary(role),
        )
    }

    /// Build a commutativity obligation for a binary operator symbol.
    pub fn commutativity(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        op: impl Into<String>,
    ) -> Self {
        let op = op.into();
        let (a, b) = (Term::var("a"), Term::var("b"));
        let left = Term::app(op.clone(), vec![a.clone(), b.clone()]);
        let right = Term::app(op, vec![b, a]);

        Self::for_property::<IsCommutative>(
            name,
            origin,
            VerificationTier::External,
            Term::eq(left, right),
        )
        .with_decl("a", carrier.clone())
        .with_decl("b", carrier)
    }

    pub fn commutativity_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
        role: &str,
    ) -> Self {
        Self::commutativity(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary(role),
        )
    }

    pub fn additive_commutativity(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        add: impl Into<String>,
    ) -> Self {
        let add = add.into();
        let (a, b) = (Term::var("a"), Term::var("b"));
        let left = Term::app(add.clone(), vec![a.clone(), b.clone()]);
        let right = Term::app(add, vec![b, a]);

        Self::for_property::<AdditivelyCommutative>(
            name,
            origin,
            VerificationTier::External,
            Term::eq(left, right),
        )
        .with_decl("a", carrier.clone())
        .with_decl("b", carrier)
    }

    pub fn additive_commutativity_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::additive_commutativity(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary("add"),
        )
    }

    /// Build a two-sided identity obligation for a binary operator symbol.
    pub fn monoid_identity(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        op: impl Into<String>,
        identity: impl Into<String>,
    ) -> Self {
        let op = op.into();
        let e = Term::var(identity);
        let a = Term::var("a");
        let left = Term::eq(Term::app(op.clone(), vec![e.clone(), a.clone()]), a.clone());
        let right = Term::eq(Term::app(op, vec![a.clone(), e]), a);

        Self::for_property::<IsMonoid>(
            name,
            origin,
            VerificationTier::External,
            Term::and(vec![left, right]),
        )
        .with_decl("a", carrier.clone())
        .with_decl("e", carrier)
    }

    pub fn monoid_identity_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::monoid_identity(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary("combine"),
            signature.require_constant("identity"),
        )
    }

    pub fn left_identity(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        op: impl Into<String>,
        identity: impl Into<String>,
    ) -> Self {
        let op = op.into();
        let e = Term::var(identity);
        let a = Term::var("a");
        let conclusion = Term::eq(Term::app(op, vec![e, a.clone()]), a);

        Self::for_property::<HasLeftIdentity>(name, origin, VerificationTier::External, conclusion)
            .with_decl("a", carrier.clone())
            .with_decl("e", carrier)
    }

    pub fn left_identity_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
        op_role: &str,
        identity_role: &str,
    ) -> Self {
        Self::left_identity(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary(op_role),
            signature.require_constant(identity_role),
        )
    }

    pub fn right_identity(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        op: impl Into<String>,
        identity: impl Into<String>,
    ) -> Self {
        let op = op.into();
        let e = Term::var(identity);
        let a = Term::var("a");
        let conclusion = Term::eq(Term::app(op, vec![a.clone(), e]), a);

        Self::for_property::<HasRightIdentity>(name, origin, VerificationTier::External, conclusion)
            .with_decl("a", carrier.clone())
            .with_decl("e", carrier)
    }

    pub fn right_identity_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
        op_role: &str,
        identity_role: &str,
    ) -> Self {
        Self::right_identity(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary(op_role),
            signature.require_constant(identity_role),
        )
    }

    pub fn left_inverse(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        op: impl Into<String>,
        inverse: impl Into<String>,
        identity: impl Into<String>,
    ) -> Self {
        let op = op.into();
        let inv = inverse.into();
        let e = Term::var(identity);
        let a = Term::var("a");
        let conclusion = Term::eq(Term::app(op, vec![Term::app(inv, vec![a.clone()]), a]), e);

        Self::for_property::<HasLeftInverse>(name, origin, VerificationTier::External, conclusion)
            .with_decl("a", carrier.clone())
            .with_decl("e", carrier)
    }

    pub fn left_inverse_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::left_inverse(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary("combine"),
            signature.require_unary("inverse"),
            signature.require_constant("identity"),
        )
    }

    pub fn right_inverse(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        op: impl Into<String>,
        inverse: impl Into<String>,
        identity: impl Into<String>,
    ) -> Self {
        let op = op.into();
        let inv = inverse.into();
        let e = Term::var(identity);
        let a = Term::var("a");
        let conclusion = Term::eq(Term::app(op, vec![a.clone(), Term::app(inv, vec![a])]), e);

        Self::for_property::<HasRightInverse>(name, origin, VerificationTier::External, conclusion)
            .with_decl("a", carrier.clone())
            .with_decl("e", carrier)
    }

    pub fn right_inverse_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::right_inverse(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary("combine"),
            signature.require_unary("inverse"),
            signature.require_constant("identity"),
        )
    }

    pub fn left_distributivity(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        add: impl Into<String>,
        mul: impl Into<String>,
    ) -> Self {
        let add = add.into();
        let mul = mul.into();
        let (a, b, c) = (Term::var("a"), Term::var("b"), Term::var("c"));
        let left = Term::app(
            mul.clone(),
            vec![
                a.clone(),
                Term::app(add.clone(), vec![b.clone(), c.clone()]),
            ],
        );
        let right = Term::app(
            add,
            vec![
                Term::app(mul.clone(), vec![a.clone(), b]),
                Term::app(mul, vec![a, c]),
            ],
        );

        Self::for_property::<IsDistributive>(
            name,
            origin,
            VerificationTier::External,
            Term::eq(left, right),
        )
        .with_decl("a", carrier.clone())
        .with_decl("b", carrier.clone())
        .with_decl("c", carrier)
    }

    pub fn left_distributivity_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::left_distributivity(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary("add"),
            signature.require_binary("mul"),
        )
    }

    pub fn right_distributivity(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        add: impl Into<String>,
        mul: impl Into<String>,
    ) -> Self {
        let add = add.into();
        let mul = mul.into();
        let (a, b, c) = (Term::var("a"), Term::var("b"), Term::var("c"));
        let left = Term::app(
            mul.clone(),
            vec![
                Term::app(add.clone(), vec![a.clone(), b.clone()]),
                c.clone(),
            ],
        );
        let right = Term::app(
            add,
            vec![
                Term::app(mul.clone(), vec![a, c.clone()]),
                Term::app(mul, vec![b, c]),
            ],
        );

        Self::for_property::<IsDistributive>(
            name,
            origin,
            VerificationTier::External,
            Term::eq(left, right),
        )
        .with_decl("a", carrier.clone())
        .with_decl("b", carrier.clone())
        .with_decl("c", carrier)
    }

    pub fn right_distributivity_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::right_distributivity(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary("add"),
            signature.require_binary("mul"),
        )
    }

    pub fn idempotency(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        op: impl Into<String>,
    ) -> Self {
        let op = op.into();
        let a = Term::var("a");
        let conclusion = Term::eq(Term::app(op, vec![a.clone(), a.clone()]), a);

        Self::for_property::<IsIdempotent>(name, origin, VerificationTier::External, conclusion)
            .with_decl("a", carrier)
    }

    pub fn idempotency_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
        role: &str,
    ) -> Self {
        Self::idempotency(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary(role),
        )
    }

    pub fn absorption(
        name: impl Into<String>,
        origin: Origin,
        carrier: Sort,
        meet: impl Into<String>,
        join: impl Into<String>,
    ) -> Self {
        let meet = meet.into();
        let join = join.into();
        let (a, b) = (Term::var("a"), Term::var("b"));
        let left = Term::eq(
            Term::app(
                meet.clone(),
                vec![
                    a.clone(),
                    Term::app(join.clone(), vec![a.clone(), b.clone()]),
                ],
            ),
            a.clone(),
        );
        let right = Term::eq(
            Term::app(join, vec![a.clone(), Term::app(meet, vec![a.clone(), b])]),
            a,
        );

        Self::for_property::<IsAbsorptive>(
            name,
            origin,
            VerificationTier::External,
            Term::and(vec![left, right]),
        )
        .with_decl("a", carrier.clone())
        .with_decl("b", carrier)
    }

    pub fn absorption_in(
        name: impl Into<String>,
        origin: Origin,
        signature: &AlgebraicSignature,
    ) -> Self {
        Self::absorption(
            name,
            origin,
            signature.carrier.clone(),
            signature.require_binary("meet"),
            signature.require_binary("join"),
        )
    }

    /// Human-readable summary used in debugging and certificate generation.
    pub fn summary(&self) -> String {
        format!(
            "{}::{} [{}]",
            self.origin.crate_name, self.origin.item_path, self.property
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::AlgebraicSignature;

    #[test]
    fn associativity_builder_populates_metadata() {
        let obligation = Obligation::associativity(
            "option_add_assoc",
            Origin::new("karpal-algebra", "Semigroup for Sum<i32>"),
            Sort::Int,
            "combine",
        );

        assert_eq!(obligation.property, IsAssociative::NAME);
        assert_eq!(obligation.declarations.len(), 3);
        assert!(obligation.summary().contains("associativity"));
    }

    #[test]
    fn monoid_identity_builder_creates_conjunction() {
        let obligation = Obligation::monoid_identity(
            "sum_identity",
            Origin::new("karpal-core", "Monoid for Sum<i32>"),
            Sort::Int,
            "combine",
            "e",
        );

        assert_eq!(obligation.property, IsMonoid::NAME);
        assert!(matches!(obligation.conclusion, Term::And(_)));
    }

    #[test]
    fn signature_driven_group_builders_use_registered_roles() {
        let sig = AlgebraicSignature::group(Sort::Int, "combine", "e", "inv");
        let left = Obligation::left_inverse_in("left_inv", Origin::new("k", "G"), &sig);
        let right = Obligation::right_inverse_in("right_inv", Origin::new("k", "G"), &sig);

        assert_eq!(left.property, HasLeftInverse::NAME);
        assert_eq!(right.property, HasRightInverse::NAME);
    }

    #[test]
    fn lattice_absorption_creates_conjunction() {
        let sig = AlgebraicSignature::lattice(Sort::Int, "meet", "join");
        let obligation = Obligation::absorption_in("lattice_absorb", Origin::new("k", "L"), &sig);
        assert_eq!(obligation.property, IsAbsorptive::NAME);
        assert!(matches!(obligation.conclusion, Term::And(_)));
    }
}
