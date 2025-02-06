use ark_ff::Field;

use crate::streams::EvaluationStream;

pub trait ProverConfig<F: Field, S: EvaluationStream<F>> {
    fn default(claim: F, num_variables: usize, stream: S) -> Self;
}
pub trait Prover<F: Field> {
    type ProverConfig;
    type ProverMessage;
    type VerifierMessage;
    fn claim(&self) -> F;
    fn new(prover_config: Self::ProverConfig) -> Self;
    fn next_message(&mut self, verifier_message: Self::VerifierMessage) -> Self::ProverMessage;
}
