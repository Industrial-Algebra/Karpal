//! Derive macros for automatic algebraic law verification.
//!
//! These macros generate `#[cfg(test)]` modules containing proptest-based
//! property tests that verify algebraic laws for user-defined types.
//!
//! # Usage
//!
//! ```ignore
//! use karpal_proof_derive::VerifySemigroup;
//!
//! #[derive(Clone, Debug, PartialEq, VerifySemigroup)]
//! #[verify(strategy = "0u32..100")]
//! struct MyWrapper(u32);
//! ```
//!
//! # Attributes
//!
//! - `#[verify(strategy = "...")]` — **Required**. A proptest strategy expression
//!   that generates values of the annotated type. Examples:
//!   - `"0u32..100"` for numeric ranges
//!   - `"any::<MyType>()"` for types implementing `Arbitrary`
//!   - `"(0i32..50).prop_map(MyWrapper)"` for newtype wrappers
//!
//! - `#[verify(epsilon = "1e-10")]` — Optional. Use approximate floating-point
//!   comparison instead of exact `PartialEq`. The generated tests will use
//!   `(left - right).abs() < epsilon` instead of `prop_assert_eq!`.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Expr, parse_macro_input};

/// Extract a string-valued attribute from `#[verify(key = "...")]`.
fn extract_verify_attr(input: &DeriveInput, key: &str) -> Option<proc_macro2::TokenStream> {
    for attr in &input.attrs {
        if !attr.path().is_ident("verify") {
            continue;
        }
        let Ok(nested) = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        ) else {
            continue;
        };
        for meta in &nested {
            if let syn::Meta::NameValue(nv) = meta
                && nv.path.is_ident(key)
                && let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
            {
                let tokens: proc_macro2::TokenStream = s
                    .value()
                    .parse()
                    .unwrap_or_else(|_| panic!("{key} must be a valid Rust expression"));
                return Some(tokens);
            }
        }
    }
    None
}

/// Extract the `strategy` string from `#[verify(strategy = "...")]`.
fn extract_strategy(input: &DeriveInput) -> proc_macro2::TokenStream {
    extract_verify_attr(input, "strategy")
        .expect("#[verify(strategy = \"...\")] attribute is required")
}

/// Extract the optional `epsilon` string from `#[verify(epsilon = "...")]`.
fn extract_epsilon(input: &DeriveInput) -> Option<proc_macro2::TokenStream> {
    extract_verify_attr(input, "epsilon")
}

/// Generate an assertion: either exact or approximate.
fn make_assert(
    epsilon: &Option<proc_macro2::TokenStream>,
    left: proc_macro2::TokenStream,
    right: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    if let Some(eps) = epsilon {
        // For types that support subtraction and abs (floats)
        quote! {
            let __left = #left;
            let __right = #right;
            prop_assert!(
                (__left - __right).abs() < #eps,
                "expected {:?} ≈ {:?} (epsilon {})", __left, __right, #eps
            );
        }
    } else {
        quote! {
            prop_assert_eq!(#left, #right);
        }
    }
}

/// Derive `VerifySemigroup`: generates proptest for associativity of `combine`.
///
/// Requires: `T: Semigroup + Clone + Debug + PartialEq`
#[proc_macro_derive(VerifySemigroup, attributes(verify))]
pub fn derive_verify_semigroup(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let strategy = extract_strategy(&input);
    let epsilon = extract_epsilon(&input);
    let mod_name = format_ident!("verify_semigroup_{}", name.to_string().to_lowercase());

    let assert_assoc = make_assert(
        &epsilon,
        quote! { a.clone().combine(b.clone()).combine(c.clone()) },
        quote! { a.combine(b.combine(c)) },
    );

    let output = quote! {
        #[cfg(test)]
        mod #mod_name {
            use super::*;
            use karpal_core::Semigroup;
            use proptest::prelude::*;

            proptest! {
                #[test]
                fn associativity(a in #strategy, b in #strategy, c in #strategy) {
                    #assert_assoc
                }
            }
        }
    };
    output.into()
}

