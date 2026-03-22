#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use crate::{LeanExport, Obligation, ObligationBundle, export_lean_module, export_smt_obligation};

/// Export an entire bundle as one SMT-LIB2 script per obligation.
pub fn export_smt_bundle(bundle: &ObligationBundle) -> Vec<(String, String)> {
    bundle
        .obligations()
        .iter()
        .map(|obligation| (obligation.name.clone(), export_smt_obligation(obligation)))
        .collect()
}

/// Export a whole bundle as a single Lean 4 module.
pub fn export_lean_bundle(module_name: &str, bundle: &ObligationBundle) -> String {
    export_lean_module(module_name, bundle.obligations())
}

/// Export a whole bundle as structured Lean module metadata plus source.
pub fn export_lean_bundle_structured(module_name: &str, bundle: &ObligationBundle) -> LeanExport {
    crate::lean::export(module_name, bundle.obligations())
}

/// Export a list of obligations as one SMT-LIB2 script per obligation.
pub fn export_smt_batch(obligations: &[Obligation]) -> Vec<(String, String)> {
    obligations
        .iter()
        .map(|obligation| (obligation.name.clone(), export_smt_obligation(obligation)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AlgebraicSignature, Origin, Sort};

    #[test]
    fn smt_bundle_export_returns_one_script_per_obligation() {
        let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
        let bundle = ObligationBundle::monoid("sum", Origin::new("karpal-core", "Sum<i32>"), &sig);
        let scripts = export_smt_bundle(&bundle);
        assert_eq!(scripts.len(), 3);
        assert!(scripts[0].1.contains("(check-sat)"));
    }

    #[test]
    fn lean_bundle_export_contains_multiple_theorems() {
        let sig = AlgebraicSignature::group(Sort::Int, "combine", "e", "inv");
        let bundle = ObligationBundle::group("sum", Origin::new("karpal-algebra", "i32"), &sig);
        let module = export_lean_bundle("KarpalVerify", &bundle);
        assert!(module.contains("theorem associativity"));
        assert!(module.contains("theorem left_inverse"));
        assert!(module.contains("theorem right_inverse"));

        let structured = export_lean_bundle_structured("KarpalVerify", &bundle);
        assert_eq!(structured.theorems.len(), 5);
        assert_eq!(
            structured
                .theorem_for_obligation("left_inverse")
                .map(|t| t.witness_ref("KarpalVerify")),
            Some("KarpalVerify.left_inverse".into())
        );
    }
}
