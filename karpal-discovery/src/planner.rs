// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The category-theoretic planner (Phase 19-F) — Karpal's abstractions
//! powering its own discovery engine, at runtime:
//!
//! - **Score aggregation** — match evidence accumulates through
//!   [`karpal_core::Semigroup`]/[`karpal_core::Monoid`] (`Score` is a monoid
//!   under componentwise addition).
//! - **Pareto ranking** — candidate ordering uses
//!   [`karpal_algebra::Lattice`]: strict dominance is decided by lattice
//!   join (`a` dominates `c` iff `a ⊔ c = a` and `a ≠ c`), the honest
//!   partial order on (relevance, weight) pairs.
//! - **Plan construction** — a candidate plan is built as the
//!   [`karpal_free::Free`] monad over a step functor (`lift_f` + `chain`
//!   sequencing) and consumed by a structural catamorphism (`fmap`-based
//!   fold); normalization collapses adjacent duplicate steps on the folded
//!   form.
//!
//! Recall runs over the curated overlay: direct text matches (ids, names,
//! aliases, math concepts, problem shapes) seed the candidates, and the
//! relation graph (the capability category: `generalizes` / `composes_with`
//! / `alternative_to` / `dual_of` edges) expands the neighborhood. Every
//! entry carries its evidence — matches say what matched, neighbors say
//! which relation and concept carried them.

use std::collections::{BTreeMap, BTreeSet};

use karpal_algebra::Lattice;
use karpal_core::{Monoid, Semigroup};
use karpal_free::Free;
use serde::{Deserialize, Serialize};

use crate::overlay::{ConceptOverlay, ConceptRecord, StabilityTier};

/// A planner score: match strength and quality weight. A monoid under
/// componentwise addition (evidence accumulation) and a lattice under
/// componentwise max/min (Pareto order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Score {
    /// Match evidence strength.
    pub relevance: u32,
    /// Quality weight (stability tier of the concept).
    pub weight: u32,
}

impl Score {
    /// Construct from parts.
    #[must_use]
    pub fn from_parts(relevance: u32, weight: u32) -> Self {
        Self { relevance, weight }
    }
}

impl Semigroup for Score {
    fn combine(self, other: Self) -> Self {
        Self {
            relevance: self.relevance.saturating_add(other.relevance),
            weight: self.weight.saturating_add(other.weight),
        }
    }
}

impl Monoid for Score {
    fn empty() -> Self {
        Self::from_parts(0, 0)
    }
}

impl Lattice for Score {
    fn join(self, other: Self) -> Self {
        Self {
            relevance: self.relevance.max(other.relevance),
            weight: self.weight.max(other.weight),
        }
    }

    fn meet(self, other: Self) -> Self {
        Self {
            relevance: self.relevance.min(other.relevance),
            weight: self.weight.min(other.weight),
        }
    }
}

impl karpal_algebra::BoundedLattice for Score {
    fn top() -> Self {
        Self::from_parts(u32::MAX, u32::MAX)
    }

    fn bottom() -> Self {
        Self::from_parts(0, 0)
    }
}

/// One ranked concept in a recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedConcept {
    /// The concept id.
    pub concept_id: String,
    /// Display name.
    pub name: String,
    /// Stability tier (`"stable"` / `"experimental"` / `"research"`).
    pub stability: String,
    /// Aggregated score.
    pub score: Score,
    /// Why this concept was recalled (match evidence, relation evidence).
    pub evidence: Vec<String>,
}

/// A recommendation: concepts recalled for a goal, Pareto-ranked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// The goal as received.
    pub goal: String,
    /// Ranked entries (dominants first, deterministic order).
    pub entries: Vec<RankedConcept>,
    /// When nothing recalled: the closest concepts by vocabulary overlap
    /// (bounded to three), so "wrong phrasing" is distinguishable from
    /// "nothing exists" (Lonis #21 R7).
    pub nearest: Vec<String>,
    /// When nothing recalled: an explanatory note pointing at rephrasing
    /// or browsing (`karpal.concepts`).
    pub note: Option<String>,
}

/// Stability-tier weight: stable concepts weigh more at equal relevance.
const fn stability_weight(tier: StabilityTier) -> u32 {
    match tier {
        StabilityTier::Stable => 3,
        StabilityTier::Experimental => 2,
        StabilityTier::Research => 1,
    }
}

