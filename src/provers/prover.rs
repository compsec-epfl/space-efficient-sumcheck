use ark_ff::Field;

use crate::provers::evaluation_stream::EvaluationStream;
pub struct ProverArgs<'a, F: Field> {
    pub stream: Box<&'a dyn EvaluationStream<F>>,
    pub num_stages: usize,
}
pub trait Prover<'a, F: Field> {
    const DEFAULT_NUM_STAGES: usize;
    fn new(prover_args: ProverArgs<'a, F>) -> Self;
    fn claimed_sum(&self) -> F;
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F)>;
    fn total_rounds(&self) -> usize;
}
