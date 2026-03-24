use std::fs;

use karpal_verify::{
    AlgebraicSignature, ArtifactLayout, DryRunner, LeanManifest, LeanManifestReportFiles,
    LeanProject, Obligation, ObligationBundle, Origin, Sort, VerificationSession,
    export_lean_module, export_smt_obligation,
};
#[cfg(feature = "amari")]
use karpal_verify::{StatisticalBound, verify_rare_event};

#[derive(Clone, Copy)]
enum GoldenFixture {
    GroupLeftInverseSmt,
    LeftDistributivityLean,
    LeanManifestJson,
    VerificationReportJson,
    VerificationReportMarkdown,
    LeanDiagnosticsJson,
    #[cfg(feature = "amari")]
    ThreeTierReportJson,
    #[cfg(feature = "amari")]
    ThreeTierReportMarkdown,
}

fn golden(fixture: GoldenFixture) -> &'static str {
    match fixture {
        GoldenFixture::GroupLeftInverseSmt => include_str!("golden/group_left_inverse.smt2"),
        GoldenFixture::LeftDistributivityLean => {
            include_str!("golden/left_distributivity.lean")
        }
        GoldenFixture::LeanManifestJson => include_str!("golden/lean_manifest.json"),
        GoldenFixture::VerificationReportJson => {
            include_str!("golden/verification_report.json")
        }
        GoldenFixture::VerificationReportMarkdown => {
            include_str!("golden/verification_report.md")
        }
        GoldenFixture::LeanDiagnosticsJson => include_str!("golden/lean_diagnostics.json"),
        #[cfg(feature = "amari")]
        GoldenFixture::ThreeTierReportJson => include_str!("golden/three_tier_report.json"),
        #[cfg(feature = "amari")]
        GoldenFixture::ThreeTierReportMarkdown => include_str!("golden/three_tier_report.md"),
    }
}

#[test]
fn smt_export_for_group_left_inverse_matches_expected_shape() {
    let sig = AlgebraicSignature::group(Sort::Int, "combine", "e", "inv");
    let obligation = Obligation::left_inverse_in(
        "group_left_inverse",
        Origin::new("karpal-algebra", "Group for i32"),
        &sig,
    );

    let rendered = export_smt_obligation(&obligation);

    assert_eq!(rendered, golden(GoldenFixture::GroupLeftInverseSmt));
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

    assert_eq!(rendered, golden(GoldenFixture::LeftDistributivityLean));
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

    assert_eq!(manifest.to_json(), golden(GoldenFixture::LeanManifestJson));
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
    let report_markdown = fs::read_to_string(&output.report_files.markdown_path)
        .expect("report markdown should be readable");
    let diagnostics_json = fs::read_to_string(
        output
            .report_files
            .lean_diagnostics_json_path
            .as_deref()
            .expect("lean diagnostics path should be present"),
    )
    .expect("lean diagnostics json should be readable");

    assert_eq!(report_json, golden(GoldenFixture::VerificationReportJson));
    assert_eq!(
        report_markdown,
        golden(GoldenFixture::VerificationReportMarkdown)
    );
    assert_eq!(diagnostics_json, golden(GoldenFixture::LeanDiagnosticsJson));

    let _ = fs::remove_dir_all(root);
}

#[cfg(feature = "amari")]
#[test]
fn three_tier_report_sidecars_match_expected_shape() {
    let root = "target/karpal-verify-golden";
    let _ = fs::remove_dir_all(root);

    let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
    let bundle = ObligationBundle::monoid(
        "sum_monoid",
        Origin::new("karpal-core", "Monoid for Sum<i32>"),
        &sig,
    );
    let statistical = verify_rare_event(
        &bundle.obligations()[0],
        &StatisticalBound::new(0.05).with_samples(128),
        || false,
    );
    let output = VerificationSession::new(bundle, ArtifactLayout::new(root), "KarpalVerify")
        .with_report_stem("summary")
        .with_statistical_verification(statistical)
        .verify_with_ci_outputs(&DryRunner)
        .expect("ci outputs should be written");

    let three_tier_json = fs::read_to_string(
        output
            .report_files
            .three_tier_json_path
            .as_deref()
            .expect("three-tier json path should be present"),
    )
    .expect("three-tier json should be readable");
    let three_tier_markdown = fs::read_to_string(
        output
            .report_files
            .three_tier_markdown_path
            .as_deref()
            .expect("three-tier markdown path should be present"),
    )
    .expect("three-tier markdown should be readable");

    assert_eq!(three_tier_json, golden(GoldenFixture::ThreeTierReportJson));
    assert_eq!(
        three_tier_markdown,
        golden(GoldenFixture::ThreeTierReportMarkdown)
    );

    let _ = fs::remove_dir_all(root);
}
