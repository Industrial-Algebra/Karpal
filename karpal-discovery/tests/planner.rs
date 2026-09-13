// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavioral tests for the category-theoretic planner — Phase 19-F. The
//! planner dogfoods Karpal's own abstractions at runtime: score aggregation
//! through `karpal_core::{Semigroup, Monoid}`, Pareto ranking through
//! `karpal_algebra::{Lattice, BoundedLattice}`, and plan
//! construction/consumption through the `karpal_free::Free` monad (built
//! with `lift_f` + `chain`, consumed by a structural catamorphism). If
//! Karpal's typeclasses cannot power its own discovery planner, they cannot
//! credibly power anyone else's.

use karpal_algebra::{BoundedLattice, Lattice};
use karpal_core::{Monoid, Semigroup};
use karpal_discovery::overlay::{
    ConceptOverlay, ConceptRecord, ConceptRelation, CostHint, RelationKind, StabilityTier,
};
use karpal_discovery::planner::{self, PlanStep, Score, StepAction};

fn concept(id: &str, aliases: &[&str], shapes: &[&str]) -> ConceptRecord {
    ConceptRecord {
        id: id.to_string(),
        name: id.to_string(),
        summary: format!("{id} fixture"),
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
        math_concepts: Vec::new(),
        problem_shapes: shapes.iter().map(|s| s.to_string()).collect(),
        symbol_refs: vec![format!("fixture-crate::{id}")],
        stability: StabilityTier::Stable,
        cost: CostHint::Free,
    }
}

fn rel(from: &str, to: &str, kind: RelationKind) -> ConceptRelation {
    ConceptRelation {
        from: from.to_string(),
        to: to.to_string(),
        kind,
    }
}

/// A fixture overlay: monad generalizes applicative and chain; freer-monad
/// is an alternative to monad.
fn fixture_overlay() -> ConceptOverlay {
    ConceptOverlay {
        catalog_version: "0.9.0-dev".to_string(),
        concepts: vec![
            concept("monad", &["bind"], &["sequence dependent effectful steps"]),
            concept("applicative", &[], &["combine independent effects"]),
            concept("chain", &[], &[]),
            concept(
                "freer-monad",
                &[],
                &["program as data with pluggable interpreters"],
            ),
        ],
        relations: vec![
            rel("monad", "applicative", RelationKind::Generalizes),
            rel("monad", "chain", RelationKind::Generalizes),
            rel("freer-monad", "monad", RelationKind::AlternativeTo),
        ],
    }
}

// -- the substrate: Karpal typeclasses at runtime --------------------------

#[test]
fn score_is_a_monoid() {
    // Aggregation of match evidence uses karpal-core's Semigroup/Monoid.
    let scores = [
        Score::from_parts(3, 2),
        Score::from_parts(1, 3),
        Monoid::empty(),
    ];
    let combined = scores
        .into_iter()
        .reduce(Semigroup::combine)
        .unwrap_or_else(Monoid::empty);
    assert_eq!(combined, Score::from_parts(4, 5));
    // identity law: empty is neutral
    let s = Score::from_parts(7, 1);
    assert_eq!(Semigroup::combine(s, Monoid::empty()), s);
    assert_eq!(Semigroup::combine(Monoid::empty(), s), s);
}

#[test]
fn score_is_a_bounded_lattice_with_pareto_dominance() {
    // Pareto ordering uses karpal-algebra's Lattice: join/meet are the
    // componentwise sup/inf of the (relevance, weight) order.
    let a = Score::from_parts(5, 1);
    let b = Score::from_parts(3, 4);
    assert_eq!(Lattice::join(a, b), Score::from_parts(5, 4));
    assert_eq!(Lattice::meet(a, b), Score::from_parts(3, 1));
    // a strictly dominates c iff join(a, c) == a and a != c
    let c = Score::from_parts(2, 0);
    assert_eq!(Lattice::join(a, c), a);
    assert_ne!(a, c);
    // bounds exist
    assert_eq!(Score::from_parts(0, 0), BoundedLattice::bottom());
    assert_eq!(Score::from_parts(u32::MAX, u32::MAX), BoundedLattice::top());
}

// -- recall + ranking --------------------------------------------------------

