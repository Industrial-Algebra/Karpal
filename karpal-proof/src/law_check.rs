//! Runtime law-checking helpers.
//!
//! These functions verify algebraic laws at runtime using concrete values.
//! They are the building blocks that derive macros (Phase 11b) will call,
//! but are also usable standalone for ad-hoc verification.

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Check that `op` is associative for the given triple.
///
/// Returns `Ok(())` if `op(op(a, b), c) == op(a, op(b, c))`.
pub fn check_associativity<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    b: T,
    c: T,
    op: impl Fn(T, T) -> T,
) -> Result<(), LawViolation> {
    let left = op(op(a.clone(), b.clone()), c.clone());
    let right = op(a, op(b, c));
    if left == right {
        Ok(())
    } else {
        Err(LawViolation {
            law: "associativity",
            left: format!("{left:?}"),
            right: format!("{right:?}"),
        })
    }
}

/// Check that `e` is a left identity for `op`.
///
/// Returns `Ok(())` if `op(e, a) == a`.
pub fn check_left_identity<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    e: T,
    op: impl Fn(T, T) -> T,
) -> Result<(), LawViolation> {
    let result = op(e, a.clone());
    if result == a {
        Ok(())
    } else {
        Err(LawViolation {
            law: "left identity",
            left: format!("{result:?}"),
            right: format!("{a:?}"),
        })
    }
}

/// Check that `e` is a right identity for `op`.
///
/// Returns `Ok(())` if `op(a, e) == a`.
pub fn check_right_identity<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    e: T,
    op: impl Fn(T, T) -> T,
) -> Result<(), LawViolation> {
    let result = op(a.clone(), e);
    if result == a {
        Ok(())
    } else {
        Err(LawViolation {
            law: "right identity",
            left: format!("{result:?}"),
            right: format!("{a:?}"),
        })
    }
}

/// Check that `op` is commutative for the given pair.
///
/// Returns `Ok(())` if `op(a, b) == op(b, a)`.
pub fn check_commutativity<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    b: T,
    op: impl Fn(T, T) -> T,
) -> Result<(), LawViolation> {
    let left = op(a.clone(), b.clone());
    let right = op(b, a);
    if left == right {
        Ok(())
    } else {
        Err(LawViolation {
            law: "commutativity",
            left: format!("{left:?}"),
            right: format!("{right:?}"),
        })
    }
}

/// Check that `inv` produces a left inverse under `op` with identity `e`.
///
/// Returns `Ok(())` if `op(inv(a), a) == e`.
pub fn check_left_inverse<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    e: T,
    op: impl Fn(T, T) -> T,
    inv: impl Fn(T) -> T,
) -> Result<(), LawViolation> {
    let result = op(inv(a.clone()), a);
    if result == e {
        Ok(())
    } else {
        Err(LawViolation {
            law: "left inverse",
            left: format!("{result:?}"),
            right: format!("{e:?}"),
        })
    }
}

/// Check that `inv` produces a right inverse under `op` with identity `e`.
///
/// Returns `Ok(())` if `op(a, inv(a)) == e`.
pub fn check_right_inverse<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    e: T,
    op: impl Fn(T, T) -> T,
    inv: impl Fn(T) -> T,
) -> Result<(), LawViolation> {
    let result = op(a.clone(), inv(a));
    if result == e {
        Ok(())
    } else {
        Err(LawViolation {
            law: "right inverse",
            left: format!("{result:?}"),
            right: format!("{e:?}"),
        })
    }
}

/// Check left distributivity: `a * (b + c) == a*b + a*c`.
pub fn check_left_distributivity<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    b: T,
    c: T,
    add: impl Fn(T, T) -> T,
    mul: impl Fn(T, T) -> T,
) -> Result<(), LawViolation> {
    let left = mul(a.clone(), add(b.clone(), c.clone()));
    let right = add(mul(a.clone(), b), mul(a, c));
    if left == right {
        Ok(())
    } else {
        Err(LawViolation {
            law: "left distributivity",
            left: format!("{left:?}"),
            right: format!("{right:?}"),
        })
    }
}