/// Derive `VerifyMonoid`: generates proptest for left/right identity of `empty`.
///
/// Requires: `T: Monoid + Clone + Debug + PartialEq`
#[proc_macro_derive(VerifyMonoid, attributes(verify))]
pub fn derive_verify_monoid(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let strategy = extract_strategy(&input);
    let epsilon = extract_epsilon(&input);
    let mod_name = format_ident!("verify_monoid_{}", name.to_string().to_lowercase());

    let assert_left = make_assert(
        &epsilon,
        quote! { <#name as Monoid>::empty().combine(a.clone()) },
        quote! { a },
    );

    let assert_right = make_assert(
        &epsilon,
        quote! { a.clone().combine(<#name as Monoid>::empty()) },
        quote! { a },
    );

    let output = quote! {
        #[cfg(test)]
        mod #mod_name {
            use super::*;
            use karpal_core::{Semigroup, Monoid};
            use proptest::prelude::*;

            proptest! {
                #[test]
                fn left_identity(a in #strategy) {
                    #assert_left
                }

                #[test]
                fn right_identity(a in #strategy) {
                    #assert_right
                }
            }
        }
    };
    output.into()
}

/// Derive `VerifyGroup`: generates proptest for left/right inverse.
///
/// Requires: `T: Group + Clone + Debug + PartialEq`
#[proc_macro_derive(VerifyGroup, attributes(verify))]
pub fn derive_verify_group(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let strategy = extract_strategy(&input);
    let epsilon = extract_epsilon(&input);
    let mod_name = format_ident!("verify_group_{}", name.to_string().to_lowercase());

    let assert_left = make_assert(
        &epsilon,
        quote! { a.clone().invert().combine(a.clone()) },
        quote! { <#name as Monoid>::empty() },
    );

    let assert_right = make_assert(
        &epsilon,
        quote! { a.clone().combine(a.clone().invert()) },
        quote! { <#name as Monoid>::empty() },
    );

    let output = quote! {
        #[cfg(test)]
        mod #mod_name {
            use super::*;
            use karpal_core::{Semigroup, Monoid};
            use karpal_algebra::Group;
            use proptest::prelude::*;

            proptest! {
                #[test]
                fn left_inverse(a in #strategy) {
                    #assert_left
                }

                #[test]
                fn right_inverse(a in #strategy) {
                    #assert_right
                }
            }
        }
    };
    output.into()
}

/// Derive `VerifyCommutative`: generates proptest for commutativity of `combine`.
///
/// Requires: `T: Semigroup + Clone + Debug + PartialEq`
#[proc_macro_derive(VerifyCommutative, attributes(verify))]
pub fn derive_verify_commutative(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let strategy = extract_strategy(&input);
    let epsilon = extract_epsilon(&input);
    let mod_name = format_ident!("verify_commutative_{}", name.to_string().to_lowercase());

    let assert_comm = make_assert(
        &epsilon,
        quote! { a.clone().combine(b.clone()) },
        quote! { b.combine(a) },
    );

    let output = quote! {
        #[cfg(test)]
        mod #mod_name {
            use super::*;
            use karpal_core::Semigroup;
            use proptest::prelude::*;

            proptest! {
                #[test]
                fn commutativity(a in #strategy, b in #strategy) {
                    #assert_comm
                }
            }
        }
    };
    output.into()
}

/// Derive `VerifySemiring`: generates proptests for additive monoid,
/// multiplicative monoid, distributivity, and zero annihilation.
///
/// Requires: `T: Semiring + Clone + Debug + PartialEq`
#[proc_macro_derive(VerifySemiring, attributes(verify))]
pub fn derive_verify_semiring(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let strategy = extract_strategy(&input);
    let epsilon = extract_epsilon(&input);
    let mod_name = format_ident!("verify_semiring_{}", name.to_string().to_lowercase());

    let assert_add_assoc = make_assert(
        &epsilon,
        quote! { a.clone().add(b.clone()).add(c.clone()) },
        quote! { a.clone().add(b.clone().add(c.clone())) },
    );

    let assert_add_comm = make_assert(
        &epsilon,
        quote! { a.clone().add(b.clone()) },
        quote! { b.clone().add(a.clone()) },
    );

    let assert_add_left_id = make_assert(
        &epsilon,
        quote! { <#name as Semiring>::zero().add(a.clone()) },
        quote! { a.clone() },
    );

    let assert_add_right_id = make_assert(
        &epsilon,
        quote! { a.clone().add(<#name as Semiring>::zero()) },
        quote! { a.clone() },
    );

    let assert_mul_assoc = make_assert(
        &epsilon,
        quote! { a.clone().mul(b.clone()).mul(c.clone()) },
        quote! { a.clone().mul(b.clone().mul(c.clone())) },
    );

    let assert_mul_left_id = make_assert(
        &epsilon,
        quote! { <#name as Semiring>::one().mul(a.clone()) },
        quote! { a.clone() },
    );

    let assert_mul_right_id = make_assert(
        &epsilon,
        quote! { a.clone().mul(<#name as Semiring>::one()) },
        quote! { a.clone() },
    );

    let assert_left_dist = make_assert(
        &epsilon,
        quote! { a.clone().mul(b.clone().add(c.clone())) },
        quote! { a.clone().mul(b.clone()).add(a.clone().mul(c.clone())) },
    );

    let assert_right_dist = make_assert(
        &epsilon,
        quote! { a.clone().add(b.clone()).mul(c.clone()) },
        quote! { a.clone().mul(c.clone()).add(b.clone().mul(c.clone())) },
    );

    let assert_zero_left = make_assert(
        &epsilon,
        quote! { <#name as Semiring>::zero().mul(a.clone()) },
        quote! { <#name as Semiring>::zero() },
    );

    let assert_zero_right = make_assert(
        &epsilon,
        quote! { a.clone().mul(<#name as Semiring>::zero()) },
        quote! { <#name as Semiring>::zero() },
    );

    let output = quote! {
        #[cfg(test)]
        mod #mod_name {
            use super::*;
            use karpal_algebra::Semiring;
            use proptest::prelude::*;

            proptest! {
                // Additive commutative monoid
                #[test]
                fn add_associativity(a in #strategy, b in #strategy, c in #strategy) {
                    #assert_add_assoc
                }

                #[test]
                fn add_commutativity(a in #strategy, b in #strategy) {
                    #assert_add_comm
                }

                #[test]
                fn add_identity(a in #strategy) {
                    #assert_add_left_id
                    #assert_add_right_id
                }

                // Multiplicative monoid
                #[test]
                fn mul_associativity(a in #strategy, b in #strategy, c in #strategy) {
                    #assert_mul_assoc
                }

                #[test]
                fn mul_identity(a in #strategy) {
                    #assert_mul_left_id
                    #assert_mul_right_id
                }

                // Distributivity
                #[test]
                fn left_distributivity(a in #strategy, b in #strategy, c in #strategy) {
                    #assert_left_dist
                }

                #[test]
                fn right_distributivity(a in #strategy, b in #strategy, c in #strategy) {
                    #assert_right_dist
                }

                // Zero annihilation
                #[test]
                fn zero_annihilation(a in #strategy) {
                    #assert_zero_left
                    #assert_zero_right
                }
            }
        }
    };
    output.into()
}

/// Derive `VerifyRing`: generates proptest for additive inverse (negate).
///
/// Requires: `T: Ring + Clone + Debug + PartialEq`
#[proc_macro_derive(VerifyRing, attributes(verify))]
pub fn derive_verify_ring(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let strategy = extract_strategy(&input);
    let epsilon = extract_epsilon(&input);
    let mod_name = format_ident!("verify_ring_{}", name.to_string().to_lowercase());

    let assert_left = make_assert(
        &epsilon,
        quote! { a.clone().negate().add(a.clone()) },
        quote! { <#name as Semiring>::zero() },
    );

    let assert_right = make_assert(
        &epsilon,
        quote! { a.clone().add(a.clone().negate()) },
        quote! { <#name as Semiring>::zero() },
    );

    let output = quote! {
        #[cfg(test)]
        mod #mod_name {
            use super::*;
            use karpal_algebra::{Semiring, Ring};
            use proptest::prelude::*;

            proptest! {
                #[test]
                fn left_additive_inverse(a in #strategy) {
                    #assert_left
                }

                #[test]
                fn right_additive_inverse(a in #strategy) {
                    #assert_right
                }
            }
        }
    };
    output.into()
}

/// Derive `VerifyLattice`: generates proptests for associativity, commutativity,
/// idempotency, and absorption of `meet`/`join`.
///
/// Requires: `T: Lattice + Clone + Debug + PartialEq`
#[proc_macro_derive(VerifyLattice, attributes(verify))]
pub fn derive_verify_lattice(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let strategy = extract_strategy(&input);
    let mod_name = format_ident!("verify_lattice_{}", name.to_string().to_lowercase());

    // Lattice laws don't use epsilon — meet/join should be exact
    let output = quote! {
        #[cfg(test)]
        mod #mod_name {
            use super::*;
            use karpal_algebra::Lattice;
            use proptest::prelude::*;

            proptest! {
                // Associativity
                #[test]
                fn join_associativity(a in #strategy, b in #strategy, c in #strategy) {
                    prop_assert_eq!(
                        a.clone().join(b.clone()).join(c.clone()),
                        a.join(b.join(c))
                    );
                }

                #[test]
                fn meet_associativity(a in #strategy, b in #strategy, c in #strategy) {
                    prop_assert_eq!(
                        a.clone().meet(b.clone()).meet(c.clone()),
                        a.meet(b.meet(c))
                    );
                }

                // Commutativity
                #[test]
                fn join_commutativity(a in #strategy, b in #strategy) {
                    prop_assert_eq!(a.clone().join(b.clone()), b.join(a));
                }

                #[test]
                fn meet_commutativity(a in #strategy, b in #strategy) {
                    prop_assert_eq!(a.clone().meet(b.clone()), b.meet(a));
                }

                // Idempotency
                #[test]
                fn join_idempotency(a in #strategy) {
                    prop_assert_eq!(a.clone().join(a.clone()), a);
                }

                #[test]
                fn meet_idempotency(a in #strategy) {
                    prop_assert_eq!(a.clone().meet(a.clone()), a);
                }

                // Absorption
                #[test]
                fn absorption_join_meet(a in #strategy, b in #strategy) {
                    prop_assert_eq!(a.clone().join(a.clone().meet(b)), a);
                }

                #[test]
                fn absorption_meet_join(a in #strategy, b in #strategy) {
                    prop_assert_eq!(a.clone().meet(a.clone().join(b)), a);
                }
            }
        }
    };
    output.into()
}
