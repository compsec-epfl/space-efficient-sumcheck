use ark_ff::Field;
use ark_std::marker::PhantomData;

use crate::streams::EvaluationStream;

pub struct ProverArgsStageInfo {
    pub num_stages: usize,
}
pub struct ProverArgs<'a, F: Field, S: EvaluationStream<F>> {
    pub stream_p: &'a S,
    pub stream_q: &'a S,
    pub claimed_sum: F,
    pub stage_info: Option<ProverArgsStageInfo>,
    pub _phantom: PhantomData<F>,
}
pub trait Prover<'a, F: Field, S: EvaluationStream<F>> {
    fn generate_default_args(
        stream_p: &'a S,
        stream_q: &'a S,
        claimed_sum: F,
    ) -> ProverArgs<'a, F, S>;
    fn new(prover_args: ProverArgs<'a, F, S>) -> Self;
    fn claimed_sum(&self) -> F;
    fn next_message(&mut self, verifier_message: Option<F>) -> Option<(F, F, F)>;
    fn total_rounds(&self) -> usize;
}
