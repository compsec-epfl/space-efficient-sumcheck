use ark_ff::Field;

use crate::provers::evaluation_stream::EvaluationStream;

pub struct ProverArgsStageInfo {
    pub num_stages: usize,
}
pub struct ProverArgs<'a, F: Field> {
    pub stream: &'a dyn EvaluationStream<F>,
    pub stage_info: Option<ProverArgsStageInfo>,
}
pub trait Prover<'a, F: Field> {
    fn generate_default_args(stream: &'a impl EvaluationStream<F>) -> ProverArgs<'a, F>;
    fn new(prover_args: ProverArgs<'a, F>) -> Self;
    fn claimed_sum(&self) -> F;
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F)>;
    fn total_rounds(&self) -> usize;
}
