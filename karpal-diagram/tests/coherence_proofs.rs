use karpal_arrow::{Category, FnA, Semigroupoid};
use karpal_diagram::Tensor;
use karpal_diagram::coherence::{
    HexagonIdentity, PentagonIdentity, TriangleIdentity, verify_hexagon, verify_pentagon,
    verify_triangle,
};
use karpal_proof::rewrite::Rewrite;

#[test]
fn pentagon_witness_compiles_for_concrete_types() {
    let _witness: Rewrite<
        (((i32, u8), bool), String),
        (i32, (u8, (bool, String))),
        PentagonIdentity,
    > = verify_pentagon::<i32, u8, bool, String>();
}

#[test]
fn triangle_witness_compiles_for_concrete_types() {
    let _witness: Rewrite<((i32, ()), String), (i32, String), TriangleIdentity> =
        verify_triangle::<i32, String>();
}

#[test]
fn pentagon_paths_produce_equal_results() {
    // Upper path: (α_{A,B,C} ⊗ id_D) ; α_{A,B⊗C,D} ; (id_A ⊗ α_{B,C,D})
    let upper_step1 = FnA::tensor(FnA::associate::<i32, u8, bool>(), FnA::id::<String>());
    let upper_step2 = FnA::associate::<i32, (u8, bool), String>();
    let upper_step3 = FnA::tensor(FnA::id::<i32>(), FnA::associate::<u8, bool, String>());
    let upper = FnA::compose(upper_step3, FnA::compose(upper_step2, upper_step1));

    // Lower path: α_{A⊗B,C,D} ; α_{A,B,C⊗D}
    let lower_step1 = FnA::associate::<(i32, u8), bool, String>();
    let lower_step2 = FnA::associate::<i32, u8, (bool, String)>();
    let lower = FnA::compose(lower_step2, lower_step1);

    let input = (((1_i32, 2_u8), true), "end".to_string());
    assert_eq!(upper(input.clone()), lower(input));
}

#[test]
fn triangle_paths_produce_equal_results() {
    // Left path: ρ_A ⊗ id_B
    let left = FnA::tensor(FnA::right_unitor::<i32>(), FnA::id::<String>());

    // Right path: α_{A,I,B} ; (id_A ⊗ λ_B)
    let alpha = FnA::associate::<i32, (), String>();
    let right_step2 = FnA::tensor(FnA::id::<i32>(), FnA::left_unitor::<String>());
    let right = FnA::compose(right_step2, alpha);

    let input = ((42_i32, ()), "hello".to_string());
    assert_eq!(left(input.clone()), right(input.clone()));
}

#[test]
fn hexagon_witness_compiles_for_concrete_types() {
    let _witness: Rewrite<((i32, u8), bool), ((u8, bool), i32), HexagonIdentity> =
        verify_hexagon::<i32, u8, bool>();
}

#[test]
fn hexagon_paths_produce_equal_results() {
    use karpal_diagram::Braiding;

    // Upper path: braid outer, then reassociate-inv, braid, reassociate-inv
    let braid_ab_c = FnA::braid::<(i32, u8), bool>(); // ((A,B),C) → (C,(A,B))
    let assoc_inv_c_a_b = FnA::associate_inv::<bool, i32, u8>(); // (C,(A,B)) → ((C,A),B)
    let braid_ca_b = FnA::braid::<(bool, i32), u8>(); // ((C,A),B) → (B,(C,A))
    let assoc_inv_b_c_a = FnA::associate_inv::<u8, bool, i32>(); // (B,(C,A)) → ((B,C),A)
    let upper = FnA::compose(
        assoc_inv_b_c_a,
        FnA::compose(braid_ca_b, FnA::compose(assoc_inv_c_a_b, braid_ab_c)),
    );

    // Lower path: associate then braid compound type
    let assoc_a_b_c = FnA::associate::<i32, u8, bool>(); // ((A,B),C) → (A,(B,C))
    let braid_a_bc = FnA::braid::<i32, (u8, bool)>(); // (A,(B,C)) → ((B,C),A)
    let lower = FnA::compose(braid_a_bc, assoc_a_b_c);

    let input = ((1_i32, 2_u8), true);
    assert_eq!(upper(input.clone()), lower(input));
}
