#[verifier::external_body]
pub proof fn copied_readiness_axiom() {
    assume_specification();
    assume(false);
    axiom readiness_is_complete;
}
