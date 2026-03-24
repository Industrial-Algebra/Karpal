#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

use crate::{
    Obligation, ObligationBundle,
    obligation::{Sort, Term},
};

/// Marker for the SMT-LIB2 backend.
pub struct SmtLib2;

impl SmtLib2 {
    /// Export a single obligation as an SMT-LIB2 script.
    pub fn export(obligation: &Obligation) -> String {
        export_obligation(obligation)
    }

    /// Export a bundle as one SMT-LIB2 script per obligation.
    pub fn export_bundle(bundle: &ObligationBundle) -> Vec<(String, String)> {
        bundle
            .obligations()
            .iter()
            .map(|obligation| (obligation.name.clone(), export_obligation(obligation)))
            .collect()
    }
}

pub fn export_obligation(obligation: &Obligation) -> String {
    let mut lines = Vec::new();
    lines.push(format!("; obligation: {}", obligation.name));
    lines.push(format!("; property: {}", obligation.property));
    lines.push(format!("; origin: {}", obligation.summary()));
    lines.push("(set-logic ALL)".to_string());

    for declaration in &obligation.declarations {
        lines.push(format!(
            "(declare-const {} {})",
            declaration.name,
            render_sort(&declaration.sort)
        ));
    }

    for assumption in &obligation.assumptions {
        lines.push(format!("(assert {})", render_term(assumption)));
    }

    lines.push("; ask the solver for a counterexample to the law".to_string());
    lines.push(format!(
        "(assert (not {}))",
        render_term(&obligation.conclusion)
    ));
    lines.push("(check-sat)".to_string());
    lines.push("(get-model)".to_string());
    lines.join("\n")
}

fn render_sort(sort: &Sort) -> String {
    match sort {
        Sort::Bool => "Bool".to_string(),
        Sort::Int => "Int".to_string(),
        Sort::Real => "Real".to_string(),
        Sort::Named(name) => name.clone(),
    }
}

fn render_term(term: &Term) -> String {
    match term {
        Term::Var(name) => name.clone(),
        Term::Bool(true) => "true".to_string(),
        Term::Bool(false) => "false".to_string(),
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
        Term::Eq(left, right) => format!("(= {} {})", render_term(left), render_term(right)),
        Term::And(terms) => match terms.as_slice() {
            [] => "true".to_string(),
            [term] => render_term(term),
            _ => format!(
                "(and {})",
                terms.iter().map(render_term).collect::<Vec<_>>().join(" ")
            ),
        },
        Term::Or(terms) => match terms.as_slice() {
            [] => "false".to_string(),
            [term] => render_term(term),
            _ => format!(
                "(or {})",
                terms.iter().map(render_term).collect::<Vec<_>>().join(" ")
            ),
        },
        Term::Not(inner) => format!("(not {})", render_term(inner)),
        Term::Implies(lhs, rhs) => format!("(=> {} {})", render_term(lhs), render_term(rhs)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obligation::{Origin, VerificationTier};
    use karpal_proof::IsAssociative;

    #[test]
    fn exports_declarations_and_negated_goal() {
        let obligation = Obligation::for_property::<IsAssociative>(
            "assoc",
            Origin::new("karpal-core", "Semigroup for i32"),
            VerificationTier::External,
            Term::eq(Term::var("x"), Term::var("y")),
        )
        .with_decl("x", Sort::Int)
        .with_decl("y", Sort::Int);

        let text = export_obligation(&obligation);
        assert!(text.contains("(declare-const x Int)"));
        assert!(text.contains("(assert (not (= x y)))"));
        assert!(text.contains("(check-sat)"));
    }
}