/// Check right distributivity: `(a + b) * c == a*c + b*c`.
pub fn check_right_distributivity<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    b: T,
    c: T,
    add: impl Fn(T, T) -> T,
    mul: impl Fn(T, T) -> T,
) -> Result<(), LawViolation> {
    let left = mul(add(a.clone(), b.clone()), c.clone());
    let right = add(mul(a, c.clone()), mul(b, c));
    if left == right {
        Ok(())
    } else {
        Err(LawViolation {
            law: "right distributivity",
            left: format!("{left:?}"),
            right: format!("{right:?}"),
        })
    }
}

/// Check idempotency: `op(a, a) == a`.
pub fn check_idempotency<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    op: impl Fn(T, T) -> T,
) -> Result<(), LawViolation> {
    let result = op(a.clone(), a.clone());
    if result == a {
        Ok(())
    } else {
        Err(LawViolation {
            law: "idempotency",
            left: format!("{result:?}"),
            right: format!("{a:?}"),
        })
    }
}

/// Check absorption: `a ∧ (a ∨ b) == a`.
pub fn check_absorption<T: Clone + PartialEq + core::fmt::Debug>(
    a: T,
    b: T,
    meet: impl Fn(T, T) -> T,
    join: impl Fn(T, T) -> T,
) -> Result<(), LawViolation> {
    let result = meet(a.clone(), join(a.clone(), b));
    if result == a {
        Ok(())
    } else {
        Err(LawViolation {
            law: "absorption",
            left: format!("{result:?}"),
            right: format!("{a:?}"),
        })
    }
}

/// A law violation with debug output.
#[derive(Debug, Clone)]
pub struct LawViolation {
    /// Name of the violated law.
    pub law: &'static str,
    /// Debug representation of the left-hand side.
    pub left: String,
    /// Debug representation of the right-hand side.
    pub right: String,
}

impl core::fmt::Display for LawViolation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Law violation ({}): {} != {}",
            self.law, self.left, self.right
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn associativity_holds_for_addition() {
        assert!(check_associativity(1i32, 2, 3, |a, b| a.wrapping_add(b)).is_ok());
    }

    #[test]
    fn associativity_fails_for_subtraction() {
        assert!(check_associativity(10i32, 5, 3, |a, b| a - b).is_err());
    }

    #[test]
    fn identity_holds() {
        assert!(check_left_identity(5i32, 0, |a, b| a + b).is_ok());
        assert!(check_right_identity(5i32, 0, |a, b| a + b).is_ok());
    }

    #[test]
    fn commutativity_holds_for_addition() {
        assert!(check_commutativity(3i32, 7, |a, b| a.wrapping_add(b)).is_ok());
    }

    #[test]
    fn commutativity_fails_for_subtraction() {
        assert!(check_commutativity(3i32, 7, |a, b| a - b).is_err());
    }

    #[test]
    fn inverse_holds() {
        assert!(check_left_inverse(5i32, 0, |a, b| a + b, |a| -a).is_ok());
        assert!(check_right_inverse(5i32, 0, |a, b| a + b, |a| -a).is_ok());
    }

    #[test]
    fn distributivity_holds_for_integers() {
        assert!(
            check_left_distributivity(
                2i32,
                3,
                4,
                |a, b| a.wrapping_add(b),
                |a, b| a.wrapping_mul(b),
            )
            .is_ok()
        );
        assert!(
            check_right_distributivity(
                2i32,
                3,
                4,
                |a, b| a.wrapping_add(b),
                |a, b| a.wrapping_mul(b),
            )
            .is_ok()
        );
    }

    #[test]
    fn idempotency_holds_for_min() {
        assert!(check_idempotency(5i32, |a, b| a.min(b)).is_ok());
    }

    #[test]
    fn idempotency_fails_for_addition() {
        // 5 + 5 = 10 != 5
        assert!(check_idempotency(5i32, |a, b| a + b).is_err());
    }

    #[test]
    fn absorption_holds_for_bool() {
        assert!(check_absorption(true, false, |a, b| a && b, |a, b| a || b).is_ok());
        assert!(check_absorption(false, true, |a, b| a && b, |a, b| a || b).is_ok());
    }

    #[test]
    fn law_violation_display() {
        let v = LawViolation {
            law: "test",
            left: "1".into(),
            right: "2".into(),
        };
        assert_eq!(format!("{v}"), "Law violation (test): 1 != 2");
    }
}
