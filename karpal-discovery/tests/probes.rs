// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for the probe registry and algebraic probes — Phase
//! 19-G. Probes are bounded, typed, read-only, deterministic executions
//! that dogfood the library's own abstractions — each one *runs real
//! Karpal code* and reports what it demonstrated.

use karpal_discovery::probes::{ProbeStatus, probe_catalog, run_probe};

#[test]
fn registry_lists_five_representative_probes() {
    let catalog = probe_catalog();
    let ids: Vec<&str> = catalog.iter().map(|p| p.id).collect();
    for expected in [
        "functor-monad-laws",
        "algebra-laws",
        "schubert-intersection",
        "recursion-eval",
        "coherence",
    ] {
        assert!(
            ids.contains(&expected),
            "probe `{expected}` registered: {ids:?}"
        );
    }
    // every probe declares what it dogfoods and is deterministic
    for probe in catalog {
        assert!(
            !probe.dogfoods.is_empty(),
            "{} dogfoods something",
            probe.id
        );
        assert!(probe.deterministic, "{} is deterministic", probe.id);
        assert!(!probe.description.is_empty(), "{} is described", probe.id);
    }
}

#[test]
fn unknown_probe_id_is_none() {
    assert!(run_probe("bogus").is_none());
}

#[test]
fn functor_monad_laws_probe_passes_on_option() {
    let outcome = run_probe("functor-monad-laws").expect("probe runs");
    assert_eq!(outcome.status, ProbeStatus::Passed);
    let joined = outcome.details.join("\n");
    assert!(joined.contains("identity"), "{joined}");
    assert!(joined.contains("composition"), "{joined}");
    assert!(joined.contains("associativity"), "{joined}");
}

#[test]
fn algebra_laws_probe_uses_karpal_proof_checkers() {
    let outcome = run_probe("algebra-laws").expect("probe runs");
    assert_eq!(outcome.status, ProbeStatus::Passed);
    let joined = outcome.details.join("\n");
    // the karpal-proof law checkers each reported
    assert!(joined.contains("associativity"), "{joined}");
    assert!(joined.contains("left identity"), "{joined}");
    assert!(joined.contains("right identity"), "{joined}");
    assert!(joined.contains("absorption"), "{joined}");
    assert!(
        outcome.dogfoods.iter().any(|d| d == "karpal-proof"),
        "{:?}",
        outcome.dogfoods
    );
}

#[test]
fn schubert_probe_distinguishes_emptiness_kinds() {
    let outcome = run_probe("schubert-intersection").expect("probe runs");
    assert_eq!(outcome.status, ProbeStatus::Passed);
    let joined = outcome.details.join("\n");
    // the structured-emptiness thesis, live: structural zero vs positive
    assert!(joined.contains("StructuralZero"), "{joined}");
    assert!(joined.contains("Positive"), "{joined}");
    // the classic two-lines result is reported with its multiplicity
    assert!(joined.contains("2"), "{joined}");
}

#[test]
fn recursion_probe_agrees_across_schemes() {
    let outcome = run_probe("recursion-eval").expect("probe runs");
    assert_eq!(outcome.status, ProbeStatus::Passed);
    let joined = outcome.details.join("\n");
    assert!(joined.contains("cata"), "{joined}");
    assert!(joined.contains("ana"), "{joined}");
    assert!(joined.contains("hylo"), "{joined}");
    // hylo ≡ cata ∘ ana on the sample
    assert!(joined.contains("agree"), "{joined}");
}

#[test]
fn coherence_probe_builds_all_three_witnesses() {
    let outcome = run_probe("coherence").expect("probe runs");
    assert_eq!(outcome.status, ProbeStatus::Passed);
    let joined = outcome.details.join("\n");
    assert!(joined.contains("pentagon"), "{joined}");
    assert!(joined.contains("triangle"), "{joined}");
    assert!(joined.contains("hexagon"), "{joined}");
}

#[test]
fn probes_are_deterministic_across_runs() {
    for probe in probe_catalog() {
        let first = run_probe(probe.id).expect("runs");
        let second = run_probe(probe.id).expect("runs again");
        assert_eq!(
            first.details, second.details,
            "{} is deterministic",
            probe.id
        );
        assert_eq!(first.summary, second.summary);
    }
}
