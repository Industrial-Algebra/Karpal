use karpal_arrow::{Arrow, FnA};
use karpal_diagram::Trace;

#[test]
fn fna_trace_closes_feedback_with_default_seed() {
    let traced = FnA::trace::<i32, i32, i32>(FnA::arr(|(input, feedback)| {
        (input + feedback, feedback + 1)
    }));

    assert_eq!(traced(7), 7);
}

#[test]
fn fna_trace_preserves_passthrough_morphism() {
    let traced = FnA::trace::<&'static str, &'static str, bool>(FnA::arr(|(input, feedback)| {
        (input, feedback)
    }));

    assert_eq!(traced("wire"), "wire");
}