const fn stability_str(tier: StabilityTier) -> &'static str {
    match tier {
        StabilityTier::Stable => "stable",
        StabilityTier::Experimental => "experimental",
        StabilityTier::Research => "research",
    }
}

/// Recall and rank concepts for a goal.
///
/// Direct text matches (id, name, aliases, math concepts, problem shapes;
/// case-insensitive) seed candidates with relevance 3 per matched field;
/// the relation graph expands the neighborhood with relevance 1 per edge
/// (both directions: a concept is recalled by what it generalizes, by what
/// generalizes it, and by its alternatives and duals). Evidence
/// accumulates through `Monoid::combine`; ranking is Pareto dominance via
/// `Lattice::join` (fewest strict dominators first, then relevance, then
/// weight, then id for determinism).
#[must_use]
pub fn recommend(goal: &str, overlay: &ConceptOverlay) -> Recommendation {
    let needle = goal.trim().to_lowercase();
    let query = crate::recall::query_tokens(&needle);
    let mut recalled: BTreeMap<String, (Score, Vec<String>, &ConceptRecord)> = BTreeMap::new();

    if !needle.is_empty() {
        for concept in &overlay.concepts {
            let mut score = Monoid::empty();
            let mut evidence = Vec::new();
            // Exact identity matches outrank substring matches: `monad`
            // should not tie with `freer-monad` for the goal "monad".
            if concept.id.to_lowercase() == needle {
                score = Semigroup::combine(
                    score,
                    Score::from_parts(4, stability_weight(concept.stability)),
                );
                evidence.push("match: id (exact)".to_string());
            } else if concept.id.to_lowercase().contains(&needle) {
                score = Semigroup::combine(
                    score,
                    Score::from_parts(3, stability_weight(concept.stability)),
                );
                evidence.push("match: id".to_string());
            }
            if concept.name.to_lowercase() == needle {
                score = Semigroup::combine(
                    score,
                    Score::from_parts(4, stability_weight(concept.stability)),
                );
                evidence.push("match: name (exact)".to_string());
            } else if concept.name.to_lowercase().contains(&needle) {
                score = Semigroup::combine(
                    score,
                    Score::from_parts(3, stability_weight(concept.stability)),
                );
                evidence.push("match: name".to_string());
            }
            if concept
                .aliases
                .iter()
                .any(|a| a.to_lowercase().contains(&needle))
            {
                score = Semigroup::combine(
                    score,
                    Score::from_parts(3, stability_weight(concept.stability)),
                );
                evidence.push("match: alias".to_string());
            }
            if concept
                .math_concepts
                .iter()
                .any(|m| m.to_lowercase().contains(&needle))
            {
                score = Semigroup::combine(
                    score,
                    Score::from_parts(2, stability_weight(concept.stability)),
                );
                evidence.push("match: math concept".to_string());
            }
            if concept
                .problem_shapes
                .iter()
                .any(|p| p.to_lowercase().contains(&needle))
            {
                score = Semigroup::combine(
                    score,
                    Score::from_parts(2, stability_weight(concept.stability)),
                );
                evidence.push("match: problem shape".to_string());
            }
            // Token tier (0.9.1): summaries and aliases are the richest
            // curated text each concept has; plain-language goals recall
            // via vocabulary overlap beneath the substring tier. The four
            // documented 0.9.0 near-misses (PR #160) all hit here. Relevance
            // is graded by matched-token count (capped at 3, substring-tier
            // confidence): a 3-token vocabulary match must outrank 2-token
            // matches — Pareto cannot separate equal scores, and alphabetical
            // order would otherwise decide.
            let token_hits = crate::recall::text_matches(
                &query,
                &format!("{} {}", concept.summary, concept.aliases.join(" ")),
            );
            if token_hits > 0 {
                let relevance = token_hits.min(3) as u32;
                score = Semigroup::combine(
                    score,
                    Score::from_parts(relevance, stability_weight(concept.stability)),
                );
                evidence.push(if token_hits == 1 {
                    "match: summary/alias token (1 query token)".to_string()
                } else {
                    format!("match: summary/alias token ({token_hits} query tokens)")
                });
            }
            if score.relevance > 0 {
                recalled.insert(concept.id.clone(), (score, evidence, concept));
            }
        }

        // Neighborhood expansion along the relation graph (one hop): edges
        // touching a directly-matched concept recall their other endpoint.
        let matched: BTreeSet<String> = recalled
            .iter()
            .filter(|(_, (score, _, _))| score.relevance >= 3)
            .map(|(id, _)| id.clone())
            .collect();
        for relation in &overlay.relations {
            for (anchor, other) in [
                (relation.from.as_str(), relation.to.as_str()),
                (relation.to.as_str(), relation.from.as_str()),
            ] {
                if !matched.contains(anchor) {
                    continue;
                }
                if let Some(concept) = overlay.concepts.iter().find(|c| c.id == other) {
                    let bonus = Score::from_parts(1, stability_weight(concept.stability));
                    let entry = recalled
                        .entry(other.to_string())
                        .or_insert_with(|| (Monoid::empty(), Vec::new(), concept));
                    entry.0 = Semigroup::combine(entry.0, bonus);
                    entry.1.push(format!(
                        "relation: {} {} ↔ {}",
                        relation.kind.as_str(),
                        anchor,
                        other
                    ));
                }
            }
        }
    }

    let entries: Vec<RankedConcept> = recalled
        .into_iter()
        .map(|(concept_id, (score, evidence, concept))| RankedConcept {
            concept_id,
            name: concept.name.clone(),
            stability: stability_str(concept.stability).to_string(),
            score,
            evidence,
        })
        .collect();

    // Pareto ranking: fewest strict dominators first, then relevance, then
    // weight, then id (deterministic).
    let dominators = |entry: &RankedConcept| -> usize {
        entries
            .iter()
            .filter(|other| {
                other.concept_id != entry.concept_id
                    && Lattice::join(other.score, entry.score) == other.score
                    && other.score != entry.score
            })
            .count()
    };
    let dominator_counts: Vec<usize> = entries.iter().map(&dominators).collect();
    let mut ranked: Vec<(RankedConcept, usize)> =
        entries.into_iter().zip(dominator_counts).collect();
    ranked.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| b.0.score.relevance.cmp(&a.0.score.relevance))
            .then_with(|| b.0.score.weight.cmp(&a.0.score.weight))
            .then_with(|| a.0.concept_id.cmp(&b.0.concept_id))
    });

    let ranked_entries: Vec<RankedConcept> = ranked.into_iter().map(|(entry, _)| entry).collect();

    // Zero-result diagnostics (0.9.1, Lonis #21 R7): a silent empty list —
    // or a lone single-token noise hit — cannot distinguish "nothing exists"
    // from "wrong phrasing". In the live run, grep outperformed the tool for
    // exactly this reason. Say what happened and offer the nearest vocabulary
    // whenever nothing recalled with substring-tier confidence (≥ 2).
    let weak = ranked_entries.is_empty()
        || ranked_entries
            .iter()
            .map(|e| e.score.relevance)
            .max()
            .is_some_and(|best| best <= 1);
    let (nearest, note) = if weak && !needle.is_empty() {
        let mut near: Vec<(usize, &ConceptRecord)> = overlay
            .concepts
            .iter()
            .map(|c| {
                (
                    crate::recall::text_matches(
                        &query,
                        &format!("{} {}", c.summary, c.aliases.join(" ")),
                    ),
                    c,
                )
            })
            .filter(|(hits, _)| *hits >= 1)
            .collect();
        near.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
        (
            near.into_iter().take(3).map(|(_, c)| c.id.clone()).collect(),
            Some(
                "no concept matched the goal with confidence; recall is vocabulary-based — try the \
                 domain terms a concept's summary would use, or browse the overlay with \
                 karpal.concepts"
                    .to_string(),
            ),
        )
    } else {
        (Vec::new(), None)
    };

    Recommendation {
        goal: goal.to_string(),
        entries: ranked_entries,
        nearest,
        note,
    }
}

