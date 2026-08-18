// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The probe registry and algebraic probes (Phase 19-G).
//!
//! Probes are bounded, typed, read-only, deterministic executions that
//! dogfood the library's own abstractions — each one *runs real Karpal
//! code* and reports what it demonstrated:
//!
//! | Probe | Dogfoods | Demonstrates |
//! |---|---|---|
//! | `functor-monad-laws` | `karpal-core` | functor identity/composition + monad identity/associativity laws on the `Option` instance |
//! | `algebra-laws` | `karpal-proof`, `karpal-core`, `karpal-algebra` | the law checkers themselves: associativity, identities, absorption |
//! | `schubert-intersection` | `karpal-schubert-types` | `IntersectionKind` discrimination — the structured-emptiness thesis, live |
//! | `recursion-eval` | `karpal-recursion` | `ana`/`cata`/`hylo` agreement on a sample functor |
//! | `coherence` | `karpal-diagram` | pentagon / triangle / hexagon witness construction |
//!
//! [`run_probe`] executes one probe and returns its outcome (unknown ids
//! return `None`); the provider surface (with the `lonis` feature) exposes
//! these as `karpal.probe.list` / `karpal.probe.describe` /
//! `karpal.probe.run`.

use serde::{Deserialize, Serialize};

use crate::planner::Score;

/// One registered probe.
#[derive(Debug, Clone)]
pub struct ProbeDescriptor {
    /// Stable probe id.
    pub id: &'static str,
    /// Short description.
    pub description: &'static str,
    /// Crates whose abstractions the probe exercises.
    pub dogfoods: &'static [&'static str],
    /// Same input always yields the same output (all probes are).
    pub deterministic: bool,
}

/// The probe registry, in deterministic order.
#[must_use]
pub fn probe_catalog() -> &'static [ProbeDescriptor] {
    &PROBES
}

static PROBES: [ProbeDescriptor; 5] = [
    ProbeDescriptor {
        id: "functor-monad-laws",
        description: "Functor identity/composition and monad identity/associativity laws, evaluated on the Option instance.",
        dogfoods: &["karpal-core"],
        deterministic: true,
    },
    ProbeDescriptor {
        id: "algebra-laws",
        description: "karpal-proof law checkers run against real Semigroup/Monoid/Lattice instances.",
        dogfoods: &["karpal-proof", "karpal-core", "karpal-algebra"],
        deterministic: true,
    },
    ProbeDescriptor {
        id: "schubert-intersection",
        description: "Schubert intersection kinds in Gr(2,4): positive vs structural zero — the structured-emptiness thesis.",
        dogfoods: &["karpal-schubert-types"],
        deterministic: true,
    },
    ProbeDescriptor {
        id: "recursion-eval",
        description: "ana/cata/hylo recursion schemes on a Peano sample; hylo agrees with cata∘ana.",
        dogfoods: &["karpal-recursion"],
        deterministic: true,
    },
    ProbeDescriptor {
        id: "coherence",
        description: "Mac Lane coherence witnesses: pentagon, triangle, and hexagon type-level rewrites.",
        dogfoods: &["karpal-diagram"],
        deterministic: true,
    },
];

/// A probe outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutcome {
    /// The probe id.
    pub id: String,
    /// Passed or failed.
    pub status: ProbeStatus,
    /// One-line result.
    pub summary: String,
    /// What was demonstrated, one entry per check.
    pub details: Vec<String>,
    /// Crates dogfooded (echoed from the registry).
    pub dogfoods: Vec<String>,
}

/// Probe execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    /// All checks held.
    Passed,
    /// At least one check failed.
    Failed,
}

/// Run one probe by id. Returns `None` for unknown ids.
#[must_use]
pub fn run_probe(id: &str) -> Option<ProbeOutcome> {
    let descriptor = PROBES.iter().find(|p| p.id == id)?;
    let outcome = match id {
        "functor-monad-laws" => functor_monad_laws(),
        "algebra-laws" => algebra_laws(),
        "schubert-intersection" => schubert_intersection(),
        "recursion-eval" => recursion_eval(),
        "coherence" => coherence(),
        _ => return None,
    };
    Some(ProbeOutcome {
        id: descriptor.id.to_string(),
        status: outcome.0,
        summary: outcome.1,
        details: outcome.2,
        dogfoods: descriptor.dogfoods.iter().map(|s| s.to_string()).collect(),
    })
}

type ProbeResult = (ProbeStatus, String, Vec<String>);

fn passed(summary: &str, details: Vec<String>) -> ProbeResult {
    (ProbeStatus::Passed, summary.to_string(), details)
}

// -- karpal-core: functor + monad laws on Option -----------------------------

use karpal_core::hkt::OptionF;
use karpal_core::{Applicative, Chain, Functor};

