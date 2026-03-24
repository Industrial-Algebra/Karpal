#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::String};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, string::String};

use crate::obligation::Sort;

/// A named binary operator in an algebraic signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinarySymbol {
    pub name: String,
}

impl BinarySymbol {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// A named unary operator in an algebraic signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnarySymbol {
    pub name: String,
}

impl UnarySymbol {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// A named constant in an algebraic signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantSymbol {
    pub name: String,
}

impl ConstantSymbol {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Backend-agnostic signature for an algebraic structure.
///
/// This lets Phase 12 build proof obligations from a reusable semantic model
/// instead of ad-hoc raw strings at each export site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgebraicSignature {
    pub carrier: Sort,
    binary: BTreeMap<String, BinarySymbol>,
    unary: BTreeMap<String, UnarySymbol>,
    constants: BTreeMap<String, ConstantSymbol>,
}

impl AlgebraicSignature {
    pub fn new(carrier: Sort) -> Self {
        Self {
            carrier,
            binary: BTreeMap::new(),
            unary: BTreeMap::new(),
            constants: BTreeMap::new(),
        }
    }

    pub fn semigroup(carrier: Sort, combine: impl Into<String>) -> Self {
        Self::new(carrier).with_binary("combine", combine)
    }

    pub fn monoid(carrier: Sort, combine: impl Into<String>, identity: impl Into<String>) -> Self {
        Self::semigroup(carrier, combine).with_constant("identity", identity)
    }

    pub fn group(
        carrier: Sort,
        combine: impl Into<String>,
        identity: impl Into<String>,
        inverse: impl Into<String>,
    ) -> Self {
        Self::monoid(carrier, combine, identity).with_unary("inverse", inverse)
    }

    pub fn semiring(
        carrier: Sort,
        add: impl Into<String>,
        mul: impl Into<String>,
        zero: impl Into<String>,
        one: impl Into<String>,
    ) -> Self {
        Self::new(carrier)
            .with_binary("add", add)
            .with_binary("mul", mul)
            .with_constant("zero", zero)
            .with_constant("one", one)
    }

    pub fn lattice(carrier: Sort, meet: impl Into<String>, join: impl Into<String>) -> Self {
        Self::new(carrier)
            .with_binary("meet", meet)
            .with_binary("join", join)
    }

    pub fn with_binary(mut self, role: impl Into<String>, symbol: impl Into<String>) -> Self {
        self.binary.insert(role.into(), BinarySymbol::new(symbol));
        self
    }

    pub fn with_unary(mut self, role: impl Into<String>, symbol: impl Into<String>) -> Self {
        self.unary.insert(role.into(), UnarySymbol::new(symbol));
        self
    }

    pub fn with_constant(mut self, role: impl Into<String>, symbol: impl Into<String>) -> Self {
        self.constants
            .insert(role.into(), ConstantSymbol::new(symbol));
        self
    }

    pub fn binary(&self, role: &str) -> Option<&str> {
        self.binary.get(role).map(|symbol| symbol.name.as_str())
    }

    pub fn unary(&self, role: &str) -> Option<&str> {
        self.unary.get(role).map(|symbol| symbol.name.as_str())
    }

    pub fn constant(&self, role: &str) -> Option<&str> {
        self.constants.get(role).map(|symbol| symbol.name.as_str())
    }

    pub fn require_binary(&self, role: &str) -> &str {
        self.binary(role)
            .unwrap_or_else(|| panic!("missing binary symbol for role `{role}`"))
    }

    pub fn require_unary(&self, role: &str) -> &str {
        self.unary(role)
            .unwrap_or_else(|| panic!("missing unary symbol for role `{role}`"))
    }

    pub fn require_constant(&self, role: &str) -> &str {
        self.constant(role)
            .unwrap_or_else(|| panic!("missing constant symbol for role `{role}`"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_signature_registers_expected_roles() {
        let sig = AlgebraicSignature::group(Sort::Int, "combine", "e", "inv");
        assert_eq!(sig.require_binary("combine"), "combine");
        assert_eq!(sig.require_constant("identity"), "e");
        assert_eq!(sig.require_unary("inverse"), "inv");
    }

    #[test]
    fn semiring_signature_registers_expected_roles() {
        let sig = AlgebraicSignature::semiring(Sort::Int, "add", "mul", "zero", "one");
        assert_eq!(sig.require_binary("add"), "add");
        assert_eq!(sig.require_binary("mul"), "mul");
        assert_eq!(sig.require_constant("zero"), "zero");
        assert_eq!(sig.require_constant("one"), "one");
    }
}
