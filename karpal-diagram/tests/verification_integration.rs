use karpal_diagram::coherence::coherence_certificates;

#[test]
fn coherence_certificates_returns_three_certificates() {
    let certs = coherence_certificates();
    assert_eq!(
        certs.len(),
        3,
        "should have pentagon, triangle, and hexagon certificates"
    );
}

#[test]
fn coherence_certificates_all_have_expected_backend() {
    for cert in coherence_certificates() {
        assert_eq!(cert.backend, "karpal-diagram-coherence");
    }
}

#[test]
fn coherence_certificates_have_distinct_obligation_names() {
    let certs = coherence_certificates();
    let names: Vec<&str> = certs.iter().map(|c| c.obligation.as_str()).collect();
    let names_lower: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
    assert!(names_lower.iter().any(|n| n.contains("pentagon")));
    assert!(names_lower.iter().any(|n| n.contains("triangle")));
    assert!(names_lower.iter().any(|n| n.contains("hexagon")));
}