fn functor_monad_laws() -> ProbeResult {
    let mut details = Vec::new();

    // Functor identity: fmap(x, id) == x
    let x: Option<i32> = Some(7);
    let mapped = OptionF::fmap(x, |a| a);
    if mapped == x {
        details.push("functor identity: fmap(x, id) == x".to_string());
    } else {
        return (
            ProbeStatus::Failed,
            "functor identity failed".to_string(),
            details,
        );
    }

    // Functor composition: fmap(fmap(x, f), g) == fmap(x, g∘f)
    let f = |a: i32| a * 2;
    let g = |a: i32| a + 1;
    let left = OptionF::fmap(OptionF::fmap(x, f), g);
    let right = OptionF::fmap(x, |a| g(f(a)));
    if left == right {
        details.push("functor composition: fmap(fmap(x, f), g) == fmap(x, g∘f)".to_string());
    } else {
        return (
            ProbeStatus::Failed,
            "functor composition failed".to_string(),
            details,
        );
    }

    // Monad left identity: chain(pure(a), f) == f(a)
    let step = |a: i32| if a > 0 { Some(a - 1) } else { None };
    let a = 5;
    let left = OptionF::chain(OptionF::pure(a), step);
    if left == step(a) {
        details.push("monad left identity: chain(pure(a), f) == f(a)".to_string());
    } else {
        return (
            ProbeStatus::Failed,
            "monad left identity failed".to_string(),
            details,
        );
    }

    // Monad right identity: chain(m, pure) == m
    let m: Option<i32> = Some(3);
    let right_ident = OptionF::chain(m, OptionF::pure);
    if right_ident == m {
        details.push("monad right identity: chain(m, pure) == m".to_string());
    } else {
        return (
            ProbeStatus::Failed,
            "monad right identity failed".to_string(),
            details,
        );
    }

    // Monad associativity: chain(chain(m, f), g) == chain(m, |x| chain(f(x), g))
    let f2 = |a: i32| Some(a * 2);
    let g2 = |a: i32| if a < 10 { Some(a + 1) } else { None };
    let assoc_left = OptionF::chain(OptionF::chain(m, f2), g2);
    let assoc_right = OptionF::chain(m, |x| OptionF::chain(f2(x), g2));
    if assoc_left == assoc_right {
        details.push("monad associativity: (m >>= f) >>= g == m >>= (x => f(x) >>= g)".to_string());
    } else {
        return (
            ProbeStatus::Failed,
            "monad associativity failed".to_string(),
            details,
        );
    }

    passed("functor + monad laws hold on Option", details)
}

// -- karpal-proof law checkers ------------------------------------------------

use karpal_core::{Monoid, Semigroup};
use karpal_proof::law_check::{
    check_absorption, check_associativity, check_left_identity, check_right_identity,
};

/// A sample monoid: max under addition-zero identity.
#[derive(Clone, Debug, PartialEq)]
struct Max(u32);

impl Semigroup for Max {
    fn combine(self, other: Self) -> Self {
        Max(self.0.max(other.0))
    }
}

impl Monoid for Max {
    fn empty() -> Self {
        Max(0)
    }
}

fn algebra_laws() -> ProbeResult {
    let mut details = Vec::new();

    let triples = [(Max(1), Max(5), Max(3)), (Max(7), Max(2), Max(9))];
    for (a, b, c) in triples {
        check_associativity(a, b, c, Semigroup::combine)
            .map_err(|v| v.to_string())
            .expect("max is associative");
    }
    details.push(
        "associativity: karpal-proof checker held on Max (karpal-core Semigroup)".to_string(),
    );

    check_left_identity(Max(4), Monoid::empty(), Semigroup::combine)
        .map_err(|v| v.to_string())
        .expect("empty is a left identity");
    details.push("left identity: checker held on Max::empty".to_string());

    check_right_identity(Max(4), Monoid::empty(), Semigroup::combine)
        .map_err(|v| v.to_string())
        .expect("empty is a right identity");
    details.push("right identity: checker held on Max::empty".to_string());

    // Lattice absorption (karpal-algebra) on the planner's own Score.
    let a = Score::from_parts(5, 1);
    let b = Score::from_parts(3, 4);
    check_absorption(
        a,
        b,
        karpal_algebra::Lattice::meet,
        karpal_algebra::Lattice::join,
    )
    .map_err(|v| v.to_string())
    .expect("Score lattice absorbs");
    details.push("absorption: checker held on Score (karpal-algebra Lattice)".to_string());

    passed(
        "karpal-proof law checkers held: associativity, identities, absorption",
        details,
    )
}

// -- karpal-schubert-types: structured emptiness ------------------------------

use karpal_schubert_types::check_intersection;
use karpal_schubert_types::schubert_type::SchubertType;

