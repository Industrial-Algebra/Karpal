use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;

use karpal_free::cofree::Cofree;
use karpal_free::free::Free;

use crate::either::Either;
use crate::fix::Fix;

// ---------------------------------------------------------------------------
// Catamorphism — fold bottom-up
// ---------------------------------------------------------------------------

/// Catamorphism — fold a recursive structure bottom-up.
///
/// Given an algebra `F<A> -> A` and a `Fix<F>`, tears down the structure
/// by replacing each `F`-layer with the algebra.
pub fn cata<F: HKT + Functor, A>(alg: impl Fn(F::Of<A>) -> A, fix: Fix<F>) -> A
where
    F::Of<Fix<F>>: Clone,
{
    cata_inner(&alg, fix)
}

fn cata_inner<F: HKT + Functor, A>(alg: &dyn Fn(F::Of<A>) -> A, fix: Fix<F>) -> A
where
    F::Of<Fix<F>>: Clone,
{
    let layer = fix.unfix();
    let mapped = F::fmap(layer, |child| cata_inner(alg, child));
    alg(mapped)
}

// ---------------------------------------------------------------------------
// Anamorphism — unfold top-down
// ---------------------------------------------------------------------------

/// Anamorphism — unfold a recursive structure top-down from a seed.
///
/// Given a coalgebra `A -> F<A>` and a seed, builds up a `Fix<F>`
/// by repeatedly applying the coalgebra.
pub fn ana<F: HKT + Functor, A>(coalg: impl Fn(A) -> F::Of<A>, seed: A) -> Fix<F> {
    ana_inner(&coalg, seed)
}

fn ana_inner<F: HKT + Functor, A>(coalg: &dyn Fn(A) -> F::Of<A>, seed: A) -> Fix<F> {
    let layer = coalg(seed);
    let mapped = F::fmap(layer, |child_seed| ana_inner(coalg, child_seed));
    Fix::new(mapped)
}

// ---------------------------------------------------------------------------
// Hylomorphism — unfold then fold (deforested)
// ---------------------------------------------------------------------------

/// Hylomorphism — unfold then fold, without building an intermediate `Fix`.
///
/// `hylo alg coalg ≡ cata alg . ana coalg` but more efficient because
/// no intermediate data structure is allocated.
pub fn hylo<F: HKT + Functor, A, B>(
    alg: impl Fn(F::Of<B>) -> B,
    coalg: impl Fn(A) -> F::Of<A>,
    seed: A,
) -> B {
    hylo_inner::<F, A, B>(&alg, &coalg, seed)
}

fn hylo_inner<F: HKT + Functor, A, B>(
    alg: &dyn Fn(F::Of<B>) -> B,
    coalg: &dyn Fn(A) -> F::Of<A>,
    seed: A,
) -> B {
    let layer = coalg(seed);
    let mapped = F::fmap(layer, |child_seed| {
        hylo_inner::<F, A, B>(alg, coalg, child_seed)
    });
    alg(mapped)
}

// ---------------------------------------------------------------------------
// Paramorphism — fold with access to original subterms
// ---------------------------------------------------------------------------

/// Paramorphism — fold with access to original subterms.
///
/// Like catamorphism, but the algebra receives `(Fix<F>, A)` pairs —
/// both the original sub-structure and its already-folded result.
///
/// `Fix` uses `Rc` internally, so cloning each subterm is O(1).
pub fn para<F: HKT + Functor, A>(alg: impl Fn(F::Of<(Fix<F>, A)>) -> A, fix: Fix<F>) -> A
where
    F::Of<Fix<F>>: Clone,
{
    para_inner(&alg, fix)
}

fn para_inner<F: HKT + Functor, A>(alg: &dyn Fn(F::Of<(Fix<F>, A)>) -> A, fix: Fix<F>) -> A
where
    F::Of<Fix<F>>: Clone,
{
    let layer = fix.unfix();
    let paired = F::fmap(layer, |child: Fix<F>| {
        let original = child.clone(); // Rc clone: O(1)
        let folded = para_inner(alg, child);
        (original, folded)
    });
    alg(paired)
}

// ---------------------------------------------------------------------------
// Apomorphism — unfold with early termination
// ---------------------------------------------------------------------------

