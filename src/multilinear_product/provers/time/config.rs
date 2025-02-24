use ark_ff::Field;

use crate::{prover::ProductProverConfig, streams::EvaluationStream};

pub struct TimeProductProverConfig<F, S>
where
    F: Field,
    S: EvaluationStream<F>,
{
    pub num_variables: usize,
    pub claim: F,
    pub stream_p: S,
    pub stream_q: S,
}

impl<'a, F, S> TimeProductProverConfig<F, S>
where
    F: Field,
    S: EvaluationStream<F>,
{
    pub fn new(claim: F, num_variables: usize, stream_p: S, stream_q: S) -> Self {
        Self {
            claim,
            num_variables,
            stream_p,
            stream_q,
        }
    }
}

impl<F: Field, S: EvaluationStream<F>> ProductProverConfig<F, S> for TimeProductProverConfig<F, S> {
    fn default(claim: F, num_variables: usize, stream_p: S, stream_q: S) -> Self {
        Self {
            claim,
            num_variables,
            stream_p,
            stream_q,
        }
    }
}
