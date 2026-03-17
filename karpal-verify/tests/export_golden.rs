use karpal_verify::{
    AlgebraicSignature, Obligation, Origin, Sort, export_lean_module, export_smt_obligation,
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
