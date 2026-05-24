#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

use crate::{Obligation, ObligationBundle, Term};

/// Generated Kani proof harness for one obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KaniHarness {
    pub obligation_name: String,
    pub harness_name: String,
    pub source: String,
}

/// Marker type for the Kani backend.
pub struct Kani;

/// Export one backend-agnostic obligation as a Kani proof harness.
pub fn export_kani_harness(obligation: &Obligation) -> KaniHarness {
    let harness_name = sanitize_ident(&obligation.name);
    KaniHarness {
        obligation_name: obligation.name.clone(),
        harness_name: harness_name.clone(),
        source: render_kani_harness(&harness_name, obligation),
    }
}

/// Export every obligation in a bundle as a separate Kani proof harness.
pub fn export_kani_bundle(bundle: &ObligationBundle) -> Vec<KaniHarness> {
    bundle
        .obligations()
        .iter()
        .map(export_kani_harness)
        .collect()
}

fn render_kani_harness(harness_name: &str, obligation: &Obligation) -> String {
    let mut source = String::new();
    source.push_str("#[kani::proof]\n");
    source.push_str(&format!("fn {harness_name}() {{\n"));
    for decl in &obligation.declarations {
        source.push_str(&format!(
            "    let {} = kani::any();\n",
            sanitize_ident(&decl.name)
        ));
    }
    source.push_str("    // Kani backend skeleton generated from Karpal obligation IR.\n");
    source.push_str(&format!("    // property: {}\n", obligation.property));
    source.push_str(&format!(
        "    kani::assert({}, \"{}\");\n",
        render_kani_bool(&obligation.conclusion),
        obligation.name
    ));
    source.push_str("}\n");
    source
}

fn render_kani_bool(term: &Term) -> String {
    match term {
        Term::Bool(value) => value.to_string(),
        Term::Eq(left, right) => {
            format!("{} == {}", render_kani_term(left), render_kani_term(right))
        }
        Term::And(terms) => join_bool_terms(terms, " && "),
        Term::Or(terms) => join_bool_terms(terms, " || "),
        Term::Not(term) => format!("!({})", render_kani_bool(term)),
        Term::Implies(lhs, rhs) => {
            format!(
                "!({}) || ({})",
                render_kani_bool(lhs),
                render_kani_bool(rhs)
            )
        }
        other => render_kani_term(other),
    }
}

fn join_bool_terms(terms: &[Term], separator: &str) -> String {
    terms
        .iter()
        .map(|term| format!("({})", render_kani_bool(term)))
        .collect::<Vec<_>>()
        .join(separator)
}

fn render_kani_term(term: &Term) -> String {
    match term {
        Term::Var(name) => sanitize_ident(name),
        Term::Bool(value) => value.to_string(),
        Term::Int(value) => value.to_string(),
        Term::App { function, args } => format!(
            "{}({})",
            sanitize_ident(function),
            args.iter()
                .map(render_kani_term)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => render_kani_bool(other),
    }
}

fn sanitize_ident(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        out.push(if ch.is_ascii_alphanumeric() || ch == '_' {
            ch
        } else {
            '_'
        });
    }
    if out.is_empty() {
        out.push('_');
    }
    if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AlgebraicSignature, Obligation, ObligationBundle, Origin, Sort};

    #[test]
    fn kani_backend_renders_harness_for_associativity_obligation() {
        let obligation = Obligation::associativity(
            "sum_assoc",
            Origin::new("karpal-core", "Sum<i32>"),
            Sort::Int,
            "combine",
        );

        let harness = export_kani_harness(&obligation);

        assert!(harness.source.contains("#[kani::proof]"));
        assert!(harness.source.contains("fn sum_assoc"));
        assert!(harness.source.contains("kani::any"));
        assert!(harness.source.contains("kani::assert"));
        assert_eq!(harness.obligation_name, "sum_assoc");
        assert_eq!(harness.harness_name, "sum_assoc");
    }

    #[test]
    fn kani_backend_exports_one_harness_per_bundle_obligation() {
        let signature = AlgebraicSignature::monoid(Sort::Int, "combine", "empty");
        let bundle =
            ObligationBundle::monoid("sum", Origin::new("karpal-core", "Sum<i32>"), &signature);

        let harnesses = export_kani_bundle(&bundle);

        assert_eq!(harnesses.len(), bundle.obligations().len());
        assert_eq!(harnesses[0].obligation_name, bundle.obligations()[0].name);
    }

    #[test]
    fn kani_backend_sanitizes_invalid_harness_names() {
        let obligation = Obligation::associativity(
            "12 sum-assoc",
            Origin::new("karpal-core", "Sum<i32>"),
            Sort::Int,
            "combine",
        );

        let harness = export_kani_harness(&obligation);

        assert_eq!(harness.harness_name, "_12_sum_assoc");
        assert!(harness.source.contains("fn _12_sum_assoc"));
    }
}