// -- planning ----------------------------------------------------------------

/// What a plan step does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepAction {
    /// Frame the goal.
    Orient,
    /// Explore one concept (its items, docs, neighbors).
    Explore,
    /// Verify invariants (the drift gate, bounds).
    Verify,
}

impl StepAction {
    /// The wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Orient => "orient",
            Self::Explore => "explore",
            Self::Verify => "verify",
        }
    }
}

/// One concrete plan step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    /// The action.
    pub action: StepAction,
    /// What it targets (a concept id, the goal, or a check name).
    pub target: String,
    /// Why the step is here.
    pub note: String,
}

impl PlanStep {
    /// Construct a step.
    #[must_use]
    pub fn new(action: StepAction, target: &str, note: &str) -> Self {
        Self {
            action,
            target: target.to_string(),
            note: note.to_string(),
        }
    }
}

/// A candidate plan: goal + sequenced steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePlan {
    /// The goal as received.
    pub goal: String,
    /// The sequenced steps (oriented, explored, verified).
    pub steps: Vec<PlanStep>,
}

/// The plan step functor: one step plus a continuation.
#[derive(Debug, Clone)]
pub enum PlanStepF<A> {
    /// One step wrapping the rest of the plan.
    Step {
        /// The action.
        action: StepAction,
        /// What it targets.
        target: String,
        /// Why it is here.
        note: String,
        /// The rest of the plan.
        next: A,
    },
    /// The end of the plan.
    Fin,
}

