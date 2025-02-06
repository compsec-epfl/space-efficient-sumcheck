use ark_ff::Field;

use crate::{prover::ProverConfig, streams::EvaluationStream};

pub struct TimeProverConfig<F, S>
where
    F: Field,
    S: EvaluationStream<F>,
{
    pub num_variables: usize,
    pub claim: F,
    pub stream: S,
}

impl<'a, F, S> TimeProverConfig<F, S>
where
    F: Field,
    S: EvaluationStream<F>,
{
    pub fn new(claim: F, num_variables: usize, stream: S) -> Self {
        Self {
            claim,
            num_variables,
            stream,
        }
    }
}

impl<F: Field, S: EvaluationStream<F>> ProverConfig<F, S> for TimeProverConfig<F, S> {
    fn default(claim: F, num_variables: usize, stream: S) -> Self {
        Self {
            claim,
            num_variables,
            stream,
        }
    }
}