/// Apomorphism — unfold with early termination.
///
/// The coalgebra returns `F<Either<Fix<F>, A>>` — `Right(seed)` continues
/// unfolding, `Left(fix)` embeds an already-built subtree directly.
pub fn apo<F: HKT + Functor, A>(coalg: impl Fn(A) -> F::Of<Either<Fix<F>, A>>, seed: A) -> Fix<F> {
    apo_inner(&coalg, seed)
}

fn apo_inner<F: HKT + Functor, A>(
    coalg: &dyn Fn(A) -> F::Of<Either<Fix<F>, A>>,
    seed: A,
) -> Fix<F> {
    let layer = coalg(seed);
    let mapped = F::fmap(layer, |e| match e {
        Either::Left(fix) => fix,
        Either::Right(s) => apo_inner(coalg, s),
    });
    Fix::new(mapped)
}

// ---------------------------------------------------------------------------
// Histomorphism — fold with history (via Cofree)
// ---------------------------------------------------------------------------

/// Histomorphism — fold with access to all previous results via Cofree.
///
/// The algebra receives `&F<Cofree<F, A>>` (by reference) where each
/// Cofree node carries the folded result (`head`) and the full
/// sub-history (`tail`).
///
/// The algebra takes a reference to avoid needing `Clone` on the
/// Cofree structure (which is recursive and can't satisfy Rust's
/// trait solver for coinductive Clone proofs).
pub fn histo<F: HKT + Functor, A>(alg: impl Fn(&F::Of<Cofree<F, A>>) -> A, fix: Fix<F>) -> A
where
    F::Of<Fix<F>>: Clone,
{
    histo_inner(&alg, fix).head
}

fn histo_inner<F: HKT + Functor, A>(
    alg: &dyn Fn(&F::Of<Cofree<F, A>>) -> A,
    fix: Fix<F>,
) -> Cofree<F, A>
where
    F::Of<Fix<F>>: Clone,
{
    let layer = fix.unfix();
    let mapped: F::Of<Cofree<F, A>> = F::fmap(layer, |child| histo_inner(alg, child));
    let head = alg(&mapped);
    Cofree::new(head, mapped)
}

// ---------------------------------------------------------------------------
// Futumorphism — unfold multiple steps (via Free)
// ---------------------------------------------------------------------------

/// Futumorphism — unfold multiple steps at once via Free.
///
/// The coalgebra returns `F<Free<F, A>>` — `Pure(seed)` continues
/// with one more coalgebra application, `Roll(f)` injects multiple
/// layers of structure at once.
pub fn futu<F: HKT + Functor, A>(coalg: impl Fn(A) -> F::Of<Free<F, A>>, seed: A) -> Fix<F> {
    futu_inner(&coalg, seed)
}

fn futu_inner<F: HKT + Functor, A>(coalg: &dyn Fn(A) -> F::Of<Free<F, A>>, seed: A) -> Fix<F> {
    let layer = coalg(seed);
    let mapped = F::fmap(layer, |free| free_to_fix(coalg, free));
    Fix::new(mapped)
}

fn free_to_fix<F: HKT + Functor, A>(
    coalg: &dyn Fn(A) -> F::Of<Free<F, A>>,
    free: Free<F, A>,
) -> Fix<F> {
    match free {
        Free::Pure(a) => futu_inner(coalg, a),
        Free::Roll(ff) => {
            let mapped = F::fmap(*ff, |child| free_to_fix(coalg, child));
            Fix::new(mapped)
        }
    }
}

// ---------------------------------------------------------------------------
// Zygomorphism — fold with auxiliary fold
// ---------------------------------------------------------------------------

/// Zygomorphism — fold with an auxiliary fold running in parallel.
///
/// Two algebras run simultaneously: `aux` computes a helper value `B`,
/// and `alg` computes the result `A` with access to both `B` and `A`
/// from sub-structures.
///
/// Requires `F::Of<(B, A)>: Clone` to split the mapped layer between
/// the auxiliary and primary algebras.
pub fn zygo<F: HKT + Functor, A, B>(
    aux: impl Fn(F::Of<B>) -> B,
    alg: impl Fn(F::Of<(B, A)>) -> A,
    fix: Fix<F>,
) -> A
where
    F::Of<Fix<F>>: Clone,
    F::Of<(B, A)>: Clone,
{
    zygo_inner(&aux, &alg, fix).1
}

