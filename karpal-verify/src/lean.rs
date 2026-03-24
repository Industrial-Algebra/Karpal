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

impl Lean4 {
    /// Export a list of obligations as a Lean 4 module skeleton.
    pub fn export_module(module_name: &str, obligations: &[Obligation]) -> String {
        export_module(module_name, obligations)
    }

    /// Export a bundle of obligations as a Lean 4 module skeleton.
    pub fn export_bundle(module_name: &str, bundle: &ObligationBundle) -> String {
        export_module(module_name, bundle.obligations())
    }
}

pub fn export_module(module_name: &str, obligations: &[Obligation]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("namespace {}", module_name));
    lines.push(String::new());

    for obligation in obligations {
        lines.push(format!("-- property: {}", obligation.property));
        lines.push(format!("-- origin: {}", obligation.summary()));
        lines.push(render_theorem(obligation));
        lines.push(String::new());
    }

    lines.push(format!("end {}", module_name));
    lines.join("\n")
}

fn render_theorem(obligation: &Obligation) -> String {
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

    let mut header = format!("theorem {}", sanitize(&obligation.name));
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
}