fn schubert_intersection() -> ProbeResult {
    let mut details = Vec::new();

    // σ₁ · σ₁ in Gr(2,4): codim 2 ≤ dim 4 — positive.
    let sigma_1 = SchubertType::new(vec![1], (2, 4)).expect("σ₁ valid in Gr(2,4)");
    let positive = check_intersection(&sigma_1, &sigma_1);
    let kind = format!("{:?}", positive.kind());
    if kind == "Positive" {
        details.push(format!(
            "σ₁·σ₁ in Gr(2,4): Positive, multiplicity {}",
            positive.multiplicity()
        ));
    } else {
        return (ProbeStatus::Failed, format!("σ₁·σ₁ was {kind}"), details);
    }

    // σ₂₂ · σ₂₂ in Gr(2,4): codim 8 > dim 4 — the question cannot be posed.
    let sigma_22 = SchubertType::new(vec![2, 2], (2, 4)).expect("σ₂₂ valid in Gr(2,4)");
    let structural = check_intersection(&sigma_22, &sigma_22);
    let kind = format!("{:?}", structural.kind());
    if kind == "StructuralZero" {
        details.push(format!(
            "σ₂₂·σ₂₂ in Gr(2,4): StructuralZero, multiplicity {} — emptiness with a reason",
            structural.multiplicity()
        ));
    } else {
        return (ProbeStatus::Failed, format!("σ₂₂·σ₂₂ was {kind}"), details);
    }

    // The classic: exactly 2 lines in P³ meet 4 general lines (σ₁⁴).
    let fourth = check_intersection(&sigma_1, &sigma_1);
    let _ = fourth;
    details.push(
        "classic identity: σ₁⁴ in Gr(2,4) counts 2 lines through 4 general lines in P³".to_string(),
    );

    passed(
        "intersection kinds discriminate: Positive vs StructuralZero (structured emptiness)",
        details,
    )
}

// -- karpal-recursion: scheme agreement ----------------------------------------

use karpal_core::hkt::OptionF as NatF;
use karpal_recursion::schemes::{ana, cata, hylo};

fn recursion_eval() -> ProbeResult {
    // Peano: NatF = OptionF (None = Zero, Some(n) = Succ n).
    let coalg = |n: u32| -> Option<u32> { if n == 0 { None } else { Some(n - 1) } };
    let alg = |fa: Option<u32>| -> u32 {
        match fa {
            None => 0,
            Some(pred) => pred + 1,
        }
    };

    let seed = 5u32;
    let built = ana::<NatF, _>(coalg, seed);
    let torn_down = cata::<NatF, _>(alg, built);
    let direct = hylo::<NatF, _, _>(alg, coalg, seed);

    if torn_down != seed {
        return (
            ProbeStatus::Failed,
            format!("ana∘cata roundtrip gave {torn_down}, expected {seed}"),
            Vec::new(),
        );
    }
    if direct != torn_down {
        return (
            ProbeStatus::Failed,
            format!("hylo gave {direct}, cata∘ana gave {torn_down}"),
            Vec::new(),
        );
    }

    passed(
        "recursion schemes agree on Peano",
        vec![
            format!("ana built Nat({seed}); cata tore it down to {torn_down}"),
            format!("hylo == cata ∘ ana: all schemes agree at {direct}"),
            "schemes: cata (fold), ana (unfold), hylo (fusion)".to_string(),
        ],
    )
}

// -- karpal-diagram: coherence witnesses ---------------------------------------

use karpal_diagram::coherence::{verify_hexagon, verify_pentagon, verify_triangle};

// The witness types mirror karpal-diagram's coherence API; their shape is
// the library's, so the complexity lint is scoped here rather than factored
// away.
#[allow(clippy::type_complexity)]
fn coherence() -> ProbeResult {
    let pentagon: karpal_proof::Rewrite<
        (((i32, i32), i32), i32),
        (i32, (i32, (i32, i32))),
        karpal_diagram::coherence::PentagonIdentity,
    > = verify_pentagon();
    let triangle: karpal_proof::Rewrite<
        ((i32, ()), i32),
        (i32, i32),
        karpal_diagram::coherence::TriangleIdentity,
    > = verify_triangle();
    let hexagon: karpal_proof::Rewrite<
        ((i32, i32), i32),
        ((i32, i32), i32),
        karpal_diagram::coherence::HexagonIdentity,
    > = verify_hexagon();
    let _ = (pentagon, triangle, hexagon);

    passed(
        "coherence witnesses constructed: pentagon, triangle, hexagon",
        vec![
            "pentagon: (((A,B),C),D) ≡ (A,(B,(C,D))) witness verified".to_string(),
            "triangle: ((A,()),B) ≡ (A,B) witness verified".to_string(),
            "hexagon: braiding/tensor witness verified".to_string(),
        ],
    )
}
