use ark_ff::Field;

use crate::{multilinear::ProverConfig, streams::EvaluationStream};

pub struct SpaceProverConfig<F, S>
where
    F: Field,
    S: EvaluationStream<F>,
{
    pub num_variables: usize,
    pub claim: F,
    pub stream: S,
}

impl<'a, F, S> SpaceProverConfig<F, S>
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

impl<F: Field, S: EvaluationStream<F>> ProverConfig<F, S> for SpaceProverConfig<F, S> {
    fn default(claim: F, num_variables: usize, stream: S) -> Self {
        Self {
            claim,
            num_variables,
            stream,
        }
    }
}