#[test]
fn recall_matches_text_and_expands_along_relations() {
    let overlay = fixture_overlay();
    let rec = planner::recommend("monad", &overlay);
    let ids: Vec<&str> = rec.entries.iter().map(|e| e.concept_id.as_str()).collect();
    assert!(ids.contains(&"monad"), "direct text match: {ids:?}");
    // neighborhood expansion along the relation graph
    assert!(ids.contains(&"applicative"), "generalizes target: {ids:?}");
    assert!(ids.contains(&"chain"), "generalizes target: {ids:?}");
    assert!(
        ids.contains(&"freer-monad"),
        "alternative_to source: {ids:?}"
    );
    // evidence distinguishes direct matches from relation neighbors
    let monad = rec
        .entries
        .iter()
        .find(|e| e.concept_id == "monad")
        .unwrap();
    assert!(
        monad.evidence.iter().any(|e| e.contains("match")),
        "monad carries match evidence: {:?}",
        monad.evidence
    );
    let applicative = rec
        .entries
        .iter()
        .find(|e| e.concept_id == "applicative")
        .unwrap();
    assert!(
        applicative.evidence.iter().any(|e| e.contains("relation")),
        "applicative carries relation evidence: {:?}",
        applicative.evidence
    );
    // direct match outranks a one-hop neighbor
    assert!(monad.score.relevance > applicative.score.relevance);
}

#[test]
fn ranking_prefers_pareto_dominants_then_stable_order() {
    let overlay = fixture_overlay();
    let rec = planner::recommend("monad", &overlay);
    let monad = rec
        .entries
        .iter()
        .find(|e| e.concept_id == "monad")
        .expect("monad present");
    // A direct match with maximal evidence dominates one-hop neighbors.
    assert_eq!(
        rec.entries.first().map(|e| e.concept_id.as_str()),
        Some("monad")
    );
    // deterministic: same input, same order
    let rec2 = planner::recommend("monad", &overlay);
    let ids: Vec<&str> = rec.entries.iter().map(|e| e.concept_id.as_str()).collect();
    let ids2: Vec<&str> = rec2.entries.iter().map(|e| e.concept_id.as_str()).collect();
    assert_eq!(ids, ids2);
    let _ = monad;
}