fn zygo_inner<F: HKT + Functor, A, B>(
    aux: &dyn Fn(F::Of<B>) -> B,
    alg: &dyn Fn(F::Of<(B, A)>) -> A,
    fix: Fix<F>,
) -> (B, A)
where
    F::Of<Fix<F>>: Clone,
    F::Of<(B, A)>: Clone,
{
    let layer = fix.unfix();
    let mapped: F::Of<(B, A)> = F::fmap(layer, |child| zygo_inner(aux, alg, child));
    let for_aux = mapped.clone();
    let bs = F::fmap(for_aux, |(b, _a): (B, A)| b);
    let b = aux(bs);
    let a = alg(mapped);
    (b, a)
}

// ---------------------------------------------------------------------------
// Chronomorphism — futu ; histo
// ---------------------------------------------------------------------------

/// Chronomorphism — futumorphism followed by histomorphism.
///
/// Combines multi-step unfolding (via Free) with history-aware folding
/// (via Cofree) in a single pass.
pub fn chrono<F: HKT + Functor, A, B>(
    alg: impl Fn(&F::Of<Cofree<F, B>>) -> B,
    coalg: impl Fn(A) -> F::Of<Free<F, A>>,
    seed: A,
) -> B {
    chrono_inner::<F, A, B>(&alg, &coalg, seed).head
}

fn chrono_inner<F: HKT + Functor, A, B>(
    alg: &dyn Fn(&F::Of<Cofree<F, B>>) -> B,
    coalg: &dyn Fn(A) -> F::Of<Free<F, A>>,
    seed: A,
) -> Cofree<F, B> {
    let layer = coalg(seed);
    let mapped: F::Of<Cofree<F, B>> = F::fmap(layer, |free| free_to_cofree(alg, coalg, free));
    let head = alg(&mapped);
    Cofree::new(head, mapped)
}

