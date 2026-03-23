use std::fs;

use karpal_verify::{
    AlgebraicSignature, ArtifactLayout, DryRunner, LeanManifest, LeanManifestReportFiles,
    LeanProject, Obligation, ObligationBundle, Origin, Sort, VerificationSession,
    export_lean_module, export_smt_obligation,
};

#[test]
fn smt_export_for_group_left_inverse_matches_expected_shape() {
    let sig = AlgebraicSignature::group(Sort::Int, "combine", "e", "inv");
    let obligation = Obligation::left_inverse_in(
        "group_left_inverse",
        Origin::new("karpal-algebra", "Group for i32"),
        &sig,
    );

    let rendered = export_smt_obligation(&obligation);
    let expected = r#"; obligation: group_left_inverse
; property: left inverse
; origin: karpal-algebra::Group for i32 [left inverse]
(set-logic ALL)
(declare-const a Int)
(declare-const e Int)
; ask the solver for a counterexample to the law
(assert (not (= (combine (inv a) a) e)))
(check-sat)
(get-model)"#;

    assert_eq!(rendered, expected);
}

#[test]
fn lean_export_for_semiring_left_distributivity_matches_expected_shape() {
    let sig = AlgebraicSignature::semiring(Sort::Int, "add", "mul", "zero", "one");
    let obligation = Obligation::left_distributivity_in(
        "left/distributivity",
        Origin::new("karpal-algebra", "Semiring for i32"),
        &sig,
    );

    let rendered = export_lean_module("KarpalVerify", &[obligation]);
    let expected = r#"namespace KarpalVerify

-- property: distributive
-- origin: karpal-algebra::Semiring for i32 [distributive]
theorem left_distributivity (a : Int) (b : Int) (c : Int) : (mul a (add b c)) = (add (mul a b) (mul a c)) := by
  sorry

end KarpalVerify"#;

    assert_eq!(rendered, expected);
}

#[test]
fn lean_manifest_json_matches_expected_shape() {
    let obligation = Obligation::associativity(
        "sum_assoc",
        Origin::new("karpal-algebra", "Semigroup for Sum<i32>"),
        Sort::Int,
        "combine",
    );
    let export = karpal_verify::Lean4::export("KarpalVerify", &[obligation]);
    let project = LeanProject::for_export(&export);
    let manifest = LeanManifest::from_export(&export, &project).with_report_files(
        LeanManifestReportFiles::new("target/verify/report.json", "target/verify/report.md")
            .with_lean_diagnostics_json_path("target/verify/report.lean-diagnostics.json"),
    );

    let expected = r#"{"schema_version":"1","module_name":"KarpalVerify","project":{"package_name":"karpalverify","toolchain":"leanprover/lean4:stable","requires_mathlib":false},"prelude":{"imports":[],"aliases":[]},"theorems":[{"obligation_name":"sum_assoc","theorem_name":"sum_assoc","witness_ref":"KarpalVerify.sum_assoc","declaration_start_line":5,"declaration_end_line":6}],"report_files":{"schema_version":"1","json_path":"target/verify/report.json","markdown_path":"target/verify/report.md","lean_diagnostics_json_path":"target/verify/report.lean-diagnostics.json"}}"#;

    assert_eq!(manifest.to_json(), expected);
}

#[test]
fn verification_report_json_files_match_expected_shape() {
    let root = "target/karpal-verify-golden";
    let _ = fs::remove_dir_all(root);

    let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
    let bundle = ObligationBundle::monoid(
        "sum_monoid",
        Origin::new("karpal-core", "Monoid for Sum<i32>"),
        &sig,
    );
    let output = VerificationSession::new(bundle, ArtifactLayout::new(root), "KarpalVerify")
        .with_report_stem("summary")
        .verify_with_ci_outputs(&DryRunner)
        .expect("ci outputs should be written");

    let report_json =
        fs::read_to_string(&output.report_files.json_path).expect("report json should be readable");
    let diagnostics_json = fs::read_to_string(
        output
            .report_files
            .lean_diagnostics_json_path
            .as_deref()
            .expect("lean diagnostics path should be present"),
    )
    .expect("lean diagnostics json should be readable");

    let expected_report = r#"{"schema_version":"1","bundle_name":"sum_monoid","root":"target/karpal-verify-golden","success_count":0,"failure_count":3,"obligations":[{"name":"associativity","summary":"karpal-core::Monoid for Sum<i32> [associativity]","status":"DryRun","artifact_path":"target/karpal-verify-golden/smt/associativity.smt2","lean_theorem_ref":"KarpalVerify.associativity","lean_diagnostic_count":0,"certificate":null,"lean_certificate":null},{"name":"left_identity","summary":"karpal-core::Monoid for Sum<i32> [left identity]","status":"DryRun","artifact_path":"target/karpal-verify-golden/smt/left_identity.smt2","lean_theorem_ref":"KarpalVerify.left_identity","lean_diagnostic_count":0,"certificate":null,"lean_certificate":null},{"name":"right_identity","summary":"karpal-core::Monoid for Sum<i32> [right identity]","status":"DryRun","artifact_path":"target/karpal-verify-golden/smt/right_identity.smt2","lean_theorem_ref":"KarpalVerify.right_identity","lean_diagnostic_count":0,"certificate":null,"lean_certificate":null}],"lean_module":{"module_name":"KarpalVerify","status":"DryRun","theorem_count":3,"import_count":0,"alias_count":0,"diagnostic_count":0,"theorem_failure_count":0,"certificate":null},"report_files":{"schema_version":"1","json_path":"target/karpal-verify-golden/summary.json","markdown_path":"target/karpal-verify-golden/summary.md","lean_diagnostics_json_path":"target/karpal-verify-golden/summary.lean-diagnostics.json","lean_manifest_path":"target/karpal-verify-golden/lean/KarpalVerify.manifest.json"}}"#;
    let expected_diagnostics = r#"{"schema_version":"1","module_name":"KarpalVerify","module_diagnostics":[],"theorem_failures":[],"obligations":[{"obligation_name":"associativity","theorem_ref":"KarpalVerify.associativity","diagnostics":[]},{"obligation_name":"left_identity","theorem_ref":"KarpalVerify.left_identity","diagnostics":[]},{"obligation_name":"right_identity","theorem_ref":"KarpalVerify.right_identity","diagnostics":[]}]}"#;

    assert_eq!(report_json, expected_report);
    assert_eq!(diagnostics_json, expected_diagnostics);

    let _ = fs::remove_dir_all(root);
}