#[test]
fn problem_shape_matches_recall_concepts() {
    // "sequence dependent effectful steps" is monad's problem shape — an
    // agent phrasing the problem, not the concept name, still recalls it.
    let overlay = fixture_overlay();
    let rec = planner::recommend("dependent effectful", &overlay);
    assert!(
        rec.entries.iter().any(|e| e.concept_id == "monad"),
        "{:?}",
        rec.entries
            .iter()
            .map(|e| e.concept_id.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn empty_goal_recalls_nothing() {
    let overlay = fixture_overlay();
    let rec = planner::recommend("", &overlay);
    assert!(rec.entries.is_empty());
}

// -- planning ----------------------------------------------------------------

#[test]
fn plan_is_sequenced_orient_explore_verify() {
    let overlay = fixture_overlay();
    let rec = planner::recommend("monad", &overlay);
    let plan = planner::plan("monad", &rec);
    assert_eq!(
        plan.steps.first().map(|s| s.action),
        Some(StepAction::Orient)
    );
    let explores: Vec<&PlanStep> = plan
        .steps
        .iter()
        .filter(|s| s.action == StepAction::Explore)
        .collect();
    // bounded exploration: at most the top three ranked concepts
    assert!(
        !explores.is_empty() && explores.len() <= 3,
        "{:?}",
        plan.steps
    );
    assert_eq!(
        plan.steps.last().map(|s| s.action),
        Some(StepAction::Verify)
    );
    // explore steps target ranked concepts, starting with the best
    assert_eq!(explores.first().map(|s| s.target.as_str()), Some("monad"));
}

#[test]
fn plan_normalization_collapses_adjacent_duplicates() {
    // Plans are built as Free<PlanStepF, ()> and consumed by a structural
    // catamorphism; normalization collapses adjacent duplicate steps.
    let steps = vec![
        PlanStep::new(StepAction::Orient, "monad", "goal"),
        PlanStep::new(StepAction::Explore, "monad", "first"),
        PlanStep::new(StepAction::Explore, "monad", "duplicate"),
        PlanStep::new(StepAction::Verify, "drift gate", "all refs validated"),
    ];
    let plan = planner::build_plan("monad", steps);
    let targets: Vec<(StepAction, &str)> = plan
        .steps
        .iter()
        .map(|s| (s.action, s.target.as_str()))
        .collect();
    assert_eq!(
        targets,
        vec![
            (StepAction::Orient, "monad"),
            (StepAction::Explore, "monad"),
            (StepAction::Verify, "drift gate"),
        ],
        "adjacent duplicate collapsed, first note kept"
    );
}

// -- the embedded overlay is planner-grade -----------------------------------

#[test]
fn embedded_overlay_supports_recommend_and_plan() {
    let overlay = karpal_discovery::load_concept_overlay();
    let rec = planner::recommend("sequence dependent effectful steps", &overlay);
    assert!(
        rec.entries.iter().any(|e| e.concept_id == "monad"),
        "the real curated overlay recalls monad from its problem shape"
    );
    let plan = planner::plan("sequence dependent effectful steps", &rec);
    assert!(plan.steps.iter().any(|s| s.target == "monad"));
}

/// The four documented near-miss queries from the Knopper practitioner
/// run (PR #160, `docs/dev/discovery-feedback-2026-08-22-knopper.md`):
/// each is plain language over words the concepts' own summaries contain,
/// and each returned zero under 0.9.0's whole-phrase substring recall.
/// 0.9.1 acceptance: token-level recall over summaries + aliases.
#[test]
fn plain_language_goals_recall_via_summaries() {
    let overlay = karpal_discovery::load_concept_overlay();
    for (goal, expected) in [
        (
            "bidirectional focus on a part of a structure, get and put",
            "optic",
        ),
        ("state and a focus position", "comonad-transformers"),
        ("least upper bound join", "lattice"),
        ("commuting two layers", "traversable"),
    ] {
        let rec = planner::recommend(goal, &overlay);
        let top3: Vec<&str> = rec
            .entries
            .iter()
            .map(|e| e.concept_id.as_str())
            .take(3)
            .collect();
        assert!(
            top3.contains(&expected),
            "goal {goal:?} should recall {expected} in the top 3; got {top3:?}"
        );
    }
}

/// The strongest of the documented queries should rank their target first.
#[test]
fn vocabulary_goals_rank_their_target_first() {
    let overlay = karpal_discovery::load_concept_overlay();
    for (goal, expected) in [
        ("least upper bound join", "lattice"),
        ("commuting two layers", "traversable"),
    ] {
        let rec = planner::recommend(goal, &overlay);
        assert_eq!(
            rec.entries.first().map(|e| e.concept_id.as_str()),
            Some(expected),
            "goal {goal:?} should rank {expected} first; got {:?}",
            rec.entries
                .iter()
                .map(|e| e.concept_id.clone())
                .take(3)
                .collect::<Vec<_>>()
        );
        assert!(
            rec.entries
                .iter()
                .any(|e| e.evidence.iter().any(|ev| ev.contains("token"))),
            "goal {goal:?} should carry token-level evidence"
        );
    }
}

/// R7-lite: a zero-recall goal must explain itself — distinguish "nothing
/// exists" from "wrong phrasing" instead of returning a silent empty list
/// (in the live run, grep outperformed the tool for exactly this reason).
/// Uses the synthetic fixture overlay so the zero case is guaranteed
/// (single-token noise in the 83-concept overlay can recall at relevance 1,
/// which ranks last and is acceptable).
#[test]
fn zero_result_recommend_explains_itself() {
    let overlay = fixture_overlay();
    let rec = planner::recommend("elephant pajamas waltzing", &overlay);
    assert!(
        rec.entries.is_empty(),
        "absent-vocabulary goal recalls nothing"
    );
    let note = rec.note.as_deref().unwrap_or_else(|| {
        panic!("zero-result recommend must carry an explanatory note; got {rec:?}")
    });
    assert!(note.contains("karpal.concepts"), "note points at browsing");
    assert!(rec.nearest.is_empty(), "no vocabulary overlap → no nearest");
}

/// Weak-recall diagnostics: a goal that only single-token noise can touch
/// (relevance ≤ 1) is effectively a miss — it must explain itself and offer
/// the nearest vocabulary, not just list the noise.
#[test]
fn zero_result_recommend_offers_nearest_vocabulary() {
    let overlay = karpal_discovery::load_concept_overlay();
    let rec = planner::recommend("elephant pajamas commuting", &overlay);
    assert!(rec.note.is_some(), "weak-recall goal must explain itself");
    assert!(
        !rec.nearest.is_empty(),
        "weak-recall goal offers nearest vocabulary"
    );
}
