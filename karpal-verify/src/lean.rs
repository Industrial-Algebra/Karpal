#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

use crate::{
    Obligation, ObligationBundle,
    obligation::{Sort, Term},
};

/// Marker for the Lean 4 backend.
pub struct Lean4;

/// Structured Lean theorem metadata derived from an obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanTheorem {
    pub obligation_name: String,
    pub theorem_name: String,
    pub property: String,
    pub origin_summary: String,
}

impl LeanTheorem {
    pub fn witness_ref(&self, module_name: &str) -> String {
        format!("{module_name}.{}", self.theorem_name)
    }
}

/// Structured Lean export result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanExport {
    pub module_name: String,
    pub source: String,
    pub theorems: Vec<LeanTheorem>,
}

impl LeanExport {
    pub fn theorem_for_obligation(&self, obligation_name: &str) -> Option<&LeanTheorem> {
        self.theorems
            .iter()
            .find(|theorem| theorem.obligation_name == obligation_name)
    }

    pub fn theorem_names(&self) -> Vec<String> {
        self.theorems
            .iter()
            .map(|theorem| theorem.theorem_name.clone())
            .collect()
    }
}

impl Lean4 {
    /// Export a list of obligations as a Lean 4 module skeleton.
    pub fn export_module(module_name: &str, obligations: &[Obligation]) -> String {
        export_module(module_name, obligations)
    }

    /// Export a list of obligations as structured Lean metadata plus source.
    pub fn export(module_name: &str, obligations: &[Obligation]) -> LeanExport {
        export(module_name, obligations)
    }

    /// Export a bundle of obligations as a Lean 4 module skeleton.
    pub fn export_bundle(module_name: &str, bundle: &ObligationBundle) -> String {
        export_module(module_name, bundle.obligations())
    }

    /// Export a bundle of obligations as structured Lean metadata plus source.
    pub fn export_bundle_structured(module_name: &str, bundle: &ObligationBundle) -> LeanExport {
        export(module_name, bundle.obligations())
    }
}

pub fn export(module_name: &str, obligations: &[Obligation]) -> LeanExport {
    let theorems = obligations
        .iter()
        .map(LeanTheorem::from)
        .collect::<Vec<_>>();

    let mut lines = Vec::new();
    lines.push(format!("namespace {}", module_name));
    lines.push(String::new());

    for (obligation, theorem) in obligations.iter().zip(&theorems) {
        lines.push(format!("-- property: {}", obligation.property));
        lines.push(format!("-- origin: {}", obligation.summary()));
        lines.push(render_theorem(obligation, theorem));
        lines.push(String::new());
    }

    lines.push(format!("end {}", module_name));

    LeanExport {
        module_name: module_name.into(),
        source: lines.join("\n"),
        theorems,
    }
}

pub fn export_module(module_name: &str, obligations: &[Obligation]) -> String {
    export(module_name, obligations).source
}

fn render_theorem(obligation: &Obligation, theorem: &LeanTheorem) -> String {
    let binders = obligation
        .declarations
        .iter()
        .map(|decl| format!("({} : {})", decl.name, render_sort(&decl.sort)))
        .collect::<Vec<_>>()
        .join(" ");

    let assumptions = obligation
        .assumptions
        .iter()
        .enumerate()
        .map(|(idx, term)| format!("(h{} : {})", idx + 1, render_term(term)))
        .collect::<Vec<_>>()
        .join(" ");

    let mut header = format!("theorem {}", theorem.theorem_name);
    if !binders.is_empty() {
        header.push(' ');
        header.push_str(&binders);
    }
    if !assumptions.is_empty() {
        header.push(' ');
        header.push_str(&assumptions);
    }
    header.push_str(&format!(" : {} := by", render_term(&obligation.conclusion)));
    header.push_str("\n  sorry");
    header
}

fn render_sort(sort: &Sort) -> String {
    match sort {
        Sort::Bool => "Bool".to_string(),
        Sort::Int => "Int".to_string(),
        Sort::Real => "Rat".to_string(),
        Sort::Named(name) => name.clone(),
    }
}

fn render_term(term: &Term) -> String {
    match term {
        Term::Var(name) => name.clone(),
        Term::Bool(true) => "True".to_string(),
        Term::Bool(false) => "False".to_string(),
        Term::Int(value) => value.to_string(),
        Term::App { function, args } => {
            if args.is_empty() {
                function.clone()
            } else {
                format!(
                    "({} {})",
                    function,
                    args.iter().map(render_term).collect::<Vec<_>>().join(" ")
                )
            }
        }
        Term::Eq(left, right) => format!("{} = {}", render_term(left), render_term(right)),
        Term::And(terms) => match terms.as_slice() {
            [] => "True".to_string(),
            [term] => render_term(term),
            _ => terms
                .iter()
                .map(render_term)
                .collect::<Vec<_>>()
                .join(" ∧ "),
        },
        Term::Or(terms) => match terms.as_slice() {
            [] => "False".to_string(),
            [term] => render_term(term),
            _ => terms
                .iter()
                .map(render_term)
                .collect::<Vec<_>>()
                .join(" ∨ "),
        },
        Term::Not(inner) => format!("¬{}", parenthesize(inner)),
        Term::Implies(lhs, rhs) => format!("{} → {}", parenthesize(lhs), render_term(rhs)),
    }
}

fn parenthesize(term: &Term) -> String {
    match term {
        Term::Var(_) | Term::Bool(_) | Term::Int(_) | Term::App { .. } => render_term(term),
        _ => format!("({})", render_term(term)),
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

impl From<&Obligation> for LeanTheorem {
    fn from(obligation: &Obligation) -> Self {
        Self {
            obligation_name: obligation.name.clone(),
            theorem_name: sanitize(&obligation.name),
            property: obligation.property.into(),
            origin_summary: obligation.summary(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obligation::{Origin, VerificationTier};
    use karpal_proof::IsCommutative;

    #[test]
    fn exports_theorem_skeleton() {
        let obligation = Obligation::for_property::<IsCommutative>(
            "sum/commutative",
            Origin::new("karpal-algebra", "AbelianGroup for i16"),
            VerificationTier::External,
            Term::eq(
                Term::app("combine", [Term::var("a"), Term::var("b")]),
                Term::app("combine", [Term::var("b"), Term::var("a")]),
            ),
        )
        .with_decl("a", Sort::Int)
        .with_decl("b", Sort::Int);

        let text = export_module("KarpalVerify", &[obligation]);
        assert!(text.contains("namespace KarpalVerify"));
        assert!(text.contains("theorem sum_commutative"));
        assert!(text.contains("sorry"));
    }

    #[test]
    fn structured_export_tracks_theorem_identity() {
        let obligation = Obligation::associativity(
            "sum-assoc",
            Origin::new("karpal-core", "Semigroup for i32"),
            Sort::Int,
            "combine",
        );

        let export = export("KarpalVerify", &[obligation]);
        assert_eq!(export.module_name, "KarpalVerify");
        assert_eq!(export.theorems.len(), 1);
        assert_eq!(export.theorems[0].theorem_name, "sum_assoc");
        assert_eq!(
            export.theorems[0].witness_ref("KarpalVerify"),
            "KarpalVerify.sum_assoc"
        );
        assert!(export.source.contains("theorem sum_assoc"));
    }
}