/// HKT marker for [`PlanStepF`] (the `OptionF` pattern).
pub struct PlanF;

impl karpal_core::hkt::HKT for PlanF {
    type Of<T> = PlanStepF<T>;
}

impl karpal_core::Functor for PlanF {
    fn fmap<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> B) -> Self::Of<B> {
        match fa {
            PlanStepF::Step {
                action,
                target,
                note,
                next,
            } => PlanStepF::Step {
                action,
                target,
                note,
                next: f(next),
            },
            PlanStepF::Fin => PlanStepF::Fin,
        }
    }
}

/// Build a candidate plan from concrete steps.
///
/// The steps are lifted into the [`Free`] monad (`lift_f` + `chain` — the
/// plan *is* a free program over the step functor), then consumed by a
/// structural catamorphism (`fmap`-based fold). Normalization collapses
/// adjacent duplicate steps on the folded form, keeping the first note.
#[must_use]
pub fn build_plan(goal: &str, steps: Vec<PlanStep>) -> CandidatePlan {
    // Sequence as Free<PlanStepF, ()>: each step is one effect layer,
    // chained onto the growing program.
    let mut program: Free<PlanF, ()> = Free::pure(());
    for step in &steps {
        program = program.chain(|_| {
            Free::lift_f(PlanStepF::Step {
                action: step.action,
                target: step.target.clone(),
                note: step.note.clone(),
                next: (),
            })
        });
    }

    // Consume: structural catamorphism over the free structure.
    let mut folded: Vec<PlanStep> = Vec::new();
    let mut cursor = program;
    loop {
        cursor = match cursor {
            Free::Pure(()) => break,
            Free::Roll(boxed) => match *boxed {
                PlanStepF::Step {
                    action,
                    target,
                    note,
                    next,
                } => {
                    folded.push(PlanStep {
                        action,
                        target,
                        note,
                    });
                    next
                }
                PlanStepF::Fin => break,
            },
        };
    }

    // Normalize: collapse adjacent duplicates (first note kept).
    let mut normalized: Vec<PlanStep> = Vec::new();
    for step in folded {
        if normalized
            .last()
            .is_some_and(|prev| prev.action == step.action && prev.target == step.target)
        {
            continue;
        }
        normalized.push(step);
    }

    CandidatePlan {
        goal: goal.to_string(),
        steps: normalized,
    }
}

/// Plan for a recommendation: orient, explore the top-ranked concepts
/// (bounded to three), verify.
#[must_use]
pub fn plan(goal: &str, recommendation: &Recommendation) -> CandidatePlan {
    let mut steps = vec![PlanStep::new(StepAction::Orient, goal, "frame the goal")];
    for entry in recommendation.entries.iter().take(3) {
        steps.push(PlanStep::new(
            StepAction::Explore,
            &entry.concept_id,
            &format!(
                "ranked concept (relevance {}, stability {})",
                entry.score.relevance, entry.stability
            ),
        ));
    }
    steps.push(PlanStep::new(
        StepAction::Verify,
        "drift gate",
        "all overlay symbol refs validate against the catalog",
    ));
    build_plan(goal, steps)
}