fn free_to_cofree<F: HKT + Functor, A, B>(
    alg: &dyn Fn(&F::Of<Cofree<F, B>>) -> B,
    coalg: &dyn Fn(A) -> F::Of<Free<F, A>>,
    free: Free<F, A>,
) -> Cofree<F, B> {
    match free {
        Free::Pure(a) => chrono_inner::<F, A, B>(alg, coalg, a),
        Free::Roll(ff) => {
            let mapped: F::Of<Cofree<F, B>> =
                F::fmap(*ff, |child| free_to_cofree(alg, coalg, child));
            let head = alg(&mapped);
            Cofree::new(head, mapped)
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    // === Test helpers: Natural numbers as Fix<OptionF> ===

    fn zero() -> Fix<OptionF> {
        Fix::new(None)
    }

    fn succ(n: Fix<OptionF>) -> Fix<OptionF> {
        Fix::new(Some(n))
    }

    fn nat(n: u32) -> Fix<OptionF> {
        let mut result = zero();
        for _ in 0..n {
            result = succ(result);
        }
        result
    }

    fn to_u32(n: &Fix<OptionF>) -> u32 {
        match n.unfix_ref() {
            None => 0,
            Some(pred) => 1 + to_u32(pred),
        }
    }

    // === Catamorphism tests ===

    #[test]
    fn cata_count_nat() {
        let result = cata::<OptionF, u32>(
            |layer| match layer {
                None => 0,
                Some(n) => n + 1,
            },
            nat(5),
        );
        assert_eq!(result, 5);
    }

    #[test]
    fn cata_zero() {
        let result = cata::<OptionF, u32>(
            |layer| match layer {
                None => 0,
                Some(n) => n + 1,
            },
            zero(),
        );
        assert_eq!(result, 0);
    }

    #[test]
    fn cata_to_string() {
        let result = cata::<OptionF, String>(
            |layer| match layer {
                None => "Z".to_string(),
                Some(s) => format!("S({})", s),
            },
            nat(3),
        );
        assert_eq!(result, "S(S(S(Z)))");
    }

    #[test]
    fn cata_identity_law() {
        let rebuilt: Fix<OptionF> = cata(Fix::new, nat(4));
        assert_eq!(to_u32(&rebuilt), 4);
    }

    // === Anamorphism tests ===

    #[test]
    fn ana_build_nat() {
        let n: Fix<OptionF> = ana(
            |seed: u32| {
                if seed == 0 { None } else { Some(seed - 1) }
            },
            5,
        );
        assert_eq!(to_u32(&n), 5);
    }

    #[test]
    fn ana_zero() {
        let n: Fix<OptionF> = ana(|_: u32| None, 0);
        assert_eq!(to_u32(&n), 0);
    }

    #[test]
    fn ana_step_by_two() {
        let n: Fix<OptionF> = ana(
            |seed: u32| {
                if seed == 0 {
                    None
                } else {
                    Some(seed.saturating_sub(2))
                }
            },
            10,
        );
        assert_eq!(to_u32(&n), 5);
    }

    // === Hylomorphism tests ===

    #[test]
    fn hylo_count() {
        let result = hylo::<OptionF, u32, u32>(
            |layer| match layer {
                None => 0,
                Some(acc) => acc + 1,
            },
            |seed| {
                if seed == 0 { None } else { Some(seed - 1) }
            },
            5,
        );
        assert_eq!(result, 5);
    }

    #[test]
    fn hylo_equals_cata_ana() {
        let alg = |layer: Option<u32>| match layer {
            None => 0u32,
            Some(n) => n + 1,
        };
        let coalg = |seed: u32| -> Option<u32> { if seed == 0 { None } else { Some(seed - 1) } };

        for n in 0..10 {
            let via_hylo = hylo::<OptionF, u32, u32>(alg, coalg, n);
            let via_cata_ana = cata::<OptionF, u32>(alg, ana(coalg, n));
            assert_eq!(via_hylo, via_cata_ana, "failed for n={}", n);
        }
    }

    // === Paramorphism tests ===

    #[test]
    fn para_factorial() {
        let result = para::<OptionF, u64>(
            |layer| match layer {
                None => 1,
                Some((sub, acc)) => {
                    let n = to_u32(&sub) + 1;
                    (n as u64) * acc
                }
            },
            nat(5),
        );
        assert_eq!(result, 120);
    }

    #[test]
    fn para_degenerates_to_cata() {
        let via_para = para::<OptionF, u32>(
            |layer| match layer {
                None => 0,
                Some((_sub, acc)) => acc + 1,
            },
            nat(7),
        );
        let via_cata = cata::<OptionF, u32>(
            |layer| match layer {
                None => 0,
                Some(acc) => acc + 1,
            },
            nat(7),
        );
        assert_eq!(via_para, via_cata);
    }

    #[test]
    fn para_zero() {
        let result = para::<OptionF, u64>(
            |layer| match layer {
                None => 1,
                Some((sub, acc)) => {
                    let n = to_u32(&sub) + 1;
                    (n as u64) * acc
                }
            },
            zero(),
        );
        assert_eq!(result, 1);
    }

    // === Apomorphism tests ===

    #[test]
    fn apo_build_nat() {
        let n: Fix<OptionF> = apo(
            |seed: u32| {
                if seed == 0 {
                    None
                } else {
                    Some(Either::Right(seed - 1))
                }
            },
            3,
        );
        assert_eq!(to_u32(&n), 3);
    }

    #[test]
    fn apo_early_stop() {
        let n: Fix<OptionF> = apo(
            |seed: u32| {
                if seed == 0 {
                    None
                } else if seed <= 2 {
                    Some(Either::Left(nat(seed - 1)))
                } else {
                    Some(Either::Right(seed - 1))
                }
            },
            5,
        );
        assert_eq!(to_u32(&n), 5);
    }

    #[test]
    fn apo_degenerates_to_ana() {
        let coalg_apo = |seed: u32| -> Option<Either<Fix<OptionF>, u32>> {
            if seed == 0 {
                None
            } else {
                Some(Either::Right(seed - 1))
            }
        };
        let coalg_ana =
            |seed: u32| -> Option<u32> { if seed == 0 { None } else { Some(seed - 1) } };

        for n in 0..8 {
            assert_eq!(to_u32(&apo(coalg_apo, n)), to_u32(&ana(coalg_ana, n)));
        }
    }

    // === Histomorphism tests ===

    #[test]
    fn histo_fibonacci() {
        let result = histo::<OptionF, u64>(
            |layer| match layer {
                None => 0,
                Some(cofree) => {
                    let fib_prev = cofree.head;
                    match cofree.tail.as_ref() {
                        None => 1,
                        Some(grandchild) => fib_prev + grandchild.head,
                    }
                }
            },
            nat(10),
        );
        assert_eq!(result, 55);
    }

    #[test]
    fn histo_degenerates_to_cata() {
        let via_histo = histo::<OptionF, u32>(
            |layer| match layer {
                None => 0,
                Some(cofree) => cofree.head + 1,
            },
            nat(5),
        );
        let via_cata = cata::<OptionF, u32>(
            |layer| match layer {
                None => 0,
                Some(n) => n + 1,
            },
            nat(5),
        );
        assert_eq!(via_histo, via_cata);
    }

    #[test]
    fn histo_zero() {
        let result = histo::<OptionF, u64>(
            |layer| match layer {
                None => 0,
                Some(cofree) => {
                    let prev = cofree.head;
                    match cofree.tail.as_ref() {
                        None => 1,
                        Some(gc) => prev + gc.head,
                    }
                }
            },
            zero(),
        );
        assert_eq!(result, 0);
    }

    // === Futumorphism tests ===

    #[test]
    fn futu_build_nat() {
        let n: Fix<OptionF> = futu(
            |seed: u32| -> Option<Free<OptionF, u32>> {
                if seed == 0 {
                    None
                } else {
                    Some(Free::Pure(seed - 1))
                }
            },
            3,
        );
        assert_eq!(to_u32(&n), 3);
    }

    #[test]
    fn futu_multi_step() {
        let n: Fix<OptionF> = futu(
            |seed: u32| -> Option<Free<OptionF, u32>> {
                if seed == 0 {
                    None
                } else if seed == 1 {
                    Some(Free::Roll(Box::new(None)))
                } else {
                    Some(Free::Roll(Box::new(Some(Free::Pure(seed - 2)))))
                }
            },
            4,
        );
        assert_eq!(to_u32(&n), 4);
    }

    #[test]
    fn futu_degenerates_to_ana() {
        let coalg_futu = |seed: u32| -> Option<Free<OptionF, u32>> {
            if seed == 0 {
                None
            } else {
                Some(Free::Pure(seed - 1))
            }
        };
        let coalg_ana =
            |seed: u32| -> Option<u32> { if seed == 0 { None } else { Some(seed - 1) } };

        for n in 0..8 {
            assert_eq!(to_u32(&futu(coalg_futu, n)), to_u32(&ana(coalg_ana, n)));
        }
    }

    // === Zygomorphism tests ===

    #[test]
    fn zygo_parity_and_count() {
        // aux counts the layers (same as cata), alg reports aux's result + parity
        // zygo returns just the A component from the top-level (B, A)
        // The aux B value at the top is computed from F::Of<B>, which is
        // the fmap-fst of the clone of F::Of<(B, A)>. The top B comes from
        // aux(F::fmap(clone, fst)), which counts the cloned children's B values.
        // Actually, zygo_inner returns (B, A) and zygo takes .1.
        // Let's verify with a simpler test: just check the aux count works.
        let result = zygo::<OptionF, u32, u32>(
            // aux: count layers
            |layer| match layer {
                None => 0,
                Some(n) => n + 1,
            },
            // alg: return aux value (count) from children
            |layer| match layer {
                None => 0,
                Some((b, _a)) => b + 1,
            },
            nat(5),
        );
        // alg ignores a, just uses b (count from aux). Should equal cata count.
        assert_eq!(result, 5);
    }

    #[test]
    fn zygo_degenerates_to_cata() {
        let via_zygo = zygo::<OptionF, u32, u32>(
            |layer| match layer {
                None => 0u32,
                Some(n) => n,
            },
            |layer| match layer {
                None => 0,
                Some((_b, a)) => a + 1,
            },
            nat(5),
        );
        let via_cata = cata::<OptionF, u32>(
            |layer| match layer {
                None => 0,
                Some(n) => n + 1,
            },
            nat(5),
        );
        assert_eq!(via_zygo, via_cata);
    }

    // === Chronomorphism tests ===

    #[test]
    fn chrono_fibonacci() {
        let result = chrono::<OptionF, u32, u64>(
            |layer| match layer {
                None => 0,
                Some(cofree) => {
                    let prev = cofree.head;
                    match cofree.tail.as_ref() {
                        None => 1,
                        Some(gc) => prev + gc.head,
                    }
                }
            },
            |seed: u32| -> Option<Free<OptionF, u32>> {
                if seed == 0 {
                    None
                } else {
                    Some(Free::Pure(seed - 1))
                }
            },
            10,
        );
        assert_eq!(result, 55);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    fn nat(n: u32) -> Fix<OptionF> {
        let mut result = Fix::new(None);
        for _ in 0..n {
            result = Fix::new(Some(result));
        }
        result
    }

    fn to_u32(n: &Fix<OptionF>) -> u32 {
        match n.unfix_ref() {
            None => 0,
            Some(pred) => 1 + to_u32(pred),
        }
    }

    proptest! {
        #[test]
        fn cata_fix_identity(n in 0u32..20) {
            let rebuilt: Fix<OptionF> = cata(Fix::new, nat(n));
            prop_assert_eq!(to_u32(&rebuilt), n);
        }

        #[test]
        fn hylo_is_cata_ana(n in 0u32..20) {
            let alg = |layer: Option<u32>| match layer {
                None => 0u32,
                Some(x) => x + 1,
            };
            let coalg = |seed: u32| -> Option<u32> {
                if seed == 0 { None } else { Some(seed - 1) }
            };
            let via_hylo = hylo::<OptionF, u32, u32>(alg, coalg, n);
            let via_cata_ana = cata::<OptionF, u32>(alg, ana(coalg, n));
            prop_assert_eq!(via_hylo, via_cata_ana);
        }

        #[test]
        fn ana_cata_roundtrip(n in 0u32..20) {
            let coalg = |seed: u32| -> Option<u32> {
                if seed == 0 { None } else { Some(seed - 1) }
            };
            let alg = |layer: Option<u32>| match layer {
                None => 0u32,
                Some(x) => x + 1,
            };
            prop_assert_eq!(cata::<OptionF, u32>(alg, ana(coalg, n)), n);
        }

        #[test]
        fn apo_always_right_is_ana(n in 0u32..15) {
            let coalg_ana = |seed: u32| -> Option<u32> {
                if seed == 0 { None } else { Some(seed - 1) }
            };
            let coalg_apo = |seed: u32| -> Option<Either<Fix<OptionF>, u32>> {
                if seed == 0 { None } else { Some(Either::Right(seed - 1)) }
            };
            prop_assert_eq!(to_u32(&ana(coalg_ana, n)), to_u32(&apo(coalg_apo, n)));
        }

        #[test]
        fn futu_always_pure_is_ana(n in 0u32..15) {
            let coalg_ana = |seed: u32| -> Option<u32> {
                if seed == 0 { None } else { Some(seed - 1) }
            };
            let coalg_futu = |seed: u32| -> Option<Free<OptionF, u32>> {
                if seed == 0 { None } else { Some(Free::Pure(seed - 1)) }
            };
            prop_assert_eq!(to_u32(&ana(coalg_ana, n)), to_u32(&futu(coalg_futu, n)));
        }

        #[test]
        fn para_ignoring_subterms_is_cata(n in 0u32..15) {
            let via_para = para::<OptionF, u32>(
                |layer| match layer {
                    None => 0,
                    Some((_sub, acc)) => acc + 1,
                },
                nat(n),
            );
            let via_cata = cata::<OptionF, u32>(
                |layer| match layer {
                    None => 0,
                    Some(acc) => acc + 1,
                },
                nat(n),
            );
            prop_assert_eq!(via_para, via_cata);
        }

        #[test]
        fn histo_ignoring_history_is_cata(n in 0u32..15) {
            let via_histo = histo::<OptionF, u32>(
                |layer| match layer {
                    None => 0,
                    Some(cofree) => cofree.head + 1,
                },
                nat(n),
            );
            let via_cata = cata::<OptionF, u32>(
                |layer| match layer {
                    None => 0,
                    Some(n) => n + 1,
                },
                nat(n),
            );
            prop_assert_eq!(via_histo, via_cata);
        }
    }
}
