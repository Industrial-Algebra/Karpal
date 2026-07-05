// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Lifetime-aware contravariant functor hierarchy.
//!
//! The standard [`Contravariant`](crate::contravariant::Contravariant) trait
//! requires `A: 'static` because its canonical instance (`PredicateF`) uses
//! `Box<dyn Fn(A) -> bool>`, and `dyn` trait objects default to `'static`.
//!
//! This module provides a lifetime-parameterized alternative that supports
//! borrowed data. The trade-off is a slightly more complex HKT encoding:
//!
//! | Encoding | `'static` required | Borrowed data | API complexity |
//! |----------|-------------------|---------------|----------------|
//! | `HKT { type Of<T> }` | Yes | No | Simple |
//! | `HKTLt { type Of<'a, T> }` | No | Yes | Slightly more complex |
//!
//! # When to use which
//!
//! - Use [`Contravariant`](crate::contravariant::Contravariant) when your
//!   types are owned (e.g., `i32`, `String`, `Vec<T>`).
//! - Use [`ContravariantLt`] when you need to work with borrowed data
//!   (e.g., `&str`, `&[u8]`) or when your predicates reference short-lived
//!   data.
//!
//! # Example
//!
//! ```
//! use karpal_core::contravariant_lt::{HKTLt, ContravariantLt, PredicateFLt};
//!
//! let len_check: Box<dyn Fn(i32) -> bool> = Box::new(|n| n > 3);
//! // contramap from String to i32 (extract length)
//! let string_pred = PredicateFLt::contramap(len_check, |s: String| s.len() as i32);
//! assert!(string_pred(String::from("hello")));
//! assert!(!string_pred(String::from("hi")));
//! ```

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

/// Lifetime-parameterized HKT: `type Of<'a, T>` can express borrowed types.
///
/// Unlike [`HKT`](crate::hkt::HKT), this trait carries a lifetime parameter
/// `'a` through the associated type, allowing `Of<'a, T>` to contain
/// references with lifetime `'a`.
pub trait HKTLt {
    type Of<'a, T: 'a>;
}

/// Lifetime-aware contravariant functor.
///
/// This is the lifetime-parameterized counterpart of
/// [`Contravariant`](crate::contravariant::Contravariant). It lifts a
/// function `B -> A` into `F<A> -> F<B>` without requiring `A: 'static`.
///
/// # Laws
///
/// - Identity: `contramap(id, fa) == fa`
/// - Composition: `contramap(f . g, fa) == contramap(g, contramap(f, fa))`
pub trait ContravariantLt: HKTLt {
    fn contramap<'a, A: 'a, B: 'a>(fa: Self::Of<'a, A>, f: impl Fn(B) -> A + 'a)
    -> Self::Of<'a, B>;
}

/// Type constructor for lifetime-aware predicates: `Of<'a, T> = Box<dyn Fn(T) -> bool + 'a>`.
///
/// Unlike [`PredicateF`](crate::contravariant::PredicateF), this marker
/// supports borrowed types and non-`'static` captured data.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct PredicateFLt;

#[cfg(any(feature = "std", feature = "alloc"))]
impl HKTLt for PredicateFLt {
    type Of<'a, T: 'a> = Box<dyn Fn(T) -> bool + 'a>;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl ContravariantLt for PredicateFLt {
    fn contramap<'a, A: 'a, B: 'a>(
        fa: Self::Of<'a, A>,
        f: impl Fn(B) -> A + 'a,
    ) -> Self::Of<'a, B> {
        Box::new(move |b| fa(f(b)))
    }
}

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests {
    use super::*;

    #[test]
    fn predicate_owned_contramap() {
        let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
        let str_pred = PredicateFLt::contramap(is_positive, |s: &str| s.len() as i32);
        assert!(str_pred("hello"));
        assert!(!str_pred(""));
    }

    #[test]
    fn predicate_string_contramap() {
        let len_check: Box<dyn Fn(i32) -> bool> = Box::new(|n| n > 3);
        let string_pred = PredicateFLt::contramap(len_check, |s: String| s.len() as i32);
        assert!(string_pred(String::from("hello")));
        assert!(!string_pred(String::from("hi")));
    }

    #[test]
    fn predicate_composition() {
        let pred: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
        let f = |a: i16| a as i32;
        let g = |a: i16| a.wrapping_add(1);

        // contramap(f . g, pred)
        let pred2: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
        let left = PredicateFLt::contramap(pred2, move |a: i16| f(g(a)));

        // contramap(g, contramap(f, pred))
        let inner = PredicateFLt::contramap(pred, f);
        let right = PredicateFLt::contramap(inner, g);

        assert_eq!(left((5i16)), right((5i16)));
    }

    #[test]
    fn predicate_identity() {
        let pred: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
        let mapped = PredicateFLt::contramap(pred, |a: i32| a);
        assert!(mapped(42));
        assert!(!mapped(-1));
    }
}
